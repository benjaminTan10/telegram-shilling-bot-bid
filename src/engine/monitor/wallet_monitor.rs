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
    solana_account_decoder::UiAccountEncoding,
    std::{str::FromStr, time::{Duration, Instant}},
    tokio::time,
    chrono::Utc,
    base64,
    bs58,
    futures_util::stream::StreamExt,
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
            
         * [Monitor Mode]: Real-time WebSocket streaming
         ",
        ws_url,
        target_wallet.to_string(),
        state.wallet.pubkey(),
        state.rpc_client.get_balance(&state.wallet.pubkey())? as f64 / 1_000_000_000.0,
        slippage,
        use_jito,
    ));

    logger.info("[STARTED. MONITORING]...".to_string());

    // Create WebSocket client
    let ws_client = solana_client::nonblocking::pubsub_client::PubsubClient::new(ws_url).await?;

    // Subscribe to account notifications for the target wallet
    let (mut notifications, unsubscribe) = ws_client.account_subscribe(
        &target_wallet,
        Some(solana_client::rpc_config::RpcAccountInfoConfig {
            encoding: Some(solana_client::rpc_config::UiAccountEncoding::Base64),
            commitment: Some(CommitmentConfig::confirmed()),
            data_slice: None,
            min_context_slot: None,
        }),
    ).await?;

    // Handle cleanup on drop
    let _cleanup = scopeguard::guard(unsubscribe, |unsub| {
        tokio::spawn(async move {
            let _ = unsub().await;
        });
    });

    // Process notifications in real-time
    while let Some(notification) = notifications.next().await {
        match notification {
            Ok(update) => {
                logger.info(format!(
                    "\n[NEW ACCOUNT UPDATE] => Time: {}", 
                    Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
                ));

                // Get recent transactions since the update
                match state.rpc_client.get_signatures_for_address(&target_wallet) {
                    Ok(signatures) => {
                        for sig in signatures.iter().take(5) {
                            if sig.err.is_none() {
                                let signature = Signature::from_str(&sig.signature)?;
                                
                                if let Ok(tx_response) = state.rpc_client.get_transaction_with_config(
                                    &signature,
                                    RpcTransactionConfig {
                                        encoding: Some(UiTransactionEncoding::Json),
                                        commitment: Some(CommitmentConfig::confirmed()),
                                        max_supported_transaction_version: Some(0),
                                    },
                                ) {
                                    // Process the transaction
                                    if let Err(e) = process_transaction(&state, &tx_response.transaction).await {
                                        logger.error(format!("Error processing transaction: {}", e));
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        logger.error(format!("Error getting signatures: {}", e));
                    }
                }
            }
            Err(e) => {
                logger.error(format!("WebSocket error: {}", e));
            }
        }
    }

    Ok(())
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

                        logger.info(format!(
                            "\n   * [PUMP-INFO] => {:?}",
                            pump_info
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
        // Extract and analyze program data
        if log.starts_with("Program data: ") {
            let data = log.trim_start_matches("Program data: ");
            if let Ok(decoded) = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, data) {
                println!("Decoded data length: {}", decoded.len());
                
                // First 8 bytes are instruction discriminator
                if decoded.len() >= 8 {
                    let discriminator = &decoded[0..8];
                    let discriminator_value = u64::from_le_bytes(discriminator.try_into().unwrap());
                    println!("Instruction discriminator: {}", discriminator_value);

                    // Match against known method discriminators
                    is_buy = match discriminator_value {
                        16927863322537952870 => true,  // PUMP_BUY_METHOD
                        12502976635542562355 => false, // PUMP_SELL_METHOD
                        _ => {
                            println!("Unknown instruction discriminator");
                            false
                        }
                    };
                }

                // Next 32 bytes after instruction data should be the mint address
                if decoded.len() >= 40 {  // 8 (discriminator) + 32 (mint)
                    let mint_bytes = &decoded[8..40];
                    mint_address = bs58::encode(mint_bytes).into_string();
                    println!("Extracted mint address: {}", mint_address);
                }

                // Analyze remaining data (amount, etc)
                if decoded.len() > 40 {
                    let amount_bytes = &decoded[40..48]; // Next 8 bytes for amount
                    if amount_bytes.len() == 8 {
                        let amount = u64::from_le_bytes(amount_bytes.try_into().unwrap());
                        println!("Transaction amount: {}", amount);
                    }
                }
            } else {
                println!("Failed to decode base64 data: {}", data);
            }
        }
        
        // Additional log analysis for confirmation
        if log.contains("Program log: Instruction: Buy") {
            println!("Confirmed Buy instruction from logs");
            is_buy = true;
        } else if log.contains("Program log: Instruction: Sell") {
            println!("Confirmed Sell instruction from logs");
            is_buy = false;
        }
    }

    if mint_address.is_empty() {
        return Err(anyhow!("Could not extract mint address from logs"));
    }

    // Validate the extracted mint address
    if let Err(_) = Pubkey::from_str(&mint_address) {
        return Err(anyhow!("Invalid mint address format"));
    }

    println!("Final extracted info - Mint: {}, Is Buy: {}", mint_address, is_buy);
    Ok((mint_address, is_buy))
}

async fn execute_swap(pump: &Pump, mint: &str, is_buy: bool, pump_info: &PumpInfo) -> Result<String> {
    let logger = Logger::new("[EXECUTE SWAP]".to_string());
    
    // Calculate copy amount (50% of virtual reserves)
    let amount = if is_buy {
        // For buys: 50% of virtual SOL reserves
        let copy_amount = (pump_info.virtual_sol_reserves as f64 * 0.5) as u64;
        logger.info(format!(
            "Attempting buy with 50% - Amount: {} SOL (from {} total virtual reserves)",
            copy_amount as f64 / 1_000_000_000.0,
            pump_info.virtual_sol_reserves as f64 / 1_000_000_000.0
        ));

        // Check wallet SOL balance
        if let Ok(wallet_balance) = pump.client.get_balance(&pump.keypair.pubkey()).await {
            logger.info(format!(
                "Current wallet SOL balance: {} SOL",
                wallet_balance as f64 / 1_000_000_000.0
            ));
        }

        copy_amount
    } else {
        // For sells: First check current token balance
        let token_balance = pump.get_token_balance(mint).await?;
        logger.info(format!(
            "Current token balance before sell: {} tokens",
            token_balance
        ));

        // For sells: 50% of token balance if we have any
        let copy_amount = (token_balance as f64 * 0.5) as u64;
        logger.info(format!(
            "Attempting sell with 50% - Amount: {} tokens (from {} total balance)",
            copy_amount,
            token_balance
        ));

        // Additional info about virtual reserves
        logger.info(format!(
            "Virtual reserves - SOL: {} SOL, Tokens: {} tokens",
            pump_info.virtual_sol_reserves as f64 / 1_000_000_000.0,
            pump_info.virtual_token_reserves
        ));

        copy_amount
    };

    // Execute the swap
    let signature = if is_buy {
        logger.info(format!("Executing buy for {} SOL", amount as f64 / 1_000_000_000.0));
        pump.buy(mint, amount).await
    } else {
        if amount == 0 {
            logger.error("Cannot execute sell - No tokens available in wallet".to_string());
            return Err(anyhow!("No tokens available to sell"));
        }
        logger.info(format!("Executing sell for {} tokens", amount));
        pump.sell(mint, amount).await
    };

    match &signature {
        Ok(sig) => logger.success(format!("Swap executed successfully: {}", sig)),
        Err(e) => logger.error(format!("Swap failed: {}", e)),
    }

    signature
}

#[derive(Debug)]
struct TradeInfo {
    mint: String,
    amount: u64,
    is_buy: bool,
} 