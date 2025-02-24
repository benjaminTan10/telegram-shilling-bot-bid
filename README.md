## Overview
A high-performance copy trading bot for PumpFun DEX on Solana, written in Rust 🦀. The bot monitors specific wallets and automatically replicates their trading activities with configurable parameters and advanced features like Jito MEV integration.

![image](https://github.com/user-attachments/assets/028dabb0-6f34-404e-9495-3fdf94835104)

# Solana PumpFun Copy Trading Bot in Rust 🚀

## Key Features

### 🚀 Performance & Architecture
- **Rust-Powered Performance**: Built with Rust for optimal speed and memory safety
- **Dual Monitoring Modes**: 
  - gRPC streaming via Yellowstone/InstantNodes
  - WebSocket-based wallet monitoring
- **Asynchronous Architecture**: Using Tokio for non-blocking operations

### 🔒 Security & Configuration
- **Environment-Based Setup**: Secure configuration via `.env` file
- **Robust Error Handling**: Comprehensive error management and logging
- **Configurable Parameters**: Customizable slippage, amounts, and monitoring settings

### 📊 Trading Features
- **Smart Copy Trading**: 
  - Automatic trade detection and replication
  - Configurable trade size (default: 50% of detected amount)
  - Support for both buy and sell operations
- **PumpFun DEX Integration**: 
  - Direct interaction with PumpFun bonding curves
  - Automatic token account creation and management
- **Jito MEV Integration**: Enhanced transaction priority

## Directory Structure

```
src/
├── common/                 # Common utilities and shared components
│   ├── logger.rs          # Logging system with colored output
│   └── utils.rs           # Configuration and utility functions
├── dex/                   # DEX integration components
│   └── pump_fun.rs        # PumpFun DEX interaction logic
├── engine/                # Core trading engine
│   └── monitor/           # Transaction monitoring systems
│       ├── grpc_monitor.rs    # gRPC-based monitoring
│       └── wallet_monitor.rs  # WebSocket-based monitoring
├── services/              # External service integrations
│   └── jito.rs           # Jito MEV service integration
└── proto/                 # Protocol definitions
    └── instantnode.rs     # InstantNode gRPC client implementation
```

## Environment Variables

```env
# Required Configuration
PRIVATE_KEY=<your_base58_private_key>
RPC_HTTPS=<your_rpc_endpoint>
RPC_WSS=<your_websocket_endpoint>
RPC_GRPC=<your_grpc_endpoint>
RPC_TOKEN=<your_rpc_auth_token>

# Optional Configuration
SLIPPAGE=10               # Slippage tolerance in percentage
LOG_LEVEL=debug          # Logging level (debug/info/error)
```

## Usage

1. **Installation**
   ```bash
   git clone <repository_url>
   cd solana-pumpfun-bot
   cargo build --release
   ```

2. **Configuration**
   - Copy `.env.example` to `.env`
   - Configure your environment variables

3. **Running the Bot**
   ```bash
   cargo run --release
   ```

### Monitoring Modes

#### gRPC Monitoring
```bash
# Monitor PumpFun transactions
cargo run -- --endpoint $RPC_GRPC --x-token $RPC_TOKEN subscribe \
  --transactions \
  --transactions-vote false \
  --transactions-failed false \
  --transactions-account-include "o7RY6P2vQMuGSu1TrLM81weuzgDjaCRTXYRaXJwWcvc"
```

#### WebSocket Monitoring
```bash
# Monitor wallet updates
cargo run -- --ws-url $RPC_WSS monitor-wallet
```

## Technical Details

### Trading Logic
- The bot monitors specified wallets for PumpFun DEX interactions
- Upon detecting a trade:
  1. Extracts transaction details (mint, amount, direction)
  2. Validates the trading parameters
  3. Executes a copy trade with configured parameters
  - For buys: Uses 50% of virtual SOL reserves
  - For sells: Uses 50% of available token balance

### Safety Features
- Transaction validation and simulation
- Automatic token account creation
- Balance checks before execution
- Comprehensive error handling and logging

### Logging System
- Colored output for different message types
- Transaction counting and tracking
- Detailed timing information
- Multiple log levels (DEBUG, INFO, ERROR, SUCCESS, WARNING)

## Support

For support and inquiries, please connect via Telegram: 📞 [Benjamin](https://t.me/blockchainDeveloper_Ben)

## License

MIT License
