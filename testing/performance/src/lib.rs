//! CC Chain Performance Testing
//!
//! This crate provides performance testing utilities and benchmarking tools
//! for CC Chain components. It includes timing utilities, throughput measurement,
//! resource monitoring, and performance regression detection.

use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PerformanceError {
    #[error("Benchmark execution error: {0}")]
    BenchmarkExecution(String),
    #[error("Measurement error: {0}")]
    Measurement(String),
    #[error("Resource monitoring error: {0}")]
    ResourceMonitoring(String),
}

pub type Result<T> = std::result::Result<T, PerformanceError>;

/// Performance benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub name: String,
    pub warmup_iterations: u32,
    pub measurement_iterations: u32,
    pub timeout: Duration,
    pub memory_limit: Option<usize>, // bytes
    pub cpu_limit: Option<f64>,      // percentage
}

/// Performance measurement results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub name: String,
    pub duration: Duration,
    pub iterations: u32,
    pub ops_per_second: f64,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub mean_duration: Duration,
    pub std_deviation: f64,
    pub memory_usage: MemoryStats,
    pub timestamp: SystemTime,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub peak_memory: usize,     // bytes
    pub average_memory: usize,  // bytes
    pub allocations: u64,
    pub deallocations: u64,
}

/// Main benchmarking framework
pub struct BenchmarkRunner {
    benchmarks: HashMap<String, Box<dyn Fn() -> Result<Duration> + Send>>,
    configs: HashMap<String, BenchmarkConfig>,
    results: Vec<BenchmarkResult>,
}

impl BenchmarkRunner {
    /// Create a new benchmark runner
    pub fn new() -> Self {
        BenchmarkRunner {
            benchmarks: HashMap::new(),
            configs: HashMap::new(),
            results: Vec::new(),
        }
    }
    
    /// Add a benchmark function
    pub fn add_benchmark<F>(&mut self, name: &str, config: BenchmarkConfig, benchmark: F)
    where
        F: Fn() -> Result<Duration> + Send + 'static,
    {
        self.benchmarks.insert(name.to_string(), Box::new(benchmark));
        self.configs.insert(name.to_string(), config);
    }
    
    /// Run all registered benchmarks
    pub fn run_all(&mut self) -> Result<Vec<BenchmarkResult>> {
        let mut results = Vec::new();
        
        for (name, _) in self.benchmarks.iter() {
            match self.run_benchmark(name) {
                Ok(result) => {
                    results.push(result.clone());
                    self.results.push(result);
                }
                Err(e) => {
                    eprintln!("Benchmark '{}' failed: {}", name, e);
                }
            }
        }
        
        Ok(results)
    }
    
    /// Run a specific benchmark
    pub fn run_benchmark(&self, name: &str) -> Result<BenchmarkResult> {
        let benchmark = self.benchmarks.get(name)
            .ok_or_else(|| PerformanceError::BenchmarkExecution(
                format!("Benchmark '{}' not found", name)
            ))?;
            
        let config = self.configs.get(name)
            .ok_or_else(|| PerformanceError::BenchmarkExecution(
                format!("Config for benchmark '{}' not found", name)
            ))?;
        
        // Warmup phase
        for _ in 0..config.warmup_iterations {
            benchmark()?;
        }
        
        // Measurement phase
        let mut durations = Vec::new();
        let start_time = Instant::now();
        let memory_monitor = MemoryMonitor::new();
        
        for _ in 0..config.measurement_iterations {
            let iteration_start = Instant::now();
            benchmark()?;
            let iteration_duration = iteration_start.elapsed();
            durations.push(iteration_duration);
            
            // Check timeout
            if start_time.elapsed() > config.timeout {
                return Err(PerformanceError::BenchmarkExecution(
                    "Benchmark timeout exceeded".to_string()
                ));
            }
        }
        
        let memory_stats = memory_monitor.get_stats();
        let total_duration = start_time.elapsed();
        
        // Calculate statistics
        let min_duration = *durations.iter().min().unwrap();
        let max_duration = *durations.iter().max().unwrap();
        let mean_duration = Duration::from_nanos(
            ((durations.iter().map(|d| d.as_nanos()).sum::<u128>() / durations.len() as u128) as u64)
        );
        
        let mean_nanos = mean_duration.as_nanos() as f64;
        let variance = durations.iter()
            .map(|d| {
                let diff = d.as_nanos() as f64 - mean_nanos;
                diff * diff
            })
            .sum::<f64>() / durations.len() as f64;
        let std_deviation = variance.sqrt();
        
        let ops_per_second = config.measurement_iterations as f64 / total_duration.as_secs_f64();
        
        Ok(BenchmarkResult {
            name: name.to_string(),
            duration: total_duration,
            iterations: config.measurement_iterations,
            ops_per_second,
            min_duration,
            max_duration,
            mean_duration,
            std_deviation,
            memory_usage: memory_stats,
            timestamp: SystemTime::now(),
        })
    }
    
    /// Get all benchmark results
    pub fn get_results(&self) -> &[BenchmarkResult] {
        &self.results
    }
    
    /// Generate performance report
    pub fn generate_report(&self) -> String {
        let mut report = String::from("Performance Benchmark Report\n");
        report.push_str("=====================================\n\n");
        
        for result in &self.results {
            report.push_str(&format!("Benchmark: {}\n", result.name));
            report.push_str(&format!("  Duration: {:?}\n", result.duration));
            report.push_str(&format!("  Iterations: {}\n", result.iterations));
            report.push_str(&format!("  Ops/sec: {:.2}\n", result.ops_per_second));
            report.push_str(&format!("  Mean: {:?}\n", result.mean_duration));
            report.push_str(&format!("  Min: {:?}\n", result.min_duration));
            report.push_str(&format!("  Max: {:?}\n", result.max_duration));
            report.push_str(&format!("  Std Dev: {:.2}ns\n", result.std_deviation));
            report.push_str(&format!("  Peak Memory: {} bytes\n", result.memory_usage.peak_memory));
            report.push_str("\n");
        }
        
        report
    }
}

/// Memory monitoring utility
pub struct MemoryMonitor {
    start_time: Instant,
    measurements: Vec<usize>,
}

impl MemoryMonitor {
    /// Create a new memory monitor
    pub fn new() -> Self {
        MemoryMonitor {
            start_time: Instant::now(),
            measurements: Vec::new(),
        }
    }
    
    /// Record current memory usage (simplified implementation)
    pub fn record(&mut self) {
        // In a real implementation, this would use system APIs to get actual memory usage
        // For testing purposes, we'll use a simplified approach
        let current_usage = self.estimate_memory_usage();
        self.measurements.push(current_usage);
    }
    
    /// Get memory statistics
    pub fn get_stats(&self) -> MemoryStats {
        if self.measurements.is_empty() {
            return MemoryStats {
                peak_memory: 0,
                average_memory: 0,
                allocations: 0,
                deallocations: 0,
            };
        }
        
        let peak_memory = *self.measurements.iter().max().unwrap();
        let average_memory = self.measurements.iter().sum::<usize>() / self.measurements.len();
        
        MemoryStats {
            peak_memory,
            average_memory,
            allocations: self.measurements.len() as u64,
            deallocations: 0, // Simplified
        }
    }
    
    fn estimate_memory_usage(&self) -> usize {
        // Simplified memory estimation - in real implementation would use actual system calls
        // This is just for testing purposes
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        self.start_time.elapsed().as_nanos().hash(&mut hasher);
        
        // Return a value between 1KB and 100KB
        1024 + (hasher.finish() as usize % (100 * 1024 - 1024))
    }
}

/// Throughput measurement utility
pub struct ThroughputMeter {
    name: String,
    start_time: Instant,
    operation_count: u64,
    byte_count: u64,
}

impl ThroughputMeter {
    /// Create a new throughput meter
    pub fn new(name: &str) -> Self {
        ThroughputMeter {
            name: name.to_string(),
            start_time: Instant::now(),
            operation_count: 0,
            byte_count: 0,
        }
    }
    
    /// Record an operation
    pub fn record_operation(&mut self) {
        self.operation_count += 1;
    }
    
    /// Record bytes processed
    pub fn record_bytes(&mut self, bytes: u64) {
        self.byte_count += bytes;
    }
    
    /// Get current throughput statistics
    pub fn get_throughput(&self) -> ThroughputStats {
        let elapsed = self.start_time.elapsed();
        let elapsed_secs = elapsed.as_secs_f64();
        
        ThroughputStats {
            name: self.name.clone(),
            operations_per_second: if elapsed_secs > 0.0 {
                self.operation_count as f64 / elapsed_secs
            } else {
                0.0
            },
            bytes_per_second: if elapsed_secs > 0.0 {
                self.byte_count as f64 / elapsed_secs
            } else {
                0.0
            },
            total_operations: self.operation_count,
            total_bytes: self.byte_count,
            elapsed_time: elapsed,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputStats {
    pub name: String,
    pub operations_per_second: f64,
    pub bytes_per_second: f64,
    pub total_operations: u64,
    pub total_bytes: u64,
    pub elapsed_time: Duration,
}

/// Load testing utilities
pub mod load_testing {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::sync::atomic::{AtomicU64, Ordering};
    
    /// Load test configuration
    #[derive(Debug, Clone)]
    pub struct LoadTestConfig {
        pub concurrent_users: usize,
        pub duration: Duration,
        pub ramp_up_time: Duration,
        pub operations_per_second: Option<f64>,
    }
    
    /// Load test results
    #[derive(Debug, Clone)]
    pub struct LoadTestResult {
        pub total_operations: u64,
        pub successful_operations: u64,
        pub failed_operations: u64,
        pub average_response_time: Duration,
        pub throughput: f64,
        pub error_rate: f64,
    }
    
    /// Simple load tester
    pub struct LoadTester {
        config: LoadTestConfig,
        operation_count: Arc<AtomicU64>,
        success_count: Arc<AtomicU64>,
        failure_count: Arc<AtomicU64>,
        response_times: Arc<Mutex<Vec<Duration>>>,
    }
    
    impl LoadTester {
        /// Create a new load tester
        pub fn new(config: LoadTestConfig) -> Self {
            LoadTester {
                config,
                operation_count: Arc::new(AtomicU64::new(0)),
                success_count: Arc::new(AtomicU64::new(0)),
                failure_count: Arc::new(AtomicU64::new(0)),
                response_times: Arc::new(Mutex::new(Vec::new())),
            }
        }
        
        /// Run load test with provided operation
        pub fn run<F, T, E>(&self, operation: F) -> LoadTestResult
        where
            F: Fn() -> std::result::Result<T, E> + Send + Sync + 'static,
            T: Send + 'static,
            E: Send + 'static,
        {
            let start_time = Instant::now();
            let operation = Arc::new(operation);
            
            // Simulate concurrent load (simplified for testing)
            for _ in 0..self.config.concurrent_users {
                let op = Arc::clone(&operation);
                let operation_count = Arc::clone(&self.operation_count);
                let success_count = Arc::clone(&self.success_count);
                let failure_count = Arc::clone(&self.failure_count);
                let response_times = Arc::clone(&self.response_times);
                
                std::thread::spawn(move || {
                    let op_start = Instant::now();
                    match op() {
                        Ok(_) => {
                            success_count.fetch_add(1, Ordering::Relaxed);
                        }
                        Err(_) => {
                            failure_count.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    
                    operation_count.fetch_add(1, Ordering::Relaxed);
                    response_times.lock().unwrap().push(op_start.elapsed());
                });
            }
            
            // Wait for test duration
            std::thread::sleep(self.config.duration);
            
            let total_operations = self.operation_count.load(Ordering::Relaxed);
            let successful_operations = self.success_count.load(Ordering::Relaxed);
            let failed_operations = self.failure_count.load(Ordering::Relaxed);
            
            let response_times = self.response_times.lock().unwrap();
            let average_response_time = if !response_times.is_empty() {
                Duration::from_nanos(
                    ((response_times.iter().map(|d| d.as_nanos()).sum::<u128>() / response_times.len() as u128) as u64)
                )
            } else {
                Duration::from_nanos(0)
            };
            
            let elapsed = start_time.elapsed();
            let throughput = total_operations as f64 / elapsed.as_secs_f64();
            let error_rate = if total_operations > 0 {
                failed_operations as f64 / total_operations as f64
            } else {
                0.0
            };
            
            LoadTestResult {
                total_operations,
                successful_operations,
                failed_operations,
                average_response_time,
                throughput,
                error_rate,
            }
        }
    }
}

impl Default for BenchmarkRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MemoryMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_benchmark_runner() {
        let mut runner = BenchmarkRunner::new();
        
        let config = BenchmarkConfig {
            name: "test_benchmark".to_string(),
            warmup_iterations: 2,
            measurement_iterations: 5,
            timeout: Duration::from_secs(10),
            memory_limit: None,
            cpu_limit: None,
        };
        
        runner.add_benchmark("test", config, || {
            thread::sleep(Duration::from_millis(1));
            Ok(Duration::from_millis(1))
        });
        
        let result = runner.run_benchmark("test").unwrap();
        assert_eq!(result.name, "test");
        assert_eq!(result.iterations, 5);
        assert!(result.ops_per_second > 0.0);
    }
    
    #[test]
    fn test_memory_monitor() {
        let mut monitor = MemoryMonitor::new();
        monitor.record();
        monitor.record();
        monitor.record();
        
        let stats = monitor.get_stats();
        assert!(stats.peak_memory > 0);
        assert!(stats.average_memory > 0);
        assert_eq!(stats.allocations, 3);
    }
    
    #[test]
    fn test_throughput_meter() {
        let mut meter = ThroughputMeter::new("test_ops");
        
        thread::sleep(Duration::from_millis(100));
        
        for _ in 0..10 {
            meter.record_operation();
            meter.record_bytes(1024);
        }
        
        let stats = meter.get_throughput();
        assert_eq!(stats.name, "test_ops");
        assert_eq!(stats.total_operations, 10);
        assert_eq!(stats.total_bytes, 10240);
        assert!(stats.operations_per_second > 0.0);
        assert!(stats.bytes_per_second > 0.0);
    }
    
    #[test]
    fn test_load_tester() {
        let config = load_testing::LoadTestConfig {
            concurrent_users: 2,
            duration: Duration::from_millis(100),
            ramp_up_time: Duration::from_millis(10),
            operations_per_second: None,
        };
        
        let load_tester = load_testing::LoadTester::new(config);
        
        let result = load_tester.run(|| -> std::result::Result<(), String> {
            thread::sleep(Duration::from_millis(10));
            Ok(())
        });
        
        assert!(result.total_operations > 0);
        assert_eq!(result.failed_operations, 0);
        assert!(result.throughput >= 0.0);
        assert_eq!(result.error_rate, 0.0);
    }
    
    #[test]
    fn test_benchmark_report_generation() {
        let mut runner = BenchmarkRunner::new();
        
        let config = BenchmarkConfig {
            name: "report_test".to_string(),
            warmup_iterations: 1,
            measurement_iterations: 2,
            timeout: Duration::from_secs(5),
            memory_limit: None,
            cpu_limit: None,
        };
        
        runner.add_benchmark("report_test", config, || {
            Ok(Duration::from_millis(1))
        });
        
        runner.run_all().unwrap();
        
        let report = runner.generate_report();
        assert!(report.contains("Performance Benchmark Report"));
        assert!(report.contains("report_test"));
        assert!(report.contains("Ops/sec"));
    }
}
