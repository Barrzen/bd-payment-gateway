# Contributing

## Prerequisites

- Rust stable 1.93.0 (workspace baseline, edition 2024)
- `cargo fmt` + `cargo clippy`
- `just` for short local commands
- `uv` for local Python tooling
- Optional:
  - Node.js/Bun/Deno for N-API smoke tests
  - Python 3.14 for wheel builds

## Setup

```bash
just quality
```

or

```bash
cargo check --workspace --all-features
./scripts/quality-check.sh
```

## Standard Commands

- Use `just --list` to see all shortcuts.
- Recommended:
  - `just quality`
  - `just fmt`
  - `just lint`
  - `just test`
  - `just py-wheel all-providers`

Raw equivalents:

- Format: `cargo fmt --all`
- Lint: `cargo clippy --workspace --all-features --all-targets -- -D warnings`
- Tests: `cargo test --workspace --all-features`
- No-feature binding warning gate:
  - `RUSTFLAGS='-D warnings' cargo check -p bd-payment-gateway-js --no-default-features`
  - `RUSTFLAGS='-D warnings' cargo check -p bd-payment-gateway-py --no-default-features`
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
