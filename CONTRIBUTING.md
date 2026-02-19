# Contributing

## Prerequisites

- Rust stable (>= workspace `rust-version`)
- `cargo fmt` + `cargo clippy`
- `uv` for local Python tooling
- Optional:
  - Node.js/Bun/Deno for N-API smoke tests
  - Python 3.14 for wheel builds

## Setup

```bash
cargo check --workspace --all-features
```

## Standard Commands

- Format: `cargo fmt --all`
- Lint: `cargo clippy --workspace --all-features --all-targets -- -D warnings`
- Tests: `cargo test --workspace --all-features`
- Examples: `cargo run -p bd-payment-gateway --example sslcommerz_initiate_verify --features sslcommerz`

## Binding Builds

- JS:
  - `cargo build -p bd-payment-gateway-js --all-features`
- Python:
  - `cargo build -p bd-payment-gateway-py --all-features`
  - `source $HOME/.local/bin/env`
  - `cd bd-payment-gateway-py && uv sync --group dev`
  - `cd bd-payment-gateway-py && uv run maturin build --release --features all-providers`

## PR Checklist

- [ ] Added/updated tests for validation + mapping logic
- [ ] Added/updated docs and examples
- [ ] No secrets logged, redaction preserved
- [ ] Error messages include actionable hint
- [ ] `cargo fmt`, `clippy`, `test` pass locally
- [ ] Feature-gating preserved (no implicit provider coupling)

## Commit & Version Notes

- Keep changes scoped by crate where possible
- Any public API change must be documented in PR summary
- Breaking changes require explicit semver note
