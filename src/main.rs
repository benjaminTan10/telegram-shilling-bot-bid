use {
    dotenvy::dotenv,
    crate::{
        common::{
            logger::Logger,
            utils::{create_nonblocking_rpc_client, create_rpc_client, import_env_var, import_wallet, AppState},
        },
        engine::monitor::grpc_monitor::monitor_transactions_grpc,
    },
    anyhow::Result,
    solana_sdk::signature::Signer,
    std::sync::Arc,
    tonic::transport::Channel,
};

mod common;
mod dex;
mod engine;
mod services;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize environment
    dotenv().ok();
    
    // Setup logging
    let logger = Logger::new("[MAIN]".to_string());
    logger.info("Starting PumpFun sniper bot...".to_string());

    // Initialize clients and state
    let rpc_client = Arc::new(create_rpc_client()?);
    let rpc_nonblocking_client = Arc::new(create_nonblocking_rpc_client().await?);
    let wallet = import_wallet()?;
    
    logger.info(format!("Bot wallet: {}", wallet.pubkey()));

    let state = AppState {
        rpc_client: rpc_client.clone(),
        rpc_nonblocking_client: rpc_nonblocking_client.clone(),
        wallet: wallet.clone(),
    };

    // Get configuration from environment
    let slippage = import_env_var("SLIPPAGE").parse::<u64>().unwrap_or(5);
    let grpc_url = import_env_var("RPC_GRPC");
    
    logger.success("Bot initialization complete".to_string());
    logger.info("Starting gRPC transaction monitor...".to_string());

    // Create gRPC channel
    let channel = Channel::from_shared(grpc_url.clone())?
        .connect()
        .await?;

    // Start gRPC monitoring
    monitor_transactions_grpc(&grpc_url, state).await?;

    Ok(())
}
