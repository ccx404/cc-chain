# CC Chain Examples and Tutorials

This crate provides comprehensive educational examples and interactive tutorials for learning CC Chain development and building blockchain applications.

## Features

### 📚 Interactive Tutorials
- Step-by-step guided learning
- Progress tracking
- Hands-on code examples
- Difficulty-based filtering

### 🎯 Built-in Tutorials

#### 1. Basic Blockchain Concepts (Beginner - 30 mins)
Learn fundamental blockchain concepts including:
- Hash functions and their role
- Block creation and linking
- Transaction basics

#### 2. Smart Contract Development (Intermediate - 60 mins)
Comprehensive smart contract tutorial covering:
- Contract basics and structure
- WASM compilation
- Deployment and interaction

#### 3. RPC API Usage (Beginner - 45 mins)
Learn to interact with CC Chain through RPC:
- Client setup and configuration
- Making RPC calls
- Handling responses

#### 4. Consensus Mechanism (Advanced - 90 mins)
Deep dive into CC Chain consensus:
- Consensus overview and principles
- Validator participation
- Fault tolerance mechanisms

#### 5. Networking (Intermediate - 75 mins)
Peer-to-peer networking fundamentals:
- Peer discovery mechanisms
- Network protocols
- Connection management

## Usage

### Basic Tutorial Runner

```rust
use examples_tutorials::{TutorialRunner, TutorialDifficulty};

let mut runner = TutorialRunner::new();

// List available tutorials
let tutorials = runner.list_tutorials();
for tutorial in tutorials {
    println!("Tutorial: {} ({})", tutorial.title, tutorial.id);
    println!("Difficulty: {:?}", tutorial.difficulty);
    println!("Estimated time: {} minutes", tutorial.estimated_time);
}

// Start a tutorial
let user_id = "student123";
runner.start_tutorial("basic-blockchain", user_id)?;

// Get next step
if let Some(step) = runner.get_next_step(user_id)? {
    println!("Step: {}", step.title);
    println!("Description: {}", step.description);
    println!("Code example:\n{}", step.code_example);
}

// Complete step and advance
let is_complete = runner.complete_step(user_id)?;
if is_complete {
    println!("Tutorial completed! 🎉");
}
```

### Filter by Difficulty

```rust
// Get only beginner tutorials
let beginner_tutorials = runner.tutorials_by_difficulty(TutorialDifficulty::Beginner);

// Get intermediate and advanced tutorials
let intermediate = runner.tutorials_by_difficulty(TutorialDifficulty::Intermediate);
let advanced = runner.tutorials_by_difficulty(TutorialDifficulty::Advanced);
```

### Tutorial Structure

Each tutorial consists of:

```rust
Tutorial {
    id: String,                    // Unique identifier
    title: String,                 // Human-readable title
    description: String,           // Tutorial description
    difficulty: TutorialDifficulty, // Beginner/Intermediate/Advanced/Expert
    estimated_time: u32,           // Minutes to complete
    steps: Vec<TutorialStep>,      // Individual steps
    tags: Vec<String>,             // Searchable tags
}
```

Each step includes:

```rust
TutorialStep {
    id: String,                    // Step identifier
    title: String,                 // Step title
    description: String,           // What you'll learn
    code_example: String,          // Executable code
    expected_output: Option<String>, // Expected result
    prerequisites: Vec<String>,    // Required previous steps
}
```

## Utilities

### Certificate Generation

```rust
use examples_tutorials::tutorial_utils;

let certificate = tutorial_utils::generate_certificate("user123", "basic-blockchain");
println!("{}", certificate);
```

### Time Estimation

```rust
let estimated = tutorial_utils::estimate_time(&TutorialDifficulty::Advanced, 60);
println!("Estimated completion time: {} minutes", estimated);
```

### Difficulty Display

```rust
let difficulty_str = tutorial_utils::difficulty_to_string(&TutorialDifficulty::Intermediate);
println!("Difficulty: {}", difficulty_str);
```

## Progress Tracking

The tutorial system automatically tracks:
- Current step position
- Completed steps
- Start and last update timestamps
- Tutorial completion status

## Creating Custom Tutorials

You can extend the tutorial system by adding custom tutorials:

```rust
let custom_tutorial = Tutorial {
    id: "custom-defi".to_string(),
    title: "Building a DeFi Application".to_string(),
    description: "Learn to build a decentralized finance application".to_string(),
    difficulty: TutorialDifficulty::Expert,
    estimated_time: 120,
    steps: vec![
        // Define your custom steps here
    ],
    tags: vec!["defi".to_string(), "advanced".to_string()],
};

runner.add_tutorial(custom_tutorial);
```

## Best Practices

1. **Start with Basics** - Complete the basic blockchain tutorial first
2. **Practice Code** - Run all code examples in your development environment
3. **Take Notes** - Document your learning progress
4. **Ask Questions** - Use the CC Chain community forums for help
5. **Build Projects** - Apply your knowledge to real projects

## Tutorial Tags

Tutorials are tagged for easy discovery:
- `beginner`, `intermediate`, `advanced`, `expert` - Difficulty levels
- `blockchain`, `consensus`, `networking` - Core concepts
- `smart-contracts`, `wasm`, `defi` - Application development
- `rpc`, `api`, `client` - Integration topics

## Contributing

Want to add more tutorials? See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines on creating educational content.