# CC Chain Consensus Performance

This crate provides comprehensive performance monitoring, optimization, and benchmarking capabilities for the CC Chain consensus mechanism.

## 🎯 Overview

The consensus performance module helps developers and validators:
- Monitor consensus performance in real-time
- Detect performance anomalies automatically
- Optimize consensus parameters dynamically
- Benchmark different consensus configurations
- Track resource utilization

## 📊 Core Components

### 🔍 Performance Monitor
Real-time monitoring of consensus operations with detailed metrics collection.

```rust
use consensus_performance::PerformanceMonitor;

let mut monitor = PerformanceMonitor::new();

// Start monitoring a consensus round
monitor.start_round(round_number);

// Monitor specific operations
monitor.start_operation("proposal");
// ... perform proposal ...
monitor.end_operation("proposal");

// End round and collect metrics
monitor.end_round(round_number, transaction_count)?;

// Get performance statistics
let avg_metrics = monitor.get_average_metrics(Duration::from_secs(60));
println!("Average round duration: {:?}", avg_metrics?.round_duration);
```

### 📈 Throughput Tracking
Monitor transaction throughput over configurable time windows.

```rust
use consensus_performance::ThroughputTracker;

let mut tracker = ThroughputTracker::new(Duration::from_secs(10));

// Record processed transactions
tracker.record_transactions(150);

// Get current throughput (TPS)
let tps = tracker.current_throughput();
println!("Current throughput: {:.2} TPS", tps);
```

### 🔧 Optimization Engine
Automated parameter tuning based on performance targets.

```rust
use consensus_performance::{OptimizationEngine, OptimizationParameters, PerformanceTargets};

let parameters = OptimizationParameters::default();
let targets = PerformanceTargets {
    target_throughput: 1000.0,  // 1000 TPS
    max_latency: Duration::from_secs(5),
    target_finality_time: Duration::from_secs(15),
    max_cpu_usage: 80.0,        // 80% CPU
    max_memory_usage: 1024,     // 1GB RAM
};

let mut engine = OptimizationEngine::new(parameters, targets);

// Analyze current performance and get suggestions
let suggestions = engine.analyze_and_optimize(&current_metrics)?;

for suggestion in suggestions {
    println!("Suggestion: {} -> {}", suggestion.parameter, suggestion.suggested_value);
    println!("Reason: {}", suggestion.reason);
    
    // Apply optimization
    engine.apply_optimization(&suggestion, current_metrics.clone())?;
}
```

### 🏁 Benchmark Suite
Comprehensive benchmarking with predefined scenarios.

```rust
use consensus_performance::ConsensusBenchmark;

let mut benchmark = ConsensusBenchmark::new();

// Run all benchmarks
let results = benchmark.run_all_benchmarks().await?;

for (name, result) in results {
    println!("Benchmark: {}", name);
    println!("Execution time: {:?}", result.execution_time);
    println!("Average TPS: {:.2}", result.average_metrics.throughput);
    println!("Success criteria met: {}", result.success_criteria_met);
}
```

## 📋 Metrics Collection

### Consensus Metrics

```rust
ConsensusMetrics {
    round_duration: Duration,           // Total round time
    proposal_time: Duration,            // Time to create proposal
    voting_time: Duration,             // Time for voting phase
    commit_time: Duration,             // Time to commit block
    throughput: f64,                   // Transactions per second
    latency_percentiles: LatencyPercentiles,
    resource_usage: ResourceUsage,
}
```

### Latency Percentiles

```rust
LatencyPercentiles {
    p50: Duration,  // 50th percentile (median)
    p90: Duration,  // 90th percentile
    p95: Duration,  // 95th percentile
    p99: Duration,  // 99th percentile
}
```

### Resource Usage

```rust
ResourceUsage {
    cpu_percent: f64,        // CPU utilization (0-100%)
    memory_mb: u64,          // Memory usage in megabytes
    network_bytes_in: u64,   // Inbound network traffic
    network_bytes_out: u64,  // Outbound network traffic
    disk_io_bytes: u64,      // Disk I/O in bytes
}
```

## 🚨 Anomaly Detection

Automatic detection of performance issues with configurable thresholds.

```rust
let anomalies = monitor.detect_anomalies();

for anomaly in anomalies {
    match anomaly.severity {
        AnomalySeverity::Critical => {
            println!("🚨 CRITICAL: {}", anomaly.description);
            // Trigger immediate response
        },
        AnomalySeverity::High => {
            println!("⚠️ HIGH: {}", anomaly.description);
            // Schedule investigation
        },
        AnomalySeverity::Medium => {
            println!("⚡ MEDIUM: {}", anomaly.description);
            // Log for review
        },
        AnomalySeverity::Low => {
            println!("ℹ️ LOW: {}", anomaly.description);
            // Monitor trend
        },
    }
}
```

### Common Anomalies Detected

- **High Round Duration**: Consensus rounds taking too long
- **Low Throughput**: Transaction processing below target
- **High CPU Usage**: Resource utilization exceeding limits
- **Memory Leaks**: Increasing memory usage over time
- **Network Issues**: Unusual network traffic patterns

## 🎛️ Optimization Parameters

### Tunable Parameters

```rust
OptimizationParameters {
    block_size_limit: u32,           // Maximum block size (bytes)
    block_interval: Duration,        // Target time between blocks
    timeout_propose: Duration,       // Proposal phase timeout
    timeout_prevote: Duration,       // Prevote phase timeout
    timeout_precommit: Duration,     // Precommit phase timeout
    batch_size: u32,                // Transaction batch size
    parallel_verification: bool,     // Enable parallel verification
}
```

### Optimization Strategies

1. **Throughput Optimization**
   - Increase batch size
   - Enable parallel verification
   - Optimize block interval

2. **Latency Optimization**
   - Reduce timeouts
   - Smaller block sizes
   - Faster networking

3. **Resource Optimization**
   - Balance CPU vs memory usage
   - Optimize network protocols
   - Efficient data structures

## 🏆 Benchmark Scenarios

### Built-in Scenarios

1. **Baseline Test**
   - 4 validators
   - 100 TPS load
   - 60-second duration
   - Normal network conditions

2. **High Load Test**
   - 7 validators
   - 1000 TPS load
   - 5-minute duration
   - Moderate network latency

3. **Byzantine Fault Test**
   - 10 validators
   - 33% Byzantine nodes
   - 500 TPS load
   - 3-minute duration

4. **Network Partition Test**
   - 7 validators
   - Simulated network partition
   - 200 TPS load
   - 4-minute duration

### Custom Benchmarks

```rust
use consensus_performance::BenchmarkScenario;

let custom_scenario = BenchmarkScenario {
    name: "custom_stress_test".to_string(),
    description: "Custom high-stress scenario".to_string(),
    validator_count: 15,
    transaction_rate: 2000.0,
    duration: Duration::from_secs(600),
    network_latency: Duration::from_millis(100),
    byzantine_ratio: 0.2,  // 20% Byzantine nodes
};

let result = benchmark.run_scenario(&custom_scenario).await?;
```

## 📊 Performance Analysis

### Metrics Visualization

```rust
// Generate performance report
let stats = monitor.get_stats();
for (metric_name, (avg, max, count)) in stats {
    println!("{}: avg={:?}, max={:?}, samples={}", 
             metric_name, avg, max, count);
}

// Detect trends
let recent_avg = monitor.get_average_metrics(Duration::from_secs(300))?;
let historical_avg = monitor.get_average_metrics(Duration::from_secs(3600))?;

if recent_avg.throughput < historical_avg.throughput * 0.9 {
    println!("⚠️ Throughput degradation detected");
}
```

### Performance Targets

```rust
let targets = PerformanceTargets {
    target_throughput: 1000.0,                    // 1000 TPS
    max_latency: Duration::from_secs(5),          // 5 second max latency
    target_finality_time: Duration::from_secs(15), // 15 second finality
    max_cpu_usage: 80.0,                          // 80% CPU max
    max_memory_usage: 1024,                       // 1GB RAM max
};

// Check if targets are met
let current_metrics = monitor.get_latest_metrics();
let meets_targets = 
    current_metrics.throughput >= targets.target_throughput &&
    current_metrics.round_duration <= targets.max_latency &&
    current_metrics.resource_usage.cpu_percent <= targets.max_cpu_usage;
```

## 🔬 Advanced Features

### Historical Analysis

```rust
// Analyze performance over time
let window = Duration::from_hours(24);
let metrics = monitor.get_metrics_window(window);

// Calculate statistics
let avg_throughput: f64 = metrics.iter()
    .map(|m| m.throughput)
    .sum::<f64>() / metrics.len() as f64;

let max_latency = metrics.iter()
    .map(|m| m.round_duration)
    .max()
    .unwrap_or_default();
```

### Predictive Analytics

```rust
// Predict future performance based on trends
let performance_trend = monitor.calculate_trend(Duration::from_hours(6));

if performance_trend.throughput_slope < -0.1 {
    println!("📉 Declining throughput trend detected");
    // Trigger proactive optimization
}
```

### Real-time Alerts

```rust
// Set up performance alerts
monitor.set_alert_threshold("throughput", 800.0);  // Alert if below 800 TPS
monitor.set_alert_threshold("latency", 8.0);       // Alert if above 8 seconds
monitor.set_alert_threshold("cpu", 90.0);          // Alert if above 90% CPU

// Check alerts
let alerts = monitor.check_alerts();
for alert in alerts {
    send_notification(&alert);
}
```

## 🔧 Configuration

### Environment Variables

```bash
# Performance monitoring settings
CC_PERFORMANCE_WINDOW_SIZE=1000      # Metrics history size
CC_PERFORMANCE_SAMPLING_RATE=1000    # Sampling rate (ms)
CC_PERFORMANCE_ALERTS_ENABLED=true   # Enable alerting

# Optimization settings
CC_AUTO_OPTIMIZATION=true            # Enable auto-optimization
CC_OPTIMIZATION_INTERVAL=300         # Optimization check interval (seconds)
CC_OPTIMIZATION_AGGRESSIVENESS=0.5   # Optimization aggressiveness (0.0-1.0)
```

### Configuration File

```toml
[performance]
monitoring_enabled = true
metrics_retention = "24h"
sampling_interval = "1s"

[optimization]
auto_tune = true
target_throughput = 1000.0
max_latency_ms = 5000
aggressive_mode = false

[benchmarks]
default_duration = "5m"
stress_test_enabled = true
```

## 🧪 Testing

Run performance tests:

```bash
# Unit tests
cargo test --package consensus-performance

# Integration tests
cargo test --package consensus-performance --test integration

# Benchmark tests
cargo test --package consensus-performance --bench consensus_bench
```

## 📈 Production Deployment

### Monitoring Setup

1. **Metrics Collection**
   ```rust
   let monitor = PerformanceMonitor::new();
   // Export metrics to Prometheus/Grafana
   ```

2. **Alert Configuration**
   ```rust
   monitor.configure_alerts(AlertConfig {
       throughput_threshold: 500.0,
       latency_threshold_ms: 10000,
       cpu_threshold: 85.0,
   });
   ```

3. **Dashboard Integration**
   ```rust
   // Expose metrics endpoint
   let metrics_endpoint = "/metrics";
   // Export Prometheus-compatible metrics
   ```

### Best Practices

1. **Baseline Establishment**: Run baseline benchmarks before deployment
2. **Gradual Optimization**: Apply optimizations incrementally
3. **Rollback Strategy**: Have rollback plans for parameter changes
4. **Regular Benchmarking**: Schedule periodic benchmark runs
5. **Alert Tuning**: Adjust alert thresholds based on historical data

## 🤝 Contributing

Want to improve consensus performance? See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

Areas for contribution:
- Additional benchmark scenarios
- New optimization algorithms
- Enhanced anomaly detection
- Performance visualization tools
- Machine learning-based optimization

## 📚 Related Documentation

- [Consensus Module](../consensus/)
- [Performance Tuning Guide](../../docs/performance/)
- [Monitoring Best Practices](../../docs/monitoring/)
- [Benchmark Results](./benchmarks/)