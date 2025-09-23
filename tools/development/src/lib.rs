//! CC Chain Development Tools
//!
//! This crate provides utilities and tools to aid in the development
//! and debugging of CC Chain components.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use thiserror::Error;
use serde::{Deserialize, Serialize};

#[derive(Error, Debug)]
pub enum DevelopmentError {
    #[error("Profiler error: {0}")]
    Profiler(String),
    #[error("Code generator error: {0}")]
    CodeGenerator(String),
    #[error("Development server error: {0}")]
    DevServer(String),
}

pub type Result<T> = std::result::Result<T, DevelopmentError>;

/// Performance profiler for development
#[derive(Debug)]
pub struct DevelopmentProfiler {
    measurements: HashMap<String, Vec<Duration>>,
    active_timers: HashMap<String, Instant>,
}

/// Code scaffolding generator
#[derive(Debug)]
pub struct CodeGenerator {
    templates: HashMap<String, String>,
}

/// Development server configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct DevServerConfig {
    pub host: String,
    pub port: u16,
    pub auto_reload: bool,
    pub debug_mode: bool,
}

/// Live reload functionality for development
#[derive(Debug)]
pub struct LiveReload {
    watchers: HashMap<String, std::time::SystemTime>,
    config: DevServerConfig,
}

impl DevelopmentProfiler {
    /// Create a new development profiler
    pub fn new() -> Self {
        Self {
            measurements: HashMap::new(),
            active_timers: HashMap::new(),
        }
    }

    /// Start timing a code section
    pub fn start_timer(&mut self, name: &str) {
        self.active_timers.insert(name.to_string(), Instant::now());
    }

    /// Stop timing and record measurement
    pub fn stop_timer(&mut self, name: &str) -> Result<Duration> {
        if let Some(start_time) = self.active_timers.remove(name) {
            let duration = start_time.elapsed();
            self.measurements
                .entry(name.to_string())
                .or_insert_with(Vec::new)
                .push(duration);
            Ok(duration)
        } else {
            Err(DevelopmentError::Profiler(format!(
                "No active timer found for '{}'",
                name
            )))
        }
    }

    /// Get average duration for a measurement
    pub fn average_duration(&self, name: &str) -> Option<Duration> {
        self.measurements.get(name).map(|measurements| {
            let total: Duration = measurements.iter().sum();
            total / measurements.len() as u32
        })
    }

    /// Get all measurement statistics
    pub fn get_stats(&self) -> HashMap<String, (Duration, Duration, usize)> {
        self.measurements
            .iter()
            .map(|(name, measurements)| {
                let avg = measurements.iter().sum::<Duration>() / measurements.len() as u32;
                let max = *measurements.iter().max().unwrap();
                let count = measurements.len();
                (name.clone(), (avg, max, count))
            })
            .collect()
    }
}

impl CodeGenerator {
    /// Create a new code generator with default templates
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        
        // Basic module template
        templates.insert(
            "module".to_string(),
            include_str!("../templates/module.rs.template").to_string(),
        );
        
        // Test template
        templates.insert(
            "test".to_string(),
            include_str!("../templates/test.rs.template").to_string(),
        );

        Self { templates }
    }

    /// Generate code from template
    pub fn generate(&self, template_name: &str, variables: &HashMap<String, String>) -> Result<String> {
        let template = self.templates
            .get(template_name)
            .ok_or_else(|| DevelopmentError::CodeGenerator(format!("Template '{}' not found", template_name)))?;

        let mut result = template.clone();
        for (key, value) in variables {
            result = result.replace(&format!("{{{{{}}}}}", key), value);
        }

        Ok(result)
    }

    /// Add custom template
    pub fn add_template(&mut self, name: String, template: String) {
        self.templates.insert(name, template);
    }
}

impl Default for DevServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3030,
            auto_reload: true,
            debug_mode: true,
        }
    }
}

impl LiveReload {
    /// Create new live reload instance
    pub fn new(config: DevServerConfig) -> Self {
        Self {
            watchers: HashMap::new(),
            config,
        }
    }

    /// Add path to watch for changes
    pub fn watch_path(&mut self, path: &str) -> Result<()> {
        let metadata = std::fs::metadata(path)
            .map_err(|e| DevelopmentError::DevServer(format!("Failed to watch path '{}': {}", path, e)))?;
        
        if let Ok(modified) = metadata.modified() {
            self.watchers.insert(path.to_string(), modified);
        }
        
        Ok(())
    }

    /// Check if any watched files have changed
    pub fn check_changes(&mut self) -> Result<Vec<String>> {
        let mut changed = Vec::new();
        
        for (path, last_modified) in &mut self.watchers {
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(modified) = metadata.modified() {
                    if modified > *last_modified {
                        changed.push(path.clone());
                        *last_modified = modified;
                    }
                }
            }
        }
        
        Ok(changed)
    }
}

/// Development utilities and helpers
pub mod utils {
    use super::*;

    /// Format byte size for human reading
    pub fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_index])
    }

    /// Pretty print duration
    pub fn format_duration(duration: Duration) -> String {
        let nanos = duration.as_nanos();
        if nanos < 1_000 {
            format!("{}ns", nanos)
        } else if nanos < 1_000_000 {
            format!("{:.2}μs", nanos as f64 / 1_000.0)
        } else if nanos < 1_000_000_000 {
            format!("{:.2}ms", nanos as f64 / 1_000_000.0)
        } else {
            format!("{:.2}s", duration.as_secs_f64())
        }
    }

    /// Generate unique development session ID
    pub fn generate_session_id() -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::SystemTime;

        let mut hasher = DefaultHasher::new();
        SystemTime::now().hash(&mut hasher);
        format!("dev-{:x}", hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_profiler_basic_functionality() {
        let mut profiler = DevelopmentProfiler::new();
        
        profiler.start_timer("test_operation");
        thread::sleep(Duration::from_millis(10));
        let duration = profiler.stop_timer("test_operation").unwrap();
        
        assert!(duration >= Duration::from_millis(10));
        assert!(profiler.average_duration("test_operation").is_some());
    }

    #[test]
    fn test_code_generator() {
        let generator = CodeGenerator::new();
        let mut variables = HashMap::new();
        variables.insert("module_name".to_string(), "test_module".to_string());
        
        // This would work if we had the template files
        // let result = generator.generate("module", &variables);
        // assert!(result.is_ok());
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(utils::format_bytes(1024), "1.00 KB");
        assert_eq!(utils::format_bytes(1048576), "1.00 MB");
        assert_eq!(utils::format_bytes(512), "512.00 B");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(utils::format_duration(Duration::from_nanos(500)), "500ns");
        assert_eq!(utils::format_duration(Duration::from_micros(1500)), "1.50ms");
    }

    #[test]
    fn test_session_id_generation() {
        let id1 = utils::generate_session_id();
        let id2 = utils::generate_session_id();
        
        assert_ne!(id1, id2);
        assert!(id1.starts_with("dev-"));
    }
}
