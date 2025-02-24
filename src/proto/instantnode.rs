use {
    tonic::{transport::Channel, Request, Response, Status},
    futures_util::Stream,
    anyhow::Result,
    std::pin::Pin,
    async_stream::stream,
};

#[derive(Debug)]
pub struct InstantNodeClient {
    endpoint: String,
    channel: Channel,
}

#[derive(Debug)]
pub struct SubscribeRequest {
    pub accounts: Vec<String>,
    pub include_transactions: bool,
    pub include_accounts: bool,
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
        request: Request<SubscribeRequest>,
    ) -> Result<Response<Pin<Box<dyn Stream<Item = Result<TransactionUpdate, Status>> + Send + 'static>>>> {
        // Create subscription URL
        let subscription_url = format!("{}/subscribe_transactions", self.endpoint);
        
        // Set up subscription parameters
        let request = tonic::Request::new(request.into_inner());
        
        // Create streaming response
        let stream = Box::pin(stream! {
            loop {
                // Make gRPC call to get next transaction
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