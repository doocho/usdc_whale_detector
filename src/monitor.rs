use crate::config::{ChainConfig, POLL_INTERVAL_SECS, TRANSFER_EVENT_SIGNATURE, WHALE_THRESHOLD_RAW};
use crate::labels::LabelStore;
use crate::types::WhaleTransfer;

use alloy::primitives::{Address, B256, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::{Filter, Log};
use eyre::Result;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;

/// Chain monitor that watches for USDC whale transfers
pub struct ChainMonitor {
    config: ChainConfig,
    labels: Arc<LabelStore>,
    tx: mpsc::Sender<WhaleTransfer>,
}

impl ChainMonitor {
    /// Create a new chain monitor
    pub fn new(
        config: ChainConfig,
        labels: Arc<LabelStore>,
        tx: mpsc::Sender<WhaleTransfer>,
    ) -> Self {
        Self { config, labels, tx }
    }

    /// Start monitoring the chain for whale transfers
    pub async fn run(&self) -> Result<()> {
        tracing::info!(
            chain = %self.config.chain,
            rpc = %self.config.rpc_url,
            usdc = ?self.config.usdc_address,
            "Starting monitor"
        );

        loop {
            match self.monitor_loop().await {
                Ok(_) => {
                    tracing::info!(chain = %self.config.chain, "Monitor stopped");
                    break;
                }
                Err(e) => {
                    tracing::error!(
                        chain = %self.config.chain,
                        error = %e,
                        "Monitor error, restarting in 10 seconds..."
                    );
                    sleep(Duration::from_secs(10)).await;
                }
            }
        }

        Ok(())
    }

    /// Main monitoring loop
    async fn monitor_loop(&self) -> Result<()> {
        let provider = ProviderBuilder::new()
            .on_http(self.config.rpc_url.parse()?);

        // Get the current block number to start from
        let mut last_block = provider.get_block_number().await?;
        tracing::info!(
            chain = %self.config.chain,
            block = last_block,
            "Starting from block"
        );

        let transfer_topic = B256::from_str(TRANSFER_EVENT_SIGNATURE)?;

        loop {
            // Get the latest block
            let latest_block = provider.get_block_number().await?;

            if latest_block > last_block {
                // Query logs for new blocks
                let filter = Filter::new()
                    .address(self.config.usdc_address)
                    .event_signature(transfer_topic)
                    .from_block(last_block + 1)
                    .to_block(latest_block);

                match provider.get_logs(&filter).await {
                    Ok(logs) => {
                        for log in logs {
                            if let Some(transfer) = self.process_log(&log) {
                                if let Err(e) = self.tx.send(transfer).await {
                                    tracing::error!(
                                        chain = %self.config.chain,
                                        error = %e,
                                        "Failed to send whale transfer"
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            chain = %self.config.chain,
                            error = %e,
                            "Failed to get logs, will retry"
                        );
                    }
                }

                last_block = latest_block;
            }

            // Wait before polling again
            sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
        }
    }

    /// Process a Transfer event log and return a WhaleTransfer if it meets the threshold
    fn process_log(&self, log: &Log) -> Option<WhaleTransfer> {
        // Transfer event has 3 topics: event signature, from, to
        // and data contains the amount
        if log.topics().len() < 3 {
            return None;
        }

        // Extract from and to addresses from topics
        let from = Address::from_slice(&log.topics()[1].as_slice()[12..]);
        let to = Address::from_slice(&log.topics()[2].as_slice()[12..]);

        // Extract amount from data
        let amount = if log.data().data.len() >= 32 {
            U256::from_be_slice(&log.data().data[..32])
        } else {
            return None;
        };

        // Check if this is a whale transfer
        let amount_u128 = amount.to::<u128>();
        if amount_u128 < WHALE_THRESHOLD_RAW {
            return None;
        }

        // Get transaction hash
        let tx_hash = log.transaction_hash?;
        let block_number = log.block_number?;

        // Create whale transfer with labels
        let transfer = WhaleTransfer::new(
            self.config.chain,
            tx_hash,
            block_number,
            from,
            to,
            amount,
        )
        .with_from_label(self.labels.get(&from))
        .with_to_label(self.labels.get(&to));

        Some(transfer)
    }
}

