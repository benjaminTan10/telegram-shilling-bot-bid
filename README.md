# Solana PumpFun Copy Trading Bot in Rust 🚀

## Overview

Introducing the **Solana PumpFun Copy Trading Bot** written in **Rust** 🦀, designed to monitor and replicate trades on **Pump.fun** with lightning speed ⚡. Built with Rust for superior performance and security, this bot tracks specific wallets for trading signals and executes copy trades using Jito MEV integration for enhanced transaction priority. Perfect for traders looking to automatically mirror successful trading strategies on PumpFun DEX.

---

## Key Features

### 🚀 Speed and Efficiency
- **Lightning-Quick Transactions**: Leveraging Rust's exceptional performance for near-instant trading execution
- **Jito MEV Integration**: Support for Jito bundles and tips for enhanced transaction priority

### 🔒 Safety First
- **Robust Security**: Rust's safety guarantees minimize bugs and vulnerabilities
- **Configurable Parameters**: Customizable slippage and amount settings for risk management



### 📊 Monitoring Capabilities
- **Wallet Tracking**: Monitor specific wallets for trading signals
- **Multiple RPC Support**: Compatible with various Solana RPC providers

### 🛠️ Core Features
- **PumpFun DEX Integration**: Specialized support for PumpFun trading
- **Flexible Configuration**: Environment-based setup for easy deployment
- **Comprehensive Logging**: Detailed transaction and error logging

---

## Directory Structure

```
src/
├── common/
│   ├── logger.rs        # Logging functionality
│   └── utils.rs         # Common utilities and configurations
├── core/
│   ├── token.rs         # Token account management
│   └── tx.rs           # Transaction handling and Jito integration
├── dex/
│   └── pump_fun.rs      # PumpFun DEX integration
├── engine/
│   └── monitor/         # Transaction and wallet monitoring
├── services/
│   └── jito.rs         # Jito MEV services integration
├── lib.rs
└── main.rs
```

## Environment Variables

```plaintext
PRIVATE_KEY=your_private_key_here
RPC_HTTPS=https://your-rpc-endpoint
RPC_WSS=wss://your-websocket-endpoint
SLIPPAGE=10
UNIT_PRICE=1
UNIT_LIMIT=300000
LOG_LEVEL=debug
```

## Usage

1. Install Rust and Cargo
2. Clone this repository
3. Set up environment variables
4. Build and run:

```bash
cargo build --release
cargo run --release
```

## Support

For support and inquiries, please connect via Telegram: 📞 [Benjamin](https://t.me/blockchainDeveloper_Ben)

## License

MIT License

# Using Yellowstone gRPC with InstantNodes

## Setup
1. Configure environment variables in `.env`:
```env
RPC_GRPC=solana-grpc-geyser.instantnodes.io:443
RPC_TOKEN=your_token_here
```

## Usage Examples

### Monitor PumpFun Transactions
```bash
cargo run -- --endpoint $RPC_GRPC --x-token $RPC_TOKEN subscribe \
  --transactions \
  --transactions-vote false \
  --transactions-failed false \
  --transactions-account-include "o7RY6P2vQMuGSu1TrLM81weuzgDjaCRTXYRaXJwWcvc"
```

### Monitor Account Updates
```bash
cargo run -- --endpoint $RPC_GRPC --x-token $RPC_TOKEN subscribe \
  --accounts \
  --accounts-account "o7RY6P2vQMuGSu1TrLM81weuzgDjaCRTXYRaXJwWcvc"
```

### Test Connection
```bash
cargo run -- --endpoint $RPC_GRPC --x-token $RPC_TOKEN ping
```

### Get Latest Blockhash
```bash
cargo run -- --endpoint $RPC_GRPC --x-token $RPC_TOKEN get-latest-blockhash
```

## Features
- Real-time transaction monitoring
- Account update tracking
- Automatic reconnection with exponential backoff
- Support for different commitment levels
