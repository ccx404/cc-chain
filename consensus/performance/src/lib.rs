//! CC Chain Consensus Performance Optimization
//!
//! This crate provides performance monitoring, optimization, and tuning
//! capabilities for the CC Chain consensus mechanism.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PerformanceError {
    #[error("Metrics collection error: {0}")]
    Metrics(String),
    #[error("Optimization error: {0}")]
    Optimization(String),
    #[error("Benchmark error: {0}")]
    Benchmark(String),
}

pub type Result<T> = std::result::Result<T, PerformanceError>;

/// Performance metrics for consensus operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusMetrics {
    pub round_duration: Duration,
    pub proposal_time: Duration,
    pub voting_time: Duration,
    pub commit_time: Duration,
    pub throughput: f64, // transactions per second
    pub latency_percentiles: LatencyPercentiles,
    pub resource_usage: ResourceUsage,
}

/// Latency percentile measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyPercentiles {
    pub p50: Duration,
    pub p90: Duration,
    pub p95: Duration,
    pub p99: Duration,
}

/// Resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_percent: f64,
    pub memory_mb: u64,
    pub network_bytes_in: u64,
    pub network_bytes_out: u64,
    pub disk_io_bytes: u64,
}

/// Performance monitor for consensus operations
#[derive(Debug)]
pub struct PerformanceMonitor {
    metrics_history: VecDeque<ConsensusMetrics>,
    round_timers: HashMap<u64, Instant>,
    operation_timers: HashMap<String, Instant>,
    throughput_tracker: ThroughputTracker,
    max_history_size: usize,
}

/// Throughput tracking utility
#[derive(Debug)]
pub struct ThroughputTracker {
    transaction_count: u64,
    window_start: Instant,
    window_duration: Duration,
    samples: VecDeque<(Instant, u64)>,
}

/// Consensus optimization engine
#[derive(Debug)]
pub struct OptimizationEngine {
    parameters: OptimizationParameters,
    performance_targets: PerformanceTargets,
    adaptation_history: Vec<AdaptationRecord>,
}

/// Tunable consensus parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationParameters {
    pub block_size_limit: u32,
    pub block_interval: Duration,
    pub timeout_propose: Duration,
    pub timeout_prevote: Duration,
    pub timeout_precommit: Duration,
    pub batch_size: u32,
    pub parallel_verification: bool,
}

/// Performance targets for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTargets {
    pub target_throughput: f64,
    pub max_latency: Duration,
    pub target_finality_time: Duration,
    pub max_cpu_usage: f64,
    pub max_memory_usage: u64,
}

/// Record of parameter adaptations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationRecord {
    pub timestamp: u64,
    pub parameters: OptimizationParameters,
    pub reason: String,
    pub performance_before: ConsensusMetrics,
    pub performance_after: Option<ConsensusMetrics>,
}

/// Consensus benchmark suite
#[derive(Debug)]
pub struct ConsensusBenchmark {
    scenarios: Vec<BenchmarkScenario>,
    results: HashMap<String, BenchmarkResult>,
}

/// Individual benchmark scenario
#[derive(Debug, Clone)]
pub struct BenchmarkScenario {
    pub name: String,
    pub description: String,
    pub validator_count: u32,
    pub transaction_rate: f64,
    pub duration: Duration,
    pub network_latency: Duration,
    pub byzantine_ratio: f64,
}

/// Benchmark execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub scenario_name: String,
    pub execution_time: Duration,
    pub average_metrics: ConsensusMetrics,
    pub peak_metrics: ConsensusMetrics,
    pub error_rate: f64,
    pub success_criteria_met: bool,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            metrics_history: VecDeque::new(),
            round_timers: HashMap::new(),
            operation_timers: HashMap::new(),
            throughput_tracker: ThroughputTracker::new(Duration::from_secs(10)),
            max_history_size: 1000,
        }
    }

    /// Start timing a consensus round
    pub fn start_round(&mut self, round: u64) {
        self.round_timers.insert(round, Instant::now());
    }

    /// End timing a consensus round and record metrics
    pub fn end_round(&mut self, round: u64, transaction_count: u64) -> Result<()> {
        if let Some(start_time) = self.round_timers.remove(&round) {
            let round_duration = start_time.elapsed();
            
            // Update throughput
            self.throughput_tracker.record_transactions(transaction_count);
            
            // Create metrics record
            let metrics = ConsensusMetrics {
                round_duration,
                proposal_time: self.get_operation_duration("proposal").unwrap_or_default(),
                voting_time: self.get_operation_duration("voting").unwrap_or_default(),
                commit_time: self.get_operation_duration("commit").unwrap_or_default(),
                throughput: self.throughput_tracker.current_throughput(),
                latency_percentiles: self.calculate_latency_percentiles(),
                resource_usage: self.collect_resource_usage(),
            };

            self.record_metrics(metrics);
            Ok(())
        } else {
            Err(PerformanceError::Metrics(format!("Round {} was not started", round)))
        }
    }

    /// Start timing a specific operation
    pub fn start_operation(&mut self, operation: &str) {
        self.operation_timers.insert(operation.to_string(), Instant::now());
    }

    /// End timing a specific operation
    pub fn end_operation(&mut self, operation: &str) -> Option<Duration> {
        self.operation_timers.remove(operation).map(|start| start.elapsed())
    }

    /// Get the average metrics over a time window
    pub fn get_average_metrics(&self, window: Duration) -> Option<ConsensusMetrics> {
        let cutoff = Instant::now() - window;
        let recent_metrics: Vec<_> = self.metrics_history
            .iter()
            .filter(|m| {
                // This is a simplified check; in practice, you'd store timestamps
                true
            })
            .collect();

        if recent_metrics.is_empty() {
            return None;
        }

        let avg_round_duration = recent_metrics.iter()
            .map(|m| m.round_duration)
            .sum::<Duration>() / recent_metrics.len() as u32;

        let avg_throughput = recent_metrics.iter()
            .map(|m| m.throughput)
            .sum::<f64>() / recent_metrics.len() as f64;

        Some(ConsensusMetrics {
            round_duration: avg_round_duration,
            proposal_time: Duration::default(),
            voting_time: Duration::default(),
            commit_time: Duration::default(),
            throughput: avg_throughput,
            latency_percentiles: LatencyPercentiles::default(),
            resource_usage: ResourceUsage::default(),
        })
    }

    /// Detect performance anomalies
    pub fn detect_anomalies(&self) -> Vec<PerformanceAnomaly> {
        let mut anomalies = Vec::new();

        if let Some(latest) = self.metrics_history.back() {
            // Check for high latency
            if latest.round_duration > Duration::from_secs(10) {
                anomalies.push(PerformanceAnomaly {
                    severity: AnomalySeverity::High,
                    description: "Consensus round duration exceeds 10 seconds".to_string(),
                    metric_name: "round_duration".to_string(),
                    value: latest.round_duration.as_secs_f64(),
                    threshold: 10.0,
                });
            }

            // Check for low throughput
            if latest.throughput < 100.0 {
                anomalies.push(PerformanceAnomaly {
                    severity: AnomalySeverity::Medium,
                    description: "Throughput below 100 TPS".to_string(),
                    metric_name: "throughput".to_string(),
                    value: latest.throughput,
                    threshold: 100.0,
                });
            }

            // Check for high resource usage
            if latest.resource_usage.cpu_percent > 90.0 {
                anomalies.push(PerformanceAnomaly {
                    severity: AnomalySeverity::High,
                    description: "CPU usage exceeds 90%".to_string(),
                    metric_name: "cpu_percent".to_string(),
                    value: latest.resource_usage.cpu_percent,
                    threshold: 90.0,
                });
            }
        }

        anomalies
    }

    fn get_operation_duration(&self, operation: &str) -> Option<Duration> {
        // In a real implementation, this would track operation durations
        Some(Duration::from_millis(50))
    }

    fn calculate_latency_percentiles(&self) -> LatencyPercentiles {
        // Simplified implementation
        LatencyPercentiles {
            p50: Duration::from_millis(100),
            p90: Duration::from_millis(200),
            p95: Duration::from_millis(300),
            p99: Duration::from_millis(500),
        }
    }

    fn collect_resource_usage(&self) -> ResourceUsage {
        // In a real implementation, this would collect actual system metrics
        ResourceUsage {
            cpu_percent: 45.0,
            memory_mb: 512,
            network_bytes_in: 1024 * 1024,
            network_bytes_out: 1024 * 1024,
            disk_io_bytes: 512 * 1024,
        }
    }

    fn record_metrics(&mut self, metrics: ConsensusMetrics) {
        self.metrics_history.push_back(metrics);
        
        // Maintain history size limit
        while self.metrics_history.len() > self.max_history_size {
            self.metrics_history.pop_front();
        }
    }
}

impl ThroughputTracker {
    /// Create a new throughput tracker
    pub fn new(window_duration: Duration) -> Self {
        Self {
            transaction_count: 0,
            window_start: Instant::now(),
            window_duration,
            samples: VecDeque::new(),
        }
    }

    /// Record processed transactions
    pub fn record_transactions(&mut self, count: u64) {
        self.transaction_count += count;
        let now = Instant::now();
        self.samples.push_back((now, count));

        // Remove old samples outside the window
        let cutoff = now - self.window_duration;
        while let Some(&(timestamp, _)) = self.samples.front() {
            if timestamp < cutoff {
                self.samples.pop_front();
            } else {
                break;
            }
        }
    }

    /// Get current throughput (transactions per second)
    pub fn current_throughput(&self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }

        let total_transactions: u64 = self.samples.iter().map(|(_, count)| count).sum();
        let window_seconds = self.window_duration.as_secs_f64();
        
        total_transactions as f64 / window_seconds
    }
}

impl OptimizationEngine {
    /// Create a new optimization engine
    pub fn new(parameters: OptimizationParameters, targets: PerformanceTargets) -> Self {
        Self {
            parameters,
            performance_targets: targets,
            adaptation_history: Vec::new(),
        }
    }

    /// Analyze current performance and suggest optimizations
    pub fn analyze_and_optimize(&mut self, current_metrics: &ConsensusMetrics) -> Result<Vec<OptimizationSuggestion>> {
        let mut suggestions = Vec::new();

        // Analyze throughput
        if current_metrics.throughput < self.performance_targets.target_throughput {
            suggestions.push(OptimizationSuggestion {
                parameter: "batch_size".to_string(),
                current_value: self.parameters.batch_size.to_string(),
                suggested_value: (self.parameters.batch_size * 2).to_string(),
                reason: "Increase batch size to improve throughput".to_string(),
                expected_impact: Impact::High,
            });
        }

        // Analyze latency
        if current_metrics.round_duration > self.performance_targets.max_latency {
            suggestions.push(OptimizationSuggestion {
                parameter: "timeout_propose".to_string(),
                current_value: format!("{:?}", self.parameters.timeout_propose),
                suggested_value: format!("{:?}", self.parameters.timeout_propose / 2),
                reason: "Reduce proposal timeout to decrease round duration".to_string(),
                expected_impact: Impact::Medium,
            });
        }

        // Analyze resource usage
        if current_metrics.resource_usage.cpu_percent > self.performance_targets.max_cpu_usage {
            suggestions.push(OptimizationSuggestion {
                parameter: "parallel_verification".to_string(),
                current_value: self.parameters.parallel_verification.to_string(),
                suggested_value: "true".to_string(),
                reason: "Enable parallel verification to distribute CPU load".to_string(),
                expected_impact: Impact::High,
            });
        }

        Ok(suggestions)
    }

    /// Apply optimization suggestions
    pub fn apply_optimization(&mut self, suggestion: &OptimizationSuggestion, current_metrics: ConsensusMetrics) -> Result<()> {
        let old_parameters = self.parameters.clone();

        match suggestion.parameter.as_str() {
            "batch_size" => {
                if let Ok(value) = suggestion.suggested_value.parse::<u32>() {
                    self.parameters.batch_size = value;
                }
            }
            "block_size_limit" => {
                if let Ok(value) = suggestion.suggested_value.parse::<u32>() {
                    self.parameters.block_size_limit = value;
                }
            }
            "parallel_verification" => {
                if let Ok(value) = suggestion.suggested_value.parse::<bool>() {
                    self.parameters.parallel_verification = value;
                }
            }
            _ => return Err(PerformanceError::Optimization(format!("Unknown parameter: {}", suggestion.parameter))),
        }

        // Record adaptation
        let adaptation = AdaptationRecord {
            timestamp: chrono::Utc::now().timestamp() as u64,
            parameters: self.parameters.clone(),
            reason: suggestion.reason.clone(),
            performance_before: current_metrics,
            performance_after: None, // Will be filled later
        };

        self.adaptation_history.push(adaptation);
        Ok(())
    }
}

impl ConsensusBenchmark {
    /// Create a new benchmark suite
    pub fn new() -> Self {
        Self {
            scenarios: Self::default_scenarios(),
            results: HashMap::new(),
        }
    }

    /// Run all benchmark scenarios
    pub async fn run_all_benchmarks(&mut self) -> Result<HashMap<String, BenchmarkResult>> {
        for scenario in &self.scenarios.clone() {
            let result = self.run_scenario(scenario).await?;
            self.results.insert(scenario.name.clone(), result);
        }
        Ok(self.results.clone())
    }

    /// Run a specific benchmark scenario
    pub async fn run_scenario(&self, scenario: &BenchmarkScenario) -> Result<BenchmarkResult> {
        println!("🏃 Running benchmark: {}", scenario.name);
        
        let start_time = Instant::now();
        
        // Simulate benchmark execution
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let execution_time = start_time.elapsed();
        
        // Generate mock results
        let result = BenchmarkResult {
            scenario_name: scenario.name.clone(),
            execution_time,
            average_metrics: self.generate_mock_metrics(scenario),
            peak_metrics: self.generate_mock_peak_metrics(scenario),
            error_rate: 0.01, // 1% error rate
            success_criteria_met: true,
        };

        println!("✅ Benchmark {} completed in {:?}", scenario.name, execution_time);
        Ok(result)
    }

    fn default_scenarios() -> Vec<BenchmarkScenario> {
        vec![
            BenchmarkScenario {
                name: "baseline".to_string(),
                description: "Baseline performance with 4 validators".to_string(),
                validator_count: 4,
                transaction_rate: 100.0,
                duration: Duration::from_secs(60),
                network_latency: Duration::from_millis(50),
                byzantine_ratio: 0.0,
            },
            BenchmarkScenario {
                name: "high_load".to_string(),
                description: "High transaction load test".to_string(),
                validator_count: 7,
                transaction_rate: 1000.0,
                duration: Duration::from_secs(300),
                network_latency: Duration::from_millis(100),
                byzantine_ratio: 0.0,
            },
            BenchmarkScenario {
                name: "byzantine_fault".to_string(),
                description: "Byzantine fault tolerance test".to_string(),
                validator_count: 10,
                transaction_rate: 500.0,
                duration: Duration::from_secs(180),
                network_latency: Duration::from_millis(75),
                byzantine_ratio: 0.33,
            },
            BenchmarkScenario {
                name: "network_partition".to_string(),
                description: "Network partition recovery test".to_string(),
                validator_count: 7,
                transaction_rate: 200.0,
                duration: Duration::from_secs(240),
                network_latency: Duration::from_millis(200),
                byzantine_ratio: 0.0,
            },
        ]
    }

    fn generate_mock_metrics(&self, scenario: &BenchmarkScenario) -> ConsensusMetrics {
        // Generate realistic metrics based on scenario parameters
        let base_latency = Duration::from_millis(500 + scenario.network_latency.as_millis() as u64);
        let throughput_factor = 1.0 - (scenario.byzantine_ratio * 0.5);
        
        ConsensusMetrics {
            round_duration: base_latency,
            proposal_time: Duration::from_millis(100),
            voting_time: Duration::from_millis(200),
            commit_time: Duration::from_millis(50),
            throughput: scenario.transaction_rate * throughput_factor,
            latency_percentiles: LatencyPercentiles {
                p50: base_latency / 2,
                p90: base_latency,
                p95: base_latency * 2,
                p99: base_latency * 3,
            },
            resource_usage: ResourceUsage {
                cpu_percent: 30.0 + (scenario.transaction_rate / 1000.0) * 50.0,
                memory_mb: 256 + (scenario.validator_count as u64 * 64),
                network_bytes_in: (scenario.transaction_rate * 1024.0) as u64,
                network_bytes_out: (scenario.transaction_rate * 1024.0) as u64,
                disk_io_bytes: (scenario.transaction_rate * 512.0) as u64,
            },
        }
    }

    fn generate_mock_peak_metrics(&self, scenario: &BenchmarkScenario) -> ConsensusMetrics {
        let base_metrics = self.generate_mock_metrics(scenario);
        
        ConsensusMetrics {
            round_duration: base_metrics.round_duration * 2,
            proposal_time: base_metrics.proposal_time * 2,
            voting_time: base_metrics.voting_time * 2,
            commit_time: base_metrics.commit_time * 2,
            throughput: base_metrics.throughput * 0.8, // Peak load reduces throughput
            latency_percentiles: LatencyPercentiles {
                p50: base_metrics.latency_percentiles.p50 * 2,
                p90: base_metrics.latency_percentiles.p90 * 2,
                p95: base_metrics.latency_percentiles.p95 * 2,
                p99: base_metrics.latency_percentiles.p99 * 2,
            },
            resource_usage: ResourceUsage {
                cpu_percent: (base_metrics.resource_usage.cpu_percent * 1.5).min(95.0),
                memory_mb: base_metrics.resource_usage.memory_mb * 2,
                network_bytes_in: base_metrics.resource_usage.network_bytes_in * 2,
                network_bytes_out: base_metrics.resource_usage.network_bytes_out * 2,
                disk_io_bytes: base_metrics.resource_usage.disk_io_bytes * 2,
            },
        }
    }
}

/// Performance anomaly detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnomaly {
    pub severity: AnomalySeverity,
    pub description: String,
    pub metric_name: String,
    pub value: f64,
    pub threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub parameter: String,
    pub current_value: String,
    pub suggested_value: String,
    pub reason: String,
    pub expected_impact: Impact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Impact {
    Low,
    Medium,
    High,
}

impl Default for LatencyPercentiles {
    fn default() -> Self {
        Self {
            p50: Duration::from_millis(100),
            p90: Duration::from_millis(200),
            p95: Duration::from_millis(300),
            p99: Duration::from_millis(500),
        }
    }
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            cpu_percent: 0.0,
            memory_mb: 0,
            network_bytes_in: 0,
            network_bytes_out: 0,
            disk_io_bytes: 0,
        }
    }
}

impl Default for OptimizationParameters {
    fn default() -> Self {
        Self {
            block_size_limit: 1024 * 1024, // 1MB
            block_interval: Duration::from_secs(5),
            timeout_propose: Duration::from_secs(3),
            timeout_prevote: Duration::from_secs(1),
            timeout_precommit: Duration::from_secs(1),
            batch_size: 100,
            parallel_verification: false,
        }
    }
}

impl Default for PerformanceTargets {
    fn default() -> Self {
        Self {
            target_throughput: 1000.0,
            max_latency: Duration::from_secs(5),
            target_finality_time: Duration::from_secs(15),
            max_cpu_usage: 80.0,
            max_memory_usage: 1024, // MB
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new();
        
        monitor.start_round(1);
        std::thread::sleep(Duration::from_millis(10));
        monitor.end_round(1, 100).unwrap();
        
        assert!(!monitor.metrics_history.is_empty());
    }

    #[test]
    fn test_throughput_tracker() {
        let mut tracker = ThroughputTracker::new(Duration::from_secs(1));
        
        tracker.record_transactions(100);
        let throughput = tracker.current_throughput();
        
        assert!(throughput > 0.0);
    }

    #[test]
    fn test_optimization_engine() {
        let params = OptimizationParameters::default();
        let targets = PerformanceTargets::default();
        let mut engine = OptimizationEngine::new(params, targets);
        
        let metrics = ConsensusMetrics {
            round_duration: Duration::from_secs(1),
            proposal_time: Duration::from_millis(100),
            voting_time: Duration::from_millis(200),
            commit_time: Duration::from_millis(50),
            throughput: 50.0, // Below target
            latency_percentiles: LatencyPercentiles::default(),
            resource_usage: ResourceUsage::default(),
        };
        
        let suggestions = engine.analyze_and_optimize(&metrics).unwrap();
        assert!(!suggestions.is_empty());
    }

    #[tokio::test]
    async fn test_consensus_benchmark() {
        let mut benchmark = ConsensusBenchmark::new();
        
        let scenario = &benchmark.scenarios[0].clone();
        let result = benchmark.run_scenario(scenario).await.unwrap();
        
        assert_eq!(result.scenario_name, scenario.name);
        assert!(result.success_criteria_met);
    }

    #[test]
    fn test_anomaly_detection() {
        let mut monitor = PerformanceMonitor::new();
        
        // Create metrics with high latency
        let metrics = ConsensusMetrics {
            round_duration: Duration::from_secs(15), // High latency
            proposal_time: Duration::from_millis(100),
            voting_time: Duration::from_millis(200),
            commit_time: Duration::from_millis(50),
            throughput: 50.0, // Low throughput
            latency_percentiles: LatencyPercentiles::default(),
            resource_usage: ResourceUsage {
                cpu_percent: 95.0, // High CPU
                memory_mb: 512,
                network_bytes_in: 1024,
                network_bytes_out: 1024,
                disk_io_bytes: 512,
            },
        };
        
        monitor.record_metrics(metrics);
        let anomalies = monitor.detect_anomalies();
        
        assert!(!anomalies.is_empty());
        assert!(anomalies.len() >= 2); // Should detect high latency and CPU
    }
}

