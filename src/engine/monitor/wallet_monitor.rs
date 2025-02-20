use {
    crate::{
        common::{logger::Logger, utils::AppState},
        dex::pump_fun::{Pump, PumpInfo, get_pump_info},
    },
    anyhow::{anyhow, Result},
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
    base64,
    bs58,
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
            if let OptionSerializer::Some(logs) = &meta.log_messages {
                if logs.iter().any(|log| log.contains(PUMP_PROGRAM_ID)) {
                    logger.success("Found PumpFun transaction!".to_string());
                    
                    // Extract transaction data
                    let program_data = extract_program_data(logs)?;
                    let trade_info = parse_trade_info(&program_data)?;
                    
                    // Log transaction details
                    logger.info(format!(
                        "\n   * [PUMP TRANSACTION FOUND] => \n   * [LOGS] => {:?}\n   * [TRADE INFO] => {:?}",
                        logs, trade_info
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
    let start_time = Instant::now();

    if let EncodedTransaction::Json(tx_data) = &transaction.transaction {
        if let Some(meta) = &transaction.meta {
            if let OptionSerializer::Some(logs) = &meta.log_messages {
                // Extract mint address and instruction type
                let (mint, is_buy) = extract_transaction_info(logs)?;
                
                logger.info(format!(
                    "\n   * [BUILD-IXN]({}) - {} :: {:?}",
                    mint, 
                    Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
                    start_time.elapsed()
                ));

                // Create Pump instance
                let pump = Pump::new(
                    state.rpc_nonblocking_client.clone(),
                    state.wallet.clone(),
                );

                // Get pump info
                match get_pump_info(state.rpc_client.clone(), &mint).await {
                    Ok(pump_info) => {
                        logger.info(format!(
                            "\n   * [SWAP-BEGIN]({}) - {} :: {:?}",
                            mint,
                            Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
                            start_time.elapsed()
                        ));

                        // Execute swap
                        match execute_swap(&pump, &mint, is_buy, &pump_info).await {
                            Ok(signature) => {
                                logger.success(format!(
                                    "\n   * [SUCCESSFUL-{}] => TX_HASH: (\"{}\") \n   * [POOL] => ({}) \n   * [COPIED] => {} :: ({:?}).",
                                    if is_buy { "BUY" } else { "SELL" },
                                    signature,
                                    mint,
                                    Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
                                    start_time.elapsed()
                                ));
                            }
                            Err(e) => {
                                logger.error(format!("Failed to execute swap: {}", e));
                            }
                        }
                    }
                    Err(e) => {
                        logger.error(format!(
                            "Skip {} by Failed to get bonding curve account data: {}", 
                            mint, e
                        ));
                    }
                }
            }
        }
    }

    Ok(())
}

// Helper functions
fn extract_program_data(logs: &[String]) -> Result<String> {
    for log in logs {
        if log.starts_with("Program data: ") {
            return Ok(log.trim_start_matches("Program data: ").to_string());
        }
    }
    Err(anyhow!("No program data found in logs"))
}

fn parse_trade_info(program_data: &str) -> Result<TradeInfo> {
    // TODO: Implement proper trade info parsing from program data
    Ok(TradeInfo {
        mint: String::new(),
        amount: 0,
        is_buy: false,
    })
}

fn extract_transaction_info(logs: &[String]) -> Result<(String, bool)> {
    let mut mint_address = String::new();
    let mut is_buy = false;
    
    for log in logs {
        // Extract mint address from program data
        if log.starts_with("Program data: ") {
            let data = log.trim_start_matches("Program data: ");
            if let Ok(decoded) = base64::decode(data) {
                // First 32 bytes contain the mint address
                if decoded.len() >= 32 {
                    let mint_bytes = &decoded[0..32];
                    mint_address = bs58::encode(mint_bytes).into_string();
                }
            }
        }
        
        // Determine if buy or sell
        if log.contains("Instruction: Buy") {
            is_buy = true;
        } else if log.contains("Instruction: Sell") {
            is_buy = false;
        }
    }

    if mint_address.is_empty() {
        return Err(anyhow!("Could not extract mint address from logs"));
    }

    Ok((mint_address, is_buy))
}

async fn execute_swap(pump: &Pump, mint: &str, is_buy: bool, pump_info: &PumpInfo) -> Result<String> {
    let amount = if is_buy {
        // Calculate buy amount based on virtual reserves
        pump_info.virtual_sol_reserves / 100 // Example: 1% of virtual reserves
    } else {
        // Calculate sell amount based on token balance
        let token_balance = pump.get_token_balance(mint).await?;
        token_balance / 2 // Example: Sell 50% of balance
    };

    let signature = if is_buy {
        pump.buy(mint, amount).await?
    } else {
        pump.sell(mint, amount).await?
    };

    Ok(signature)
}

#[derive(Debug)]
struct TradeInfo {
    mint: String,
    amount: u64,
    is_buy: bool,
} 