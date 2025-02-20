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
        UiInstruction,
        UiTransactionStatusMeta,
    },
    std::{str::FromStr, time::{Duration, Instant}},
    tokio::time,
    chrono::Utc,
};

const RETRY_DELAY: u64 = 5; // seconds
const MONITOR_INTERVAL: u64 = 2; // seconds
const MAX_RETRIES: u32 = 3;
const TARGET_WALLET: &str = "o7RY6P2vQMuGSu1TrLM81weuzgDjaCRTXYRaXJwWcvc";

pub async fn monitor_wallet(
    ws_url: &str,
    state: AppState,
    slippage: u64,
    use_jito: bool,
) -> Result<()> {
    let logger = Logger::new("[PUMPFUN-MONITOR]".to_string());
    let target_wallet = Pubkey::from_str(TARGET_WALLET)?;
    
    // Initialize monitoring
    logger.info(format!("[INIT] =>  [SNIPER ENVIRONMENT]: 
         [Web Socket RPC]: {},
            
         * [Wallet]: {}, * [Balance]: {} Sol, 
            
         * [Slippage]: {}, * [Use Jito]: {},
            
         * [Time Exceed]: {}, * [Retries]: {},
            ",
        ws_url,
        state.wallet.pubkey(),
        state.rpc_client.get_balance(&state.wallet.pubkey())? as f64 / 1_000_000_000.0,
        slippage,
        use_jito,
        RETRY_DELAY,
        MAX_RETRIES,
    ));

    logger.info("[STARTED. MONITORING]...".to_string());
    
    let mut interval = time::interval(Duration::from_secs(MONITOR_INTERVAL));

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
                time::sleep(Duration::from_secs(RETRY_DELAY)).await;
            }
        }
    }
}

async fn monitor_transactions(state: &AppState, target_wallet: &Pubkey) -> Result<u64> {
    let logger = Logger::new("[TX MONITOR]".to_string());
    let start_time = Instant::now();
    
    let config = RpcTransactionConfig {
        encoding: Some(UiTransactionEncoding::Json),
        commitment: Some(CommitmentConfig::confirmed()),
        max_supported_transaction_version: Some(0),
    };

    let signatures = state.rpc_client.get_signatures_for_address(target_wallet)?;
    let mut tx_count = 0;

    for sig in signatures.iter().take(5) {
        if sig.err.is_none() {
            let signature = Signature::from_str(&sig.signature)?;
            
            // Log new transaction detection
            logger.info(format!(
                "\n   * [NEW POOL|BUY] => (\"{}\") - SLOT:({}) \n   * [DETECT] => ({}) \n   * [BUYING] => {} :: ({:?}).",
                sig.signature,
                sig.slot,
                target_wallet.to_string(),
                Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
                start_time.elapsed()
            ));

            match state.rpc_client.get_transaction_with_config(&signature, config.clone()) {
                Ok(tx_response) => {
                    tx_count += 1;
                    
                    // Process successful transaction
                    match process_transaction(&state, tx_response.transaction).await {
                        Ok(_) => {
                            logger.success(format!(
                                "\n   * [SUCCESSFUL-BUY] => TX_HASH: (\"{}\") \n   * [SLOT] => ({}) \n   * [TIME] => {} :: ({:?}).",
                                sig.signature,
                                tx_response.slot,
                                Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
                                start_time.elapsed()
                            ));
                        }
                        Err(e) => {
                            logger.error(format!("Skip {} by {}", target_wallet, e));
                        }
                    }
                }
                Err(e) => {
                    logger.error(format!("Failed to get transaction {}: {}", sig.signature, e));
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
            let message = tx_data.message;
            let pump_program_id = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
            
            // Get the accounts from the message
            let accounts = message.account_keys
                .iter()
                .map(|key| key.pubkey.clone())
                .collect::<Vec<String>>();

            // Check if PumpFun program is involved
            if accounts.iter().any(|acc| acc == pump_program_id) {
                logger.success("Found PumpFun transaction!".to_string());
                
                if let Some(meta) = transaction.meta {
                    let instructions = meta.inner_instructions.unwrap_or_default();
                    logger.info(format!(
                        "Instructions count: {}", 
                        instructions.len()
                    ));

                    // Process each instruction set
                    for (idx, inner_ix_set) in instructions.iter().enumerate() {
                        logger.info(format!("Processing instruction set {}", idx));

                        for (inner_idx, instruction) in inner_ix_set.instructions.iter().enumerate() {
                            match instruction {
                                UiInstruction::Parsed(parsed_ix) => {
                                    logger.info(format!(
                                        "Instruction {}.{}: {:?}", 
                                        idx, inner_idx, parsed_ix
                                    ));
                                }
                                UiInstruction::Compiled(compiled_ix) => {
                                    logger.info(format!(
                                        "Instruction {}.{}: Program: {}, Data: {:?}", 
                                        idx, inner_idx,
                                        accounts[compiled_ix.program_id_index as usize],
                                        compiled_ix.data
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {
            logger.warning("Unsupported transaction encoding".to_string());
        }
    }
    
    Ok(())
} 