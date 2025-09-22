use std::time::Duration;

/// Utility functions for CC Chain
/// Calculate target block time based on network conditions
pub fn calculate_block_time(network_size: usize, avg_latency_ms: u64) -> Duration {
    // Base block time of 1 second for optimal conditions
    let base_time_ms = 1000u64;

    // Adjust based on network size (more nodes = slightly longer time for consensus)
    let network_factor = 1.0 + (network_size as f64 / 1000.0) * 0.1;

    // Adjust based on network latency
    let latency_factor = 1.0 + (avg_latency_ms as f64 / 100.0) * 0.05;

    let adjusted_time_ms = (base_time_ms as f64 * network_factor * latency_factor) as u64;

    // Cap between 500ms and 10 seconds
    let final_time_ms = adjusted_time_ms.max(500).min(10000);

    Duration::from_millis(final_time_ms)
}

/// Calculate optimal gas limit based on network capacity
pub fn calculate_gas_limit(network_throughput: u64, _avg_tx_gas: u64) -> u64 {
    // Target: process transactions efficiently without overwhelming validators
    let base_gas_limit = 10_000_000u64; // 10M gas base limit

    // Adjust based on network throughput
    let throughput_factor = (network_throughput / 1000).max(1);

    base_gas_limit * throughput_factor
}

/// Estimate transaction fees based on network conditions
pub fn estimate_fee(tx_size: usize, network_congestion: f64, base_fee: u64) -> u64 {
    // Base fee calculation
    let size_fee = (tx_size as u64 * base_fee) / 1000; // Fee per KB

    // Congestion multiplier (1.0 = no congestion, 2.0 = high congestion)
    let congestion_multiplier = 1.0 + network_congestion;

    ((size_fee as f64) * congestion_multiplier) as u64
}

/// Performance metrics for monitoring
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Transactions per second
    pub tps: f64,
    /// Average block time
    pub avg_block_time: Duration,
    /// Average transaction confirmation time
    pub avg_confirmation_time: Duration,
    /// Network throughput in bytes/sec
    pub network_throughput: u64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// CPU usage percentage
    pub cpu_usage: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            tps: 0.0,
            avg_block_time: Duration::from_secs(1),
            avg_confirmation_time: Duration::from_secs(2),
            network_throughput: 0,
            memory_usage: 0,
            cpu_usage: 0.0,
        }
    }
}

/// Performance monitor for tracking blockchain metrics
pub struct PerformanceMonitor {
    metrics: parking_lot::RwLock<PerformanceMetrics>,
    tx_count: parking_lot::RwLock<u64>,
    block_times: parking_lot::RwLock<Vec<Duration>>,
    confirmation_times: parking_lot::RwLock<Vec<Duration>>,
    start_time: std::time::Instant,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: parking_lot::RwLock::new(PerformanceMetrics::default()),
            tx_count: parking_lot::RwLock::new(0),
            block_times: parking_lot::RwLock::new(Vec::new()),
            confirmation_times: parking_lot::RwLock::new(Vec::new()),
            start_time: std::time::Instant::now(),
        }
    }

    /// Record a new block
    pub fn record_block(&self, tx_count: usize, block_time: Duration) {
        *self.tx_count.write() += tx_count as u64;

        let mut block_times = self.block_times.write();
        block_times.push(block_time);
        if block_times.len() > 100 {
            block_times.remove(0);
        }

        self.update_metrics();
    }

    /// Record transaction confirmation
    pub fn record_confirmation(&self, confirmation_time: Duration) {
        let mut confirmation_times = self.confirmation_times.write();
        confirmation_times.push(confirmation_time);
        if confirmation_times.len() > 1000 {
            confirmation_times.remove(0);
        }

        self.update_metrics();
    }

    /// Update computed metrics
    fn update_metrics(&self) {
        let mut metrics = self.metrics.write();

        // Calculate TPS
        let total_time = self.start_time.elapsed().as_secs_f64();
        if total_time > 0.0 {
            metrics.tps = *self.tx_count.read() as f64 / total_time;
        }

        // Calculate average block time
        let block_times = self.block_times.read();
        if !block_times.is_empty() {
            let total_block_time: Duration = block_times.iter().sum();
            metrics.avg_block_time = total_block_time / block_times.len() as u32;
        }

        // Calculate average confirmation time
        let confirmation_times = self.confirmation_times.read();
        if !confirmation_times.is_empty() {
            let total_confirmation_time: Duration = confirmation_times.iter().sum();
            metrics.avg_confirmation_time =
                total_confirmation_time / confirmation_times.len() as u32;
        }
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().clone()
    }

    /// Reset metrics
    pub fn reset(&self) {
        *self.metrics.write() = PerformanceMetrics::default();
        *self.tx_count.write() = 0;
        self.block_times.write().clear();
        self.confirmation_times.write().clear();
    }
}

/// Adaptive parameter manager for optimizing performance
pub struct AdaptiveParams {
    /// Current block time target
    pub block_time_target: Duration,
    /// Current gas limit
    pub gas_limit: u64,
    /// Current base fee
    pub base_fee: u64,
    /// Performance monitor
    monitor: PerformanceMonitor,
}

impl AdaptiveParams {
    pub fn new() -> Self {
        Self {
            block_time_target: Duration::from_secs(1),
            gas_limit: 10_000_000,
            base_fee: 1000,
            monitor: PerformanceMonitor::new(),
        }
    }

    /// Adapt parameters based on network performance
    pub fn adapt(&mut self, _network_size: usize, _network_latency: Duration) {
        let metrics = self.monitor.get_metrics();

        // Adapt block time based on network conditions
        if metrics.avg_block_time > Duration::from_millis(2000) {
            // Blocks taking too long, increase target time
            self.block_time_target = self.block_time_target + Duration::from_millis(100);
        } else if metrics.avg_block_time < Duration::from_millis(500) {
            // Blocks too fast, might be unsafe, but allow for high performance
            self.block_time_target = (self.block_time_target - Duration::from_millis(50))
                .max(Duration::from_millis(300));
        }

        // Adapt gas limit based on throughput
        if metrics.tps > 10000.0 {
            // High throughput, can increase gas limit
            self.gas_limit = (self.gas_limit as f64 * 1.1) as u64;
        } else if metrics.tps < 1000.0 && self.gas_limit > 1_000_000 {
            // Low throughput, decrease gas limit
            self.gas_limit = (self.gas_limit as f64 * 0.9) as u64;
        }

        // Adapt base fee based on network usage
        if metrics.avg_confirmation_time > Duration::from_secs(5) {
            // Slow confirmations, increase fee to prioritize
            self.base_fee = (self.base_fee as f64 * 1.2) as u64;
        } else if metrics.avg_confirmation_time < Duration::from_secs(1) {
            // Fast confirmations, can reduce fee
            self.base_fee = (self.base_fee as f64 * 0.95) as u64;
        }

        // Ensure reasonable bounds
        self.gas_limit = self.gas_limit.max(1_000_000).min(100_000_000);
        self.base_fee = self.base_fee.max(100).min(100_000);
    }

    /// Get current parameters
    pub fn get_params(&self) -> (Duration, u64, u64) {
        (self.block_time_target, self.gas_limit, self.base_fee)
    }

    /// Get performance monitor
    pub fn get_monitor(&self) -> &PerformanceMonitor {
        &self.monitor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_time_calculation() {
        // Small network, low latency
        let time1 = calculate_block_time(100, 50);
        assert!(time1 >= Duration::from_millis(500));
        assert!(time1 <= Duration::from_millis(2000));

        // Large network, high latency
        let time2 = calculate_block_time(10000, 500);
        assert!(time2 > time1);
        assert!(time2 <= Duration::from_millis(10000));
    }

    #[test]
    fn test_gas_limit_calculation() {
        let limit1 = calculate_gas_limit(1000, 21000);
        let limit2 = calculate_gas_limit(10000, 21000);
        assert!(limit2 > limit1);
    }

    #[test]
    fn test_fee_estimation() {
        let fee1 = estimate_fee(250, 0.0, 1000); // No congestion
        let fee2 = estimate_fee(250, 1.0, 1000); // High congestion
        assert!(fee2 > fee1);
    }
}
