from typing import Any, Mapping, TypedDict
from typing_extensions import NotRequired, TypeAlias

JsonInput: TypeAlias = str | Mapping[str, Any]


class PaymentGatewayError(Exception): ...


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


class EnvironmentConfig(TypedDict):
    mode: str
    custom_base_url: NotRequired[str]


class HttpSettingsConfig(TypedDict):
    timeout_ms: NotRequired[int]
    max_retries: NotRequired[int]
    initial_backoff_ms: NotRequired[int]
    max_backoff_ms: NotRequired[int]
    user_agent: NotRequired[str]


class ShurjopayConfig(TypedDict):
    username: str
    password: str
    prefix: str
    environment: EnvironmentConfig
    http_settings: NotRequired[HttpSettingsConfig]


class ShurjopayInitiateRequest(TypedDict):
    amount: str
    order_id: str
    currency: str
    return_url: str
    cancel_url: str
    client_ip: str
    customer_name: str
    customer_phone: str
    customer_email: str
    customer_address: str
    customer_city: str
    customer_state: str
    customer_postcode: str
    customer_country: str
    value1: NotRequired[str]
    value2: NotRequired[str]
    value3: NotRequired[str]
    value4: NotRequired[str]
    discount_amount: NotRequired[str]
    discount_percent: NotRequired[str]
    correlation_id: NotRequired[str]


class ShurjopayVerifyRequest(TypedDict):
    sp_order_id: str
    correlation_id: NotRequired[str]


class PortwalletConfig(TypedDict):
    app_key: str
    app_secret: str
    environment: EnvironmentConfig
    http_settings: NotRequired[HttpSettingsConfig]


class PortwalletCustomer(TypedDict):
    name: str
    email: str
    phone: str
    address: NotRequired[str]
    city: NotRequired[str]
    zip_code: NotRequired[str]
    country: NotRequired[str]


class PortwalletInitiateRequest(TypedDict):
    order: str
    amount: str
    currency: str
    redirect_url: str
    ipn_url: str
    customer: PortwalletCustomer
    reference: NotRequired[str]
    correlation_id: NotRequired[str]


class PortwalletVerifyRequest(TypedDict):
    invoice_id: str
    correlation_id: NotRequired[str]


class PortwalletRefundRequest(TypedDict):
    invoice_id: str
    amount: str
    reason: NotRequired[str]
    correlation_id: NotRequired[str]


class AamarpayConfig(TypedDict):
    store_id: str
    signature_key: str
    environment: EnvironmentConfig
    http_settings: NotRequired[HttpSettingsConfig]


class AamarpayInitiateRequest(TypedDict):
    tran_id: str
    amount: str
    currency: str
    success_url: str
    fail_url: str
    cancel_url: str
    cus_name: str
    cus_email: str
    cus_add1: str
    cus_city: str
    cus_country: str
    cus_phone: str
    desc: NotRequired[str]
    cus_add2: NotRequired[str]
    cus_state: NotRequired[str]
    cus_postcode: NotRequired[str]
    opt_a: NotRequired[str]
    opt_b: NotRequired[str]
    opt_c: NotRequired[str]
    opt_d: NotRequired[str]
    signature_key: NotRequired[str]


class AamarpayVerifyRequest(TypedDict):
    request_id: str


class SslcommerzConfig(TypedDict):
    store_id: str
    store_passwd: str
    environment: EnvironmentConfig
    http_settings: NotRequired[HttpSettingsConfig]


class SslcommerzInitiateRequest(TypedDict):
    total_amount: str
    currency: str
    tran_id: str
    success_url: str
    fail_url: str
    cancel_url: str
    product_name: str
    product_category: str
    product_profile: str
    cus_name: str
    cus_email: str
    cus_add1: str
    cus_city: str
    cus_country: str
    cus_phone: str
    ipn_url: NotRequired[str]
    shipping_method: NotRequired[str]
    value_a: NotRequired[str]
    value_b: NotRequired[str]
    value_c: NotRequired[str]
    value_d: NotRequired[str]


class _SslcommerzValIdRef(TypedDict):
    ValId: str


class _SslcommerzSessionKeyRef(TypedDict):
    SessionKey: str


class _SslcommerzTranIdRef(TypedDict):
    TranId: str


class SslcommerzVerifyByValId(TypedDict):
    reference: _SslcommerzValIdRef


class SslcommerzVerifyBySessionKey(TypedDict):
    reference: _SslcommerzSessionKeyRef


class SslcommerzVerifyByTranId(TypedDict):
    reference: _SslcommerzTranIdRef


SslcommerzVerifyRequest: TypeAlias = (
    SslcommerzVerifyByValId | SslcommerzVerifyBySessionKey | SslcommerzVerifyByTranId
)


class _SslcommerzRefundInitiateBody(TypedDict):
    bank_tran_id: str
    refund_amount: str
    refund_remarks: str


class _SslcommerzRefundQueryBody(TypedDict):
    refund_ref_id: str


class SslcommerzRefundInitiate(TypedDict):
    Initiate: _SslcommerzRefundInitiateBody


class SslcommerzRefundQuery(TypedDict):
    Query: _SslcommerzRefundQueryBody


SslcommerzRefundRequest: TypeAlias = SslcommerzRefundInitiate | SslcommerzRefundQuery


class SslcommerzClient:
    def __init__(self, config: JsonInput | SslcommerzConfig) -> None: ...
    def initiate_payment(
        self, request: JsonInput | SslcommerzInitiateRequest
    ) -> InitiatePaymentResponse: ...
    def verify_payment(
        self, request: JsonInput | SslcommerzVerifyRequest
    ) -> VerifyPaymentResponse: ...
    def refund(self, request: JsonInput | SslcommerzRefundRequest) -> RefundResponse: ...


class ShurjopayClient:
    def __init__(self, config: JsonInput | ShurjopayConfig) -> None: ...
    def initiate_payment(
        self, request: JsonInput | ShurjopayInitiateRequest
    ) -> InitiatePaymentResponse: ...
    def verify_payment(
        self, request: JsonInput | ShurjopayVerifyRequest
    ) -> VerifyPaymentResponse: ...


class PortwalletClient:
    def __init__(self, config: JsonInput | PortwalletConfig) -> None: ...
    def initiate_payment(
        self, request: JsonInput | PortwalletInitiateRequest
    ) -> InitiatePaymentResponse: ...
    def verify_payment(
        self, request: JsonInput | PortwalletVerifyRequest
    ) -> VerifyPaymentResponse: ...
    def refund(self, request: JsonInput | PortwalletRefundRequest) -> RefundResponse: ...


class AamarpayClient:
    def __init__(self, config: JsonInput | AamarpayConfig) -> None: ...
    def initiate_payment(
        self, request: JsonInput | AamarpayInitiateRequest
    ) -> InitiatePaymentResponse: ...
    def verify_payment(
        self, request: JsonInput | AamarpayVerifyRequest
    ) -> VerifyPaymentResponse: ...
