use alloy::primitives::{Address, B256, U256};
use std::fmt;

/// Supported blockchain networks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Chain {
    Ethereum,
    Arbitrum,
    Base,
}

impl Chain {
    /// Get the display name of the chain
    pub fn name(&self) -> &'static str {
        match self {
            Chain::Ethereum => "ETHEREUM",
            Chain::Arbitrum => "ARBITRUM",
            Chain::Base => "BASE",
        }
    }

    /// Get the block explorer URL for transactions
    pub fn explorer_tx_url(&self, tx_hash: &B256) -> String {
        let base_url = match self {
            Chain::Ethereum => "https://etherscan.io/tx/",
            Chain::Arbitrum => "https://arbiscan.io/tx/",
            Chain::Base => "https://basescan.org/tx/",
        };
        format!("{}{:?}", base_url, tx_hash)
    }

    /// Get the block explorer URL for addresses
    pub fn explorer_address_url(&self, address: &Address) -> String {
        let base_url = match self {
            Chain::Ethereum => "https://etherscan.io/address/",
            Chain::Arbitrum => "https://arbiscan.io/address/",
            Chain::Base => "https://basescan.org/address/",
        };
        format!("{}{:?}", base_url, address)
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Represents a detected whale transfer
#[derive(Debug, Clone)]
pub struct WhaleTransfer {
    /// The blockchain where the transfer occurred
    pub chain: Chain,
    /// Transaction hash
    pub tx_hash: B256,
    /// Block number
    pub block_number: u64,
    /// Sender address
    pub from: Address,
    /// Sender label (if known)
    pub from_label: Option<String>,
    /// Recipient address
    pub to: Address,
    /// Recipient label (if known)
    pub to_label: Option<String>,
    /// Transfer amount in raw units (6 decimals for USDC)
    pub amount_raw: U256,
    /// Transfer amount in USD
    pub amount_usd: f64,
}

impl WhaleTransfer {
    /// Create a new WhaleTransfer
    pub fn new(
        chain: Chain,
        tx_hash: B256,
        block_number: u64,
        from: Address,
        to: Address,
        amount_raw: U256,
    ) -> Self {
        // USDC has 6 decimals
        let amount_usd = amount_raw.to::<u128>() as f64 / 1_000_000.0;

        Self {
            chain,
            tx_hash,
            block_number,
            from,
            from_label: None,
            to,
            to_label: None,
            amount_raw,
            amount_usd,
        }
    }

    /// Set the from address label
    pub fn with_from_label(mut self, label: Option<String>) -> Self {
        self.from_label = label;
        self
    }

    /// Set the to address label
    pub fn with_to_label(mut self, label: Option<String>) -> Self {
        self.to_label = label;
        self
    }

    /// Format the address with optional label
    fn format_address(address: &Address, label: &Option<String>) -> String {
        let addr_str = format!("{:?}", address);
        let short_addr = format!("{}...{}", &addr_str[..10], &addr_str[addr_str.len() - 8..]);

        match label {
            Some(l) => format!("{} ({})", short_addr, l),
            None => format!("{} (Unknown)", short_addr),
        }
    }

    /// Get formatted from address
    pub fn formatted_from(&self) -> String {
        Self::format_address(&self.from, &self.from_label)
    }

    /// Get formatted to address
    pub fn formatted_to(&self) -> String {
        Self::format_address(&self.to, &self.to_label)
    }

    /// Get formatted amount with thousands separator
    pub fn formatted_amount(&self) -> String {
        let formatted = format_with_commas(self.amount_usd);
        format!("${} USDC", formatted)
    }

    /// Get short transaction hash
    pub fn short_tx_hash(&self) -> String {
        let tx_str = format!("{:?}", self.tx_hash);
        format!("{}...{}", &tx_str[..10], &tx_str[tx_str.len() - 8..])
    }
}

/// Format a number with commas as thousands separators
fn format_with_commas(value: f64) -> String {
    let integer_part = value.trunc() as i64;
    let decimal_part = ((value.fract() * 100.0).round() as i64).abs();

    let int_str = integer_part.abs().to_string();
    let mut result = String::new();
    let chars: Vec<char> = int_str.chars().collect();

    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }

    if integer_part < 0 {
        result = format!("-{}", result);
    }

    format!("{}.{:02}", result, decimal_part)
}
