#!/usr/bin/env python3
"""End-to-end SSLCOMMERZ sandbox flow test using bd_payment_gateway_py.

Flow aligned with SSLCOMMERZ v4 backend process:
1) Initiate transaction (Create Session)
2) Redirect to hosted gateway
3) Complete payment in sandbox gateway
4) Poll transaction query API by SessionKey until final status

Credential loading priority:
- SSLCOMMERZ_STORE_ID / SSLCOMMERZ_STORE_PASSWD env vars
- local .env file next to this script (gitignored)
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import time
import uuid
import webbrowser
from pathlib import Path
from typing import Any

import bd_payment_gateway_py as sdk

DEFAULT_RETURN_BASE_URL = "https://www.alibto.com"
DEFAULT_ENV_FILE = Path(__file__).resolve().parent / ".env"

HTTP_SETTINGS = {
    "timeout_ms": 30_000,
    "max_retries": 2,
    "initial_backoff_ms": 200,
    "max_backoff_ms": 2_000,
    "user_agent": "bd-payment-gateway-py-sslcommerz-fullflow-test",
}


class FlowError(RuntimeError):
    """Raised when the E2E flow cannot continue."""



def parse_error_payload(exc: Exception) -> dict[str, Any]:
    try:
        return json.loads(str(exc))
    except json.JSONDecodeError:
        return {"message": str(exc), "code": "UNKNOWN", "hint": ""}


def load_env_file(path: Path) -> dict[str, str]:
    if not path.exists():
        return {}

    parsed: dict[str, str] = {}
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        parsed[key.strip()] = value.strip().strip("'\"")
    return parsed



def load_credentials(env_file: Path) -> tuple[str, str, str]:
    env_store_id = os.getenv("SSLCOMMERZ_STORE_ID", "").strip()
    env_store_passwd = os.getenv("SSLCOMMERZ_STORE_PASSWD", "").strip()

    if env_store_id and env_store_passwd:
        return env_store_id, env_store_passwd, "env"

    local_env = load_env_file(env_file)
    file_store_id = local_env.get("SSLCOMMERZ_STORE_ID", "").strip()
    file_store_passwd = local_env.get("SSLCOMMERZ_STORE_PASSWD", "").strip()
    if file_store_id and file_store_passwd:
        return file_store_id, file_store_passwd, f"env_file:{env_file}"

    raise FlowError(
        "Missing SSLCOMMERZ credentials. "
        "Set SSLCOMMERZ_STORE_ID and SSLCOMMERZ_STORE_PASSWD in environment "
        f"or add them to {env_file}."
    )



def build_client(store_id: str, store_passwd: str) -> sdk.SslcommerzClient:
    return sdk.SslcommerzClient(
        {
            "store_id": store_id,
            "store_passwd": store_passwd,
            "environment": {"mode": "sandbox"},
            "http_settings": HTTP_SETTINGS,
        }
    )



def run_full_flow(
    client: sdk.SslcommerzClient,
    *,
    amount: str,
    return_base_url: str,
    open_browser: bool,
    poll_interval: int,
    timeout_seconds: int,
) -> int:
    tran_id = f"SMOKE{int(time.time())}{uuid.uuid4().hex[:6]}"[:30]
    return_base_url = return_base_url.rstrip("/")

    print("Initiating SSLCOMMERZ sandbox transaction...")
    try:
        initiated = client.initiate_payment(
            {
                "total_amount": amount,
                "currency": "BDT",
                "tran_id": tran_id,
                "success_url": f"{return_base_url}/success",
                "fail_url": f"{return_base_url}/fail",
                "cancel_url": f"{return_base_url}/cancel",
                "ipn_url": f"{return_base_url}/ipn",
                "shipping_method": "NO",
                "product_name": "SDK E2E Test",
                "product_category": "Testing",
                "product_profile": "general",
                "cus_name": "Sandbox Tester",
                "cus_email": "tester@example.com",
                "cus_add1": "Dhaka",
                "cus_city": "Dhaka",
                "cus_country": "Bangladesh",
                "cus_phone": "01700000000",
                "value_a": "sdk-e2e",
            }
        )
    except sdk.PaymentGatewayError as exc:
        payload = parse_error_payload(exc)
        raise FlowError(
            f"initiate_payment failed: code={payload.get('code')} message={payload.get('message')} hint={payload.get('hint')}"
        ) from exc

    session_key = initiated.provider_reference
    redirect_url = initiated.redirect_url

    print("\nSession created successfully")
    print(f"- tran_id: {tran_id}")
    print(f"- session_key: {session_key}")
    print(f"- redirect_url: {redirect_url}")

    print("\nSandbox card details (from SSLCOMMERZ docs):")
    print("- VISA: 4111111111111111, Exp: 12/26, CVV: 111")
    print("- MasterCard: 5111111111111111, Exp: 12/26, CVV: 111")
    print("- AMEX: 371111111111111, Exp: 12/26, CVV: 111")
    print("- OTP: 111111 or 123456")

    if open_browser:
        opened = webbrowser.open(redirect_url)
        if opened:
            print("\nOpened payment page in your default browser.")
        else:
            print("\nCould not auto-open browser. Open redirect_url manually.")

    print("\nComplete the payment now. Polling transaction status by SessionKey...")

    deadline = time.time() + timeout_seconds
    last_status = "unknown"
    while time.time() < deadline:
        try:
            verified = client.verify_payment({"reference": {"SessionKey": session_key}})
        except sdk.PaymentGatewayError as exc:
            payload = parse_error_payload(exc)
            print(
                "verify_payment error: "
                f"code={payload.get('code')} message={payload.get('message')}"
            )
            time.sleep(poll_interval)
            continue

        status = (verified.status or "unknown").strip().lower()
        last_status = status
        print(f"- status={status}")

        if status in {"paid", "failed", "cancelled"}:
            if status == "paid":
                print("\nPayment completed successfully.")
                return 0
            print(f"\nPayment finished with non-success status: {status}")
            return 1

        time.sleep(poll_interval)

    print(f"\nTimed out waiting for final transaction status. Last status={last_status}")
    return 1



def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="SSLCOMMERZ sandbox full-flow SDK test")
    parser.add_argument(
        "--amount",
        default="10.00",
        help="Transaction amount in BDT (default: 10.00)",
    )
    parser.add_argument(
        "--return-base-url",
        default=os.getenv("SSLCOMMERZ_RETURN_BASE_URL", DEFAULT_RETURN_BASE_URL),
        help=(
            "Base URL used to build success/fail/cancel/ipn URLs "
            f"(default: {DEFAULT_RETURN_BASE_URL})"
        ),
    )
    parser.add_argument(
        "--env-file",
        default=str(DEFAULT_ENV_FILE),
        help=f"Path to local .env file (default: {DEFAULT_ENV_FILE})",
    )
    parser.add_argument(
        "--no-browser",
        action="store_true",
        help="Do not auto-open browser; print URL only",
    )
    parser.add_argument(
        "--poll-interval",
        type=int,
        default=5,
        help="Seconds between verify polls (default: 5)",
    )
    parser.add_argument(
        "--timeout",
        type=int,
        default=300,
        help="Timeout in seconds waiting for final status (default: 300)",
    )
    return parser.parse_args()



def main() -> int:
    required = ["PaymentGatewayError", "SslcommerzClient"]
    missing = [name for name in required if not hasattr(sdk, name)]
    if missing:
        print(
            "Missing SSLCommerz binding in this build: "
            + ", ".join(missing)
            + ". Rebuild Rust extension with '--features sslcommerz' or '--features all-providers'."
        )
        return 1

    args = parse_args()
    env_file_path = Path(args.env_file)
    env_file_values = load_env_file(env_file_path)

    # Use SSLCOMMERZ_RETURN_BASE_URL from .env when CLI/env did not override.
    if (
        not os.getenv("SSLCOMMERZ_RETURN_BASE_URL")
        and args.return_base_url == DEFAULT_RETURN_BASE_URL
        and env_file_values.get("SSLCOMMERZ_RETURN_BASE_URL")
    ):
        args.return_base_url = env_file_values["SSLCOMMERZ_RETURN_BASE_URL"].strip()

    try:
        store_id, store_passwd, source = load_credentials(env_file_path)
    except FlowError as exc:
        print(f"Credential load error: {exc}")
        return 1

    print(f"Loaded credentials from: {source}")
    print(f"Using store_id: {store_id}")

    try:
        client = build_client(store_id, store_passwd)
        return run_full_flow(
            client,
            amount=args.amount,
            return_base_url=args.return_base_url,
            open_browser=not args.no_browser,
            poll_interval=args.poll_interval,
            timeout_seconds=args.timeout,
        )
    except FlowError as exc:
        print(f"Flow error: {exc}")
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
