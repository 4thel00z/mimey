from collections.abc import Callable

__all__ = [
    "clear_registrations",
    "detect_mime",
    "detect_type",
    "register",
    "registered",
]

def detect_mime(data: bytes) -> str:
    """Detects the mime type of a byte array."""

def detect_type(data: bytes) -> str:
    """Detects the extension of a byte array."""

def register(
    mime: str,
    extension: str,
    *,
    magic: bytes | None = ...,
    offset: int = ...,
    matcher: Callable[[bytes], bool] | None = ...,
) -> None:
    """Registers a custom type, matched by magic bytes or by a Python callable.

    Exactly one of magic or matcher must be given. Registered types are checked
    before the built-in table, in registration order.
    """

def registered() -> list[tuple[str, str]]:
    """Returns the registered (mime, extension) pairs in registration order."""

def clear_registrations() -> None:
    """Drops every registered type."""
