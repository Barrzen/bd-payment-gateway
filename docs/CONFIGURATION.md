# Configuration (Python)

This page is the source of truth for Python environment variables.

## Provider support reminder

Only SSLCOMMERZ is supported right now in Python.

## Environment variables

| Env var | Required | Meaning | Example |
| --- | --- | --- | --- |
| `BDPG_SSLCOMMERZ_STORE_ID` | Yes | SSLCOMMERZ store ID from merchant panel | `testbox` |
| `BDPG_SSLCOMMERZ_STORE_PASSWD` | Yes | SSLCOMMERZ store password | `qwerty` |
| `BDPG_SSLCOMMERZ_ENVIRONMENT` | Yes | `sandbox`, `production`, or `custom` | `sandbox` |
| `BDPG_SSLCOMMERZ_CUSTOM_BASE_URL` | Only for `custom` | Custom API base URL | `https://sandbox.sslcommerz.com` |

## `.env` example

```dotenv
BDPG_SSLCOMMERZ_STORE_ID=your_store_id
BDPG_SSLCOMMERZ_STORE_PASSWD=your_store_password
BDPG_SSLCOMMERZ_ENVIRONMENT=sandbox
# Only if environment=custom:
# BDPG_SSLCOMMERZ_CUSTOM_BASE_URL=https://sandbox.sslcommerz.com
```

## Sandbox vs production

- `sandbox`:
  - Use for local development and test payments.
  - Do not treat transactions as real settlements.
- `production`:
  - Use only after SSLCOMMERZ merchant onboarding is complete.
  - Verify callback URLs are public and HTTPS.
- `custom`:
  - Advanced mode for controlled environments.
  - Must set `BDPG_SSLCOMMERZ_CUSTOM_BASE_URL`.

## Loading config in code

```python
from bd_payment_gateway.sslcommerz.models import Settings

settings = Settings()  # Reads env and optional .env
print(settings.environment)
```

## Important notes

- Use only the `BDPG_SSLCOMMERZ_*` names in application code and deployment config.
- Do not commit real credentials to git.
- Keep credentials in secrets manager or CI/CD secret store for production.
