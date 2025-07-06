# justfile

# Check code formatting
fmt-check:
    cargo fmt -- --check

# Fix formatting
fmt-fix:
    cargo fmt

# Run clippy with denied warnings
clippy:
    cargo clippy -- -D warnings

# Build project with verbose output
build:
    cargo build --verbose

# Run tests with verbose output
test:
    cargo test --verbose

# CI pipeline: run all checks
ci:
    just fmt-check
    just clippy
    just build
    just test

# Fix format, check clippy, run tests
fix-check-test:
    just fmt-fix
    just clippy
    just test