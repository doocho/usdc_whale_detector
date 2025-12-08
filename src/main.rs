mod config;
mod labels;
mod monitor;
mod types;

use crate::config::{get_all_chains, WHALE_THRESHOLD_USD};
use crate::labels::LabelStore;
use crate::monitor::ChainMonitor;
use crate::types::WhaleTransfer;

use chrono::Local;
use colored::Colorize;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Print a whale transfer to the console with formatting
fn print_whale_transfer(transfer: &WhaleTransfer) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let chain_color = match transfer.chain {
        types::Chain::Ethereum => "blue",
        types::Chain::Arbitrum => "cyan",
        types::Chain::Base => "magenta",
    };

    println!();
    println!(
        "{} {} 🐋 {}",
        format!("[{}]", timestamp).bright_black(),
        format!("[{}]", transfer.chain.name()).color(chain_color).bold(),
        "WHALE TRANSFER DETECTED".bright_yellow().bold()
    );
    println!(
        "  {} {}",
        "Amount:".bright_white(),
        transfer.formatted_amount().bright_green().bold()
    );
    println!(
        "  {} {}",
        "From:  ".bright_white(),
        transfer.formatted_from()
    );
    println!(
        "  {} {}",
        "To:    ".bright_white(),
        transfer.formatted_to()
    );
    println!(
        "  {} {}",
        "Tx:    ".bright_white(),
        transfer.short_tx_hash().bright_blue()
    );
    println!(
        "  {} {}",
        "Block: ".bright_white(),
        transfer.block_number.to_string().bright_black()
    );
    println!(
        "  {} {}",
        "Link:  ".bright_white(),
        transfer.chain.explorer_tx_url(&transfer.tx_hash).bright_blue().underline()
    );
}

/// Print startup banner
fn print_banner() {
    println!();
    println!("{}", "╔═══════════════════════════════════════════════════════════════╗".bright_cyan());
    println!("{}", "║                                                               ║".bright_cyan());
    println!("{}", "║   🐋  USDC WHALE DETECTOR  🐋                                 ║".bright_cyan());
    println!("{}", "║                                                               ║".bright_cyan());
    println!("{}", "║   Monitoring large USDC transfers across chains               ║".bright_cyan());
    println!("{}", "║                                                               ║".bright_cyan());
    println!("{}", "╚═══════════════════════════════════════════════════════════════╝".bright_cyan());
    println!();
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("usdc_whale_detector=info".parse()?)
                .add_directive("alloy=warn".parse()?),
        )
        .with_target(false)
        .init();

    print_banner();

    // Load address labels
    let labels = Arc::new(LabelStore::default());
    println!(
        "{} {} address labels",
        "✓".bright_green(),
        format!("Loaded {}", labels.len()).bright_white()
    );

    // Print configuration
    println!(
        "{} {} ${} USDC (~100M KRW)",
        "✓".bright_green(),
        "Whale threshold:".bright_white(),
        WHALE_THRESHOLD_USD.to_string().bright_yellow()
    );

    // Create channel for whale transfers
    let (tx, mut rx) = mpsc::channel::<WhaleTransfer>(100);

    // Get chain configurations
    let chains = get_all_chains();
    println!(
        "{} {} chains: {}",
        "✓".bright_green(),
        "Monitoring".bright_white(),
        chains
            .iter()
            .map(|c| c.chain.name())
            .collect::<Vec<_>>()
            .join(", ")
            .bright_cyan()
    );

    println!();
    println!("{}", "Starting monitors...".bright_white());
    println!("{}", "─".repeat(65).bright_black());

    // Spawn monitors for each chain
    let mut handles = Vec::new();

    for chain_config in chains {
        let labels_clone = Arc::clone(&labels);
        let tx_clone = tx.clone();

        let handle = tokio::spawn(async move {
            let monitor = ChainMonitor::new(chain_config, labels_clone, tx_clone);
            if let Err(e) = monitor.run().await {
                tracing::error!(error = %e, "Monitor failed");
            }
        });

        handles.push(handle);
    }

    // Drop the original sender so the receiver knows when all monitors are done
    drop(tx);

    // Process whale transfers from all chains
    let printer_handle = tokio::spawn(async move {
        while let Some(transfer) = rx.recv().await {
            print_whale_transfer(&transfer);
        }
    });

    // Wait for all monitors (they run indefinitely unless there's an error)
    // In practice, this will run forever
    for handle in handles {
        let _ = handle.await;
    }

    // Wait for the printer to finish
    let _ = printer_handle.await;

    Ok(())
}

