from typing import Any

import pytest

import mimey

PNG = b"\x89PNG\r\n\x1a\n"
GARBAGE = b"\x00\x01\x02\x03\x04\x05\x06\x07"


@pytest.fixture(autouse=True)
def clean_registry() -> Any:
    mimey.clear_registrations()
    yield
    mimey.clear_registrations()


def test_detects_builtin_mime() -> None:
    assert mimey.detect_mime(PNG) == "image/png"


def test_detects_builtin_extension() -> None:
    assert mimey.detect_type(PNG) == ".png"


def test_unknown_bytes_fall_back_to_octet_stream() -> None:
    assert mimey.detect_mime(GARBAGE) == "application/octet-stream"


def test_plain_ascii_is_detected_as_text() -> None:
    assert mimey.detect_mime(b"hello world") == "text/plain; charset=utf-8"


def test_empty_input_does_not_raise() -> None:
    assert isinstance(mimey.detect_mime(b""), str)
    assert isinstance(mimey.detect_type(b""), str)


def test_register_magic_adds_a_new_type() -> None:
    mimey.register("application/x-nes-rom", ".nes", magic=b"NES\x1a")
    payload = b"NES\x1a" + b"\x00" * 32
    assert mimey.detect_mime(payload) == "application/x-nes-rom"
    assert mimey.detect_type(payload) == ".nes"


def test_register_magic_honours_offset() -> None:
    mimey.register("application/x-offset", ".off", magic=b"HERE", offset=4)
    assert mimey.detect_mime(b"\x00\x00\x00\x00HERE") == "application/x-offset"
    assert mimey.detect_mime(b"HERE\x00\x00\x00\x00") != "application/x-offset"


def test_magic_shorter_than_offset_does_not_match_or_panic() -> None:
    mimey.register("application/x-far", ".far", magic=b"X", offset=64)
    assert mimey.detect_mime(b"X") != "application/x-far"
    assert mimey.detect_mime(b"") != "application/x-far"


def test_registered_type_beats_builtin() -> None:
    mimey.register("image/x-mine", ".mine", magic=b"\x89PNG")
    assert mimey.detect_mime(PNG) == "image/x-mine"
    assert mimey.detect_type(PNG) == ".mine"


def test_first_registration_wins() -> None:
    mimey.register("application/x-first", ".one", magic=b"AB")
    mimey.register("application/x-second", ".two", magic=b"AB")
    assert mimey.detect_mime(b"ABCD") == "application/x-first"


def test_callable_matcher() -> None:
    mimey.register("application/x-even", ".even", matcher=lambda data: len(data) % 2 == 0)
    assert mimey.detect_mime(b"ab") == "application/x-even"
    assert mimey.detect_mime(b"abc") != "application/x-even"


def test_callable_receives_bytes() -> None:
    seen: list[object] = []

    def matcher(data: object) -> bool:
        seen.append(data)
        return False

    mimey.register("application/x-spy", ".spy", matcher=matcher)
    mimey.detect_mime(b"hello")
    assert seen == [b"hello"]


def test_callable_exception_propagates() -> None:
    def boom(data: bytes) -> bool:
        raise RuntimeError("matcher exploded")

    mimey.register("application/x-boom", ".boom", matcher=boom)
    with pytest.raises(RuntimeError, match="matcher exploded"):
        mimey.detect_mime(PNG)


def test_matcher_may_register_without_deadlocking() -> None:
    def matcher(data: bytes) -> bool:
        mimey.register("application/x-nested", ".nested", magic=b"zzz")
        return False

    mimey.register("application/x-outer", ".outer", matcher=matcher)
    assert mimey.detect_mime(PNG) == "image/png"


def test_register_rejects_both_magic_and_matcher() -> None:
    with pytest.raises(ValueError):
        mimey.register("a/b", ".b", magic=b"X", matcher=lambda data: True)


def test_register_rejects_neither() -> None:
    with pytest.raises(ValueError):
        mimey.register("a/b", ".b")


def test_register_rejects_empty_magic() -> None:
    with pytest.raises(ValueError):
        mimey.register("a/b", ".b", magic=b"")


def test_registered_reports_entries_in_order() -> None:
    mimey.register("application/x-a", ".a", magic=b"AAA")
    mimey.register("application/x-b", ".b", magic=b"BBB")
    assert mimey.registered() == [("application/x-a", ".a"), ("application/x-b", ".b")]


def test_clear_registrations_restores_builtin_behaviour() -> None:
    mimey.register("image/x-mine", ".mine", magic=b"\x89PNG")
    assert mimey.detect_mime(PNG) == "image/x-mine"
    mimey.clear_registrations()
    assert mimey.detect_mime(PNG) == "image/png"
    assert mimey.registered() == []
