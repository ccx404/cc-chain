//! CC Chain Examples and Tutorials
//!
//! This crate provides educational examples and tutorials for learning
//! how to use CC Chain and build applications on top of it.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TutorialError {
    #[error("Tutorial step error: {0}")]
    StepError(String),
    #[error("Prerequisites not met: {0}")]
    PrerequisitesError(String),
    #[error("Tutorial execution error: {0}")]
    ExecutionError(String),
}

pub type Result<T> = std::result::Result<T, TutorialError>;

/// Tutorial step definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialStep {
    pub id: String,
    pub title: String,
    pub description: String,
    pub code_example: String,
    pub expected_output: Option<String>,
    pub prerequisites: Vec<String>,
}

/// Complete tutorial definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tutorial {
    pub id: String,
    pub title: String,
    pub description: String,
    pub difficulty: TutorialDifficulty,
    pub estimated_time: u32, // minutes
    pub steps: Vec<TutorialStep>,
    pub tags: Vec<String>,
}

/// Tutorial difficulty levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TutorialDifficulty {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// Tutorial execution engine
#[derive(Debug)]
pub struct TutorialRunner {
    tutorials: HashMap<String, Tutorial>,
    progress: HashMap<String, TutorialProgress>,
}

/// Tracks progress through a tutorial
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialProgress {
    pub tutorial_id: String,
    pub current_step: usize,
    pub completed_steps: Vec<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl TutorialRunner {
    /// Create a new tutorial runner
    pub fn new() -> Self {
        let mut runner = Self {
            tutorials: HashMap::new(),
            progress: HashMap::new(),
        };
        
        // Load built-in tutorials
        runner.load_built_in_tutorials();
        runner
    }

    /// Load built-in tutorials
    fn load_built_in_tutorials(&mut self) {
        // Basic blockchain concepts tutorial
        self.add_tutorial(self.create_basic_blockchain_tutorial());
        
        // Smart contract development tutorial
        self.add_tutorial(self.create_smart_contract_tutorial());
        
        // RPC API tutorial
        self.add_tutorial(self.create_rpc_tutorial());
        
        // Consensus mechanism tutorial
        self.add_tutorial(self.create_consensus_tutorial());
        
        // Network integration tutorial
        self.add_tutorial(self.create_networking_tutorial());
    }

    /// Add a tutorial to the runner
    pub fn add_tutorial(&mut self, tutorial: Tutorial) {
        self.tutorials.insert(tutorial.id.clone(), tutorial);
    }

    /// Get available tutorials
    pub fn list_tutorials(&self) -> Vec<&Tutorial> {
        self.tutorials.values().collect()
    }

    /// Get tutorials by difficulty
    pub fn tutorials_by_difficulty(&self, difficulty: TutorialDifficulty) -> Vec<&Tutorial> {
        self.tutorials
            .values()
            .filter(|t| std::mem::discriminant(&t.difficulty) == std::mem::discriminant(&difficulty))
            .collect()
    }

    /// Start a tutorial
    pub fn start_tutorial(&mut self, tutorial_id: &str, user_id: &str) -> Result<()> {
        if !self.tutorials.contains_key(tutorial_id) {
            return Err(TutorialError::StepError(format!("Tutorial '{}' not found", tutorial_id)));
        }

        let progress = TutorialProgress {
            tutorial_id: tutorial_id.to_string(),
            current_step: 0,
            completed_steps: Vec::new(),
            started_at: chrono::Utc::now(),
            last_updated: chrono::Utc::now(),
        };

        self.progress.insert(user_id.to_string(), progress);
        Ok(())
    }

    /// Get next step for user
    pub fn get_next_step(&self, user_id: &str) -> Result<Option<&TutorialStep>> {
        let progress = self.progress.get(user_id)
            .ok_or_else(|| TutorialError::StepError("No active tutorial for user".to_string()))?;

        let tutorial = self.tutorials.get(&progress.tutorial_id)
            .ok_or_else(|| TutorialError::StepError("Tutorial not found".to_string()))?;

        Ok(tutorial.steps.get(progress.current_step))
    }

    /// Complete current step and advance
    pub fn complete_step(&mut self, user_id: &str) -> Result<bool> {
        let progress = self.progress.get_mut(user_id)
            .ok_or_else(|| TutorialError::StepError("No active tutorial for user".to_string()))?;

        let tutorial = self.tutorials.get(&progress.tutorial_id)
            .ok_or_else(|| TutorialError::StepError("Tutorial not found".to_string()))?;

        if progress.current_step < tutorial.steps.len() {
            let step_id = tutorial.steps[progress.current_step].id.clone();
            progress.completed_steps.push(step_id);
            progress.current_step += 1;
            progress.last_updated = chrono::Utc::now();
            
            // Return true if tutorial is complete
            Ok(progress.current_step >= tutorial.steps.len())
        } else {
            Ok(true) // Already complete
        }
    }

    /// Create basic blockchain tutorial
    fn create_basic_blockchain_tutorial(&self) -> Tutorial {
        Tutorial {
            id: "basic-blockchain".to_string(),
            title: "Basic Blockchain Concepts".to_string(),
            description: "Learn the fundamental concepts of blockchain technology using CC Chain".to_string(),
            difficulty: TutorialDifficulty::Beginner,
            estimated_time: 30,
            steps: vec![
                TutorialStep {
                    id: "hash-functions".to_string(),
                    title: "Understanding Hash Functions".to_string(),
                    description: "Learn how hash functions work in blockchain".to_string(),
                    code_example: r#"
use cc_core::hash::CCHash;

// Create a hash from data
let data = b"Hello, CC Chain!";
let hash = CCHash::hash(data);
println!("Hash: {}", hash);
"#.to_string(),
                    expected_output: Some("Hash: [hash value]".to_string()),
                    prerequisites: vec![],
                },
                TutorialStep {
                    id: "blocks".to_string(),
                    title: "Creating Blocks".to_string(),
                    description: "Learn how to create and link blocks".to_string(),
                    code_example: r#"
use cc_core::block::Block;
use cc_core::transaction::Transaction;

// Create a new block
let mut block = Block::new(0, CCHash::default());
block.add_transaction(Transaction::new(/* ... */));
let block_hash = block.calculate_hash();
"#.to_string(),
                    expected_output: Some("Block created with hash".to_string()),
                    prerequisites: vec!["hash-functions".to_string()],
                },
            ],
            tags: vec!["beginner".to_string(), "blockchain".to_string(), "fundamentals".to_string()],
        }
    }

    /// Create smart contract tutorial
    fn create_smart_contract_tutorial(&self) -> Tutorial {
        Tutorial {
            id: "smart-contracts".to_string(),
            title: "Smart Contract Development".to_string(),
            description: "Learn to develop and deploy smart contracts on CC Chain".to_string(),
            difficulty: TutorialDifficulty::Intermediate,
            estimated_time: 60,
            steps: vec![
                TutorialStep {
                    id: "contract-basics".to_string(),
                    title: "Smart Contract Basics".to_string(),
                    description: "Understanding smart contract fundamentals".to_string(),
                    code_example: r#"
// Example smart contract in Rust/WASM
#[no_mangle]
pub extern "C" fn init() -> i32 {
    // Contract initialization
    0
}

#[no_mangle]
pub extern "C" fn call() -> i32 {
    // Contract execution
    0
}
"#.to_string(),
                    expected_output: Some("Contract functions defined".to_string()),
                    prerequisites: vec![],
                },
            ],
            tags: vec!["intermediate".to_string(), "smart-contracts".to_string(), "wasm".to_string()],
        }
    }

    /// Create RPC tutorial
    fn create_rpc_tutorial(&self) -> Tutorial {
        Tutorial {
            id: "rpc-api".to_string(),
            title: "Using CC Chain RPC API".to_string(),
            description: "Learn to interact with CC Chain through RPC calls".to_string(),
            difficulty: TutorialDifficulty::Beginner,
            estimated_time: 45,
            steps: vec![
                TutorialStep {
                    id: "rpc-client".to_string(),
                    title: "Setting up RPC Client".to_string(),
                    description: "Configure and use the CC Chain RPC client".to_string(),
                    code_example: r#"
use cc_rpc::client::RpcClient;

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = RpcClient::new("http://localhost:8545").await?;
    let block_number = client.get_block_number().await?;
    println!("Current block: {}", block_number);
    Ok(())
}
"#.to_string(),
                    expected_output: Some("Current block: [number]".to_string()),
                    prerequisites: vec![],
                },
            ],
            tags: vec!["beginner".to_string(), "rpc".to_string(), "api".to_string()],
        }
    }

    /// Create consensus tutorial
    fn create_consensus_tutorial(&self) -> Tutorial {
        Tutorial {
            id: "consensus-mechanism".to_string(),
            title: "Understanding CC Chain Consensus".to_string(),
            description: "Deep dive into CC Chain's consensus mechanism".to_string(),
            difficulty: TutorialDifficulty::Advanced,
            estimated_time: 90,
            steps: vec![
                TutorialStep {
                    id: "consensus-overview".to_string(),
                    title: "Consensus Overview".to_string(),
                    description: "Understanding how consensus works in CC Chain".to_string(),
                    code_example: r#"
use consensus::CCConsensus;

// Initialize consensus engine
let consensus = CCConsensus::new(validator_keys);
let proposal = consensus.create_proposal(transactions);
"#.to_string(),
                    expected_output: Some("Consensus proposal created".to_string()),
                    prerequisites: vec![],
                },
            ],
            tags: vec!["advanced".to_string(), "consensus".to_string(), "validator".to_string()],
        }
    }

    /// Create networking tutorial
    fn create_networking_tutorial(&self) -> Tutorial {
        Tutorial {
            id: "networking".to_string(),
            title: "CC Chain Networking".to_string(),
            description: "Learn about peer-to-peer networking in CC Chain".to_string(),
            difficulty: TutorialDifficulty::Intermediate,
            estimated_time: 75,
            steps: vec![
                TutorialStep {
                    id: "peer-discovery".to_string(),
                    title: "Peer Discovery".to_string(),
                    description: "Understanding how nodes find each other".to_string(),
                    code_example: r#"
use networking::discovery::PeerDiscovery;

let discovery = PeerDiscovery::new();
let peers = discovery.discover_peers().await?;
println!("Found {} peers", peers.len());
"#.to_string(),
                    expected_output: Some("Found [n] peers".to_string()),
                    prerequisites: vec![],
                },
            ],
            tags: vec!["intermediate".to_string(), "networking".to_string(), "p2p".to_string()],
        }
    }
}

impl Default for TutorialRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// Tutorial utilities and helpers
pub mod tutorial_utils {
    use super::*;

    /// Format tutorial difficulty as string
    pub fn difficulty_to_string(difficulty: &TutorialDifficulty) -> &'static str {
        match difficulty {
            TutorialDifficulty::Beginner => "Beginner",
            TutorialDifficulty::Intermediate => "Intermediate", 
            TutorialDifficulty::Advanced => "Advanced",
            TutorialDifficulty::Expert => "Expert",
        }
    }

    /// Estimate completion time based on difficulty
    pub fn estimate_time(difficulty: &TutorialDifficulty, base_time: u32) -> u32 {
        match difficulty {
            TutorialDifficulty::Beginner => base_time,
            TutorialDifficulty::Intermediate => (base_time as f32 * 1.5) as u32,
            TutorialDifficulty::Advanced => base_time * 2,
            TutorialDifficulty::Expert => base_time * 3,
        }
    }

    /// Generate tutorial completion certificate
    pub fn generate_certificate(user_id: &str, tutorial_id: &str) -> String {
        format!(
            "Certificate of Completion\n\
             User: {}\n\
             Tutorial: {}\n\
             Completed: {}\n\
             ---\n\
             CC Chain Tutorial System",
            user_id,
            tutorial_id,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tutorial_runner_creation() {
        let runner = TutorialRunner::new();
        assert!(!runner.list_tutorials().is_empty());
    }

    #[test]
    fn test_tutorial_filtering_by_difficulty() {
        let runner = TutorialRunner::new();
        let beginner_tutorials = runner.tutorials_by_difficulty(TutorialDifficulty::Beginner);
        assert!(!beginner_tutorials.is_empty());
    }

    #[test]
    fn test_tutorial_progress() {
        let mut runner = TutorialRunner::new();
        let user_id = "test_user";
        let tutorial_id = "basic-blockchain";

        // Start tutorial
        runner.start_tutorial(tutorial_id, user_id).unwrap();
        
        // Get first step
        let step = runner.get_next_step(user_id).unwrap();
        assert!(step.is_some());
        
        // Complete step
        let is_complete = runner.complete_step(user_id).unwrap();
        assert!(!is_complete); // Should not be complete after first step
    }

    #[test]
    fn test_difficulty_formatting() {
        assert_eq!(tutorial_utils::difficulty_to_string(&TutorialDifficulty::Beginner), "Beginner");
        assert_eq!(tutorial_utils::difficulty_to_string(&TutorialDifficulty::Advanced), "Advanced");
    }

    #[test]
    fn test_time_estimation() {
        assert_eq!(tutorial_utils::estimate_time(&TutorialDifficulty::Beginner, 30), 30);
        assert_eq!(tutorial_utils::estimate_time(&TutorialDifficulty::Advanced, 30), 60);
    }

    #[test]
    fn test_certificate_generation() {
        let cert = tutorial_utils::generate_certificate("user123", "basic-blockchain");
        assert!(cert.contains("user123"));
        assert!(cert.contains("basic-blockchain"));
        assert!(cert.contains("Certificate of Completion"));
    }
}
