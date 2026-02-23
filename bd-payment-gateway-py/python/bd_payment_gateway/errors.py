from __future__ import annotations

import json
from dataclasses import dataclass
from typing import Any


@dataclass(eq=False)
class PaymentGatewayError(Exception):
    """Structured payment gateway error for Python users."""

    code: str
    message: str
    hint: str
    provider_payload: dict[str, Any] | None = None

    def __str__(self) -> str:
        return f"{self.code}: {self.message}. Hint: {self.hint}"

    @classmethod
    def timeout(cls, message: str, hint: str) -> "PaymentGatewayError":
        return cls(code="TIMEOUT", message=message, hint=hint, provider_payload=None)

    @classmethod
    def from_exception(cls, exc: BaseException) -> "PaymentGatewayError":
        if isinstance(exc, cls):
            return exc

        code = _string_or_none(getattr(exc, "code", None))
        message = _string_or_none(getattr(exc, "message", None))
        hint = _string_or_none(getattr(exc, "hint", None))

        provider_payload_raw = getattr(exc, "provider_payload", None)
        provider_payload = _payload_or_none(provider_payload_raw)

        if code and message and hint:
            return cls(
                code=code,
                message=message,
                hint=hint,
                provider_payload=provider_payload,
            )

        parsed = _parse_error_message(str(exc))
        if parsed is not None:
            return cls(
                code=parsed.get("code", "UNKNOWN_ERROR"),
                message=parsed.get("message", str(exc)),
                hint=parsed.get("hint", "Inspect provider payload and request fields."),
                provider_payload=_payload_or_none(parsed.get("provider_payload")),
            )

        return cls(
            code=code or "UNKNOWN_ERROR",
            message=message or str(exc),
            hint=hint or "Inspect stack trace and provider request/response payloads.",
            provider_payload=provider_payload,
        )


def _payload_or_none(payload: Any) -> dict[str, Any] | None:
    if payload is None:
        return None
    if isinstance(payload, dict):
        return payload
    if isinstance(payload, str):
        try:
            parsed = json.loads(payload)
        except json.JSONDecodeError:
            return {"raw": payload}
        return parsed if isinstance(parsed, dict) else {"raw": parsed}
    return {"raw": payload}


def _string_or_none(value: Any) -> str | None:
    if value is None:
        return None
    if isinstance(value, str):
        return value
    return str(value)


def _parse_error_message(message: str) -> dict[str, Any] | None:
    try:
        parsed = json.loads(message)
    except json.JSONDecodeError:
        return None
    return parsed if isinstance(parsed, dict) else None
