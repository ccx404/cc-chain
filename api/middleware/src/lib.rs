//! CC Chain API Middleware
//!
//! This module provides comprehensive middleware functionality for the CC Chain API,
//! including authentication, logging, CORS, rate limiting, and request/response processing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MiddlewareError {
    #[error("Authentication failed: {reason}")]
    Authentication { reason: String },
    #[error("Authorization failed: {reason}")]
    Authorization { reason: String },
    #[error("Rate limit exceeded: {message}")]
    RateLimit { message: String },
    #[error("CORS validation failed: {reason}")]
    Cors { reason: String },
    #[error("Request validation failed: {reason}")]
    Validation { reason: String },
    #[error("Middleware error: {0}")]
    Generic(String),
}

pub type Result<T> = std::result::Result<T, MiddlewareError>;

/// HTTP request context
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_id: String,
    pub method: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub remote_addr: Option<String>,
    pub user_agent: Option<String>,
    pub content_type: Option<String>,
    pub start_time: Instant,
}

impl RequestContext {
    pub fn new(method: String, path: String) -> Self {
        Self {
            request_id: generate_request_id(),
            method,
            path,
            query_params: HashMap::new(),
            headers: HashMap::new(),
            remote_addr: None,
            user_agent: None,
            content_type: None,
            start_time: Instant::now(),
        }
    }

    pub fn duration(&self) -> Duration {
        self.start_time.elapsed()
    }
}

/// Authentication middleware
pub struct AuthMiddleware {
    pub required_permissions: Vec<String>,
    pub allow_anonymous: bool,
    pub api_key_header: String,
    pub token_header: String,
}

impl AuthMiddleware {
    pub fn new() -> Self {
        Self {
            required_permissions: vec!["read".to_string()],
            allow_anonymous: false,
            api_key_header: "X-API-Key".to_string(),
            token_header: "Authorization".to_string(),
        }
    }

    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.required_permissions = permissions;
        self
    }

    pub fn allow_anonymous(mut self) -> Self {
        self.allow_anonymous = true;
        self
    }

    /// Process authentication for request
    pub fn process(&self, context: &RequestContext) -> Result<AuthResult> {
        // Check for API key
        if let Some(api_key) = context.headers.get(&self.api_key_header) {
            return self.validate_api_key(api_key);
        }

        // Check for JWT token
        if let Some(auth_header) = context.headers.get(&self.token_header) {
            if auth_header.starts_with("Bearer ") {
                let token = &auth_header[7..];
                return self.validate_jwt_token(token);
            }
        }

        // No authentication provided
        if self.allow_anonymous {
            Ok(AuthResult::Anonymous)
        } else {
            Err(MiddlewareError::Authentication {
                reason: "No authentication provided".to_string(),
            })
        }
    }

    fn validate_api_key(&self, _api_key: &str) -> Result<AuthResult> {
        // In a real implementation, this would validate against a key store
        Ok(AuthResult::ApiKey {
            key_id: "test_key".to_string(),
            user_id: "test_user".to_string(),
            permissions: vec!["read".to_string(), "write".to_string()],
        })
    }

    fn validate_jwt_token(&self, _token: &str) -> Result<AuthResult> {
        // In a real implementation, this would validate JWT signature and expiration
        Ok(AuthResult::JwtToken {
            user_id: "jwt_user".to_string(),
            permissions: vec!["read".to_string(), "write".to_string()],
            expires_at: std::time::SystemTime::now() + Duration::from_secs(3600),
        })
    }
}

impl Default for AuthMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

/// Authentication result
#[derive(Debug, Clone)]
pub enum AuthResult {
    Anonymous,
    ApiKey {
        key_id: String,
        user_id: String,
        permissions: Vec<String>,
    },
    JwtToken {
        user_id: String,
        permissions: Vec<String>,
        expires_at: std::time::SystemTime,
    },
}

impl AuthResult {
    pub fn has_permission(&self, permission: &str) -> bool {
        match self {
            AuthResult::Anonymous => permission == "read", // Anonymous users can only read
            AuthResult::ApiKey { permissions, .. } => permissions.contains(&permission.to_string()),
            AuthResult::JwtToken { permissions, .. } => permissions.contains(&permission.to_string()),
        }
    }

    pub fn user_id(&self) -> Option<&str> {
        match self {
            AuthResult::Anonymous => None,
            AuthResult::ApiKey { user_id, .. } => Some(user_id),
            AuthResult::JwtToken { user_id, .. } => Some(user_id),
        }
    }
}

/// CORS middleware configuration
#[derive(Debug, Clone)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub exposed_headers: Vec<String>,
    pub allow_credentials: bool,
    pub max_age: Option<Duration>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "OPTIONS".to_string(),
            ],
            allowed_headers: vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
                "X-API-Key".to_string(),
            ],
            exposed_headers: vec!["X-Request-ID".to_string()],
            allow_credentials: false,
            max_age: Some(Duration::from_secs(86400)), // 24 hours
        }
    }
}

/// CORS middleware
pub struct CorsMiddleware {
    config: CorsConfig,
}

impl CorsMiddleware {
    pub fn new(config: CorsConfig) -> Self {
        Self { config }
    }

    /// Process CORS for request
    pub fn process(&self, context: &RequestContext) -> Result<CorsResponse> {
        let origin = context.headers.get("Origin");

        // Check if origin is allowed
        if let Some(origin) = origin {
            if !self.is_origin_allowed(origin) {
                return Err(MiddlewareError::Cors {
                    reason: format!("Origin not allowed: {}", origin),
                });
            }
        }

        // Handle preflight request
        if context.method == "OPTIONS" {
            return Ok(CorsResponse::Preflight {
                allowed_origin: origin.cloned(),
                allowed_methods: self.config.allowed_methods.clone(),
                allowed_headers: self.config.allowed_headers.clone(),
                max_age: self.config.max_age,
            });
        }

        // Handle regular request
        Ok(CorsResponse::Regular {
            allowed_origin: origin.cloned(),
            exposed_headers: self.config.exposed_headers.clone(),
        })
    }

    fn is_origin_allowed(&self, origin: &str) -> bool {
        self.config.allowed_origins.contains(&"*".to_string()) 
            || self.config.allowed_origins.contains(&origin.to_string())
    }
}

impl Default for CorsMiddleware {
    fn default() -> Self {
        Self::new(CorsConfig::default())
    }
}

/// CORS response
#[derive(Debug, Clone)]
pub enum CorsResponse {
    Preflight {
        allowed_origin: Option<String>,
        allowed_methods: Vec<String>,
        allowed_headers: Vec<String>,
        max_age: Option<Duration>,
    },
    Regular {
        allowed_origin: Option<String>,
        exposed_headers: Vec<String>,
    },
}

/// Rate limiting middleware
pub struct RateLimitMiddleware {
    limits: HashMap<String, RateLimit>,
    global_limit: Option<RateLimit>,
}

#[derive(Debug, Clone)]
pub struct RateLimit {
    pub requests_per_window: u32,
    pub window_duration: Duration,
    pub current_count: u32,
    pub window_start: Instant,
}

impl RateLimit {
    pub fn new(requests_per_window: u32, window_duration: Duration) -> Self {
        Self {
            requests_per_window,
            window_duration,
            current_count: 0,
            window_start: Instant::now(),
        }
    }

    pub fn check_and_increment(&mut self) -> bool {
        let now = Instant::now();
        
        // Reset window if expired
        if now.duration_since(self.window_start) >= self.window_duration {
            self.current_count = 0;
            self.window_start = now;
        }

        // Check if limit exceeded
        if self.current_count >= self.requests_per_window {
            return false;
        }

        self.current_count += 1;
        true
    }

    pub fn remaining(&self) -> u32 {
        self.requests_per_window.saturating_sub(self.current_count)
    }

    pub fn reset_time(&self) -> Duration {
        self.window_duration.saturating_sub(
            Instant::now().duration_since(self.window_start)
        )
    }
}

impl RateLimitMiddleware {
    pub fn new() -> Self {
        Self {
            limits: HashMap::new(),
            global_limit: Some(RateLimit::new(1000, Duration::from_secs(60))), // 1000 req/min
        }
    }

    pub fn with_global_limit(mut self, requests_per_minute: u32) -> Self {
        self.global_limit = Some(RateLimit::new(requests_per_minute, Duration::from_secs(60)));
        self
    }

    pub fn with_user_limit(mut self, user_id: String, requests_per_minute: u32) -> Self {
        self.limits.insert(user_id, RateLimit::new(requests_per_minute, Duration::from_secs(60)));
        self
    }

    /// Process rate limiting for request
    pub fn process(&mut self, _context: &RequestContext, auth_result: &AuthResult) -> Result<RateLimitInfo> {
        // Check global limit first
        if let Some(ref mut global_limit) = self.global_limit {
            if !global_limit.check_and_increment() {
                return Err(MiddlewareError::RateLimit {
                    message: "Global rate limit exceeded".to_string(),
                });
            }
        }

        // Check user-specific limit
        if let Some(user_id) = auth_result.user_id() {
            if let Some(user_limit) = self.limits.get_mut(user_id) {
                if !user_limit.check_and_increment() {
                    return Err(MiddlewareError::RateLimit {
                        message: format!("User rate limit exceeded for {}", user_id),
                    });
                }

                return Ok(RateLimitInfo {
                    remaining: user_limit.remaining(),
                    reset_time: user_limit.reset_time(),
                    limit: user_limit.requests_per_window,
                });
            }
        }

        // Return global limit info if no user-specific limit
        if let Some(ref global_limit) = self.global_limit {
            Ok(RateLimitInfo {
                remaining: global_limit.remaining(),
                reset_time: global_limit.reset_time(),
                limit: global_limit.requests_per_window,
            })
        } else {
            Ok(RateLimitInfo {
                remaining: u32::MAX,
                reset_time: Duration::from_secs(0),
                limit: u32::MAX,
            })
        }
    }
}

impl Default for RateLimitMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate limit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    pub remaining: u32,
    pub reset_time: Duration,
    pub limit: u32,
}

/// Logging middleware
pub struct LoggingMiddleware {
    pub log_requests: bool,
    pub log_responses: bool,
    pub log_body: bool,
    pub sensitive_headers: Vec<String>,
}

impl LoggingMiddleware {
    pub fn new() -> Self {
        Self {
            log_requests: true,
            log_responses: true,
            log_body: false,
            sensitive_headers: vec![
                "Authorization".to_string(),
                "X-API-Key".to_string(),
                "Cookie".to_string(),
            ],
        }
    }

    /// Log incoming request
    pub fn log_request(&self, context: &RequestContext) {
        if !self.log_requests {
            return;
        }

        let headers = self.filter_sensitive_headers(&context.headers);
        
        println!(
            "[REQUEST] {} {} {} - Headers: {:?} - User-Agent: {:?}",
            context.request_id,
            context.method,
            context.path,
            headers,
            context.user_agent
        );
    }

    /// Log outgoing response
    pub fn log_response(&self, context: &RequestContext, status: u16, size: Option<usize>) {
        if !self.log_responses {
            return;
        }

        println!(
            "[RESPONSE] {} - Status: {} - Duration: {:?} - Size: {:?}",
            context.request_id,
            status,
            context.duration(),
            size
        );
    }

    fn filter_sensitive_headers(&self, headers: &HashMap<String, String>) -> HashMap<String, String> {
        headers
            .iter()
            .map(|(key, value)| {
                if self.sensitive_headers.contains(key) {
                    (key.clone(), "***".to_string())
                } else {
                    (key.clone(), value.clone())
                }
            })
            .collect()
    }
}

impl Default for LoggingMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

/// Middleware chain for processing requests
pub struct MiddlewareChain {
    pub auth: AuthMiddleware,
    pub cors: CorsMiddleware,
    pub rate_limit: RateLimitMiddleware,
    pub logging: LoggingMiddleware,
}

impl MiddlewareChain {
    pub fn new() -> Self {
        Self {
            auth: AuthMiddleware::new(),
            cors: CorsMiddleware::new(CorsConfig::default()),
            rate_limit: RateLimitMiddleware::new(),
            logging: LoggingMiddleware::new(),
        }
    }

    /// Process request through all middleware
    pub fn process_request(&mut self, context: &RequestContext) -> Result<MiddlewareResult> {
        // Log request
        self.logging.log_request(context);

        // Process CORS
        let cors_response = self.cors.process(context)?;

        // Skip auth for preflight requests
        if let CorsResponse::Preflight { .. } = cors_response {
            return Ok(MiddlewareResult {
                auth_result: AuthResult::Anonymous,
                cors_response,
                rate_limit_info: RateLimitInfo {
                    remaining: u32::MAX,
                    reset_time: Duration::from_secs(0),
                    limit: u32::MAX,
                },
            });
        }

        // Process authentication
        let auth_result = self.auth.process(context)?;

        // Process rate limiting
        let rate_limit_info = self.rate_limit.process(context, &auth_result)?;

        Ok(MiddlewareResult {
            auth_result,
            cors_response,
            rate_limit_info,
        })
    }

    /// Log response
    pub fn log_response(&self, context: &RequestContext, status: u16, size: Option<usize>) {
        self.logging.log_response(context, status, size);
    }
}

impl Default for MiddlewareChain {
    fn default() -> Self {
        Self::new()
    }
}

/// Middleware processing result
#[derive(Debug, Clone)]
pub struct MiddlewareResult {
    pub auth_result: AuthResult,
    pub cors_response: CorsResponse,
    pub rate_limit_info: RateLimitInfo,
}

/// Generate unique request ID
fn generate_request_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("req_{}", timestamp % 1_000_000)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_context() -> RequestContext {
        let mut context = RequestContext::new("GET".to_string(), "/api/v1/blocks".to_string());
        context.headers.insert("User-Agent".to_string(), "test-client/1.0".to_string());
        context.user_agent = Some("test-client/1.0".to_string());
        context
    }

    #[test]
    fn test_auth_middleware_anonymous() {
        let auth = AuthMiddleware::new().allow_anonymous();
        let context = create_test_context();
        
        let result = auth.process(&context);
        assert!(result.is_ok());
        
        if let Ok(AuthResult::Anonymous) = result {
            // Expected
        } else {
            panic!("Expected anonymous auth result");
        }
    }

    #[test]
    fn test_auth_middleware_api_key() {
        let auth = AuthMiddleware::new();
        let mut context = create_test_context();
        context.headers.insert("X-API-Key".to_string(), "test-key".to_string());
        
        let result = auth.process(&context);
        assert!(result.is_ok());
        
        if let Ok(AuthResult::ApiKey { key_id, .. }) = result {
            assert_eq!(key_id, "test_key");
        } else {
            panic!("Expected API key auth result");
        }
    }

    #[test]
    fn test_auth_middleware_jwt_token() {
        let auth = AuthMiddleware::new();
        let mut context = create_test_context();
        context.headers.insert("Authorization".to_string(), "Bearer test-token".to_string());
        
        let result = auth.process(&context);
        assert!(result.is_ok());
        
        if let Ok(AuthResult::JwtToken { user_id, .. }) = result {
            assert_eq!(user_id, "jwt_user");
        } else {
            panic!("Expected JWT token auth result");
        }
    }

    #[test]
    fn test_cors_middleware_regular_request() {
        let cors = CorsMiddleware::default();
        let mut context = create_test_context();
        context.headers.insert("Origin".to_string(), "https://example.com".to_string());
        
        let result = cors.process(&context);
        assert!(result.is_ok());
        
        if let Ok(CorsResponse::Regular { allowed_origin, .. }) = result {
            assert_eq!(allowed_origin, Some("https://example.com".to_string()));
        } else {
            panic!("Expected regular CORS response");
        }
    }

    #[test]
    fn test_cors_middleware_preflight_request() {
        let cors = CorsMiddleware::default();
        let mut context = RequestContext::new("OPTIONS".to_string(), "/api/v1/blocks".to_string());
        context.headers.insert("Origin".to_string(), "https://example.com".to_string());
        
        let result = cors.process(&context);
        assert!(result.is_ok());
        
        if let Ok(CorsResponse::Preflight { allowed_origin, .. }) = result {
            assert_eq!(allowed_origin, Some("https://example.com".to_string()));
        } else {
            panic!("Expected preflight CORS response");
        }
    }

    #[test]
    fn test_rate_limit_basic() {
        let mut rate_limit = RateLimit::new(2, Duration::from_secs(60));
        
        assert!(rate_limit.check_and_increment());
        assert_eq!(rate_limit.remaining(), 1);
        
        assert!(rate_limit.check_and_increment());
        assert_eq!(rate_limit.remaining(), 0);
        
        assert!(!rate_limit.check_and_increment());
        assert_eq!(rate_limit.remaining(), 0);
    }

    #[test]
    fn test_rate_limit_middleware() {
        let mut middleware = RateLimitMiddleware::new().with_global_limit(2);
        let context = create_test_context();
        let auth_result = AuthResult::Anonymous;
        
        // First request should succeed
        let result1 = middleware.process(&context, &auth_result);
        assert!(result1.is_ok());
        
        // Second request should succeed
        let result2 = middleware.process(&context, &auth_result);
        assert!(result2.is_ok());
        
        // Third request should fail
        let result3 = middleware.process(&context, &auth_result);
        assert!(result3.is_err());
    }

    #[test]
    fn test_logging_middleware() {
        let logging = LoggingMiddleware::new();
        let mut context = create_test_context();
        context.headers.insert("Authorization".to_string(), "Bearer secret-token".to_string());
        
        // This should not panic and should filter sensitive headers
        logging.log_request(&context);
        logging.log_response(&context, 200, Some(1024));
    }

    #[test]
    fn test_middleware_chain() {
        let mut chain = MiddlewareChain::new();
        chain.auth = AuthMiddleware::new().allow_anonymous();
        
        let context = create_test_context();
        let result = chain.process_request(&context);
        
        assert!(result.is_ok());
        
        let middleware_result = result.unwrap();
        assert!(matches!(middleware_result.auth_result, AuthResult::Anonymous));
    }

    #[test]
    fn test_auth_result_permissions() {
        let api_key_result = AuthResult::ApiKey {
            key_id: "test".to_string(),
            user_id: "user".to_string(),
            permissions: vec!["read".to_string(), "write".to_string()],
        };
        
        assert!(api_key_result.has_permission("read"));
        assert!(api_key_result.has_permission("write"));
        assert!(!api_key_result.has_permission("admin"));
        
        let anonymous_result = AuthResult::Anonymous;
        assert!(anonymous_result.has_permission("read"));
        assert!(!anonymous_result.has_permission("write"));
    }

    #[test]
    fn test_request_context_duration() {
        let context = create_test_context();
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let duration = context.duration();
        assert!(duration >= std::time::Duration::from_millis(10));
    }
}
