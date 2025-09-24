//! CC Chain Integration Testing
//!
//! This crate provides integration testing utilities for testing interactions
//! between different CC Chain components and external systems.

use std::collections::HashMap;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IntegrationError {
    #[error("Integration test setup error: {0}")]
    Setup(String),
    #[error("Component interaction error: {0}")]
    ComponentInteraction(String),
    #[error("Test environment error: {0}")]
    Environment(String),
}

pub type Result<T> = std::result::Result<T, IntegrationError>;

/// Integration test suite runner
pub struct IntegrationTestSuite {
    name: String,
    components: HashMap<String, Box<dyn TestComponent>>,
    test_cases: Vec<IntegrationTestCase>,
}

/// Test component interface
pub trait TestComponent {
    fn name(&self) -> &str;
    fn setup(&mut self) -> Result<()>;
    fn teardown(&mut self) -> Result<()>;
    fn is_ready(&self) -> bool;
}

/// Integration test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationTestCase {
    pub name: String,
    pub description: String,
    pub components: Vec<String>,
    pub timeout: Duration,
}

impl IntegrationTestSuite {
    /// Create a new integration test suite
    pub fn new(name: &str) -> Self {
        IntegrationTestSuite {
            name: name.to_string(),
            components: HashMap::new(),
            test_cases: Vec::new(),
        }
    }
    
    /// Add a component to the test suite
    pub fn add_component(&mut self, component: Box<dyn TestComponent>) {
        let name = component.name().to_string();
        self.components.insert(name, component);
    }
    
    /// Add a test case
    pub fn add_test_case(&mut self, test_case: IntegrationTestCase) {
        self.test_cases.push(test_case);
    }
    
    /// Run all integration tests
    pub fn run_all(&mut self) -> Result<Vec<TestResult>> {
        let mut results = Vec::new();
        
        // Setup all components
        for component in self.components.values_mut() {
            component.setup()?;
        }
        
        // Run each test case
        for test_case in &self.test_cases {
            let result = self.run_test_case(test_case)?;
            results.push(result);
        }
        
        // Teardown all components
        for component in self.components.values_mut() {
            component.teardown()?;
        }
        
        Ok(results)
    }
    
    fn run_test_case(&self, test_case: &IntegrationTestCase) -> Result<TestResult> {
        let start_time = std::time::Instant::now();
        
        // Verify all required components are ready
        for component_name in &test_case.components {
            let component = self.components.get(component_name)
                .ok_or_else(|| IntegrationError::ComponentInteraction(
                    format!("Component '{}' not found", component_name)
                ))?;
            
            if !component.is_ready() {
                return Ok(TestResult {
                    name: test_case.name.clone(),
                    success: false,
                    duration: start_time.elapsed(),
                    error_message: Some(format!("Component '{}' not ready", component_name)),
                });
            }
        }
        
        // Simulate test execution
        std::thread::sleep(Duration::from_millis(10));
        
        Ok(TestResult {
            name: test_case.name.clone(),
            success: true,
            duration: start_time.elapsed(),
            error_message: None,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub success: bool,
    pub duration: Duration,
    pub error_message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct MockComponent {
        name: String,
        ready: bool,
    }
    
    impl TestComponent for MockComponent {
        fn name(&self) -> &str {
            &self.name
        }
        
        fn setup(&mut self) -> Result<()> {
            self.ready = true;
            Ok(())
        }
        
        fn teardown(&mut self) -> Result<()> {
            self.ready = false;
            Ok(())
        }
        
        fn is_ready(&self) -> bool {
            self.ready
        }
    }
    
    #[test]
    fn test_integration_suite() {
        let mut suite = IntegrationTestSuite::new("test_suite");
        
        let component1 = MockComponent {
            name: "component1".to_string(),
            ready: false,
        };
        
        suite.add_component(Box::new(component1));
        
        let test_case = IntegrationTestCase {
            name: "test_case_1".to_string(),
            description: "Test component interaction".to_string(),
            components: vec!["component1".to_string()],
            timeout: Duration::from_secs(30),
        };
        
        suite.add_test_case(test_case);
        
        let results = suite.run_all().unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
    }
}
