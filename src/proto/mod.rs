mod instantnode;

pub use instantnode::{
    InstantNodeClient,
    SubscribeRequest,
    TransactionUpdate,
};

// Re-export common types
pub use tonic::{transport::Channel, Request, Response};
pub use futures_util::Stream;
pub use anyhow::Result;

// Remove duplicate type definitions since they're now in instantnode.rs 