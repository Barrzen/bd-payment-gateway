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

class ShurjopayConfig(TypedDict):
    username: str
    password: str
    prefix: str
    environment: EnvironmentConfig

class ShurjopayInitiateRequest(TypedDict, total=False):
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
    value1: str
    value2: str
    value3: str
    value4: str
    discount_amount: str
    discount_percent: str
    correlation_id: str

class ShurjopayVerifyRequest(TypedDict, total=False):
    sp_order_id: str
    correlation_id: str

class PortwalletConfig(TypedDict):
    app_key: str
    app_secret: str
    environment: EnvironmentConfig

class PortwalletCustomer(TypedDict, total=False):
    name: str
    email: str
    phone: str
    address: str
    city: str
    zip_code: str
    country: str

class PortwalletInitiateRequest(TypedDict, total=False):
    order: str
    amount: str
    currency: str
    redirect_url: str
    ipn_url: str
    reference: str
    customer: PortwalletCustomer
    correlation_id: str

class PortwalletVerifyRequest(TypedDict, total=False):
    invoice_id: str
    correlation_id: str

class PortwalletRefundRequest(TypedDict, total=False):
    invoice_id: str
    amount: str
    reason: str
    correlation_id: str

class AamarpayConfig(TypedDict):
    store_id: str
    signature_key: str
    environment: EnvironmentConfig

class AamarpayInitiateRequest(TypedDict, total=False):
    tran_id: str
    amount: str
    currency: str
    success_url: str
    fail_url: str
    cancel_url: str
    desc: str
    cus_name: str
    cus_email: str
    cus_add1: str
    cus_add2: str
    cus_city: str
    cus_state: str
    cus_postcode: str
    cus_country: str
    cus_phone: str
    opt_a: str
    opt_b: str
    opt_c: str
    opt_d: str
    signature_key: str

class AamarpayVerifyRequest(TypedDict):
    request_id: str

class SslcommerzConfig(TypedDict):
    store_id: str
    store_passwd: str
    environment: EnvironmentConfig

class SslcommerzInitiateRequest(TypedDict, total=False):
    total_amount: str
    currency: str
    tran_id: str
    success_url: str
    fail_url: str
    cancel_url: str
    ipn_url: str
    shipping_method: str
    product_name: str
    product_category: str
    product_profile: str
    cus_name: str
    cus_email: str
    cus_add1: str
    cus_city: str
    cus_country: str
    cus_phone: str
    value_a: str
    value_b: str
    value_c: str
    value_d: str

class SslcommerzClient:
    def __init__(self, config: JsonInput | SslcommerzConfig) -> None: ...
    def initiate_payment(
        self, request: JsonInput | SslcommerzInitiateRequest
    ) -> InitiatePaymentResponse: ...
    def verify_payment(self, request: JsonInput | Mapping[str, Any]) -> VerifyPaymentResponse: ...
    def refund(self, request: JsonInput | Mapping[str, Any]) -> RefundResponse: ...

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
