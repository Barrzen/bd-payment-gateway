#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "[1/8] cargo fmt --check"
cargo fmt --all -- --check

echo "[2/8] cargo clippy (all features, deny warnings)"
cargo clippy --workspace --all-features --all-targets -- -D warnings

echo "[3/8] cargo check bindings with no default features and denied warnings"
RUSTFLAGS="-D warnings" cargo check -p bd-payment-gateway-js --no-default-features
RUSTFLAGS="-D warnings" cargo check -p bd-payment-gateway-py --no-default-features

echo "[4/8] cargo test workspace all features"
cargo test --workspace --all-features

echo "[5/8] cargo test providers individually"
cargo test -p bd-payment-gateway-shurjopay
cargo test -p bd-payment-gateway-portwallet
cargo test -p bd-payment-gateway-aamarpay
cargo test -p bd-payment-gateway-sslcommerz

echo "[6/8] cargo build JS/Python bindings"
cargo build -p bd-payment-gateway-js --all-features
cargo build -p bd-payment-gateway-py --all-features

echo "[7/8] uv+maturin Python wheel build (if uv available)"
if [[ -f "$HOME/.local/bin/env" ]]; then
  # shellcheck disable=SC1090
  source "$HOME/.local/bin/env"
fi

if command -v uv >/dev/null 2>&1; then
  (
    cd bd-payment-gateway-py
    uv sync --group dev
    uv run maturin build --release --features all-providers
  )
else
  echo "uv not found; skipping Python uv/maturin verification"
fi

echo "[8/8] done"
