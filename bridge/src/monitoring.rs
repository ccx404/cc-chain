//! Bridge monitoring functionality

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Bridge monitoring metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeMetrics {
    /// Transfer metrics
    pub transfers: TransferMetrics,
    /// Validator metrics
    pub validators: ValidatorMetrics,
    /// Performance metrics
    pub performance: PerformanceMetrics,
    /// Security metrics
    pub security: SecurityMetrics,
    /// System health metrics
    pub health: HealthMetrics,
}

/// Transfer-related metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferMetrics {
    /// Total transfers processed
    pub total_transfers: u64,
    /// Successful transfers
    pub successful_transfers: u64,
    /// Failed transfers
    pub failed_transfers: u64,
    /// Pending transfers
    pub pending_transfers: u64,
    /// Total volume transferred
    pub total_volume: u64,
    /// Average transfer time (seconds)
    pub avg_transfer_time: f64,
    /// Transfer success rate
    pub success_rate: f64,
    /// Transfers per hour
    pub transfers_per_hour: f64,
}

/// Validator-related metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorMetrics {
    /// Total validators
    pub total_validators: u32,
    /// Active validators
    pub active_validators: u32,
    /// Inactive validators
    pub inactive_validators: u32,
    /// Jailed validators
    pub jailed_validators: u32,
    /// Average validator uptime
    pub avg_uptime: f64,
    /// Validator consensus participation rate
    pub consensus_participation_rate: f64,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Average response time (ms)
    pub avg_response_time: f64,
    /// Throughput (operations per second)
    pub throughput: f64,
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage percentage
    pub memory_usage: f64,
    /// Network latency (ms)
    pub network_latency: f64,
    /// Database response time (ms)
    pub db_response_time: f64,
}

/// Security metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMetrics {
    /// Failed authentication attempts
    pub failed_auth_attempts: u64,
    /// Suspicious activity count
    pub suspicious_activities: u64,
    /// Security alerts
    pub security_alerts: u64,
    /// Last security scan timestamp
    pub last_security_scan: u64,
    /// Known vulnerabilities
    pub known_vulnerabilities: u32,
}

/// Health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    /// System uptime (seconds)
    pub uptime: u64,
    /// Last restart timestamp
    pub last_restart: u64,
    /// Error rate
    pub error_rate: f64,
    /// Available disk space (bytes)
    pub available_disk_space: u64,
    /// Network connectivity status
    pub network_status: NetworkStatus,
    /// Database connection status
    pub database_status: DatabaseStatus,
}

/// Network connectivity status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkStatus {
    /// All connections healthy
    Healthy,
    /// Some connections degraded
    Degraded,
    /// Network issues detected
    Unhealthy,
    /// Network completely down
    Down,
}

/// Database connection status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseStatus {
    /// Database connections healthy
    Healthy,
    /// Database performance degraded
    Degraded,
    /// Database connection issues
    Unhealthy,
    /// Database unavailable
    Down,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    /// Information only
    Info,
    /// Warning condition
    Warning,
    /// Error condition
    Error,
    /// Critical condition requiring immediate attention
    Critical,
}

/// Bridge alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeAlert {
    /// Alert ID
    pub id: String,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert title
    pub title: String,
    /// Alert description
    pub description: String,
    /// Alert category
    pub category: String,
    /// Alert metadata
    pub metadata: HashMap<String, String>,
    /// Alert timestamp
    pub timestamp: u64,
    /// Alert acknowledged flag
    pub acknowledged: bool,
    /// Alert resolved flag
    pub resolved: bool,
}

/// Bridge monitoring system
pub struct BridgeMonitor {
    /// Current metrics
    metrics: BridgeMetrics,
    /// Active alerts
    active_alerts: HashMap<String, BridgeAlert>,
    /// Alert history
    alert_history: Vec<BridgeAlert>,
    /// Monitoring configuration
    config: MonitoringConfig,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Metrics collection interval (seconds)
    pub metrics_interval: u64,
    /// Alert check interval (seconds)
    pub alert_interval: u64,
    /// Maximum alerts to keep in memory
    pub max_active_alerts: usize,
    /// Alert thresholds
    pub thresholds: AlertThresholds,
}

/// Alert threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Transfer failure rate threshold (0.0-1.0)
    pub transfer_failure_rate: f64,
    /// Response time threshold (ms)
    pub response_time_threshold: f64,
    /// CPU usage threshold (0.0-1.0)
    pub cpu_usage_threshold: f64,
    /// Memory usage threshold (0.0-1.0)
    pub memory_usage_threshold: f64,
    /// Validator participation threshold (0.0-1.0)
    pub validator_participation_threshold: f64,
    /// Security alert threshold
    pub security_alert_threshold: u64,
}

impl BridgeMonitor {
    /// Create a new bridge monitor
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            metrics: BridgeMetrics::default(),
            active_alerts: HashMap::new(),
            alert_history: Vec::new(),
            config,
        }
    }
    
    /// Update metrics
    pub fn update_metrics(&mut self, metrics: BridgeMetrics) {
        self.metrics = metrics;
        self.check_alerts();
    }
    
    /// Get current metrics
    pub fn get_metrics(&self) -> &BridgeMetrics {
        &self.metrics
    }
    
    /// Check for alert conditions
    fn check_alerts(&mut self) {
        // Check transfer failure rate
        if self.metrics.transfers.success_rate < self.config.thresholds.transfer_failure_rate {
            self.create_alert(
                AlertSeverity::Error,
                "High Transfer Failure Rate".to_string(),
                format!("Transfer success rate is {:.2}%, below threshold of {:.2}%", 
                    self.metrics.transfers.success_rate * 100.0,
                    self.config.thresholds.transfer_failure_rate * 100.0),
                "transfers".to_string(),
            );
        }
        
        // Check response time
        if self.metrics.performance.avg_response_time > self.config.thresholds.response_time_threshold {
            self.create_alert(
                AlertSeverity::Warning,
                "High Response Time".to_string(),
                format!("Average response time is {:.2}ms, above threshold of {:.2}ms",
                    self.metrics.performance.avg_response_time,
                    self.config.thresholds.response_time_threshold),
                "performance".to_string(),
            );
        }
        
        // Check CPU usage
        if self.metrics.performance.cpu_usage > self.config.thresholds.cpu_usage_threshold {
            self.create_alert(
                AlertSeverity::Warning,
                "High CPU Usage".to_string(),
                format!("CPU usage is {:.2}%, above threshold of {:.2}%",
                    self.metrics.performance.cpu_usage * 100.0,
                    self.config.thresholds.cpu_usage_threshold * 100.0),
                "performance".to_string(),
            );
        }
        
        // Check memory usage
        if self.metrics.performance.memory_usage > self.config.thresholds.memory_usage_threshold {
            self.create_alert(
                AlertSeverity::Warning,
                "High Memory Usage".to_string(),
                format!("Memory usage is {:.2}%, above threshold of {:.2}%",
                    self.metrics.performance.memory_usage * 100.0,
                    self.config.thresholds.memory_usage_threshold * 100.0),
                "performance".to_string(),
            );
        }
        
        // Check validator participation
        if self.metrics.validators.consensus_participation_rate < self.config.thresholds.validator_participation_threshold {
            self.create_alert(
                AlertSeverity::Error,
                "Low Validator Participation".to_string(),
                format!("Validator participation rate is {:.2}%, below threshold of {:.2}%",
                    self.metrics.validators.consensus_participation_rate * 100.0,
                    self.config.thresholds.validator_participation_threshold * 100.0),
                "validators".to_string(),
            );
        }
    }
    
    /// Create a new alert
    fn create_alert(
        &mut self,
        severity: AlertSeverity,
        title: String,
        description: String,
        category: String,
    ) {
        let alert_id = format!("alert_{}_{}", 
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            rand::random::<u32>()
        );
        
        let alert = BridgeAlert {
            id: alert_id.clone(),
            severity,
            title,
            description,
            category,
            metadata: HashMap::new(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            acknowledged: false,
            resolved: false,
        };
        
        // Only add if we don't have too many alerts
        if self.active_alerts.len() < self.config.max_active_alerts {
            self.active_alerts.insert(alert_id, alert);
        }
    }
    
    /// Get active alerts
    pub fn get_active_alerts(&self) -> Vec<&BridgeAlert> {
        self.active_alerts.values().collect()
    }
    
    /// Acknowledge an alert
    pub fn acknowledge_alert(&mut self, alert_id: &str) -> Result<(), String> {
        if let Some(alert) = self.active_alerts.get_mut(alert_id) {
            alert.acknowledged = true;
            Ok(())
        } else {
            Err("Alert not found".to_string())
        }
    }
    
    /// Resolve an alert
    pub fn resolve_alert(&mut self, alert_id: &str) -> Result<(), String> {
        if let Some(mut alert) = self.active_alerts.remove(alert_id) {
            alert.resolved = true;
            self.alert_history.push(alert);
            Ok(())
        } else {
            Err("Alert not found".to_string())
        }
    }
    
    /// Get alert history
    pub fn get_alert_history(&self) -> &[BridgeAlert] {
        &self.alert_history
    }
}

impl Default for BridgeMetrics {
    fn default() -> Self {
        Self {
            transfers: TransferMetrics {
                total_transfers: 0,
                successful_transfers: 0,
                failed_transfers: 0,
                pending_transfers: 0,
                total_volume: 0,
                avg_transfer_time: 0.0,
                success_rate: 1.0,
                transfers_per_hour: 0.0,
            },
            validators: ValidatorMetrics {
                total_validators: 0,
                active_validators: 0,
                inactive_validators: 0,
                jailed_validators: 0,
                avg_uptime: 1.0,
                consensus_participation_rate: 1.0,
            },
            performance: PerformanceMetrics {
                avg_response_time: 0.0,
                throughput: 0.0,
                cpu_usage: 0.0,
                memory_usage: 0.0,
                network_latency: 0.0,
                db_response_time: 0.0,
            },
            security: SecurityMetrics {
                failed_auth_attempts: 0,
                suspicious_activities: 0,
                security_alerts: 0,
                last_security_scan: 0,
                known_vulnerabilities: 0,
            },
            health: HealthMetrics {
                uptime: 0,
                last_restart: 0,
                error_rate: 0.0,
                available_disk_space: 0,
                network_status: NetworkStatus::Healthy,
                database_status: DatabaseStatus::Healthy,
            },
        }
    }
}