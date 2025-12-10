use crate::types::Chain;
use alloy::primitives::Address;
use std::str::FromStr;

/// USDC has 6 decimal places
pub const USDC_DECIMALS: u8 = 6;

/// Whale threshold: 1,000,000 USDC
pub const WHALE_THRESHOLD_USD: u64 = 1_000_000;

/// Whale threshold in raw units (1,000,000 * 10^6)
pub const WHALE_THRESHOLD_RAW: u128 = WHALE_THRESHOLD_USD as u128 * 1_000_000;

/// Polling interval in seconds for checking new blocks
pub const POLL_INTERVAL_SECS: u64 = 3;

/// Configuration for a specific chain
#[derive(Debug, Clone)]
pub struct ChainConfig {
    /// The chain identifier
    pub chain: Chain,
    /// RPC endpoint URL
    pub rpc_url: String,
    /// USDC contract address
    pub usdc_address: Address,
}

impl ChainConfig {
    /// Create a new chain configuration
    pub fn new(chain: Chain, rpc_url: &str, usdc_address: &str) -> Self {
        Self {
            chain,
            rpc_url: rpc_url.to_string(),
            usdc_address: Address::from_str(usdc_address).expect("Invalid USDC address"),
        }
    }
}

/// Get all supported chain configurations
pub fn get_all_chains() -> Vec<ChainConfig> {
    vec![
        // Ethereum Mainnet
        ChainConfig::new(
            Chain::Ethereum,
            "https://eth.llamarpc.com",
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        ),
        // Arbitrum One
        ChainConfig::new(
            Chain::Arbitrum,
            "https://arb1.arbitrum.io/rpc",
            "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
        ),
        // Base
        ChainConfig::new(
            Chain::Base,
            "https://mainnet.base.org",
            "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
        ),
    ]
}

/// ERC20 Transfer event signature
/// keccak256("Transfer(address,address,uint256)")
pub const TRANSFER_EVENT_SIGNATURE: &str =
    "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";
