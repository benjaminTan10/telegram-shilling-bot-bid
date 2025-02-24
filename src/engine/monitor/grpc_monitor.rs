use {
    crate::{
        common::{logger::Logger, utils::AppState},
        dex::pump_fun::{Pump, get_pump_info, execute_swap, PUMP_PROGRAM_ID},
    },
    anyhow::Result,
    std::time::Duration,
    yellowstone_grpc_client::GeyserGrpcClient,
    yellowstone_grpc_proto::{
        prelude::{
            subscribe_update::UpdateOneof,
            CommitmentLevel,
            SubscribeRequest,
            SubscribeRequestFilterTransactions,
        },
        geyser::geyser_client::GeyserClient,
    },
    base64::Engine as _,
    bs58,
    chrono::Utc,
    solana_sdk::signature::Signer,
    tonic::{
        transport::{Channel, ClientTlsConfig},
        metadata::MetadataValue,
        Response,
    },
    tonic_health::proto::health_client::HealthClient,
};

const TARGET_WALLET: &str = "o7RY6P2vQMuGSu1TrLM81weuzgDjaCRTXYRaXJwWcvc";

pub async fn monitor_transactions_grpc(
    grpc_url: &str,
    state: AppState,
) -> Result<()> {
    let logger = Logger::new("[GRPC-MONITOR]".to_string());
    
    // Create gRPC channel
    let channel = Channel::from_shared(grpc_url.to_string())?
        .connect()
        .await?;

    // Create health and geyser clients
    let health_client = HealthClient::new(channel.clone());
    let geyser_client = GeyserClient::new(channel);

    // Create gRPC client
    let mut client = GeyserGrpcClient::new(health_client, geyser_client);

    // Add auth token
    let token = MetadataValue::try_from(std::env::var("RPC_TOKEN")?)?;
    client.add_headers(vec![("x-token", token)])?;

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
    let request = SubscribeRequest {
        transactions: maplit::hashmap! {
            "pump_fun".to_string() => SubscribeRequestFilterTransactions {
                vote: Some(false),
                failed: Some(false),
                signature: None,
                account_include: vec![TARGET_WALLET.to_string()],
                account_exclude: vec![],
                account_required: vec![],
            }
        },
        commitment: Some(CommitmentLevel::Confirmed as i32),
        ..Default::default()
    };

    // Subscribe to updates
    let (mut subscribe_tx, mut stream) = client.subscribe().await?;
    subscribe_tx.send(request).await?;

    // Process updates
    while let Some(message) = stream.next().await {
        match message {
            Ok(msg) => {
                if let Some(UpdateOneof::Transaction(tx)) = msg.update_oneof {
                    // Get signature from transaction data
                    let signature = if let Some(tx_data) = &tx.transaction {
                        tx_data.signature.clone()
                    } else {
                        continue;
                    };

                    logger.info(format!(
                        "\n[NEW TRANSACTION] => Time: {}, Signature: {}", 
                        Utc::now(),
                        bs58::encode(&signature).into_string()
                    ));

                    // Process transaction logs
                    if let Some(logs) = tx.transaction.and_then(|t| t.meta.logs) {
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
                if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(program_data) {
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