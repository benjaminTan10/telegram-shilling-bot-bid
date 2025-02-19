use {
    crate::common::{logger::Logger, utils::AppState},
    anyhow::Result,
    solana_client::rpc_config::{RpcTransactionConfig, RpcProgramAccountsConfig},
    solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey},
    std::{str::FromStr, time::Duration},
    tokio::time,
};

pub async fn monitor_wallet(
    ws_url: &str,
    state: AppState,
    slippage: u64,
    use_jito: bool,
) -> Result<()> {
    let logger = Logger::new("[WALLET MONITOR]".to_string());
    logger.info("Initializing wallet monitor...".to_string());

    // Target wallet to monitor
    let target_wallet = Pubkey::from_str("o7RY6P2vQMuGSu1TrLM81weuzgDjaCRTXYRaXJwWcvc")?;
    logger.info(format!("Target wallet set to: {}", target_wallet));

    // Configuration info
    logger.info(format!("Slippage: {}%", slippage));
    logger.info(format!("Jito MEV protection: {}", use_jito));
    logger.info(format!("RPC URL: {}", ws_url));
    logger.info(format!("Monitoring wallet: {}", state.wallet.pubkey()));

    // Start monitoring loop
    logger.success("Monitoring started successfully".to_string());
    
    let mut interval = time::interval(Duration::from_secs(2));

    loop {
        interval.tick().await;
        
        match monitor_transactions(&state, &target_wallet).await {
            Ok(count) => {
                if count > 0 {
                    logger.transaction(format!("Found {} new transactions", count));
                }
            }
            Err(e) => {
                logger.error(format!("Error monitoring transactions: {}", e));
                // Wait before retrying
                time::sleep(Duration::from_secs(5)).await;
            }
        }

        // Monitor account balance changes
        match state.rpc_client.get_balance(&target_wallet) {
            Ok(balance) => {
                logger.debug(format!(
                    "Target wallet balance: {} SOL",
                    balance as f64 / 1_000_000_000.0
                ));
            }
            Err(e) => {
                logger.error(format!("Failed to get wallet balance: {}", e));
            }
        }
    }
}

async fn monitor_transactions(state: &AppState, target_wallet: &Pubkey) -> Result<u64> {
    let logger = Logger::new("[TX MONITOR]".to_string());
    
    // Get recent transactions
    let config = RpcTransactionConfig {
        encoding: None,
        commitment: Some(CommitmentConfig::confirmed()),
        max_supported_transaction_version: Some(0),
    };

    let signatures = state.rpc_client
        .get_signatures_for_address(target_wallet)?;

    let mut tx_count = 0;

    for sig in signatures.iter().take(5) {
        if let Ok(tx) = state.rpc_client.get_transaction(
            &sig.signature,
            config.commitment.unwrap(),
        ) {
            tx_count += 1;
            logger.transaction(format!(
                "Transaction: {} | Slot: {} | Status: {}",
                sig.signature,
                tx.slot,
                if sig.err.is_none() { "Success" } else { "Failed" }
            ));
        }
    }

    Ok(tx_count)
} 