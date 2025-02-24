use {
    crate::{
        common::{logger::Logger, utils::AppState},
        dex::pump_fun::{Pump, get_pump_info, PUMP_PROGRAM_ID},
        proto::{GeyserClient, SubscribeRequest, TransactionStatus, TransactionUpdate},
    },
    anyhow::{anyhow, Result},
    tokio_stream::StreamExt,
    tonic::{transport::Channel, Request},
    std::time::Duration,
    solana_sdk::pubkey::Pubkey,
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

    // Create gRPC client
    let mut client = geyser_client::GeyserClient::new(channel);

    // Subscribe to transaction updates
    let request = Request::new(SubscribeRequest {
        accounts: vec![TARGET_WALLET.to_string()],
        transaction_details: true,
        show_events: true,
    });

    let mut stream = client
        .subscribe(request)
        .await?
        .into_inner();

    // Process transaction stream
    while let Some(update) = stream.next().await {
        match update {
            Ok(tx_update) => {
                logger.info(format!(
                    "\n[NEW TRANSACTION] => Time: {}", 
                    chrono::Utc::now()
                ));

                // Process transaction
                if let Err(e) = process_transaction_update(&state, tx_update).await {
                    logger.error(format!("Error processing transaction: {}", e));
                }
            }
            Err(e) => {
                logger.error(format!("Stream error: {}", e));
                
                // Reconnect after error
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }

    Ok(())
}

async fn process_transaction_update(
    state: &AppState,
    update: TransactionUpdate,
) -> Result<()> {
    let logger = Logger::new("[PROCESS TX]".to_string());

    // Check if transaction is successful
    if update.status != TransactionStatus::Confirmed {
        return Ok(());
    }

    // Check if transaction involves PumpFun program
    if !update.instructions.iter().any(|ix| ix.program_id == PUMP_PROGRAM_ID) {
        return Ok(());
    }

    logger.success("Found PumpFun transaction!".to_string());

    // Extract transaction data and execute copy trade
    let (mint, is_buy) = extract_transaction_info_grpc(&update)?;
    
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

    Ok(())
}

fn extract_transaction_info_grpc(update: &TransactionUpdate) -> Result<(String, bool)> {
    // Extract mint address and trade direction from transaction data
    // Implementation will depend on exact gRPC message format
    // ...
} 