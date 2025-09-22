use thiserror::Error;

pub type Result<T> = std::result::Result<T, CCError>;

#[derive(Error, Debug)]
pub enum CCError {
    #[error("Consensus error: {0}")]
    Consensus(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Block error: {0}")]
    Block(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("State error: {0}")]
    State(String),

    #[error("Cryptographic error: {0}")]
    Crypto(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Hex decode error: {0}")]
    HexDecode(#[from] hex::FromHexError),

    #[error("Network timeout")]
    NetworkTimeout(#[from] tokio::time::error::Elapsed),

    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Out of gas: required {required}, available {available}")]
    OutOfGas { required: u64, available: u64 },

    #[error("Contract execution failed: {0}")]
    ContractExecutionFailed(String),

    #[error("Other error: {0}")]
    Other(String),
}
