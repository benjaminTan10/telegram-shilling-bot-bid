use tonic::transport::Channel;
use solana_sdk::pubkey::Pubkey;

#[derive(Debug)]
pub struct SubscribeRequest {
    pub accounts: Vec<String>,
    pub transaction_details: bool,
    pub show_events: bool,
}

#[derive(Debug)]
pub struct TransactionUpdate {
    pub status: TransactionStatus,
    pub instructions: Vec<InstructionInfo>,
}

#[derive(Debug, PartialEq)]
pub enum TransactionStatus {
    Confirmed,
    Failed,
    Pending,
}

#[derive(Debug)]
pub struct InstructionInfo {
    pub program_id: String,
    pub data: Vec<u8>,
}

pub struct GeyserClient {
    channel: Channel,
}

impl GeyserClient {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
} 