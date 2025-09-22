# Contributing to CC Chain

Thank you for your interest in contributing to CC Chain! This guide will help you get started with contributing to our high-performance blockchain platform.

## Code of Conduct

We are committed to fostering a welcoming and inclusive community. Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## How to Contribute

### 🐛 Reporting Bugs

Before creating bug reports, please check the [issue tracker](https://github.com/ccx404/cc-chain/issues) to see if the issue already exists.

**When submitting a bug report, please include:**

- **Summary**: Clear and descriptive title
- **Environment**: OS, Rust version, CC Chain version
- **Steps to reproduce**: Minimal steps to trigger the bug
- **Expected behavior**: What you expected to happen
- **Actual behavior**: What actually happened
- **Logs**: Relevant log output (use `RUST_LOG=debug`)

**Bug Report Template:**
```markdown
## Bug Summary
Brief description of the issue

## Environment
- OS: [e.g., Ubuntu 22.04]
- Rust version: [e.g., 1.89.0]
- CC Chain version: [e.g., 1.0.0]

## Steps to Reproduce
1. Start node with `cargo run --bin cc-node -- start`
2. Send transaction with `cargo run --bin cc-node -- send-tx ...`
3. Observe error

## Expected Behavior
Transaction should be processed successfully

## Actual Behavior
Transaction fails with error: ...

## Logs
```
RUST_LOG=debug logs here
```
```

### 🚀 Feature Requests

We welcome feature requests! Please provide:

- **Use case**: Why is this feature needed?
- **Description**: What should the feature do?
- **Implementation**: Any ideas on how it could be implemented?
- **Alternatives**: Other solutions you've considered

### 🔧 Code Contributions

#### Prerequisites

- **Rust 1.89+**: Install from [rustup.rs](https://rustup.rs/)
- **Git**: For version control
- **VS Code** (recommended): With rust-analyzer extension

#### Development Setup

1. **Fork the repository**
   ```bash
   # Fork on GitHub, then clone your fork
   git clone https://github.com/YOUR_USERNAME/cc-chain.git
   cd cc-chain
   ```

2. **Set up the development environment**
   ```bash
   # Build the project
   cargo build
   
   # Run tests
   cargo test
   
   # Check code formatting
   cargo fmt --check
   
   # Run linter
   cargo clippy -- -D warnings
   ```

3. **Create a feature branch**
   ```bash
   git checkout -b feature/amazing-new-feature
   ```

#### Development Workflow

1. **Write tests first** (TDD approach recommended)
2. **Implement the feature**
3. **Run the full test suite**
4. **Update documentation** if needed
5. **Submit a pull request**

#### Code Standards

**Code Formatting:**
```bash
# Format all code
cargo fmt

# Check formatting
cargo fmt --check
```

**Linting:**
```bash
# Run clippy with strict settings
cargo clippy -- -D warnings -D clippy::all -D clippy::pedantic
```

**Testing:**
```bash
# Run all tests
cargo test

# Run tests with coverage
cargo test --all-features

# Run specific test
cargo test test_consensus_basic

# Run integration tests
cargo test --test ccbft_integration
```

**Documentation:**
```bash
# Generate docs
cargo doc --open

# Test doctests
cargo test --doc
```

### 🧪 Testing Guidelines

#### Unit Tests

- Test individual functions and modules
- Use descriptive test names: `test_consensus_handles_byzantine_validators`
- Include edge cases and error conditions
- Maintain >90% code coverage

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_validation_with_valid_signature() {
        let keypair = CCKeypair::generate();
        let mut tx = Transaction::new(/* ... */);
        tx.sign(&keypair);
        
        assert!(tx.validate().is_ok());
        assert!(tx.verify_signature());
    }

    #[test]
    fn test_transaction_validation_fails_with_invalid_signature() {
        let tx = Transaction::new(/* ... */);
        // Don't sign the transaction
        
        assert!(tx.validate().is_err());
        assert!(!tx.verify_signature());
    }
}
```

#### Integration Tests

- Test cross-module functionality
- Place in `tests/` directory
- Test realistic scenarios

```rust
// tests/consensus_integration.rs
#[tokio::test]
async fn test_consensus_with_network_partition() {
    // Test that consensus handles network partitions gracefully
}
```

#### Performance Tests

- Use criterion for benchmarking
- Test critical paths like consensus and transaction processing
- Compare against baselines

```rust
// benches/transaction_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_transaction_validation(c: &mut Criterion) {
    c.bench_function("transaction_validation", |b| {
        let tx = create_sample_transaction();
        b.iter(|| black_box(tx.validate()));
    });
}
```

### 📝 Documentation

#### Code Documentation

- Document all public APIs with rustdoc
- Include examples in documentation
- Explain complex algorithms and data structures

```rust
/// Creates a new transaction with specified parameters.
///
/// # Arguments
///
/// * `from` - Sender's public key
/// * `to` - Recipient's public key
/// * `amount` - Transfer amount in smallest units
/// * `fee` - Transaction fee
/// * `nonce` - Sender's current nonce
///
/// # Examples
///
/// ```
/// use cc_core::{Transaction, crypto::CCKeypair};
/// 
/// let keypair = CCKeypair::generate();
/// let tx = Transaction::new(
///     keypair.public_key(),
///     keypair.public_key(),
///     1000000,
///     1000,
///     0,
///     Vec::new(),
/// );
/// ```
///
/// # Errors
///
/// Returns an error if the signature is invalid.
pub fn new(/* ... */) -> Transaction {
    // Implementation
}
```

#### Markdown Documentation

- Update relevant `.md` files in `docs/`
- Include examples and clear explanations
- Keep documentation current with code changes

### 🔍 Pull Request Process

#### Before Submitting

1. **Rebase on latest main**
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run full test suite**
   ```bash
   cargo test --all-features
   cargo clippy -- -D warnings
   cargo fmt --check
   ```

3. **Update documentation** if needed

4. **Write clear commit messages**
   ```
   feat: add ccBFT timeout adaptation mechanism
   
   - Implement adaptive timeout based on network latency
   - Add performance metrics for timeout tuning
   - Include tests for various network conditions
   
   Closes #123
   ```

#### Pull Request Template

```markdown
## Description
Brief description of the changes

## Type of Change
- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that changes existing functionality)
- [ ] Documentation update
- [ ] Performance improvement
- [ ] Refactoring (no functional changes)

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Performance tests pass (if applicable)
- [ ] Manual testing completed

## Documentation
- [ ] Code is self-documenting
- [ ] Rustdoc comments added/updated
- [ ] Relevant markdown docs updated
- [ ] Examples updated (if applicable)

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] No new compiler warnings
- [ ] Breaking changes are documented
- [ ] Performance impact considered

## Related Issues
Closes #123
Relates to #456
```

#### Review Process

1. **Automated checks** must pass (CI/CD)
2. **Peer review** by maintainers
3. **Testing** on test networks
4. **Documentation review**
5. **Final approval** and merge

## 🏗️ Architecture Guidelines

### Module Organization

- **Core** (`cc-core`): Fundamental blockchain primitives
- **Consensus** (`consensus`): ccBFT and safety systems
- **Networking** (`networking`): P2P and bridge functionality
- **CLI** (`cli`): Command-line tools and node binary

### Coding Principles

1. **Safety First**: Use Rust's type system to prevent bugs
2. **Performance**: Profile and optimize critical paths
3. **Modularity**: Clear separation of concerns
4. **Testability**: Design for easy testing
5. **Documentation**: Code should be self-documenting

### Design Patterns

- **Error Handling**: Use `Result<T, CCError>` consistently
- **Async/Await**: Use Tokio for async operations
- **Serialization**: Use Serde for data structures
- **Concurrency**: Prefer message passing over shared state

## 🔒 Security

### Security Review Process

- All cryptographic code requires security review
- Consensus changes need thorough analysis
- Network protocol changes must be audited
- Performance optimizations reviewed for side-channel attacks

### Reporting Security Issues

**Do not open public issues for security vulnerabilities.**

Email security issues to: **security@cc-chain.org**

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

We will respond within 48 hours and provide updates on resolution.

## 🚀 Release Process

### Version Numbering

We follow [Semantic Versioning](https://semver.org/):
- **MAJOR**: Breaking API changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Release Checklist

1. **Version bump** in `Cargo.toml`
2. **Update CHANGELOG.md**
3. **Full test suite** passes
4. **Documentation** is current
5. **Security review** completed
6. **Performance benchmarks** acceptable
7. **Create release** on GitHub

## 📞 Getting Help

- **Documentation**: https://docs.cc-chain.org
- **Discord**: https://discord.gg/cc-chain
- **GitHub Discussions**: https://github.com/ccx404/cc-chain/discussions
- **Email**: dev@cc-chain.org

## 🙏 Recognition

Contributors are recognized in:
- **CONTRIBUTORS.md** file
- **Release notes**
- **Annual contributor report**

Significant contributors may be invited to join the core team.

---

Thank you for contributing to CC Chain! Your efforts help build the future of decentralized systems. 🚀