from __future__ import annotations

from decimal import Decimal

import pytest
from pydantic import ValidationError

from bd_payment_gateway.sslcommerz.models import (
    Customer,
    InitiatePaymentRequest,
    Product,
    Settings,
    Urls,
    VerifyPaymentRequest,
)


@pytest.fixture
def valid_initiate_kwargs() -> dict:
    return {
        "total_amount": Decimal("100.00"),
        "tran_id": "TXN-100",
        "urls": Urls.model_validate(
            {
                "success_url": "https://merchant.test/success",
                "fail_url": "https://merchant.test/fail",
                "cancel_url": "https://merchant.test/cancel",
                "ipn_url": "https://merchant.test/ipn",
            }
        ),
        "customer": Customer(
            name="Demo User",
            email="demo@example.com",
            address_line_1="Dhaka",
            city="Dhaka",
            country="Bangladesh",
            phone="01700000000",
        ),
        "product": Product(name="Course", category="Education", profile="general"),
    }


def test_initiate_amount_rejects_more_than_two_decimals(valid_initiate_kwargs: dict) -> None:
    payload = {**valid_initiate_kwargs, "total_amount": Decimal("100.001")}

    with pytest.raises(ValidationError):
        InitiatePaymentRequest(**payload)


def test_initiate_tran_id_max_length(valid_initiate_kwargs: dict) -> None:
    payload = {**valid_initiate_kwargs, "tran_id": "x" * 31}

    with pytest.raises(ValidationError):
        InitiatePaymentRequest(**payload)


def test_urls_reject_invalid_url() -> None:
    with pytest.raises(ValidationError):
        Urls.model_validate(
            {
                "success_url": "not-a-url",
                "fail_url": "https://merchant.test/f",
                "cancel_url": "https://merchant.test/c",
            }
        )


def test_initiate_payload_mapping_contains_required_keys(
    valid_initiate_kwargs: dict,
) -> None:
    request = InitiatePaymentRequest(**valid_initiate_kwargs)

    mapped = request.to_provider_payload()

    assert mapped["total_amount"] == "100.00"
    assert mapped["currency"] == "BDT"
    assert mapped["tran_id"] == "TXN-100"
    assert mapped["success_url"] == "https://merchant.test/success"
    assert mapped["cus_name"] == "Demo User"
    assert mapped["product_name"] == "Course"


def test_verify_request_requires_exactly_one_reference() -> None:
    with pytest.raises(ValidationError):
        VerifyPaymentRequest(session_key="s", tran_id="t")


def test_verify_request_mapping_for_session_key() -> None:
    request = VerifyPaymentRequest(session_key="SESSION-1")

    assert request.to_provider_payload() == {"reference": {"SessionKey": "SESSION-1"}}


def test_settings_load_from_env(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("BDPG_SSLCOMMERZ_STORE_ID", "store-env")
    monkeypatch.setenv("BDPG_SSLCOMMERZ_STORE_PASSWD", "secret-env")
    monkeypatch.setenv("BDPG_SSLCOMMERZ_ENVIRONMENT", "sandbox")

    settings = Settings()  # type: ignore[call-arg]

    assert settings.store_id == "store-env"
    assert settings.store_passwd.get_secret_value() == "secret-env"
    assert settings.environment == "sandbox"


def test_settings_load_from_dotenv(tmp_path, monkeypatch: pytest.MonkeyPatch) -> None:
    env_file = tmp_path / ".env"
    env_file.write_text(
        "\n".join(
            [
                "BDPG_SSLCOMMERZ_STORE_ID=store-dotenv",
                "BDPG_SSLCOMMERZ_STORE_PASSWD=secret-dotenv",
                "BDPG_SSLCOMMERZ_ENVIRONMENT=sandbox",
            ]
        ),
        encoding="utf-8",
    )

    monkeypatch.delenv("BDPG_SSLCOMMERZ_STORE_ID", raising=False)
    monkeypatch.delenv("BDPG_SSLCOMMERZ_STORE_PASSWD", raising=False)
    monkeypatch.delenv("BDPG_SSLCOMMERZ_ENVIRONMENT", raising=False)
    monkeypatch.chdir(tmp_path)

    settings = Settings()  # type: ignore[call-arg]

    assert settings.store_id == "store-dotenv"
    assert settings.store_passwd.get_secret_value() == "secret-dotenv"
    assert settings.environment == "sandbox"
