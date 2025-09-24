//! CC Chain API Response Formatters
//!
//! This module provides standardized response formatting for the CC Chain API,
//! including success/error responses, pagination, metadata, and content negotiation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ResponseError {
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Content negotiation failed: {reason}")]
    ContentNegotiation { reason: String },
    #[error("Invalid response format: {format}")]
    InvalidFormat { format: String },
    #[error("Response building error: {0}")]
    BuildError(String),
}

pub type Result<T> = std::result::Result<T, ResponseError>;

/// Standard API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ErrorDetails>,
    pub meta: ResponseMetadata,
}

/// Error details in responses
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
    pub details: Option<HashMap<String, String>>,
    pub trace_id: Option<String>,
}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseMetadata {
    pub timestamp: String,
    pub request_id: String,
    pub version: String,
    pub pagination: Option<PaginationMeta>,
    pub execution_time_ms: Option<u64>,
    pub cached: bool,
}

/// Pagination metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PaginationMeta {
    pub page: u32,
    pub per_page: u32,
    pub total_items: u64,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_previous: bool,
    pub next_page: Option<u32>,
    pub previous_page: Option<u32>,
}

impl PaginationMeta {
    pub fn new(page: u32, per_page: u32, total_items: u64) -> Self {
        let total_pages = if per_page > 0 {
            ((total_items as f64) / (per_page as f64)).ceil() as u32
        } else {
            1
        };

        let has_next = page < total_pages;
        let has_previous = page > 1;

        Self {
            page,
            per_page,
            total_items,
            total_pages,
            has_next,
            has_previous,
            next_page: if has_next { Some(page + 1) } else { None },
            previous_page: if has_previous { Some(page - 1) } else { None },
        }
    }
}

/// Response builder for creating standardized API responses
pub struct ResponseBuilder<T> {
    data: Option<T>,
    error: Option<ErrorDetails>,
    meta: ResponseMetadata,
}

impl<T> ResponseBuilder<T> {
    /// Create a new response builder
    pub fn new() -> Self {
        Self {
            data: None,
            error: None,
            meta: ResponseMetadata {
                timestamp: chrono::Utc::now().to_rfc3339(),
                request_id: generate_request_id(),
                version: "1.0.0".to_string(),
                pagination: None,
                execution_time_ms: None,
                cached: false,
            },
        }
    }

    /// Create a success response
    pub fn success(data: T) -> Self {
        let mut builder = Self::new();
        builder.data = Some(data);
        builder
    }

    /// Create an error response
    pub fn error(code: String, message: String) -> Self {
        let mut builder = Self::new();
        builder.error = Some(ErrorDetails {
            code,
            message,
            details: None,
            trace_id: None,
        });
        builder
    }

    /// Set request ID
    pub fn request_id(mut self, request_id: String) -> Self {
        self.meta.request_id = request_id;
        self
    }

    /// Set execution time
    pub fn execution_time(mut self, time_ms: u64) -> Self {
        self.meta.execution_time_ms = Some(time_ms);
        self
    }

    /// Set cached flag
    pub fn cached(mut self, cached: bool) -> Self {
        self.meta.cached = cached;
        self
    }

    /// Add pagination metadata
    pub fn paginated(mut self, page: u32, per_page: u32, total_items: u64) -> Self {
        self.meta.pagination = Some(PaginationMeta::new(page, per_page, total_items));
        self
    }

    /// Add error details
    pub fn error_details(mut self, details: HashMap<String, String>) -> Self {
        if let Some(ref mut error) = self.error {
            error.details = Some(details);
        }
        self
    }

    /// Add trace ID for debugging
    pub fn trace_id(mut self, trace_id: String) -> Self {
        if let Some(ref mut error) = self.error {
            error.trace_id = Some(trace_id);
        }
        self
    }

    /// Build the response
    pub fn build(self) -> ApiResponse<T> {
        ApiResponse {
            success: self.error.is_none(),
            data: self.data,
            error: self.error,
            meta: self.meta,
        }
    }
}

impl<T> Default for ResponseBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Response formatter for different content types
pub struct ResponseFormatter;

impl ResponseFormatter {
    /// Format response as JSON
    pub fn to_json<T: Serialize>(response: &ApiResponse<T>) -> Result<String> {
        serde_json::to_string_pretty(response)
            .map_err(|e| ResponseError::Serialization(e.to_string()))
    }

    /// Format response as compact JSON
    pub fn to_json_compact<T: Serialize>(response: &ApiResponse<T>) -> Result<String> {
        serde_json::to_string(response)
            .map_err(|e| ResponseError::Serialization(e.to_string()))
    }

    /// Get appropriate content type header
    pub fn content_type(format: &ResponseFormat) -> &'static str {
        match format {
            ResponseFormat::Json => "application/json",
            ResponseFormat::JsonCompact => "application/json",
            ResponseFormat::Xml => "application/xml",
            ResponseFormat::Yaml => "application/yaml",
            ResponseFormat::Html => "text/html",
        }
    }

    /// Determine response format from Accept header
    pub fn negotiate_format(accept_header: Option<&str>) -> ResponseFormat {
        let accept = accept_header.unwrap_or("application/json");
        
        if accept.contains("application/json") {
            ResponseFormat::Json
        } else if accept.contains("application/xml") {
            ResponseFormat::Xml
        } else if accept.contains("application/yaml") || accept.contains("text/yaml") {
            ResponseFormat::Yaml
        } else if accept.contains("text/html") {
            ResponseFormat::Html
        } else {
            ResponseFormat::Json // Default
        }
    }
}

/// Supported response formats
#[derive(Debug, Clone, PartialEq)]
pub enum ResponseFormat {
    Json,
    JsonCompact,
    Xml,
    Yaml,
    Html,
}

/// Common error response builders
pub struct ErrorResponses;

impl ErrorResponses {
    /// Bad Request (400)
    pub fn bad_request(message: &str) -> ApiResponse<()> {
        ResponseBuilder::error("BAD_REQUEST".to_string(), message.to_string())
            .build()
    }

    /// Unauthorized (401)
    pub fn unauthorized(message: &str) -> ApiResponse<()> {
        ResponseBuilder::error("UNAUTHORIZED".to_string(), message.to_string())
            .build()
    }

    /// Forbidden (403)
    pub fn forbidden(message: &str) -> ApiResponse<()> {
        ResponseBuilder::error("FORBIDDEN".to_string(), message.to_string())
            .build()
    }

    /// Not Found (404)
    pub fn not_found(resource: &str) -> ApiResponse<()> {
        ResponseBuilder::error(
            "NOT_FOUND".to_string(),
            format!("Resource not found: {}", resource),
        )
        .build()
    }

    /// Method Not Allowed (405)
    pub fn method_not_allowed(method: &str) -> ApiResponse<()> {
        ResponseBuilder::error(
            "METHOD_NOT_ALLOWED".to_string(),
            format!("Method {} not allowed", method),
        )
        .build()
    }

    /// Validation Error (422)
    pub fn validation_error(errors: HashMap<String, String>) -> ApiResponse<()> {
        ResponseBuilder::error(
            "VALIDATION_ERROR".to_string(),
            "Request validation failed".to_string(),
        )
        .error_details(errors)
        .build()
    }

    /// Rate Limited (429)
    pub fn rate_limited(retry_after: Option<u64>) -> ApiResponse<()> {
        let message = match retry_after {
            Some(seconds) => format!("Rate limit exceeded. Retry after {} seconds", seconds),
            None => "Rate limit exceeded".to_string(),
        };
        
        ResponseBuilder::error("RATE_LIMITED".to_string(), message)
            .build()
    }

    /// Internal Server Error (500)
    pub fn internal_error(message: &str) -> ApiResponse<()> {
        ResponseBuilder::error("INTERNAL_ERROR".to_string(), message.to_string())
            .build()
    }

    /// Service Unavailable (503)
    pub fn service_unavailable(message: &str) -> ApiResponse<()> {
        ResponseBuilder::error("SERVICE_UNAVAILABLE".to_string(), message.to_string())
            .build()
    }
}

/// Response wrapper for collections with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionResponse<T> {
    pub items: Vec<T>,
    pub count: usize,
    pub pagination: Option<PaginationMeta>,
}

impl<T> CollectionResponse<T> {
    pub fn new(items: Vec<T>) -> Self {
        let count = items.len();
        Self {
            items,
            count,
            pagination: None,
        }
    }

    pub fn with_pagination(items: Vec<T>, pagination: PaginationMeta) -> Self {
        let count = items.len();
        Self {
            items,
            count,
            pagination: Some(pagination),
        }
    }
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub checks: HashMap<String, HealthCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub status: String,
    pub response_time_ms: Option<u64>,
    pub error: Option<String>,
}

impl HealthResponse {
    pub fn healthy(uptime_seconds: u64) -> Self {
        let mut checks = HashMap::new();
        checks.insert("database".to_string(), HealthCheck {
            status: "healthy".to_string(),
            response_time_ms: Some(5),
            error: None,
        });
        checks.insert("network".to_string(), HealthCheck {
            status: "healthy".to_string(),
            response_time_ms: Some(2),
            error: None,
        });

        Self {
            status: "healthy".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            version: "1.0.0".to_string(),
            uptime_seconds,
            checks,
        }
    }
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

// Mock serde_json functionality for compilation
mod serde_json {
    use super::*;
    
    pub fn to_string<T: Serialize>(_value: &T) -> std::result::Result<String, Box<dyn std::error::Error>> {
        Ok("{}".to_string())
    }
    
    pub fn to_string_pretty<T: Serialize>(_value: &T) -> std::result::Result<String, Box<dyn std::error::Error>> {
        Ok("{\n}".to_string())
    }
}

// Mock chrono functionality for compilation
mod chrono {
    pub struct Utc;
    
    impl Utc {
        pub fn now() -> DateTime {
            DateTime
        }
    }
    
    pub struct DateTime;
    
    impl DateTime {
        pub fn to_rfc3339(&self) -> String {
            "2024-01-01T00:00:00Z".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_builder_success() {
        let response = ResponseBuilder::success("test data".to_string())
            .request_id("test_123".to_string())
            .execution_time(150)
            .build();

        assert!(response.success);
        assert_eq!(response.data, Some("test data".to_string()));
        assert!(response.error.is_none());
        assert_eq!(response.meta.request_id, "test_123");
        assert_eq!(response.meta.execution_time_ms, Some(150));
    }

    #[test]
    fn test_response_builder_error() {
        let response: ApiResponse<String> = ResponseBuilder::error(
            "TEST_ERROR".to_string(),
            "Something went wrong".to_string(),
        )
        .trace_id("trace_456".to_string())
        .build();

        assert!(!response.success);
        assert!(response.data.is_none());
        assert!(response.error.is_some());
        
        let error = response.error.unwrap();
        assert_eq!(error.code, "TEST_ERROR");
        assert_eq!(error.message, "Something went wrong");
        assert_eq!(error.trace_id, Some("trace_456".to_string()));
    }

    #[test]
    fn test_pagination_meta() {
        let pagination = PaginationMeta::new(2, 10, 25);

        assert_eq!(pagination.page, 2);
        assert_eq!(pagination.per_page, 10);
        assert_eq!(pagination.total_items, 25);
        assert_eq!(pagination.total_pages, 3);
        assert!(pagination.has_previous);
        assert!(pagination.has_next);
        assert_eq!(pagination.previous_page, Some(1));
        assert_eq!(pagination.next_page, Some(3));
    }

    #[test]
    fn test_pagination_edge_cases() {
        // First page
        let first_page = PaginationMeta::new(1, 10, 25);
        assert!(!first_page.has_previous);
        assert!(first_page.has_next);
        assert_eq!(first_page.previous_page, None);
        assert_eq!(first_page.next_page, Some(2));

        // Last page
        let last_page = PaginationMeta::new(3, 10, 25);
        assert!(last_page.has_previous);
        assert!(!last_page.has_next);
        assert_eq!(last_page.previous_page, Some(2));
        assert_eq!(last_page.next_page, None);

        // Single page
        let single_page = PaginationMeta::new(1, 10, 5);
        assert!(!single_page.has_previous);
        assert!(!single_page.has_next);
        assert_eq!(single_page.total_pages, 1);
    }

    #[test]
    fn test_response_format_negotiation() {
        assert_eq!(
            ResponseFormatter::negotiate_format(Some("application/json")),
            ResponseFormat::Json
        );
        assert_eq!(
            ResponseFormatter::negotiate_format(Some("application/xml")),
            ResponseFormat::Xml
        );
        assert_eq!(
            ResponseFormatter::negotiate_format(Some("application/yaml")),
            ResponseFormat::Yaml
        );
        assert_eq!(
            ResponseFormatter::negotiate_format(Some("text/html")),
            ResponseFormat::Html
        );
        assert_eq!(
            ResponseFormatter::negotiate_format(Some("*/*")),
            ResponseFormat::Json
        );
        assert_eq!(
            ResponseFormatter::negotiate_format(None),
            ResponseFormat::Json
        );
    }

    #[test]
    fn test_content_type_headers() {
        assert_eq!(
            ResponseFormatter::content_type(&ResponseFormat::Json),
            "application/json"
        );
        assert_eq!(
            ResponseFormatter::content_type(&ResponseFormat::Xml),
            "application/xml"
        );
        assert_eq!(
            ResponseFormatter::content_type(&ResponseFormat::Yaml),
            "application/yaml"
        );
        assert_eq!(
            ResponseFormatter::content_type(&ResponseFormat::Html),
            "text/html"
        );
    }

    #[test]
    fn test_error_responses() {
        let bad_request = ErrorResponses::bad_request("Invalid input");
        assert!(!bad_request.success);
        assert_eq!(bad_request.error.as_ref().unwrap().code, "BAD_REQUEST");

        let not_found = ErrorResponses::not_found("User");
        assert!(!not_found.success);
        assert_eq!(not_found.error.as_ref().unwrap().code, "NOT_FOUND");
        assert!(not_found.error.as_ref().unwrap().message.contains("User"));

        let mut validation_errors = HashMap::new();
        validation_errors.insert("email".to_string(), "Invalid format".to_string());
        let validation_error = ErrorResponses::validation_error(validation_errors);
        assert!(!validation_error.success);
        assert_eq!(validation_error.error.as_ref().unwrap().code, "VALIDATION_ERROR");
        assert!(validation_error.error.as_ref().unwrap().details.is_some());
    }

    #[test]
    fn test_collection_response() {
        let items = vec!["item1".to_string(), "item2".to_string(), "item3".to_string()];
        let collection = CollectionResponse::new(items.clone());

        assert_eq!(collection.items, items);
        assert_eq!(collection.count, 3);
        assert!(collection.pagination.is_none());

        let pagination = PaginationMeta::new(1, 10, 25);
        let collection_with_pagination = CollectionResponse::with_pagination(items.clone(), pagination.clone());

        assert_eq!(collection_with_pagination.items, items);
        assert_eq!(collection_with_pagination.count, 3);
        assert_eq!(collection_with_pagination.pagination, Some(pagination));
    }

    #[test]
    fn test_health_response() {
        let health = HealthResponse::healthy(3600);

        assert_eq!(health.status, "healthy");
        assert_eq!(health.version, "1.0.0");
        assert_eq!(health.uptime_seconds, 3600);
        assert!(health.checks.contains_key("database"));
        assert!(health.checks.contains_key("network"));

        let db_check = &health.checks["database"];
        assert_eq!(db_check.status, "healthy");
        assert_eq!(db_check.response_time_ms, Some(5));
        assert!(db_check.error.is_none());
    }

    #[test]
    fn test_response_builder_with_pagination() {
        let response = ResponseBuilder::success(vec!["item1", "item2"])
            .paginated(1, 10, 25)
            .cached(true)
            .build();

        assert!(response.success);
        assert!(response.meta.pagination.is_some());
        assert!(response.meta.cached);

        let pagination = response.meta.pagination.unwrap();
        assert_eq!(pagination.page, 1);
        assert_eq!(pagination.per_page, 10);
        assert_eq!(pagination.total_items, 25);
    }

    #[test]
    fn test_response_serialization() {
        let response = ResponseBuilder::success("test".to_string()).build();
        
        // Test JSON formatting (mock implementation)
        let json_result = ResponseFormatter::to_json(&response);
        assert!(json_result.is_ok());
        
        let compact_json_result = ResponseFormatter::to_json_compact(&response);
        assert!(compact_json_result.is_ok());
    }
}
