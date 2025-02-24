use {
    tonic::{transport::Channel, Request, Response, Status},
    futures_util::Stream,
    anyhow::Result,
    std::{pin::Pin, collections::HashMap},
    async_stream::stream,
    std::str::FromStr,
};

#[derive(Debug)]
pub struct InstantNodeClient {
    endpoint: String,
    channel: Channel,
}

#[derive(Debug)]
pub struct SubscribeRequest {
    pub accounts: HashMap<String, SubscribeRequestFilterAccounts>,
    pub slots: HashMap<String, SubscribeRequestFilterSlots>,
    pub transactions: HashMap<String, SubscribeRequestFilterTransactions>,
    pub commitment: Option<CommitmentLevel>,
}

#[derive(Debug)]
pub struct SubscribeRequestFilterAccounts {
    pub account: Vec<String>,
    pub owner: Vec<String>,
}

#[derive(Debug)]
pub struct SubscribeRequestFilterSlots {
    pub filter_by_commitment: Option<bool>,
}

#[derive(Debug)]
pub struct SubscribeRequestFilterTransactions {
    pub vote: Option<bool>,
    pub failed: Option<bool>,
    pub account_include: Vec<String>,
}

#[derive(Debug)]
pub enum CommitmentLevel {
    Processed = 0,
    Confirmed = 1,
    Finalized = 2,
}

#[derive(Debug)]
pub struct TransactionUpdate {
    pub signature: String,
    pub slot: u64,
    pub err: Option<String>,
    pub logs: Option<Vec<String>>,
    pub accounts: Vec<String>,
    pub timestamp: i64,
}

impl InstantNodeClient {
    pub fn new(channel: Channel, endpoint: String) -> Self {
        Self { channel, endpoint }
    }

    pub async fn subscribe_transactions(
        &mut self,
        mut request: Request<SubscribeRequest>,
    ) -> Result<Response<Pin<Box<dyn Stream<Item = Result<TransactionUpdate, Status>> + Send + 'static>>>> {
        // Add authentication header
        request.metadata_mut().insert(
            "x-token",
            tonic::metadata::MetadataValue::from_static(&std::env::var("RPC_TOKEN")?)
        );
        
        // Create streaming response
        let stream = Box::pin(stream! {
            loop {
                match self.get_next_transaction().await {
                    Ok(update) => yield Ok(update),
                    Err(e) => {
                        yield Err(Status::internal(format!("Stream error: {}", e)));
                        break;
                    }
                }
            }
        });

        Ok(Response::new(stream))
    }

    async fn get_next_transaction(&self) -> Result<TransactionUpdate> {
        // TODO: Implement actual gRPC call to get next transaction
        // This is where you would make the actual gRPC call to InstantNode
        
        // For now, return a placeholder error
        Err(anyhow::anyhow!("gRPC subscription not yet implemented"))
    }
}

impl From<SubscribeRequest> for tonic::Request<SubscribeRequest> {
    fn from(req: SubscribeRequest) -> Self {
        tonic::Request::new(req)
    }
} 