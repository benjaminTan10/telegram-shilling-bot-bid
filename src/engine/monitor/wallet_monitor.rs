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
        option_serializer::OptionSerializer,
    },
    std::{str::FromStr, time::{Duration, Instant}},
    tokio::time,
    chrono::Utc,
};

const RETRY_DELAY: u64 = 5; // seconds
const MONITOR_INTERVAL: u64 = 2; // seconds
const TARGET_WALLET: &str = "o7RY6P2vQMuGSu1TrLM81weuzgDjaCRTXYRaXJwWcvc";
const PUMP_PROGRAM_ID: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";

pub async fn monitor_wallet(
    ws_url: &str,
    state: AppState,
    slippage: u64,
    use_jito: bool,
) -> Result<()> {
    let logger = Logger::new("[PUMPFUN-MONITOR]".to_string());
    let target_wallet = Pubkey::from_str(TARGET_WALLET)?;
    
    // Log initial configuration
    logger.info(format!("\n[INIT] =>  [SNIPER ENVIRONMENT]: 
         [Web Socket RPC]: {},
            
         * [Target Wallet]: {}, 
         * [Bot Wallet]: {}, * [Balance]: {} Sol,
            
         * [Slippage]: {}%, * [Use Jito]: {},
            
         * [Monitor Interval]: {}s, * [Retry Delay]: {}s
         ",
        ws_url,
        target_wallet.to_string(),
        state.wallet.pubkey(),
        state.rpc_client.get_balance(&state.wallet.pubkey())? as f64 / 1_000_000_000.0,
        slippage,
        use_jito,
        MONITOR_INTERVAL,
        RETRY_DELAY,
    ));

    logger.info("[STARTED. MONITORING]...".to_string());
    
    let mut interval = time::interval(Duration::from_secs(MONITOR_INTERVAL));
    let mut last_signature = None;

    loop {
        interval.tick().await;
        let start_time = Instant::now();
        
        // Log monitoring cycle
        logger.info(format!(
            "\n[MONITORING CYCLE] => Time: {}", 
            Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
        ));

        // Monitor transactions
        match monitor_transactions(&state, &target_wallet, last_signature).await {
            Ok((count, latest_sig)) => {
                if count > 0 {
                    logger.transaction(format!(
                        "Found {} new transactions from target wallet", 
                        count
                    ));
                    last_signature = latest_sig;
                }
            }
            Err(e) => {
                logger.error(format!("Error monitoring transactions: {}", e));
                time::sleep(Duration::from_secs(RETRY_DELAY)).await;
            }
        }

        // Monitor balances
        if let Ok(target_balance) = state.rpc_client.get_balance(&target_wallet) {
            if let Ok(bot_balance) = state.rpc_client.get_balance(&state.wallet.pubkey()) {
                logger.info(format!(
                    "[BALANCES] => Target: {} SOL, Bot: {} SOL",
                    target_balance as f64 / 1_000_000_000.0,
                    bot_balance as f64 / 1_000_000_000.0
                ));
            }
        }

        // Log cycle completion
        logger.info(format!(
            "[CYCLE COMPLETE] => Duration: {:?}\n",
            start_time.elapsed()
        ));
    }
}

async fn monitor_transactions(
    state: &AppState, 
    target_wallet: &Pubkey,
    last_sig: Option<Signature>
) -> Result<(u64, Option<Signature>)> {
    let logger = Logger::new("[TX MONITOR]".to_string());
    let start_time = Instant::now();
    
    let config = RpcTransactionConfig {
        encoding: Some(UiTransactionEncoding::Json),
        commitment: Some(CommitmentConfig::confirmed()),
        max_supported_transaction_version: Some(0),
    };

    let signatures = state.rpc_client.get_signatures_for_address(target_wallet)?;
    let mut tx_count = 0;
    let mut latest_signature = None;

    for sig in signatures.iter().take(5) {
        if sig.err.is_none() {
            let signature = Signature::from_str(&sig.signature)?;
            
            // Skip if we've seen this transaction
            if let Some(last_seen) = last_sig {
                if signature == last_seen {
                    break;
                }
            }

            // Update latest signature
            if latest_signature.is_none() {
                latest_signature = Some(signature);
            }

            // Log new transaction detection
            logger.info(format!(
                "\n   * [NEW TX] => (\"{}\") - SLOT:({}) \n   * [FROM] => ({}) \n   * [TIME] => {} :: ({:?}).",
                sig.signature,
                sig.slot,
                target_wallet.to_string(),
                Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
                start_time.elapsed()
            ));

            if let Ok(tx_response) = state.rpc_client.get_transaction_with_config(&signature, config.clone()) {
                tx_count += 1;
                
                // Process transaction
                if let Ok(()) = process_transaction(&state, &tx_response.transaction).await {
                    logger.success(format!(
                        "\n   * [COPYING TX] => Hash: (\"{}\") \n   * [SLOT] => ({}) \n   * [TIME] => {} :: ({:?}).",
                        sig.signature,
                        tx_response.slot,
                        Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
                        start_time.elapsed()
                    ));

                    // Copy the transaction
                    if let Err(e) = copy_transaction(&state, &tx_response.transaction).await {
                        logger.error(format!("Failed to copy transaction: {}", e));
                    }
                }
            }
        }
    }

    Ok((tx_count, latest_signature))
}

async fn process_transaction(state: &AppState, transaction: &EncodedTransactionWithStatusMeta) -> Result<()> {
    let logger = Logger::new("[PROCESS TX]".to_string());
    
    if let EncodedTransaction::Json(tx_data) = &transaction.transaction {
        if let Some(meta) = &transaction.meta {
            // Check if transaction involves PumpFun program
            if let OptionSerializer::Some(logs) = &meta.log_messages {
                if logs.iter().any(|log| log.contains(PUMP_PROGRAM_ID)) {
                    logger.success("Found PumpFun transaction!".to_string());
                    
                    // Log transaction details
                    logger.info(format!(
                        "\n   * [PUMP TRANSACTION FOUND] => \n   * [LOGS] => {:?}",
                        logs
                    ));
                    
                    return Ok(());
                }
            }
        }
    }
    
    Ok(())
}

async fn copy_transaction(state: &AppState, transaction: &EncodedTransactionWithStatusMeta) -> Result<()> {
    let logger = Logger::new("[COPY TX]".to_string());
    
    // TODO: Implement transaction copying logic
    // 1. Extract transaction instructions
    // 2. Modify instructions for bot wallet
    // 3. Build and send new transaction
    
    logger.info("Transaction copying not yet implemented".to_string());
    Ok(())
} 