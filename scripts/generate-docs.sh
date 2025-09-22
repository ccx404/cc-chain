#!/bin/bash
# Generate and verify all documentation for CC Chain

set -e

echo "🔧 CC Chain Documentation Generator"
echo "=================================="

# Check prerequisites
echo "📋 Checking prerequisites..."
command -v cargo >/dev/null 2>&1 || { echo "❌ Rust/Cargo is required but not installed. Install from https://rustup.rs/"; exit 1; }
echo "✅ Rust/Cargo found"

# Build the project
echo ""
echo "🏗️  Building the project..."
cargo build --all --exclude contracts

# Run tests
echo ""
echo "🧪 Running tests..."
cargo test --all --exclude contracts

# Generate Rust documentation
echo ""
echo "📚 Generating Rust documentation..."
cargo doc --all --exclude contracts --no-deps --open

# Check documentation examples
echo ""
echo "🔍 Testing documentation examples..."
cargo test --doc --all --exclude contracts

# Run example binaries
echo ""
echo "🚀 Testing examples..."
cargo run --package cc-chain-examples --bin basic_transaction
echo ""
cargo run --package cc-chain-examples --bin key_generation

# Check formatting
echo ""
echo "🎨 Checking code formatting..."
cargo fmt --check || {
    echo "❌ Code formatting issues found. Run 'cargo fmt' to fix."
    exit 1
}

# Run linter
echo ""
echo "🔍 Running linter..."
cargo clippy --all --exclude contracts -- -D warnings || {
    echo "❌ Linting issues found. Fix clippy warnings."
    exit 1
}

# Verify markdown files exist
echo ""
echo "📄 Verifying documentation files..."
docs=(
    "README.md"
    "CONTRIBUTING.md"
    "docs/architecture.md"
    "docs/consensus.md"
    "docs/api.md"
    "docs/development.md"
    "examples/README.md"
)

for doc in "${docs[@]}"; do
    if [[ -f "$doc" ]]; then
        echo "✅ $doc"
    else
        echo "❌ Missing: $doc"
        exit 1
    fi
done

# Check documentation size and completeness
echo ""
echo "📊 Documentation statistics:"
total_lines=0
for doc in "${docs[@]}"; do
    lines=$(wc -l < "$doc")
    echo "   $doc: $lines lines"
    total_lines=$((total_lines + lines))
done
echo "   Total documentation: $total_lines lines"

# Verify examples compile
echo ""
echo "🔧 Verifying examples compile..."
cargo check --package cc-chain-examples

echo ""
echo "✅ All documentation generated and verified successfully!"
echo ""
echo "📖 Documentation available at:"
echo "   - README.md (Project overview)"
echo "   - docs/architecture.md (System design)"
echo "   - docs/consensus.md (ccBFT specification)"
echo "   - docs/api.md (API reference)"
echo "   - docs/development.md (Developer guide)"
echo "   - CONTRIBUTING.md (Contribution guidelines)"
echo "   - examples/ (Working code examples)"
echo "   - target/doc/ (Generated Rust docs)"
echo ""
echo "🎉 Ready for development and production!"