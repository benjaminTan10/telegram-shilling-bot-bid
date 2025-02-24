use {
    crate::{
        common::{logger::Logger, utils::AppState},
        dex::pump_fun::{Pump, get_pump_info, execute_swap, PUMP_PROGRAM_ID},
        proto::{InstantNodeClient, SubscribeRequest, SubscribeRequestFilterAccounts, CommitmentLevel},
    },
    anyhow::Result,
    tokio_stream::StreamExt,
    tonic::{transport::Channel, Request},
    std::time::Duration,
    solana_sdk::signature::Signer,
    bs58,
    base64_engine::{engine::general_purpose::STANDARD, Engine as _},
    chrono::Utc,
    std::collections::HashMap,
};

const TARGET_WALLET: &str = "o7RY6P2vQMuGSu1TrLM81weuzgDjaCRTXYRaXJwWcvc";

pub async fn monitor_transactions_grpc(
    grpc_url: &str,
    state: AppState,
) -> Result<()> {
    let logger = Logger::new("[GRPC-MONITOR]".to_string());
    
    // Connect to gRPC endpoint
    let channel = Channel::from_shared(grpc_url.to_string())?
        .connect()
        .await?;

    logger.info(format!(
        "\n[INIT] => [GRPC MONITOR ENVIRONMENT]: 
         [gRPC URL]: {},
         [Bot Wallet]: {},
         [Monitor Mode]: Real-time gRPC streaming
         ",
        grpc_url,
        state.wallet.pubkey(),
    ));

    // Create subscription request
    let mut accounts_filter = HashMap::new();
    accounts_filter.insert("pump_fun".to_string(), SubscribeRequestFilterAccounts {
        account: vec![TARGET_WALLET.to_string()],
        owner: vec![],
    });

    let request = Request::new(SubscribeRequest {
        accounts: accounts_filter,
        slots: HashMap::new(),
        transactions: HashMap::new(),
        commitment: Some(CommitmentLevel::Confirmed),
    });

    // Create InstantNode client with correct endpoint
    let mut client = InstantNodeClient::new(
        channel,
        "solana-grpc-geyser.instantnodes.io:443".to_string(),
    );

    // Subscribe to transaction updates
    let mut stream = client
        .subscribe_transactions(request)
        .await?
        .into_inner();

    // Process transaction stream
    while let Some(update) = stream.next().await {
        match update {
            Ok(tx_update) => {
                logger.info(format!(
                    "\n[NEW TRANSACTION] => Time: {}, Signature: {}", 
                    chrono::Utc::now(),
                    tx_update.signature
                ));

                // Process transaction logs
                if let Some(logs) = tx_update.logs {
                    if logs.iter().any(|log| log.contains(PUMP_PROGRAM_ID)) {
                        logger.success("Found PumpFun transaction!".to_string());

                        // Extract transaction data and execute copy trade
                        if let Ok((mint, is_buy)) = extract_transaction_info_from_logs(&logs) {
                            // Create Pump instance and execute swap
                            let pump = Pump::new(
                                state.rpc_nonblocking_client.clone(),
                                state.wallet.clone(),
                            );

                            if let Ok(pump_info) = get_pump_info(state.rpc_client.clone(), &mint).await {
                                match execute_swap(&pump, &mint, is_buy, &pump_info).await {
                                    Ok(signature) => {
                                        logger.success(format!("Successfully copied trade: {}", signature));
                                    }
                                    Err(e) => {
                                        logger.error(format!("Failed to copy trade: {}", e));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                logger.error(format!("Stream error: {}", e));
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }

    Ok(())
}

fn extract_transaction_info_from_logs(logs: &[String]) -> Result<(String, bool)> {
    for log in logs {
        if log.contains(PUMP_PROGRAM_ID) {
            if let Some(program_data) = log.strip_prefix("Program data: ") {
                if let Ok(decoded) = STANDARD.decode(program_data) {
                    // First 8 bytes are instruction discriminator
                    if decoded.len() >= 8 {
                        let discriminator = &decoded[0..8];
                        let discriminator_value = u64::from_le_bytes(discriminator.try_into().unwrap());
                        let is_buy = discriminator_value == 16927863322537952870; // PUMP_BUY_METHOD

                        // Extract mint address from instruction data
                        if decoded.len() >= 40 {
                            let mint_bytes = &decoded[8..40];
                            let mint_address = bs58::encode(mint_bytes).into_string();
                            return Ok((mint_address, is_buy));
                        }
                    }
                }
            }
        }
    }
    
    Err(anyhow::anyhow!("No valid PumpFun instruction found in logs"))
} 