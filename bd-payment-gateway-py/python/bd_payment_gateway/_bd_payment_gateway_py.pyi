from typing import Any, Mapping, TypeAlias

JsonInput: TypeAlias = str | Mapping[str, Any]

__version__: str


class PaymentGatewayError(Exception):
    code: str
    message: str
    hint: str
    provider_payload: dict[str, Any] | None


class InitiatePaymentResponse:
    redirect_url: str
    provider_reference: str
    raw: str
    request_id: str | None


class VerifyPaymentResponse:
    status: str
    provider_reference: str
    amount: str | None
    currency: str | None
    raw: str
    request_id: str | None


class RefundResponse:
    status: str
    provider_reference: str
    raw: str
    request_id: str | None


class SslcommerzClient:
    def __init__(self, config: JsonInput | Mapping[str, Any]) -> None: ...
    def initiate_payment(
        self,
        request: JsonInput | Mapping[str, Any],
    ) -> InitiatePaymentResponse: ...
    def verify_payment(
        self,
        request: JsonInput | Mapping[str, Any],
    ) -> VerifyPaymentResponse: ...
    def refund(self, request: JsonInput | Mapping[str, Any]) -> RefundResponse: ...


class ShurjopayClient:
    def __init__(self, config: JsonInput | Mapping[str, Any]) -> None: ...
    def initiate_payment(
        self,
        request: JsonInput | Mapping[str, Any],
    ) -> InitiatePaymentResponse: ...
    def verify_payment(
        self,
        request: JsonInput | Mapping[str, Any],
    ) -> VerifyPaymentResponse: ...


class PortwalletClient:
    def __init__(self, config: JsonInput | Mapping[str, Any]) -> None: ...
    def initiate_payment(
        self,
        request: JsonInput | Mapping[str, Any],
    ) -> InitiatePaymentResponse: ...
    def verify_payment(
        self,
        request: JsonInput | Mapping[str, Any],
    ) -> VerifyPaymentResponse: ...
    def refund(self, request: JsonInput | Mapping[str, Any]) -> RefundResponse: ...


class AamarpayClient:
    def __init__(self, config: JsonInput | Mapping[str, Any]) -> None: ...
    def initiate_payment(
        self,
        request: JsonInput | Mapping[str, Any],
    ) -> InitiatePaymentResponse: ...
    def verify_payment(
        self,
        request: JsonInput | Mapping[str, Any],
    ) -> VerifyPaymentResponse: ...
