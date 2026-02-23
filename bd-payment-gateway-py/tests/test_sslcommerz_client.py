from __future__ import annotations

from decimal import Decimal
from types import SimpleNamespace
from typing import Any

import pytest

from bd_payment_gateway.errors import PaymentGatewayError
from bd_payment_gateway.sslcommerz import (
    Customer,
    FinalStatus,
    InitiatePaymentRequest,
    Product,
    SslcommerzClient,
    Urls,
    VerifyPaymentRequest,
)


class FakeBackend:
    def __init__(self) -> None:
        self.initiate_calls: list[dict[str, Any]] = []
        self.verify_calls: list[dict[str, Any]] = []
        self.verify_responses: list[Any] = []

    def initiate_payment(self, payload: dict):
        self.initiate_calls.append(payload)
        return SimpleNamespace(
            redirect_url="https://sandbox.sslcommerz.com/gw/test",
            provider_reference="SESSION-123",
            request_id="REQ-1",
            raw='{"status":"SUCCESS"}',
        )

    def verify_payment(self, payload: dict):
        self.verify_calls.append(payload)
        if not self.verify_responses:
            raise AssertionError("No verify responses configured")
        return self.verify_responses.pop(0)


@pytest.fixture
def initiate_request() -> InitiatePaymentRequest:
    return InitiatePaymentRequest(
        total_amount=Decimal("150.00"),
        tran_id="TXN-150",
        urls=Urls.model_validate(
            {
                "success_url": "https://merchant.test/success",
                "fail_url": "https://merchant.test/fail",
                "cancel_url": "https://merchant.test/cancel",
            }
        ),
        customer=Customer(
            name="Demo",
            email="demo@example.com",
            address_line_1="Address",
            city="Dhaka",
            country="Bangladesh",
            phone="01700000000",
        ),
        product=Product(name="Course", category="Education", profile="general"),
    )


def test_initiate_payment_maps_payload(initiate_request: InitiatePaymentRequest) -> None:
    backend = FakeBackend()
    client = SslcommerzClient.from_backend(backend)

    response = client.initiate_payment(initiate_request)

    assert backend.initiate_calls
    mapped = backend.initiate_calls[0]
    assert mapped["total_amount"] == "150.00"
    assert mapped["tran_id"] == "TXN-150"
    assert response.provider_reference == "SESSION-123"


def test_verify_payment_maps_payload() -> None:
    backend = FakeBackend()
    backend.verify_responses = [
        SimpleNamespace(
            status="pending",
            provider_reference="SESSION-123",
            amount="150.00",
            currency="BDT",
            request_id="REQ-2",
            raw='{"status":"VALID"}',
        )
    ]
    client = SslcommerzClient.from_backend(backend)

    result = client.verify_payment(VerifyPaymentRequest(session_key="SESSION-123"))

    assert backend.verify_calls == [{"reference": {"SessionKey": "SESSION-123"}}]
    assert result.status == "pending"


def test_wait_for_final_status_stops_on_paid(monkeypatch: pytest.MonkeyPatch) -> None:
    backend = FakeBackend()
    backend.verify_responses = [
        SimpleNamespace(
            status="pending",
            provider_reference="SESSION-123",
            amount=None,
            currency=None,
            request_id=None,
            raw="{}",
        ),
        SimpleNamespace(
            status="paid",
            provider_reference="SESSION-123",
            amount="150.00",
            currency="BDT",
            request_id=None,
            raw="{}",
        ),
    ]

    monkeypatch.setattr("bd_payment_gateway.sslcommerz.client.time.sleep", lambda *_: None)

    client = SslcommerzClient.from_backend(backend)
    status = client.wait_for_final_status("SESSION-123", timeout_s=1, interval_s=1)

    assert status is FinalStatus.PAID


def test_initiate_requires_model_only() -> None:
    backend = FakeBackend()
    client = SslcommerzClient.from_backend(backend)

    with pytest.raises(TypeError):
        client.initiate_payment({"tran_id": "TXN-1"})  # type: ignore[arg-type]


def test_native_error_is_normalized(initiate_request: InitiatePaymentRequest) -> None:
    class NativeError(Exception):
        code = "PROVIDER_REJECTED"
        message = "Provider rejected request"
        hint = "Check merchant credentials"
        provider_payload = {"provider_code": "INVALID"}

    class FailingBackend(FakeBackend):
        def initiate_payment(self, payload: dict):  # noqa: ARG002
            raise NativeError("native")

    client = SslcommerzClient.from_backend(FailingBackend())

    with pytest.raises(PaymentGatewayError) as exc:
        client.initiate_payment(initiate_request)

    assert exc.value.code == "PROVIDER_REJECTED"
    assert exc.value.provider_payload == {"provider_code": "INVALID"}
