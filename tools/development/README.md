# CC Chain Development Tools

This crate provides essential development tools and utilities for CC Chain development, including performance profiling, code generation, and live reload capabilities.

## Features

### 🔧 Development Profiler
- Time measurement for code sections
- Performance statistics collection
- Average, maximum, and count metrics

### 🏗️ Code Generator
- Template-based code scaffolding
- Module and test template generation
- Variable substitution in templates

### 🔄 Live Reload
- File system watching
- Automatic change detection
- Development server support

## Usage

### Performance Profiling

```rust
use tools_development::DevelopmentProfiler;

let mut profiler = DevelopmentProfiler::new();

// Start timing an operation
profiler.start_timer("database_query");

// ... perform operation ...

// Stop timing and get duration
let duration = profiler.stop_timer("database_query")?;
println!("Operation took: {:?}", duration);

// Get statistics
let stats = profiler.get_stats();
for (name, (avg, max, count)) in stats {
    println!("{}: avg={:?}, max={:?}, count={}", name, avg, max, count);
}
```

### Code Generation

```rust
use tools_development::CodeGenerator;
use std::collections::HashMap;

let generator = CodeGenerator::new();
let mut variables = HashMap::new();
variables.insert("module_name".to_string(), "MyModule".to_string());

let code = generator.generate("module", &variables)?;
println!("{}", code);
```

### Live Reload

```rust
use tools_development::{LiveReload, DevServerConfig};

let config = DevServerConfig::default();
let mut reload = LiveReload::new(config);

// Watch files
reload.watch_path("src/")?;

// Check for changes
let changed_files = reload.check_changes()?;
if !changed_files.is_empty() {
    println!("Changed files: {:?}", changed_files);
}
```

## Utilities

The `utils` module provides helper functions:

- `format_bytes(bytes: u64)` - Human-readable byte formatting
- `format_duration(duration: Duration)` - Pretty duration formatting  
- `generate_session_id()` - Unique development session IDs

## Templates

The crate includes built-in templates for:

- **module.rs.template** - Basic Rust module structure
- **test.rs.template** - Comprehensive test suite template

## Configuration

Development server configuration options:

```rust
DevServerConfig {
    host: "127.0.0.1".to_string(),
    port: 3030,
    auto_reload: true,
    debug_mode: true,
}
```

## Examples

See the [examples](examples/) directory for complete usage examples.