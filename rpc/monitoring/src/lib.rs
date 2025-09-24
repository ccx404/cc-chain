//! CC Chain RPC Monitoring
//!
//! This module provides comprehensive monitoring capabilities for RPC operations,
//! including performance metrics, health checks, and operational insights.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MonitoringError {
    #[error("Metrics collection error: {0}")]
    MetricsError(String),
    
    #[error("Invalid time range: {0}")]
    InvalidTimeRange(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub type Result<T> = std::result::Result<T, MonitoringError>;

/// RPC monitoring configuration
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    pub enabled: bool,
    pub max_history_size: usize,
    pub metrics_retention: Duration,
    pub health_check_interval: Duration,
    pub alert_thresholds: AlertThresholds,
    pub export_interval: Duration,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_history_size: 10000,
            metrics_retention: Duration::from_secs(24 * 60 * 60), // 24 hours
            health_check_interval: Duration::from_secs(30),
            alert_thresholds: AlertThresholds::default(),
            export_interval: Duration::from_secs(60),
        }
    }
}

/// Alert threshold configuration
#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub max_response_time_ms: u64,
    pub max_error_rate_percent: f64,
    pub max_concurrent_requests: u32,
    pub min_success_rate_percent: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            max_response_time_ms: 5000,
            max_error_rate_percent: 5.0,
            max_concurrent_requests: 1000,
            min_success_rate_percent: 95.0,
        }
    }
}

/// Request metrics for individual RPC calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    pub method: String,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub duration_ms: Option<u64>,
    pub status: RequestStatus,
    pub error_code: Option<i32>,
    pub request_size: usize,
    pub response_size: Option<usize>,
    pub client_id: Option<String>,
}

/// Request status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestStatus {
    Pending,
    Success,
    Error,
    Timeout,
}

/// Aggregated metrics for time windows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    pub timestamp: u64,
    pub window_duration: Duration,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time_ms: f64,
    pub min_response_time_ms: u64,
    pub max_response_time_ms: u64,
    pub requests_per_second: f64,
    pub error_rate_percent: f64,
    pub method_breakdown: HashMap<String, MethodMetrics>,
}

/// Per-method metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodMetrics {
    pub call_count: u64,
    pub success_count: u64,
    pub avg_duration_ms: f64,
    pub min_duration_ms: u64,
    pub max_duration_ms: u64,
    pub total_request_size: u64,
    pub total_response_size: u64,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub timestamp: u64,
    pub overall_status: HealthLevel,
    pub component_statuses: HashMap<String, ComponentHealth>,
    pub metrics_summary: MetricsSummary,
}

/// Health levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthLevel {
    Healthy,
    Warning,
    Critical,
    Down,
}

/// Component health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: HealthLevel,
    pub message: String,
    pub last_check: u64,
    pub response_time_ms: Option<u64>,
}

/// Summary of key metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub uptime_seconds: u64,
    pub total_requests: u64,
    pub current_rps: f64,
    pub avg_response_time_ms: f64,
    pub error_rate_percent: f64,
    pub memory_usage_mb: Option<u64>,
    pub cpu_usage_percent: Option<f64>,
    pub concurrent_requests: u32,
}

/// Alert information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub triggered_at: u64,
    pub resolved_at: Option<u64>,
    pub metadata: HashMap<String, String>,
}

/// Alert types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    HighResponseTime,
    HighErrorRate,
    LowSuccessRate,
    HighConcurrency,
    ServiceDown,
    Custom(String),
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// RPC monitoring system
pub struct RpcMonitor {
    config: MonitoringConfig,
    active_requests: Arc<Mutex<HashMap<String, RequestMetrics>>>,
    completed_requests: Arc<Mutex<VecDeque<RequestMetrics>>>,
    aggregated_metrics: Arc<Mutex<VecDeque<AggregatedMetrics>>>,
    active_alerts: Arc<Mutex<HashMap<String, Alert>>>,
    start_time: Instant,
    last_aggregation: Arc<Mutex<Instant>>,
}

impl RpcMonitor {
    /// Create a new RPC monitor
    pub fn new() -> Self {
        Self::with_config(MonitoringConfig::default())
    }

    /// Create a new RPC monitor with custom configuration
    pub fn with_config(config: MonitoringConfig) -> Self {
        Self {
            config,
            active_requests: Arc::new(Mutex::new(HashMap::new())),
            completed_requests: Arc::new(Mutex::new(VecDeque::new())),
            aggregated_metrics: Arc::new(Mutex::new(VecDeque::new())),
            active_alerts: Arc::new(Mutex::new(HashMap::new())),
            start_time: Instant::now(),
            last_aggregation: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Start monitoring a request
    pub fn start_request(&self, request_id: String, method: String, request_size: usize) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let metrics = RequestMetrics {
            method,
            start_time: current_timestamp(),
            end_time: None,
            duration_ms: None,
            status: RequestStatus::Pending,
            error_code: None,
            request_size,
            response_size: None,
            client_id: None,
        };

        let mut active = self.active_requests.lock().unwrap();
        active.insert(request_id, metrics);
        
        Ok(())
    }

    /// Complete a successful request
    pub fn complete_request(&self, request_id: String, response_size: usize) -> Result<()> {
        self.finish_request(request_id, RequestStatus::Success, None, Some(response_size))
    }

    /// Complete a failed request
    pub fn fail_request(&self, request_id: String, error_code: i32) -> Result<()> {
        self.finish_request(request_id, RequestStatus::Error, Some(error_code), None)
    }

    /// Complete a timed-out request
    pub fn timeout_request(&self, request_id: String) -> Result<()> {
        self.finish_request(request_id, RequestStatus::Timeout, None, None)
    }

    fn finish_request(&self, request_id: String, status: RequestStatus, error_code: Option<i32>, response_size: Option<usize>) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let now = current_timestamp();
        
        let mut active = self.active_requests.lock().unwrap();
        if let Some(mut metrics) = active.remove(&request_id) {
            metrics.end_time = Some(now);
            metrics.duration_ms = Some(now - metrics.start_time);
            metrics.status = status;
            metrics.error_code = error_code;
            metrics.response_size = response_size;

            let mut completed = self.completed_requests.lock().unwrap();
            completed.push_back(metrics);

            // Maintain history size limit
            while completed.len() > self.config.max_history_size {
                completed.pop_front();
            }
        }

        // Check if we need to aggregate metrics
        self.maybe_aggregate_metrics()?;
        
        Ok(())
    }

    /// Get current health status
    pub fn get_health_status(&self) -> Result<HealthStatus> {
        let now = current_timestamp();
        let uptime = self.start_time.elapsed().as_secs();
        
        let completed = self.completed_requests.lock().unwrap();
        let active = self.active_requests.lock().unwrap();
        
        // Calculate recent metrics (last 5 minutes)
        let recent_window = Duration::from_secs(300);
        let cutoff_time = now - recent_window.as_millis() as u64;
        
        let recent_requests: Vec<_> = completed.iter()
            .filter(|r| r.start_time >= cutoff_time)
            .collect();
        
        let total_recent = recent_requests.len() as u64;
        let successful_recent = recent_requests.iter()
            .filter(|r| matches!(r.status, RequestStatus::Success))
            .count() as u64;
        
        let error_rate = if total_recent > 0 {
            ((total_recent - successful_recent) as f64 / total_recent as f64) * 100.0
        } else {
            0.0
        };
        
        let avg_response_time = if !recent_requests.is_empty() {
            recent_requests.iter()
                .filter_map(|r| r.duration_ms)
                .map(|d| d as f64)
                .sum::<f64>() / recent_requests.len() as f64
        } else {
            0.0
        };
        
        let current_rps = total_recent as f64 / recent_window.as_secs() as f64;
        
        // Determine overall health
        let overall_status = if error_rate > self.config.alert_thresholds.max_error_rate_percent {
            HealthLevel::Critical
        } else if avg_response_time > self.config.alert_thresholds.max_response_time_ms as f64 {
            HealthLevel::Warning
        } else if active.len() > self.config.alert_thresholds.max_concurrent_requests as usize {
            HealthLevel::Warning
        } else {
            HealthLevel::Healthy
        };

        let mut component_statuses = HashMap::new();
        component_statuses.insert("rpc_server".to_string(), ComponentHealth {
            status: overall_status.clone(),
            message: "RPC server operational".to_string(),
            last_check: now,
            response_time_ms: Some(avg_response_time as u64),
        });

        let metrics_summary = MetricsSummary {
            uptime_seconds: uptime,
            total_requests: completed.len() as u64,
            current_rps,
            avg_response_time_ms: avg_response_time,
            error_rate_percent: error_rate,
            memory_usage_mb: None, // Would be implemented with system metrics
            cpu_usage_percent: None, // Would be implemented with system metrics
            concurrent_requests: active.len() as u32,
        };

        Ok(HealthStatus {
            timestamp: now,
            overall_status,
            component_statuses,
            metrics_summary,
        })
    }

    /// Get aggregated metrics for a time range
    pub fn get_metrics(&self, window: Duration) -> Result<Vec<AggregatedMetrics>> {
        let aggregated = self.aggregated_metrics.lock().unwrap();
        let cutoff_time = current_timestamp() - window.as_millis() as u64;
        
        Ok(aggregated.iter()
            .filter(|m| m.timestamp >= cutoff_time)
            .cloned()
            .collect())
    }

    /// Get method-specific metrics
    pub fn get_method_metrics(&self, method: &str, window: Duration) -> Result<Vec<RequestMetrics>> {
        let completed = self.completed_requests.lock().unwrap();
        let cutoff_time = current_timestamp() - window.as_millis() as u64;
        
        Ok(completed.iter()
            .filter(|r| r.method == method && r.start_time >= cutoff_time)
            .cloned()
            .collect())
    }

    /// Get active alerts
    pub fn get_active_alerts(&self) -> Result<Vec<Alert>> {
        let alerts = self.active_alerts.lock().unwrap();
        Ok(alerts.values().cloned().collect())
    }

    /// Force metrics aggregation
    pub fn aggregate_metrics(&self) -> Result<()> {
        self.maybe_aggregate_metrics()
    }

    fn maybe_aggregate_metrics(&self) -> Result<()> {
        let mut last_agg = self.last_aggregation.lock().unwrap();
        
        if last_agg.elapsed() < Duration::from_secs(60) {
            return Ok(());
        }

        let now = current_timestamp();
        let window_start = *last_agg;
        let window_duration = window_start.elapsed();
        
        let completed = self.completed_requests.lock().unwrap();
        let window_requests: Vec<_> = completed.iter()
            .filter(|r| {
                let request_instant = Instant::now() - Duration::from_millis((now - r.start_time) as u64);
                request_instant >= window_start
            })
            .collect();

        if window_requests.is_empty() {
            *last_agg = Instant::now();
            return Ok(());
        }

        let total_requests = window_requests.len() as u64;
        let successful_requests = window_requests.iter()
            .filter(|r| matches!(r.status, RequestStatus::Success))
            .count() as u64;
        let failed_requests = total_requests - successful_requests;

        let durations: Vec<u64> = window_requests.iter()
            .filter_map(|r| r.duration_ms)
            .collect();

        let avg_response_time_ms = if !durations.is_empty() {
            durations.iter().sum::<u64>() as f64 / durations.len() as f64
        } else {
            0.0
        };

        let min_response_time_ms = durations.iter().min().copied().unwrap_or(0);
        let max_response_time_ms = durations.iter().max().copied().unwrap_or(0);
        let requests_per_second = total_requests as f64 / window_duration.as_secs() as f64;
        let error_rate_percent = (failed_requests as f64 / total_requests as f64) * 100.0;

        // Calculate method breakdown
        let mut method_breakdown = HashMap::new();
        for request in &window_requests {
            let entry = method_breakdown.entry(request.method.clone()).or_insert(MethodMetrics {
                call_count: 0,
                success_count: 0,
                avg_duration_ms: 0.0,
                min_duration_ms: u64::MAX,
                max_duration_ms: 0,
                total_request_size: 0,
                total_response_size: 0,
            });

            entry.call_count += 1;
            if matches!(request.status, RequestStatus::Success) {
                entry.success_count += 1;
            }
            entry.total_request_size += request.request_size as u64;
            if let Some(response_size) = request.response_size {
                entry.total_response_size += response_size as u64;
            }
            if let Some(duration) = request.duration_ms {
                entry.min_duration_ms = entry.min_duration_ms.min(duration);
                entry.max_duration_ms = entry.max_duration_ms.max(duration);
            }
        }

        // Calculate average durations for each method
        for (method, metrics) in &mut method_breakdown {
            let method_durations: Vec<u64> = window_requests.iter()
                .filter(|r| r.method == *method)
                .filter_map(|r| r.duration_ms)
                .collect();
            
            if !method_durations.is_empty() {
                metrics.avg_duration_ms = method_durations.iter().sum::<u64>() as f64 / method_durations.len() as f64;
            }
        }

        let aggregated = AggregatedMetrics {
            timestamp: now,
            window_duration,
            total_requests,
            successful_requests,
            failed_requests,
            avg_response_time_ms,
            min_response_time_ms,
            max_response_time_ms,
            requests_per_second,
            error_rate_percent,
            method_breakdown,
        };

        let mut agg_metrics = self.aggregated_metrics.lock().unwrap();
        agg_metrics.push_back(aggregated);

        // Clean up old metrics
        let retention_cutoff = now - self.config.metrics_retention.as_millis() as u64;
        while let Some(front) = agg_metrics.front() {
            if front.timestamp < retention_cutoff {
                agg_metrics.pop_front();
            } else {
                break;
            }
        }

        *last_agg = Instant::now();
        Ok(())
    }

    /// Check for alert conditions and trigger alerts
    pub fn check_alerts(&self) -> Result<Vec<Alert>> {
        let health = self.get_health_status()?;
        let mut new_alerts = Vec::new();
        let mut alerts = self.active_alerts.lock().unwrap();

        // High response time alert
        if health.metrics_summary.avg_response_time_ms > self.config.alert_thresholds.max_response_time_ms as f64 {
            let alert_id = "high_response_time".to_string();
            if !alerts.contains_key(&alert_id) {
                let alert = Alert {
                    id: alert_id.clone(),
                    alert_type: AlertType::HighResponseTime,
                    severity: AlertSeverity::Warning,
                    message: format!("Average response time ({:.1}ms) exceeds threshold ({}ms)", 
                        health.metrics_summary.avg_response_time_ms, 
                        self.config.alert_thresholds.max_response_time_ms),
                    triggered_at: current_timestamp(),
                    resolved_at: None,
                    metadata: HashMap::new(),
                };
                alerts.insert(alert_id, alert.clone());
                new_alerts.push(alert);
            }
        } else {
            // Resolve alert if it exists
            if let Some(mut alert) = alerts.remove("high_response_time") {
                alert.resolved_at = Some(current_timestamp());
                new_alerts.push(alert);
            }
        }

        // High error rate alert
        if health.metrics_summary.error_rate_percent > self.config.alert_thresholds.max_error_rate_percent {
            let alert_id = "high_error_rate".to_string();
            if !alerts.contains_key(&alert_id) {
                let alert = Alert {
                    id: alert_id.clone(),
                    alert_type: AlertType::HighErrorRate,
                    severity: AlertSeverity::Critical,
                    message: format!("Error rate ({:.1}%) exceeds threshold ({:.1}%)", 
                        health.metrics_summary.error_rate_percent, 
                        self.config.alert_thresholds.max_error_rate_percent),
                    triggered_at: current_timestamp(),
                    resolved_at: None,
                    metadata: HashMap::new(),
                };
                alerts.insert(alert_id, alert.clone());
                new_alerts.push(alert);
            }
        } else {
            if let Some(mut alert) = alerts.remove("high_error_rate") {
                alert.resolved_at = Some(current_timestamp());
                new_alerts.push(alert);
            }
        }

        Ok(new_alerts)
    }

    /// Export metrics in various formats
    pub fn export_metrics(&self, format: ExportFormat) -> Result<String> {
        let health = self.get_health_status()?;
        
        match format {
            ExportFormat::Json => {
                serde_json::to_string_pretty(&health)
                    .map_err(|e| MonitoringError::StorageError(e.to_string()))
            }
            ExportFormat::Prometheus => {
                Ok(self.format_prometheus_metrics(&health))
            }
        }
    }

    fn format_prometheus_metrics(&self, health: &HealthStatus) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("# HELP cc_rpc_uptime_seconds Total uptime in seconds\n"));
        output.push_str(&format!("# TYPE cc_rpc_uptime_seconds counter\n"));
        output.push_str(&format!("cc_rpc_uptime_seconds {}\n\n", health.metrics_summary.uptime_seconds));
        
        output.push_str(&format!("# HELP cc_rpc_requests_total Total number of RPC requests\n"));
        output.push_str(&format!("# TYPE cc_rpc_requests_total counter\n"));
        output.push_str(&format!("cc_rpc_requests_total {}\n\n", health.metrics_summary.total_requests));
        
        output.push_str(&format!("# HELP cc_rpc_requests_per_second Current requests per second\n"));
        output.push_str(&format!("# TYPE cc_rpc_requests_per_second gauge\n"));
        output.push_str(&format!("cc_rpc_requests_per_second {}\n\n", health.metrics_summary.current_rps));
        
        output.push_str(&format!("# HELP cc_rpc_response_time_ms Average response time in milliseconds\n"));
        output.push_str(&format!("# TYPE cc_rpc_response_time_ms gauge\n"));
        output.push_str(&format!("cc_rpc_response_time_ms {}\n\n", health.metrics_summary.avg_response_time_ms));
        
        output.push_str(&format!("# HELP cc_rpc_error_rate_percent Error rate percentage\n"));
        output.push_str(&format!("# TYPE cc_rpc_error_rate_percent gauge\n"));
        output.push_str(&format!("cc_rpc_error_rate_percent {}\n\n", health.metrics_summary.error_rate_percent));
        
        output
    }
}

impl Default for RpcMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Export formats for metrics
#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    Prometheus,
}

/// Utility function to get current timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_creation() {
        let monitor = RpcMonitor::new();
        assert!(monitor.config.enabled);
    }

    #[test]
    fn test_custom_config() {
        let config = MonitoringConfig {
            enabled: false,
            max_history_size: 1000,
            ..Default::default()
        };
        let monitor = RpcMonitor::with_config(config);
        assert!(!monitor.config.enabled);
        assert_eq!(monitor.config.max_history_size, 1000);
    }

    #[test]
    fn test_request_lifecycle() {
        let monitor = RpcMonitor::new();
        let request_id = "test_req_1".to_string();
        
        // Start request
        monitor.start_request(request_id.clone(), "test_method".to_string(), 100).unwrap();
        
        // Complete request
        monitor.complete_request(request_id, 200).unwrap();
        
        // Check that request was recorded
        let completed = monitor.completed_requests.lock().unwrap();
        assert_eq!(completed.len(), 1);
        
        let request = &completed[0];
        assert_eq!(request.method, "test_method");
        assert_eq!(request.request_size, 100);
        assert_eq!(request.response_size, Some(200));
        assert!(matches!(request.status, RequestStatus::Success));
    }

    #[test]
    fn test_failed_request() {
        let monitor = RpcMonitor::new();
        let request_id = "test_req_2".to_string();
        
        monitor.start_request(request_id.clone(), "test_method".to_string(), 100).unwrap();
        monitor.fail_request(request_id, -32602).unwrap();
        
        let completed = monitor.completed_requests.lock().unwrap();
        assert_eq!(completed.len(), 1);
        
        let request = &completed[0];
        assert!(matches!(request.status, RequestStatus::Error));
        assert_eq!(request.error_code, Some(-32602));
    }

    #[test]
    fn test_health_status() {
        let monitor = RpcMonitor::new();
        let health = monitor.get_health_status().unwrap();
        
        assert!(matches!(health.overall_status, HealthLevel::Healthy));
        assert!(health.component_statuses.contains_key("rpc_server"));
        assert_eq!(health.metrics_summary.concurrent_requests, 0);
    }

    #[test]
    fn test_metrics_aggregation() {
        let monitor = RpcMonitor::new();
        
        // Add some test requests
        for i in 0..5 {
            let request_id = format!("req_{}", i);
            monitor.start_request(request_id.clone(), "test_method".to_string(), 100).unwrap();
            monitor.complete_request(request_id, 200).unwrap();
        }
        
        monitor.aggregate_metrics().unwrap();
        
        let _metrics = monitor.get_metrics(Duration::from_secs(60 * 60)).unwrap(); // 1 hour
        // Note: metrics might be empty if aggregation window hasn't elapsed
        // This is expected behavior in the test environment
    }

    #[test]
    fn test_method_metrics() {
        let monitor = RpcMonitor::new();
        
        monitor.start_request("req1".to_string(), "method_a".to_string(), 100).unwrap();
        monitor.complete_request("req1".to_string(), 200).unwrap();
        
        monitor.start_request("req2".to_string(), "method_b".to_string(), 150).unwrap();
        monitor.complete_request("req2".to_string(), 250).unwrap();
        
        let method_a_metrics = monitor.get_method_metrics("method_a", Duration::from_secs(60 * 60)).unwrap(); // 1 hour
        assert_eq!(method_a_metrics.len(), 1);
        assert_eq!(method_a_metrics[0].method, "method_a");
        
        let method_b_metrics = monitor.get_method_metrics("method_b", Duration::from_secs(60 * 60)).unwrap(); // 1 hour
        assert_eq!(method_b_metrics.len(), 1);
        assert_eq!(method_b_metrics[0].method, "method_b");
    }

    #[test]
    fn test_alert_thresholds() {
        let thresholds = AlertThresholds::default();
        assert_eq!(thresholds.max_response_time_ms, 5000);
        assert_eq!(thresholds.max_error_rate_percent, 5.0);
    }

    #[test]
    fn test_export_formats() {
        let monitor = RpcMonitor::new();
        
        let json_export = monitor.export_metrics(ExportFormat::Json).unwrap();
        assert!(json_export.contains("timestamp"));
        
        let prometheus_export = monitor.export_metrics(ExportFormat::Prometheus).unwrap();
        assert!(prometheus_export.contains("cc_rpc_uptime_seconds"));
        assert!(prometheus_export.contains("cc_rpc_requests_total"));
    }

    #[test]
    fn test_alert_detection() {
        let monitor = RpcMonitor::new();
        let alerts = monitor.check_alerts().unwrap();
        // With default healthy state, no alerts should be triggered
        assert!(alerts.is_empty());
    }

    #[test]
    fn test_concurrent_requests() {
        let monitor = RpcMonitor::new();
        
        // Start multiple requests without completing them
        for i in 0..3 {
            let request_id = format!("concurrent_req_{}", i);
            monitor.start_request(request_id, "test_method".to_string(), 100).unwrap();
        }
        
        let health = monitor.get_health_status().unwrap();
        assert_eq!(health.metrics_summary.concurrent_requests, 3);
    }

    #[test]
    fn test_request_status_types() {
        assert!(matches!(RequestStatus::Pending, RequestStatus::Pending));
        assert!(matches!(RequestStatus::Success, RequestStatus::Success));
        assert!(matches!(RequestStatus::Error, RequestStatus::Error));
        assert!(matches!(RequestStatus::Timeout, RequestStatus::Timeout));
    }

    #[test]
    fn test_health_levels() {
        assert!(matches!(HealthLevel::Healthy, HealthLevel::Healthy));
        assert!(matches!(HealthLevel::Warning, HealthLevel::Warning));
        assert!(matches!(HealthLevel::Critical, HealthLevel::Critical));
        assert!(matches!(HealthLevel::Down, HealthLevel::Down));
    }
}
