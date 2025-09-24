//! RPC Performance Monitoring and Metrics
//!
//! This module provides comprehensive performance monitoring capabilities for RPC operations,
//! including latency tracking, throughput measurement, and resource utilization metrics.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use thiserror::Error;

/// RPC performance monitoring error types
#[derive(Error, Debug)]
pub enum PerformanceError {
    #[error("Invalid metric name: {0}")]
    InvalidMetric(String),
    
    #[error("Timer not found: {0}")]
    TimerNotFound(String),
    
    #[error("Metric collection failed: {0}")]
    CollectionFailed(String),
}

pub type Result<T> = std::result::Result<T, PerformanceError>;

/// RPC performance metrics collector
#[derive(Debug, Clone)]
pub struct RpcPerformanceCollector {
    inner: Arc<Mutex<RpcPerformanceInner>>,
}

#[derive(Debug)]
struct RpcPerformanceInner {
    /// Active timing operations
    active_timers: HashMap<String, Instant>,
    
    /// Completed request metrics
    metrics: HashMap<String, RequestMetrics>,
    
    /// Global performance statistics
    global_stats: GlobalStats,
}

/// Performance metrics for a specific RPC method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    /// Method name
    pub method: String,
    
    /// Total number of requests
    pub total_requests: u64,
    
    /// Successful requests
    pub successful_requests: u64,
    
    /// Failed requests  
    pub failed_requests: u64,
    
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    
    /// Minimum response time in milliseconds
    pub min_response_time_ms: f64,
    
    /// Maximum response time in milliseconds
    pub max_response_time_ms: f64,
    
    /// Total response time (for calculating averages)
    total_response_time_ms: f64,
    
    /// Requests per second (calculated over last window)
    pub requests_per_second: f64,
    
    /// Last request timestamp
    pub last_request_time: Option<u64>,
}

/// Global RPC performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalStats {
    /// Total requests across all methods
    pub total_requests: u64,
    
    /// Total successful requests
    pub total_successful: u64,
    
    /// Total failed requests
    pub total_failed: u64,
    
    /// Overall success rate percentage
    pub success_rate: f64,
    
    /// Average response time across all methods
    pub global_avg_response_time_ms: f64,
    
    /// Peak requests per second
    pub peak_rps: f64,
    
    /// Current active connections
    pub active_connections: u64,
    
    /// Uptime in seconds
    pub uptime_seconds: u64,
    
    /// Start time timestamp (not serialized, reconstructed on creation)
    #[serde(skip, default = "Instant::now")]
    start_time: Instant,
}

/// Performance window for rate calculations
#[derive(Debug, Clone)]
pub struct PerformanceWindow {
    /// Window duration
    pub duration: Duration,
    
    /// Request count in this window
    pub request_count: u64,
    
    /// Window start time
    pub start_time: Instant,
}

impl RpcPerformanceCollector {
    /// Create a new performance collector
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(RpcPerformanceInner {
                active_timers: HashMap::new(),
                metrics: HashMap::new(),
                global_stats: GlobalStats::new(),
            })),
        }
    }
    
    /// Start timing an RPC request
    pub fn start_request(&self, method: &str, request_id: &str) -> Result<()> {
        let mut inner = self.inner.lock().unwrap();
        let timer_key = format!("{}:{}", method, request_id);
        inner.active_timers.insert(timer_key, Instant::now());
        Ok(())
    }
    
    /// Complete timing an RPC request
    pub fn complete_request(&self, method: &str, request_id: &str, success: bool) -> Result<Duration> {
        let mut inner = self.inner.lock().unwrap();
        let timer_key = format!("{}:{}", method, request_id);
        
        let start_time = inner.active_timers.remove(&timer_key)
            .ok_or_else(|| PerformanceError::TimerNotFound(timer_key))?;
            
        let duration = start_time.elapsed();
        let duration_ms = duration.as_secs_f64() * 1000.0;
        
        // Update method-specific metrics
        let metrics = inner.metrics.entry(method.to_string()).or_insert_with(|| {
            RequestMetrics::new(method.to_string())
        });
        
        metrics.total_requests += 1;
        if success {
            metrics.successful_requests += 1;
        } else {
            metrics.failed_requests += 1;
        }
        
        // Update timing statistics
        metrics.total_response_time_ms += duration_ms;
        metrics.avg_response_time_ms = metrics.total_response_time_ms / metrics.total_requests as f64;
        
        if metrics.min_response_time_ms == 0.0 || duration_ms < metrics.min_response_time_ms {
            metrics.min_response_time_ms = duration_ms;
        }
        
        if duration_ms > metrics.max_response_time_ms {
            metrics.max_response_time_ms = duration_ms;
        }
        
        metrics.last_request_time = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        );
        
        // Update global statistics
        inner.global_stats.total_requests += 1;
        if success {
            inner.global_stats.total_successful += 1;
        } else {
            inner.global_stats.total_failed += 1;
        }
        
        inner.global_stats.success_rate = 
            (inner.global_stats.total_successful as f64 / inner.global_stats.total_requests as f64) * 100.0;
        
        Ok(duration)
    }
    
    /// Get metrics for a specific RPC method
    pub fn get_method_metrics(&self, method: &str) -> Option<RequestMetrics> {
        let inner = self.inner.lock().unwrap();
        inner.metrics.get(method).cloned()
    }
    
    /// Get all method metrics
    pub fn get_all_metrics(&self) -> HashMap<String, RequestMetrics> {
        let inner = self.inner.lock().unwrap();
        inner.metrics.clone()
    }
    
    /// Get global performance statistics
    pub fn get_global_stats(&self) -> GlobalStats {
        let mut inner = self.inner.lock().unwrap();
        inner.global_stats.uptime_seconds = inner.global_stats.start_time.elapsed().as_secs();
        inner.global_stats.clone()
    }
    
    /// Reset all metrics
    pub fn reset_metrics(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.metrics.clear();
        inner.global_stats = GlobalStats::new();
    }
    
    /// Get top slowest methods
    pub fn get_slowest_methods(&self, limit: usize) -> Vec<RequestMetrics> {
        let inner = self.inner.lock().unwrap();
        let mut methods: Vec<_> = inner.metrics.values().cloned().collect();
        methods.sort_by(|a, b| b.avg_response_time_ms.partial_cmp(&a.avg_response_time_ms).unwrap_or(std::cmp::Ordering::Equal));
        methods.into_iter().take(limit).collect()
    }
    
    /// Get methods with highest error rates
    pub fn get_highest_error_methods(&self, limit: usize) -> Vec<RequestMetrics> {
        let inner = self.inner.lock().unwrap();
        let mut methods: Vec<_> = inner.metrics.values().cloned().collect();
        methods.sort_by(|a, b| {
            let error_rate_a = a.failed_requests as f64 / a.total_requests as f64;
            let error_rate_b = b.failed_requests as f64 / b.total_requests as f64;
            error_rate_b.partial_cmp(&error_rate_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        methods.into_iter().take(limit).collect()
    }
}

impl RequestMetrics {
    fn new(method: String) -> Self {
        Self {
            method,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_response_time_ms: 0.0,
            min_response_time_ms: 0.0,
            max_response_time_ms: 0.0,
            total_response_time_ms: 0.0,
            requests_per_second: 0.0,
            last_request_time: None,
        }
    }
    
    /// Calculate error rate percentage
    pub fn error_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.failed_requests as f64 / self.total_requests as f64) * 100.0
        }
    }
    
    /// Calculate success rate percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.successful_requests as f64 / self.total_requests as f64) * 100.0
        }
    }
}

impl GlobalStats {
    fn new() -> Self {
        Self {
            total_requests: 0,
            total_successful: 0,
            total_failed: 0,
            success_rate: 0.0,
            global_avg_response_time_ms: 0.0,
            peak_rps: 0.0,
            active_connections: 0,
            uptime_seconds: 0,
            start_time: Instant::now(),
        }
    }
}

impl Default for GlobalStats {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for RpcPerformanceCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    
    #[test]
    fn test_performance_collector_creation() {
        let collector = RpcPerformanceCollector::new();
        let stats = collector.get_global_stats();
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.success_rate, 0.0);
    }
    
    #[test]
    fn test_request_timing() {
        let collector = RpcPerformanceCollector::new();
        
        // Start timing a request
        collector.start_request("test_method", "req_1").unwrap();
        
        // Simulate some processing time
        thread::sleep(Duration::from_millis(10));
        
        // Complete the request
        let duration = collector.complete_request("test_method", "req_1", true).unwrap();
        assert!(duration.as_millis() >= 10);
        
        // Check metrics
        let metrics = collector.get_method_metrics("test_method").unwrap();
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.successful_requests, 1);
        assert_eq!(metrics.failed_requests, 0);
        assert!(metrics.avg_response_time_ms >= 10.0);
    }
    
    #[test]
    fn test_global_statistics() {
        let collector = RpcPerformanceCollector::new();
        
        // Process multiple requests
        for i in 0..5 {
            let request_id = format!("req_{}", i);
            collector.start_request("test_method", &request_id).unwrap();
            collector.complete_request("test_method", &request_id, i % 2 == 0).unwrap();
        }
        
        let stats = collector.get_global_stats();
        assert_eq!(stats.total_requests, 5);
        assert_eq!(stats.total_successful, 3);
        assert_eq!(stats.total_failed, 2);
        assert_eq!(stats.success_rate, 60.0);
    }
    
    #[test]
    fn test_error_rate_calculation() {
        let collector = RpcPerformanceCollector::new();
        
        // Add successful request
        collector.start_request("test_method", "req_1").unwrap();
        collector.complete_request("test_method", "req_1", true).unwrap();
        
        // Add failed request
        collector.start_request("test_method", "req_2").unwrap();
        collector.complete_request("test_method", "req_2", false).unwrap();
        
        let metrics = collector.get_method_metrics("test_method").unwrap();
        assert_eq!(metrics.error_rate(), 50.0);
        assert_eq!(metrics.success_rate(), 50.0);
    }
    
    #[test]
    fn test_slowest_methods() {
        let collector = RpcPerformanceCollector::new();
        
        // Create requests with different response times
        collector.start_request("fast_method", "req_1").unwrap();
        collector.complete_request("fast_method", "req_1", true).unwrap();
        
        collector.start_request("slow_method", "req_2").unwrap();
        thread::sleep(Duration::from_millis(20));
        collector.complete_request("slow_method", "req_2", true).unwrap();
        
        let slowest = collector.get_slowest_methods(2);
        assert_eq!(slowest.len(), 2);
        assert_eq!(slowest[0].method, "slow_method");
        assert!(slowest[0].avg_response_time_ms > slowest[1].avg_response_time_ms);
    }
}
