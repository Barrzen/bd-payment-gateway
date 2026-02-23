from __future__ import annotations

import time
from typing import Any

from bd_payment_gateway.errors import PaymentGatewayError

from .models import (
    FinalStatus,
    InitiatePaymentRequest,
    InitiatePaymentResponse,
    Settings,
    VerifyPaymentRequest,
    VerifyPaymentResponse,
)

_native: Any
try:
    from bd_payment_gateway import _bd_payment_gateway_py as _native
except ModuleNotFoundError:  # pragma: no cover - exercised in packaging/import checks
    _native = None


class SslcommerzClient:
    """Typed-first SSLCOMMERZ client facade."""

    def __init__(self, backend: Any) -> None:
        self._backend = backend

    @classmethod
    def from_settings(cls, settings: Settings) -> "SslcommerzClient":
        native = _ensure_native_module()
        try:
            backend = native.SslcommerzClient(settings.to_provider_config())
        except Exception as exc:  # pragma: no cover - depends on native failure paths
            raise PaymentGatewayError.from_exception(exc) from exc
        return cls(backend=backend)

    @classmethod
    def from_backend(cls, backend: Any) -> "SslcommerzClient":
        """Testing hook for injecting a fake backend."""

        return cls(backend=backend)

    def initiate_payment(
        self, request: InitiatePaymentRequest
    ) -> InitiatePaymentResponse:
        if not isinstance(request, InitiatePaymentRequest):
            raise TypeError("request must be InitiatePaymentRequest")

        try:
            raw = self._backend.initiate_payment(request.to_provider_payload())
        except Exception as exc:
            raise PaymentGatewayError.from_exception(exc) from exc

        return InitiatePaymentResponse.from_native(raw)

    def verify_payment(self, request: VerifyPaymentRequest) -> VerifyPaymentResponse:
        if not isinstance(request, VerifyPaymentRequest):
            raise TypeError("request must be VerifyPaymentRequest")

        try:
            raw = self._backend.verify_payment(request.to_provider_payload())
        except Exception as exc:
            raise PaymentGatewayError.from_exception(exc) from exc

        return VerifyPaymentResponse.from_native(raw)

    def wait_for_final_status(
        self,
        session_key: str,
        timeout_s: int = 300,
        interval_s: int = 5,
    ) -> FinalStatus:
        if not session_key:
            raise ValueError("session_key is required")
        if timeout_s <= 0:
            raise ValueError("timeout_s must be > 0")
        if interval_s <= 0:
            raise ValueError("interval_s must be > 0")

        deadline = time.monotonic() + timeout_s

        while True:
            response = self.verify_payment(
                VerifyPaymentRequest(session_key=session_key)
            )
            final_status = response.final_status
            if final_status is not None:
                return final_status

            if time.monotonic() >= deadline:
                raise PaymentGatewayError.timeout(
                    message=(
                        f"Timed out waiting for final status for session_key '{session_key}'."
                    ),
                    hint="Increase timeout_s or inspect verify_payment status transitions.",
                )

            time.sleep(interval_s)


def _ensure_native_module() -> Any:
    if _native is None:
        raise RuntimeError(
            "bd_payment_gateway._bd_payment_gateway_py extension is not available. "
            "Install with `uv run maturin develop --features sslcommerz`."
        )
    return _native
