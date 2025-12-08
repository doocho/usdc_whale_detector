# USDC Whale Detector 🐋

A Rust program that monitors large USDC transfers in real-time across Ethereum and L2 chains (Arbitrum, Base).

## Features

- **Multi-chain Support**: Ethereum, Arbitrum, Base
- **Parallel Monitoring**: Independent monitor running for each chain
- **Whale Detection**: Detects USDC transfers over $74,000 (approximately 100 million KRW)
- **Address Labeling**: Automatic identification of known exchange/protocol addresses

## Installation & Running

```bash
# Build
cargo build --release

# Run
cargo run --release
```

## Supported Chains

| Chain | USDC Contract |
|-------|---------------|
| Ethereum | `0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48` |
| Arbitrum | `0xaf88d065e77c8cC2239327C5EDb3A432268e5831` |
| Base | `0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913` |

## Output Example

```
[2024-12-08 15:30:45] [ETHEREUM] 🐋 WHALE TRANSFER DETECTED
  Amount: $500,000.00 USDC
  From: 0x1234...abcd (Binance Hot Wallet)
  To: 0x5678...efgh (Unknown)
  Tx: 0xabcd...1234
```

## Customizing Address Labels

You can add custom address labels by editing the `data/labels.json` file:

```json
{
  "0x1234...": "My Custom Wallet",
  "0x5678...": "Protocol Treasury"
}
```

## Threshold Configuration

The default threshold is 74,000 USDC (approximately 100 million KRW). You can modify it in `src/config.rs`.

## License

MIT
