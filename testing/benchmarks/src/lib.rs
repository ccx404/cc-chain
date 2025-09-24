//! CC Chain Benchmarking Framework
//!
//! This crate provides comprehensive benchmarking utilities for CC Chain components.
//! It includes micro-benchmarks, macro-benchmarks, and performance regression detection.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BenchmarkError {
    #[error("Benchmark execution error: {0}")]
    Execution(String),
    #[error("Configuration error: {0}")]
    Configuration(String),
    #[error("Measurement error: {0}")]
    Measurement(String),
}

pub type Result<T> = std::result::Result<T, BenchmarkError>;

/// Benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub name: String,
    pub warmup_iterations: u32,
    pub measurement_iterations: u32,
    pub timeout: Duration,
    pub sample_size: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        BenchmarkConfig {
            name: "default_benchmark".to_string(),
            warmup_iterations: 10,
            measurement_iterations: 100,
            timeout: Duration::from_secs(60),
            sample_size: 100,
        }
    }
}

/// Benchmark measurement result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMeasurement {
    pub name: String,
    pub mean: Duration,
    pub median: Duration,
    pub min: Duration,
    pub max: Duration,
    pub std_dev: f64,
    pub samples: u32,
    pub throughput: Option<f64>, // operations per second
}

/// Main benchmarking framework
pub struct BenchmarkSuite {
    name: String,
    benchmarks: HashMap<String, BenchmarkFunction>,
    configs: HashMap<String, BenchmarkConfig>,
    results: Vec<BenchmarkMeasurement>,
}

type BenchmarkFunction = Box<dyn Fn() -> Result<Duration> + Send>;

impl BenchmarkSuite {
    /// Create a new benchmark suite
    pub fn new(name: &str) -> Self {
        BenchmarkSuite {
            name: name.to_string(),
            benchmarks: HashMap::new(),
            configs: HashMap::new(),
            results: Vec::new(),
        }
    }
    
    /// Add a benchmark to the suite
    pub fn add_benchmark<F>(&mut self, name: &str, config: BenchmarkConfig, benchmark: F)
    where
        F: Fn() -> Result<Duration> + Send + 'static,
    {
        self.benchmarks.insert(name.to_string(), Box::new(benchmark));
        self.configs.insert(name.to_string(), config);
    }
    
    /// Run all benchmarks in the suite
    pub fn run_all(&mut self) -> Result<Vec<BenchmarkMeasurement>> {
        let mut results = Vec::new();
        
        for name in self.benchmarks.keys().cloned().collect::<Vec<_>>() {
            match self.run_benchmark(&name) {
                Ok(measurement) => {
                    results.push(measurement.clone());
                    self.results.push(measurement);
                }
                Err(e) => {
                    eprintln!("Benchmark '{}' failed: {}", name, e);
                }
            }
        }
        
        Ok(results)
    }
    
    /// Run a specific benchmark
    pub fn run_benchmark(&self, name: &str) -> Result<BenchmarkMeasurement> {
        let benchmark = self.benchmarks.get(name)
            .ok_or_else(|| BenchmarkError::Execution(
                format!("Benchmark '{}' not found", name)
            ))?;
            
        let config = self.configs.get(name)
            .ok_or_else(|| BenchmarkError::Configuration(
                format!("Config for '{}' not found", name)
            ))?;
        
        println!("Running benchmark: {}", name);
        
        // Warmup phase
        println!("  Warmup ({} iterations)...", config.warmup_iterations);
        for _ in 0..config.warmup_iterations {
            benchmark()?;
        }
        
        // Measurement phase
        println!("  Measuring ({} iterations)...", config.measurement_iterations);
        let mut measurements = Vec::new();
        let start_time = Instant::now();
        
        for _ in 0..config.measurement_iterations {
            let measurement = benchmark()?;
            measurements.push(measurement);
            
            if start_time.elapsed() > config.timeout {
                return Err(BenchmarkError::Execution(
                    "Benchmark timeout exceeded".to_string()
                ));
            }
        }
        
        // Calculate statistics
        let stats = self.calculate_statistics(name, &measurements)?;
        println!("  Completed: mean={:?}", stats.mean);
        
        Ok(stats)
    }
    
    /// Calculate benchmark statistics
    fn calculate_statistics(&self, name: &str, measurements: &[Duration]) -> Result<BenchmarkMeasurement> {
        if measurements.is_empty() {
            return Err(BenchmarkError::Measurement("No measurements available".to_string()));
        }
        
        let mut sorted = measurements.to_vec();
        sorted.sort();
        
        let min = sorted[0];
        let max = sorted[sorted.len() - 1];
        let median = sorted[sorted.len() / 2];
        
        let total_nanos: u128 = measurements.iter().map(|d| d.as_nanos()).sum();
        let mean = Duration::from_nanos((total_nanos / measurements.len() as u128) as u64);
        
        // Calculate standard deviation
        let mean_nanos = mean.as_nanos() as f64;
        let variance = measurements.iter()
            .map(|d| {
                let diff = d.as_nanos() as f64 - mean_nanos;
                diff * diff
            })
            .sum::<f64>() / measurements.len() as f64;
        let std_dev = variance.sqrt();
        
        // Calculate throughput (operations per second)
        let throughput = if mean.as_secs_f64() > 0.0 {
            Some(1.0 / mean.as_secs_f64())
        } else {
            None
        };
        
        Ok(BenchmarkMeasurement {
            name: name.to_string(),
            mean,
            median,
            min,
            max,
            std_dev,
            samples: measurements.len() as u32,
            throughput,
        })
    }
    
    /// Generate a benchmark report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str(&format!("Benchmark Suite: {}\n", self.name));
        report.push_str("=".repeat(50).as_str());
        report.push('\n');
        
        for result in &self.results {
            report.push_str(&format!("\nBenchmark: {}\n", result.name));
            report.push_str(&format!("  Mean:     {:?}\n", result.mean));
            report.push_str(&format!("  Median:   {:?}\n", result.median));
            report.push_str(&format!("  Min:      {:?}\n", result.min));
            report.push_str(&format!("  Max:      {:?}\n", result.max));
            report.push_str(&format!("  Std Dev:  {:.2}ns\n", result.std_dev));
            report.push_str(&format!("  Samples:  {}\n", result.samples));
            if let Some(throughput) = result.throughput {
                report.push_str(&format!("  Throughput: {:.2} ops/sec\n", throughput));
            }
        }
        
        report
    }
    
    /// Compare benchmark results with baseline
    pub fn compare_with_baseline(&self, baseline_results: &[BenchmarkMeasurement]) -> ComparisonReport {
        let mut comparisons = Vec::new();
        
        for current in &self.results {
            if let Some(baseline) = baseline_results.iter().find(|b| b.name == current.name) {
                let improvement = baseline.mean.as_nanos() as f64 / current.mean.as_nanos() as f64;
                let change_percent = (improvement - 1.0) * 100.0;
                
                comparisons.push(BenchmarkComparison {
                    name: current.name.clone(),
                    current_mean: current.mean,
                    baseline_mean: baseline.mean,
                    improvement_factor: improvement,
                    change_percent,
                    is_improvement: improvement > 1.0,
                    is_regression: improvement < 0.95, // 5% threshold
                });
            }
        }
        
        ComparisonReport { comparisons }
    }
}

/// Benchmark comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkComparison {
    pub name: String,
    pub current_mean: Duration,
    pub baseline_mean: Duration,
    pub improvement_factor: f64,
    pub change_percent: f64,
    pub is_improvement: bool,
    pub is_regression: bool,
}

/// Comparison report
#[derive(Debug, Clone)]
pub struct ComparisonReport {
    pub comparisons: Vec<BenchmarkComparison>,
}

impl ComparisonReport {
    /// Check if any regressions were detected
    pub fn has_regressions(&self) -> bool {
        self.comparisons.iter().any(|c| c.is_regression)
    }
    
    /// Get all regressions
    pub fn get_regressions(&self) -> Vec<&BenchmarkComparison> {
        self.comparisons.iter().filter(|c| c.is_regression).collect()
    }
    
    /// Generate comparison report text
    pub fn generate_report(&self) -> String {
        let mut report = String::from("Benchmark Comparison Report\n");
        report.push_str("===========================\n\n");
        
        for comparison in &self.comparisons {
            report.push_str(&format!("Benchmark: {}\n", comparison.name));
            report.push_str(&format!("  Current:  {:?}\n", comparison.current_mean));
            report.push_str(&format!("  Baseline: {:?}\n", comparison.baseline_mean));
            report.push_str(&format!("  Change:   {:.2}%", comparison.change_percent));
            
            if comparison.is_improvement {
                report.push_str(" (IMPROVEMENT)");
            } else if comparison.is_regression {
                report.push_str(" (REGRESSION)");
            }
            
            report.push_str("\n\n");
        }
        
        if self.has_regressions() {
            report.push_str("⚠️  REGRESSIONS DETECTED!\n");
        }
        
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_benchmark_suite() {
        let mut suite = BenchmarkSuite::new("test_suite");
        
        let config = BenchmarkConfig {
            name: "fast_operation".to_string(),
            warmup_iterations: 2,
            measurement_iterations: 5,
            timeout: Duration::from_secs(10),
            sample_size: 5,
        };
        
        suite.add_benchmark("fast_operation", config, || {
            thread::sleep(Duration::from_millis(1));
            Ok(Duration::from_millis(1))
        });
        
        let results = suite.run_all().unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "fast_operation");
        assert!(results[0].mean >= Duration::from_millis(1));
    }
    
    #[test]
    fn test_benchmark_statistics() {
        let measurements = vec![
            Duration::from_millis(10),
            Duration::from_millis(15),
            Duration::from_millis(12),
            Duration::from_millis(18),
            Duration::from_millis(11),
        ];
        
        let suite = BenchmarkSuite::new("test");
        let stats = suite.calculate_statistics("test_bench", &measurements).unwrap();
        
        assert_eq!(stats.samples, 5);
        assert!(stats.min <= stats.mean);
        assert!(stats.mean <= stats.max);
        assert!(stats.throughput.is_some());
    }
    
    #[test]
    fn test_benchmark_comparison() {
        let current = vec![BenchmarkMeasurement {
            name: "test".to_string(),
            mean: Duration::from_millis(100),
            median: Duration::from_millis(100),
            min: Duration::from_millis(90),
            max: Duration::from_millis(110),
            std_dev: 5.0,
            samples: 10,
            throughput: Some(10.0),
        }];
        
        let baseline = vec![BenchmarkMeasurement {
            name: "test".to_string(),
            mean: Duration::from_millis(120), // Slower baseline
            median: Duration::from_millis(120),
            min: Duration::from_millis(110),
            max: Duration::from_millis(130),
            std_dev: 6.0,
            samples: 10,
            throughput: Some(8.33),
        }];
        
        let suite = BenchmarkSuite::new("test");
        let mut test_suite = suite;
        test_suite.results = current;
        
        let comparison = test_suite.compare_with_baseline(&baseline);
        
        assert_eq!(comparison.comparisons.len(), 1);
        assert!(comparison.comparisons[0].is_improvement);
        assert!(!comparison.comparisons[0].is_regression);
        assert!(!comparison.has_regressions());
    }
    
    #[test]
    fn test_report_generation() {
        let mut suite = BenchmarkSuite::new("test_suite");
        suite.results.push(BenchmarkMeasurement {
            name: "test_benchmark".to_string(),
            mean: Duration::from_millis(50),
            median: Duration::from_millis(48),
            min: Duration::from_millis(45),
            max: Duration::from_millis(60),
            std_dev: 3.5,
            samples: 100,
            throughput: Some(20.0),
        });
        
        let report = suite.generate_report();
        assert!(report.contains("Benchmark Suite: test_suite"));
        assert!(report.contains("test_benchmark"));
        assert!(report.contains("Mean:"));
        assert!(report.contains("Throughput: 20.00 ops/sec"));
    }
}
