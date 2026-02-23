from __future__ import annotations

import json
from decimal import Decimal, ROUND_HALF_UP
from enum import Enum
from typing import Any, Annotated, Literal

from pydantic import (
    AnyHttpUrl,
    BaseModel,
    ConfigDict,
    Field,
    SecretStr,
    model_validator,
)
from pydantic_settings import BaseSettings, SettingsConfigDict

AmountDecimal = Annotated[
    Decimal,
    Field(gt=Decimal("0"), max_digits=12, decimal_places=2),
]


class ShippingMethod(str, Enum):
    NO = "NO"
    YES = "YES"
    COURIER = "Courier"


class FinalStatus(str, Enum):
    PAID = "paid"
    FAILED = "failed"
    CANCELLED = "cancelled"
    REFUNDED = "refunded"


class Settings(BaseSettings):
    """SSLCOMMERZ settings loaded from env/.env."""

    model_config = SettingsConfigDict(
        env_prefix="BDPG_SSLCOMMERZ_",
        env_file=".env",
        env_file_encoding="utf-8",
        extra="ignore",
        str_strip_whitespace=True,
        case_sensitive=False,
    )

    store_id: str = Field(min_length=1)
    store_passwd: SecretStr
    environment: Literal["sandbox", "production", "custom"] = "sandbox"
    custom_base_url: AnyHttpUrl | None = None

    @model_validator(mode="after")
    def validate_custom_environment(self) -> "Settings":
        if self.environment == "custom" and self.custom_base_url is None:
            raise ValueError("custom_base_url is required when environment is 'custom'")
        return self

    def to_provider_config(self) -> dict[str, Any]:
        environment: dict[str, Any] = {"mode": self.environment}
        if self.custom_base_url is not None:
            environment["custom_base_url"] = str(self.custom_base_url)

        return {
            "store_id": self.store_id,
            "store_passwd": self.store_passwd.get_secret_value(),
            "environment": environment,
        }


class Urls(BaseModel):
    model_config = ConfigDict(extra="forbid", str_strip_whitespace=True)

    success_url: AnyHttpUrl
    fail_url: AnyHttpUrl
    cancel_url: AnyHttpUrl
    ipn_url: AnyHttpUrl | None = None


class Customer(BaseModel):
    model_config = ConfigDict(extra="forbid", str_strip_whitespace=True)

    name: str = Field(min_length=1, max_length=100)
    email: str = Field(min_length=3, max_length=100)
    address_line_1: str = Field(min_length=1, max_length=255)
    city: str = Field(min_length=1, max_length=100)
    country: str = Field(min_length=1, max_length=100)
    phone: str = Field(min_length=3, max_length=32)


class Product(BaseModel):
    model_config = ConfigDict(extra="forbid", str_strip_whitespace=True)

    name: str = Field(min_length=1, max_length=255)
    category: str = Field(min_length=1, max_length=255)
    profile: str = Field(default="general", min_length=1, max_length=64)


class InitiatePaymentRequest(BaseModel):
    model_config = ConfigDict(extra="forbid", str_strip_whitespace=True)

    total_amount: AmountDecimal
    currency: Literal["BDT"] = "BDT"
    tran_id: str = Field(min_length=1, max_length=30)
    urls: Urls
    customer: Customer
    product: Product
    shipping_method: ShippingMethod = ShippingMethod.NO
    value_a: str | None = Field(default=None, max_length=255)
    value_b: str | None = Field(default=None, max_length=255)
    value_c: str | None = Field(default=None, max_length=255)
    value_d: str | None = Field(default=None, max_length=255)

    def to_provider_payload(self) -> dict[str, Any]:
        payload: dict[str, Any] = {
            "total_amount": _format_amount(self.total_amount),
            "currency": self.currency,
            "tran_id": self.tran_id,
            "success_url": str(self.urls.success_url),
            "fail_url": str(self.urls.fail_url),
            "cancel_url": str(self.urls.cancel_url),
            "shipping_method": self.shipping_method.value,
            "product_name": self.product.name,
            "product_category": self.product.category,
            "product_profile": self.product.profile,
            "cus_name": self.customer.name,
            "cus_email": self.customer.email,
            "cus_add1": self.customer.address_line_1,
            "cus_city": self.customer.city,
            "cus_country": self.customer.country,
            "cus_phone": self.customer.phone,
            "value_a": self.value_a,
            "value_b": self.value_b,
            "value_c": self.value_c,
            "value_d": self.value_d,
        }

        if self.urls.ipn_url is not None:
            payload["ipn_url"] = str(self.urls.ipn_url)

        return payload


class VerifyPaymentRequest(BaseModel):
    model_config = ConfigDict(extra="forbid", str_strip_whitespace=True)

    session_key: str | None = Field(default=None, min_length=1, max_length=255)
    val_id: str | None = Field(default=None, min_length=1, max_length=255)
    tran_id: str | None = Field(default=None, min_length=1, max_length=30)

    @model_validator(mode="after")
    def exactly_one_reference(self) -> "VerifyPaymentRequest":
        values = [self.session_key, self.val_id, self.tran_id]
        present = [value for value in values if value is not None]
        if len(present) != 1:
            raise ValueError(
                "Provide exactly one reference field: session_key, val_id, or tran_id"
            )
        return self

    def to_provider_payload(self) -> dict[str, Any]:
        if self.session_key is not None:
            return {"reference": {"SessionKey": self.session_key}}
        if self.val_id is not None:
            return {"reference": {"ValId": self.val_id}}
        return {"reference": {"TranId": self.tran_id}}


class InitiatePaymentResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")

    redirect_url: AnyHttpUrl
    provider_reference: str
    request_id: str | None = None
    raw: dict[str, Any] = Field(default_factory=dict)

    @classmethod
    def from_native(cls, native_response: Any) -> "InitiatePaymentResponse":
        return cls(
            redirect_url=getattr(native_response, "redirect_url"),
            provider_reference=getattr(native_response, "provider_reference"),
            request_id=getattr(native_response, "request_id", None),
            raw=_decode_raw_payload(getattr(native_response, "raw", None)),
        )


class VerifyPaymentResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")

    status: str
    provider_reference: str
    amount: Decimal | None = None
    currency: str | None = None
    request_id: str | None = None
    raw: dict[str, Any] = Field(default_factory=dict)

    @classmethod
    def from_native(cls, native_response: Any) -> "VerifyPaymentResponse":
        amount_value = _to_decimal_or_none(getattr(native_response, "amount", None))
        status = str(getattr(native_response, "status", "unknown")).lower()

        return cls(
            status=status,
            provider_reference=getattr(native_response, "provider_reference"),
            amount=amount_value,
            currency=getattr(native_response, "currency", None),
            request_id=getattr(native_response, "request_id", None),
            raw=_decode_raw_payload(getattr(native_response, "raw", None)),
        )

    @property
    def final_status(self) -> FinalStatus | None:
        if self.status == FinalStatus.PAID.value:
            return FinalStatus.PAID
        if self.status == FinalStatus.FAILED.value:
            return FinalStatus.FAILED
        if self.status == FinalStatus.CANCELLED.value:
            return FinalStatus.CANCELLED
        if self.status == FinalStatus.REFUNDED.value:
            return FinalStatus.REFUNDED
        return None


def _format_amount(amount: Decimal) -> str:
    quantized = amount.quantize(Decimal("0.01"), rounding=ROUND_HALF_UP)
    return f"{quantized:.2f}"


def _to_decimal_or_none(value: Any) -> Decimal | None:
    if value is None:
        return None
    if isinstance(value, Decimal):
        return value
    try:
        return Decimal(str(value))
    except Exception:
        return None


def _decode_raw_payload(value: Any) -> dict[str, Any]:
    if value is None:
        return {}
    if isinstance(value, dict):
        return value
    if isinstance(value, str):
        try:
            parsed = json.loads(value)
        except json.JSONDecodeError:
            return {"raw": value}
        return parsed if isinstance(parsed, dict) else {"raw": parsed}
    return {"raw": value}
