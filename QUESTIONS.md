# Questions and Safe Defaults

## Open Questions

1. Which SSLCOMMERZ verification statuses should be treated as terminal for `wait_for_final_status()`?
Safe default used: terminal set = `paid`, `failed`, `cancelled`, `refunded`; non-terminal values continue polling until timeout.

2. Should `currency` be strictly `BDT` or allow additional values?
Safe default used: typed API enforces `BDT` for SSLCOMMERZ request model for payment-critical safety.

3. What is the preferred long-term location for facade docs (root README vs `bd-payment-gateway-py/README.md`)?
Safe default used: update root `README.md` with Python typed usage and keep `bd-payment-gateway-py/README.md` focused on build/runtime notes.
