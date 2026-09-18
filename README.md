# mimey

## Motivation

A fast mime parser written in Rust and exposed as python package.

## Installation

```
uv add mimey
```

or

```
pip install mimey
```

## Usage


### Detect a type
```python
>>> import mimey
>>> mimey.detect_type(b"\x89PNG\r\n\x1a\n")
'.png'
```

### Detect the mimetype

```python
>>> import mimey
>>> mimey.detect_mime(b"\x89PNG\r\n\x1a\n")
'image/png'
```

## Registering your own types

Registered types are checked before the built-in table, in registration order,
so they can teach `mimey` a format it doesn't know or override a built-in verdict.

### By magic bytes

```python
import mimey

mimey.register("application/x-nes-rom", ".nes", magic=b"NES\x1a")

rom = open("game.nes", "rb").read()
mimey.detect_mime(rom)  # 'application/x-nes-rom'
```

Pass `offset` when the signature does not start at byte 0:

```python
mimey.register("application/x-offset", ".off", magic=b"HERE", offset=4)
```

Matching stays in Rust, so a registered type costs a few nanoseconds — a
signature that hits is actually faster than built-in detection, because it
skips the detection tree.

### By callable

For anything a fixed signature cannot express:

```python
mimey.register("application/x-even", ".even", matcher=lambda data: len(data) % 2 == 0)
```

The callable receives the same `bytes` you passed in and its exceptions
propagate to the caller. Every detection re-enters Python once per registered
callable (~40ns each), so prefer `magic` when a signature is enough.

### Inspecting and clearing

```python
mimey.registered()          # [('application/x-nes-rom', '.nes')]
mimey.clear_registrations()
```
