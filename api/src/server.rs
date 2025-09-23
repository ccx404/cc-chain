//! REST API server implementation

use crate::error::ApiError;
use crate::models::*;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

/// API server state containing node handle
#[derive(Clone)]
pub struct ApiState {
    /// Reference to the blockchain node
    pub node: Arc<dyn NodeApi + Send + Sync>,
}

/// Trait defining the interface between API and the node
pub trait NodeApi {
    /// Get blockchain height
    fn get_height(&self) -> u64;
    
    /// Get account balance  
    fn get_balance(&self, address: &str) -> Result<u64, ApiError>;
    
    /// Submit a transaction
    fn submit_transaction(&self, tx_data: TransactionRequest) -> Result<String, ApiError>;
    
    /// Get block by height
    fn get_block(&self, height: u64) -> Result<Option<BlockResponse>, ApiError>;
    
    /// Get transaction by hash
    fn get_transaction(&self, hash: &str) -> Result<Option<TransactionResponse>, ApiError>;
    
    /// Get chain info
    fn get_chain_info(&self) -> Result<ChainInfo, ApiError>;
    
    /// Get mempool status
    fn get_mempool_status(&self) -> Result<MempoolStatus, ApiError>;
    
    /// Get network peers
    fn get_peers(&self) -> Result<Vec<PeerInfo>, ApiError>;
}

/// CC Chain REST API server
pub struct ApiServer {
    state: ApiState,
    router: Router,
}

impl ApiServer {
    /// Create a new API server
    pub fn new(node: Arc<dyn NodeApi + Send + Sync>) -> Self {
        let state = ApiState { node };
        let router = create_router(state.clone());
        
        Self { state, router }
    }
    
    /// Start the API server
    pub async fn start(self, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        tracing::info!("API server listening on {}", addr);
        
        axum::serve(listener, self.router).await?;
        Ok(())
    }
}

/// Create the API router with all endpoints
fn create_router(state: ApiState) -> Router {
    Router::new()
        // Chain information endpoints
        .route("/api/v1/chain/info", get(get_chain_info))
        .route("/api/v1/chain/height", get(get_height))
        
        // Block endpoints
        .route("/api/v1/blocks/:height", get(get_block))
        .route("/api/v1/blocks/latest", get(get_latest_block))
        
        // Transaction endpoints
        .route("/api/v1/transactions", post(submit_transaction))
        .route("/api/v1/transactions/:hash", get(get_transaction))
        
        // Account endpoints
        .route("/api/v1/accounts/:address/balance", get(get_balance))
        
        // Mempool endpoints  
        .route("/api/v1/mempool/status", get(get_mempool_status))
        
        // Network endpoints
        .route("/api/v1/network/peers", get(get_peers))
        
        // Health check
        .route("/health", get(health_check))
        
        // Add middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )
        .with_state(state)
}

// Handler functions

/// Get chain information
async fn get_chain_info(State(state): State<ApiState>) -> Result<Json<ChainInfo>, ApiError> {
    let info = state.node.get_chain_info()?;
    Ok(Json(info))
}

/// Get current blockchain height
async fn get_height(State(state): State<ApiState>) -> Result<Json<HeightResponse>, ApiError> {
    let height = state.node.get_height();
    Ok(Json(HeightResponse { height }))
}

/// Get block by height
async fn get_block(
    Path(height): Path<u64>,
    State(state): State<ApiState>,
) -> Result<Json<BlockResponse>, ApiError> {
    match state.node.get_block(height)? {
        Some(block) => Ok(Json(block)),
        None => Err(ApiError::NotFound("Block not found".to_string())),
    }
}

/// Get latest block
async fn get_latest_block(State(state): State<ApiState>) -> Result<Json<BlockResponse>, ApiError> {
    let height = state.node.get_height();
    match state.node.get_block(height)? {
        Some(block) => Ok(Json(block)),
        None => Err(ApiError::NotFound("Latest block not found".to_string())),
    }
}

/// Submit a transaction
async fn submit_transaction(
    State(state): State<ApiState>,
    Json(tx_request): Json<TransactionRequest>,
) -> Result<Json<TransactionSubmitResponse>, ApiError> {
    let tx_hash = state.node.submit_transaction(tx_request)?;
    Ok(Json(TransactionSubmitResponse { transaction_hash: tx_hash }))
}

/// Get transaction by hash
async fn get_transaction(
    Path(hash): Path<String>,
    State(state): State<ApiState>,
) -> Result<Json<TransactionResponse>, ApiError> {
    match state.node.get_transaction(&hash)? {
        Some(tx) => Ok(Json(tx)),
        None => Err(ApiError::NotFound("Transaction not found".to_string())),
    }
}

/// Get account balance
async fn get_balance(
    Path(address): Path<String>,
    State(state): State<ApiState>,
) -> Result<Json<BalanceResponse>, ApiError> {
    let balance = state.node.get_balance(&address)?;
    Ok(Json(BalanceResponse { address, balance }))
}

/// Get mempool status
async fn get_mempool_status(State(state): State<ApiState>) -> Result<Json<MempoolStatus>, ApiError> {
    let status = state.node.get_mempool_status()?;
    Ok(Json(status))
}

/// Get network peers
async fn get_peers(State(state): State<ApiState>) -> Result<Json<PeersResponse>, ApiError> {
    let peers = state.node.get_peers()?;
    Ok(Json(PeersResponse { peers }))
}

/// Health check endpoint
async fn health_check() -> Result<Json<HealthResponse>, StatusCode> {
    Ok(Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now(),
    }))
}