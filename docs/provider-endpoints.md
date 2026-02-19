# Provider Endpoint Map

This file records the endpoint contracts implemented in this workspace.

## shurjoPay

- `POST /api/get_token`
- `POST /api/secret-pay`
- `POST /api/verification`

Base URLs:

- Sandbox: `https://sandbox.shurjopayment.com`
- Production: `https://engine.shurjopayment.com`

## PortWallet v2

- `POST /v2/invoice`
- `GET /v2/invoice/ipn/{invoice_id}`
- `POST /v2/invoice/refund`

Base URLs:

- Sandbox: `https://api-sandbox.portwallet.com`
- Production: `https://api.portwallet.com`

## aamarPay

- `POST /jsonpost.php`
- `GET /api/v1/trxcheck/request.php?request_id=...`

Base URLs:

- Sandbox: `https://sandbox.aamarpay.com`
- Production: `https://secure.aamarpay.com`

## SSLCOMMERZ

- `POST /gwprocess/v4/api.php`
- `GET /validator/api/validationserverAPI.php`
- `GET /validator/api/merchantTransIDvalidationAPI.php`

Base URLs:

- Sandbox: `https://sandbox.sslcommerz.com`
- Production: `https://securepay.sslcommerz.com`
