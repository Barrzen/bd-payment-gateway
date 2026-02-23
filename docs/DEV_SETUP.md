# Development Setup

This document defines the local setup flow for the Python binding crate (`bd-payment-gateway-py`) and typed facade package.

## Prerequisites

- Rust toolchain `1.93.0` (`rustup default 1.93.0`)
- Python `>=3.9` (tested with Python `3.14`)
- `uv`
- `maturin>=1.8,<2`

## Install Dev Dependencies

```bash
source "$HOME/.local/bin/env"
cd bd-payment-gateway-py
uv sync --group dev
```

## Build Wheel

All providers:

```bash
source "$HOME/.local/bin/env"
cd bd-payment-gateway-py
uv run maturin build --release --features all-providers
```

Single provider (SSLCOMMERZ only):

```bash
source "$HOME/.local/bin/env"
cd bd-payment-gateway-py
uv run maturin build --release --features sslcommerz
```

## Develop Install (editable for local iteration)

```bash
source "$HOME/.local/bin/env"
cd bd-payment-gateway-py
uv run maturin develop --features sslcommerz
```

## Quick Import Check

```bash
source "$HOME/.local/bin/env"
cd bd-payment-gateway-py
uv run python -c "import bd_payment_gateway; import bd_payment_gateway.sslcommerz; print('ok')"
```

## Run Tests

Python tests:

```bash
source "$HOME/.local/bin/env"
cd bd-payment-gateway-py
uv run pytest
```

Rust tests for binding crate:

```bash
cargo test -p bd-payment-gateway-py --all-features
```

## Linters and Type Checking

Rust lint:

```bash
cargo clippy -p bd-payment-gateway-py --all-features --all-targets -- -D warnings
```

Python lint:

```bash
source "$HOME/.local/bin/env"
cd bd-payment-gateway-py
uv run ruff check .
```

Python type check:

```bash
source "$HOME/.local/bin/env"
cd bd-payment-gateway-py
uv run mypy python tests
```
