use {
    dotenvy::dotenv,
    crate::{
        common::{
            logger::Logger,
            utils::{create_nonblocking_rpc_client, create_rpc_client, import_env_var, import_wallet, AppState},
        },
        engine::monitor::wallet_monitor::monitor_wallet,
    },
    anyhow::Result,
    solana_sdk::signature::Signer,
    std::sync::Arc,
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
    let ws_url = import_env_var("RPC_WSS");
    
    logger.success("Bot initialization complete".to_string());
    logger.info("Starting wallet monitor...".to_string());

    // Start monitoring
    monitor_wallet(&ws_url, state, slippage, true).await?;

    Ok(())
}
