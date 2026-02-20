set shell := ["bash", "-cu"]

# Show available recipes
default:
    @just --list

# Format all workspace crates
fmt:
    cargo fmt --all

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Lint all targets/features and deny warnings
lint:
    cargo clippy --workspace --all-features --all-targets -- -D warnings

# Verify JS/Python binding crates compile cleanly with no provider features
check-bindings-no-features:
    RUSTFLAGS="-D warnings" cargo check -p bd-payment-gateway-js --no-default-features
    RUSTFLAGS="-D warnings" cargo check -p bd-payment-gateway-py --no-default-features

# Run workspace tests
test:
    cargo test --workspace --all-features

# Run provider crate tests directly
test-providers:
    cargo test -p bd-payment-gateway-shurjopay
    cargo test -p bd-payment-gateway-portwallet
    cargo test -p bd-payment-gateway-aamarpay
    cargo test -p bd-payment-gateway-sslcommerz

# Build Rust JS/Python binding crates
build-bindings:
    cargo build -p bd-payment-gateway-js --all-features
    cargo build -p bd-payment-gateway-py --all-features

# Build JS binding with selected provider features
js-build feature="all-providers":
    cargo build -p bd-payment-gateway-js --features {{feature}}

# Build Python wheel via uv+maturin with selected provider features
py-wheel feature="all-providers":
    source "$HOME/.local/bin/env" && cd bd-payment-gateway-py && uv sync --group dev && uv run maturin build --release --features {{feature}}

# Run one provider example from facade crate
example provider="portwallet":
    cargo run -p bd-payment-gateway --example {{provider}}_initiate_verify --features {{provider}}

# Full root quality suite
quality:
    ./scripts/quality-check.sh
