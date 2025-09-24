//! CC Chain API Authentication
//!
//! This module provides comprehensive authentication functionality for the CC Chain API,
//! including JWT token management, API key validation, and role-based access control.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials provided")]
    InvalidCredentials,
    #[error("Token has expired")]
    TokenExpired,
    #[error("Invalid token format")]
    InvalidToken,
    #[error("Insufficient permissions for operation")]
    InsufficientPermissions,
    #[error("API key not found or inactive")]
    InvalidApiKey,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Authentication error: {0}")]
    Generic(String),
}

pub type Result<T> = std::result::Result<T, AuthError>;

/// User roles for role-based access control
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    Developer,
    ReadOnly,
    Validator,
}

/// Authentication token containing user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub user_id: String,
    pub role: UserRole,
    pub expires_at: u64,
    pub permissions: Vec<String>,
}

/// API key information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub key_id: String,
    pub user_id: String,
    pub role: UserRole,
    pub created_at: u64,
    pub last_used: Option<u64>,
    pub is_active: bool,
    pub rate_limit: u32, // requests per minute
}

/// Authentication request
#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

/// Authentication response
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub expires_in: u64,
    pub user_id: String,
    pub role: UserRole,
}

/// API key creation request
#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub role: UserRole,
    pub rate_limit: Option<u32>,
}

/// API key response
#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
    pub key_id: String,
    pub api_key: String,
    pub role: UserRole,
    pub rate_limit: u32,
    pub created_at: u64,
}

/// Rate limiting information
#[derive(Debug, Clone)]
pub struct RateLimit {
    pub requests: u32,
    pub window_start: SystemTime,
    pub limit: u32,
}

/// CC Chain API Authenticator
pub struct Authenticator {
    secret_key: String,
    token_duration: Duration,
    users: HashMap<String, UserInfo>,
    api_keys: HashMap<String, ApiKey>,
    rate_limits: HashMap<String, RateLimit>,
}

#[derive(Debug, Clone)]
struct UserInfo {
    user_id: String,
    username: String,
    password_hash: String,
    role: UserRole,
    is_active: bool,
}

impl Authenticator {
    /// Create a new authenticator with a secret key
    pub fn new(secret_key: String) -> Self {
        let mut users = HashMap::new();
        
        // Add default admin user for testing
        users.insert("admin".to_string(), UserInfo {
            user_id: "admin_001".to_string(),
            username: "admin".to_string(),
            password_hash: "admin_hash".to_string(), // In real implementation, use proper hashing
            role: UserRole::Admin,
            is_active: true,
        });
        
        users.insert("developer".to_string(), UserInfo {
            user_id: "dev_001".to_string(),
            username: "developer".to_string(),
            password_hash: "dev_hash".to_string(),
            role: UserRole::Developer,
            is_active: true,
        });

        Self {
            secret_key,
            token_duration: Duration::from_secs(24 * 3600), // 24 hours
            users,
            api_keys: HashMap::new(),
            rate_limits: HashMap::new(),
        }
    }

    /// Authenticate user and return a token
    pub fn authenticate(&mut self, request: AuthRequest) -> Result<AuthResponse> {
        let user = self.users.get(&request.username)
            .ok_or(AuthError::InvalidCredentials)?;

        if !user.is_active {
            return Err(AuthError::InvalidCredentials);
        }

        // Simple password check (in real implementation, use proper password hashing)
        if self.hash_password(&request.password) != user.password_hash {
            return Err(AuthError::InvalidCredentials);
        }

        let expires_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() + self.token_duration.as_secs();

        let token = AuthToken {
            user_id: user.user_id.clone(),
            role: user.role.clone(),
            expires_at,
            permissions: self.get_permissions_for_role(&user.role),
        };

        let token_string = self.create_token(&token)?;

        Ok(AuthResponse {
            token: token_string,
            expires_in: self.token_duration.as_secs(),
            user_id: user.user_id.clone(),
            role: user.role.clone(),
        })
    }

    /// Validate an authentication token
    pub fn validate_token(&self, token_string: &str) -> Result<AuthToken> {
        let token = self.parse_token(token_string)?;
        
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if token.expires_at < current_time {
            return Err(AuthError::TokenExpired);
        }

        Ok(token)
    }

    /// Validate an API key
    pub fn validate_api_key(&mut self, api_key: &str) -> Result<ApiKey> {
        // First check if key exists and is active, and collect needed info
        let (key_id, rate_limit) = {
            let key_info = self.api_keys.get(api_key)
                .ok_or(AuthError::InvalidApiKey)?;

            if !key_info.is_active {
                return Err(AuthError::InvalidApiKey);
            }

            (key_info.key_id.clone(), key_info.rate_limit)
        };

        // Check rate limit
        self.check_rate_limit(&key_id, rate_limit)?;

        // Update last used timestamp and return updated key info
        if let Some(key_info_mut) = self.api_keys.get_mut(api_key) {
            key_info_mut.last_used = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            );
            Ok(key_info_mut.clone())
        } else {
            Err(AuthError::InvalidApiKey)
        }
    }

    /// Create a new API key
    pub fn create_api_key(&mut self, user_id: &str, request: CreateApiKeyRequest) -> Result<ApiKeyResponse> {
        let key_id = format!("key_{}", self.generate_id());
        let api_key = format!("cc_{}_{}", key_id, self.generate_secret());
        
        let rate_limit = request.rate_limit.unwrap_or(1000); // Default 1000 requests per minute
        
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        let api_key_info = ApiKey {
            key_id: key_id.clone(),
            user_id: user_id.to_string(),
            role: request.role.clone(),
            created_at,
            last_used: None,
            is_active: true,
            rate_limit,
        };

        self.api_keys.insert(api_key.clone(), api_key_info);

        Ok(ApiKeyResponse {
            key_id,
            api_key,
            role: request.role,
            rate_limit,
            created_at,
        })
    }

    /// Check if user has required permission
    pub fn has_permission(&self, token: &AuthToken, permission: &str) -> bool {
        token.permissions.contains(&permission.to_string())
    }

    /// Revoke an API key
    pub fn revoke_api_key(&mut self, api_key: &str) -> Result<()> {
        let key_info = self.api_keys.get_mut(api_key)
            .ok_or(AuthError::InvalidApiKey)?;
        
        key_info.is_active = false;
        Ok(())
    }

    /// Get user permissions based on role
    fn get_permissions_for_role(&self, role: &UserRole) -> Vec<String> {
        match role {
            UserRole::Admin => vec![
                "read".to_string(),
                "write".to_string(),
                "admin".to_string(),
                "manage_users".to_string(),
                "manage_keys".to_string(),
            ],
            UserRole::Developer => vec![
                "read".to_string(),
                "write".to_string(),
                "deploy".to_string(),
            ],
            UserRole::Validator => vec![
                "read".to_string(),
                "validate".to_string(),
                "consensus".to_string(),
            ],
            UserRole::ReadOnly => vec![
                "read".to_string(),
            ],
        }
    }

    /// Check rate limit for API key
    fn check_rate_limit(&mut self, key_id: &str, limit: u32) -> Result<()> {
        let now = SystemTime::now();
        
        let rate_limit = self.rate_limits.entry(key_id.to_string())
            .or_insert(RateLimit {
                requests: 0,
                window_start: now,
                limit,
            });

        // Reset window if more than 1 minute has passed
        if now.duration_since(rate_limit.window_start).unwrap() > Duration::from_secs(60) {
            rate_limit.requests = 0;
            rate_limit.window_start = now;
        }

        if rate_limit.requests >= limit {
            return Err(AuthError::RateLimitExceeded);
        }

        rate_limit.requests += 1;
        Ok(())
    }

    /// Simple token creation (in real implementation, use proper JWT library)
    fn create_token(&self, token: &AuthToken) -> Result<String> {
        let token_data = serde_json::to_string(token)
            .map_err(|e| AuthError::Generic(format!("Token serialization error: {}", e)))?;
        
        // Simple encoding (in real implementation, use proper JWT signing)
        Ok(format!("{}:{}", self.secret_key, base64_encode(&token_data)))
    }

    /// Parse token from string
    fn parse_token(&self, token_string: &str) -> Result<AuthToken> {
        let parts: Vec<&str> = token_string.split(':').collect();
        if parts.len() != 2 || parts[0] != self.secret_key {
            return Err(AuthError::InvalidToken);
        }

        let token_data = base64_decode(parts[1])
            .map_err(|_| AuthError::InvalidToken)?;
        
        serde_json::from_str(&token_data)
            .map_err(|_| AuthError::InvalidToken)
    }

    /// Hash password (simplified for demo)
    fn hash_password(&self, password: &str) -> String {
        format!("{}_hash", password)
    }

    /// Generate unique ID
    fn generate_id(&self) -> String {
        format!("{}", SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() % 1000000)
    }

    /// Generate secret key
    fn generate_secret(&self) -> String {
        format!("secret_{}", self.generate_id())
    }
}

// Simple base64 encoding/decoding (in real implementation, use proper library)
fn base64_encode(data: &str) -> String {
    data.bytes().map(|b| format!("{:02x}", b)).collect()
}

fn base64_decode(data: &str) -> Result<String> {
    if data.len() % 2 != 0 {
        return Err(AuthError::InvalidToken);
    }
    
    let mut result = String::new();
    for chunk in data.as_bytes().chunks(2) {
        if chunk.len() == 2 {
            let hex_str = std::str::from_utf8(chunk).unwrap();
            if let Ok(byte) = u8::from_str_radix(hex_str, 16) {
                result.push(byte as char);
            } else {
                return Err(AuthError::InvalidToken);
            }
        }
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_authenticator() -> Authenticator {
        Authenticator::new("test_secret".to_string())
    }

    #[test]
    fn test_authenticate_valid_user() {
        let mut auth = create_test_authenticator();
        let request = AuthRequest {
            username: "admin".to_string(),
            password: "admin".to_string(),
        };

        let result = auth.authenticate(request);
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response.user_id, "admin_001");
        assert_eq!(response.role, UserRole::Admin);
        assert!(!response.token.is_empty());
    }

    #[test]
    fn test_authenticate_invalid_user() {
        let mut auth = create_test_authenticator();
        let request = AuthRequest {
            username: "nonexistent".to_string(),
            password: "password".to_string(),
        };

        let result = auth.authenticate(request);
        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }

    #[test]
    fn test_authenticate_wrong_password() {
        let mut auth = create_test_authenticator();
        let request = AuthRequest {
            username: "admin".to_string(),
            password: "wrong_password".to_string(),
        };

        let result = auth.authenticate(request);
        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }

    #[test]
    fn test_validate_token() {
        let mut auth = create_test_authenticator();
        let request = AuthRequest {
            username: "admin".to_string(),
            password: "admin".to_string(),
        };

        let auth_response = auth.authenticate(request).unwrap();
        let validation_result = auth.validate_token(&auth_response.token);
        
        assert!(validation_result.is_ok());
        let token = validation_result.unwrap();
        assert_eq!(token.user_id, "admin_001");
        assert_eq!(token.role, UserRole::Admin);
    }

    #[test]
    fn test_create_api_key() {
        let mut auth = create_test_authenticator();
        let request = CreateApiKeyRequest {
            name: "test_key".to_string(),
            role: UserRole::Developer,
            rate_limit: Some(500),
        };

        let result = auth.create_api_key("dev_001", request);
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response.role, UserRole::Developer);
        assert_eq!(response.rate_limit, 500);
        assert!(response.api_key.starts_with("cc_key_"));
    }

    #[test]
    fn test_validate_api_key() {
        let mut auth = create_test_authenticator();
        let request = CreateApiKeyRequest {
            name: "test_key".to_string(),
            role: UserRole::Developer,
            rate_limit: Some(1000),
        };

        let key_response = auth.create_api_key("dev_001", request).unwrap();
        let validation_result = auth.validate_api_key(&key_response.api_key);
        
        assert!(validation_result.is_ok());
        let key_info = validation_result.unwrap();
        assert_eq!(key_info.user_id, "dev_001");
        assert_eq!(key_info.role, UserRole::Developer);
        assert!(key_info.last_used.is_some());
    }

    #[test]
    fn test_revoke_api_key() {
        let mut auth = create_test_authenticator();
        let request = CreateApiKeyRequest {
            name: "test_key".to_string(),
            role: UserRole::Developer,
            rate_limit: Some(1000),
        };

        let key_response = auth.create_api_key("dev_001", request).unwrap();
        
        // Revoke the key
        let revoke_result = auth.revoke_api_key(&key_response.api_key);
        assert!(revoke_result.is_ok());
        
        // Try to validate revoked key
        let validation_result = auth.validate_api_key(&key_response.api_key);
        assert!(matches!(validation_result, Err(AuthError::InvalidApiKey)));
    }

    #[test]
    fn test_permissions() {
        let auth = create_test_authenticator();
        
        // Create admin token
        let admin_token = AuthToken {
            user_id: "admin_001".to_string(),
            role: UserRole::Admin,
            expires_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() + 3600,
            permissions: auth.get_permissions_for_role(&UserRole::Admin),
        };
        
        assert!(auth.has_permission(&admin_token, "admin"));
        assert!(auth.has_permission(&admin_token, "read"));
        assert!(auth.has_permission(&admin_token, "write"));
        
        // Create read-only token
        let readonly_token = AuthToken {
            user_id: "readonly_001".to_string(),
            role: UserRole::ReadOnly,
            expires_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() + 3600,
            permissions: auth.get_permissions_for_role(&UserRole::ReadOnly),
        };
        
        assert!(auth.has_permission(&readonly_token, "read"));
        assert!(!auth.has_permission(&readonly_token, "write"));
        assert!(!auth.has_permission(&readonly_token, "admin"));
    }
}
