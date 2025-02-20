use {
    crate::common::{logger::Logger, utils::AppState},
    anyhow::Result,
    solana_client::rpc_config::RpcTransactionConfig,
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::{Signature, Signer},
    },
    solana_transaction_status::{
        EncodedTransaction, 
        UiTransactionEncoding,
        EncodedTransactionWithStatusMeta,
    },
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
                time::sleep(Duration::from_secs(5)).await;
            }
        }

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
    
    let config = RpcTransactionConfig {
        encoding: Some(UiTransactionEncoding::Json),
        commitment: Some(CommitmentConfig::confirmed()),
        max_supported_transaction_version: Some(0),
    };

    let signatures = state.rpc_client
        .get_signatures_for_address(target_wallet)?;

    let mut tx_count = 0;

    for sig in signatures.iter().take(5) {
        if sig.err.is_none() {
            if let Ok(signature) = Signature::from_str(&sig.signature) {
                if let Ok(tx_response) = state.rpc_client.get_transaction_with_config(
                    &signature,
                    config.clone(),
                ) {
                    tx_count += 1;
                    logger.success(format!(
                        "Found successful transaction: {} | Slot: {}",
                        sig.signature,
                        tx_response.slot,
                    ));

                    match process_transaction(&state, tx_response.transaction).await {
                        Ok(_) => logger.success("Successfully processed transaction".to_string()),
                        Err(e) => logger.error(format!("Failed to process transaction: {}", e)),
                    }
                }
            }
        }
    }

    Ok(tx_count)
}

async fn process_transaction(
    state: &AppState, 
    transaction: EncodedTransactionWithStatusMeta
) -> Result<()> {
    let logger = Logger::new("[PROCESS TX]".to_string());
    
    // Extract transaction data based on encoding
    match transaction.transaction {
        EncodedTransaction::Json(tx_data) => {
            if let Some(message) = tx_data.message {
                // Check if it's a PumpFun transaction
                if message.account_keys.iter().any(|key| 
                    key.pubkey == "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P"
                ) {
                    logger.success("Found PumpFun transaction!".to_string());
                    // TODO: Extract and copy transaction parameters
                }
            }
        }
        _ => {
            logger.warning("Unsupported transaction encoding".to_string());
        }
    }
    
    Ok(())
} 