<p align="center">
  <img src="https://raw.githubusercontent.com/4thel00z/mimey/master/assets/logo.svg" width="180" alt="mimey logo">
</p>

<h1 align="center">mimey</h1>

<p align="center">
  <strong>Tells you what a file is from its first bytes. Rust underneath.</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/pypi/v/mimey?logo=pypi&logoColor=white&color=2a66e0" alt="PyPI">
  <img src="https://img.shields.io/pypi/pyversions/mimey?logo=python&logoColor=white&color=3776AB" alt="Python versions">
  <img src="https://img.shields.io/badge/built%20with-rust-292420?logo=rust&logoColor=white" alt="Built with Rust">
  <img src="https://img.shields.io/badge/license-MIT-2a66e0" alt="MIT license">
</p>

---

```sh
uv add mimey     # or: pip install mimey
```

```python
>>> import mimey
>>> mimey.detect_mime(b"\x89PNG\r\n\x1a\n")
'image/png'
>>> mimey.detect_type(b"\x89PNG\r\n\x1a\n")
'.png'
```

No filename, no `libmagic`, no shelling out to `file`. It reads the bytes you
hand it and answers. Detection looks at the first 3072 bytes at most, so an
8-byte header and a 1 MB payload cost the same.

Wheels ship for CPython 3.10–3.14 on manylinux, musllinux, macOS and Windows.
Linux additionally gets free-threaded 3.14t and PyPy.

## Register your own types

Registered types are checked before the built-in table, in registration order,
so they can teach `mimey` a format it does not know or override a verdict it
gets wrong.

```python
mimey.register("application/x-nes-rom", ".nes", magic=b"NES\x1a")

rom = open("game.nes", "rb").read()
mimey.detect_mime(rom)   # 'application/x-nes-rom'
```

Pass `offset` when the signature does not start at byte 0:

```python
mimey.register("application/x-offset", ".off", magic=b"HERE", offset=4)
```

Matching stays in Rust. A signature that hits is *faster* than built-in
detection, because it never walks the detection tree.

For anything a fixed signature cannot express, hand it a callable instead:

```python
mimey.register("application/x-even", ".even", matcher=lambda data: len(data) % 2 == 0)
```

The callable gets the same `bytes` you passed in, and its exceptions propagate
to the caller. Every detection re-enters Python once per registered callable, so
prefer `magic` when a signature is enough.

```python
mimey.registered()          # [('application/x-nes-rom', '.nes')]
mimey.clear_registrations()
```

## Performance

Per call, macOS arm64 / CPython 3.12, min-of-7 over 200k calls:

| | ns/call |
|---|--:|
| `detect_mime`, no registrations | **78** |
| one registered signature, no match | 83 |
| one registered signature, **matches** | **37** |
| one registered callable | 120 |

Cost is flat in payload size — the detector sniffs a prefix, so the per-call
number is dominated by the Python↔Rust boundary, not by your data.

## License

MIT. Built on [mimetype-detector](https://crates.io/crates/mimetype-detector)
and [PyO3](https://pyo3.rs).
