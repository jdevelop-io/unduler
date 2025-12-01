# Default command
default: check

# Run all checks (format, lint, test)
check: fmt-check lint test

# Format code
fmt:
    cargo fmt --all

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Run clippy
lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run tests
test:
    cargo test --workspace --all-features

# Run tests with coverage (requires cargo-llvm-cov)
coverage:
    cargo llvm-cov --workspace --all-features --html
    @echo "Coverage report: target/llvm-cov/html/index.html"

# Build in release mode
build:
    cargo build --release

# Build all targets
build-all:
    cargo build --workspace --all-targets

# Run dependency audit (requires cargo-deny)
audit:
    cargo deny check

# Update dependencies
update:
    cargo update

# Clean build artifacts
clean:
    cargo clean

# Run benchmarks (if any)
bench:
    cargo bench --workspace

# Generate documentation
doc:
    cargo doc --workspace --no-deps --open

# Check for outdated dependencies (requires cargo-outdated)
outdated:
    cargo outdated --workspace

# Run all CI checks locally
ci: fmt-check lint test audit

# Install development dependencies
setup:
    rustup component add llvm-tools-preview
    cargo install cargo-deny cargo-llvm-cov cargo-outdated

# Watch and run tests on changes (requires cargo-watch)
watch:
    cargo watch -x "test --workspace"

# Run a specific test
test-one name:
    cargo test --workspace {{name}}

# Create a new release (dry-run)
release-dry:
    cargo run -- release --dry-run

# Show dependency tree
deps:
    cargo tree --workspace

# Check minimum supported Rust version (requires cargo-msrv)
msrv:
    cargo msrv verify

# Run security audit (requires cargo-audit)
security:
    cargo audit
