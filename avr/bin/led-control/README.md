# `led-control`

Control the debug led via serial, with single byte commands.

Send `\1` to turn it off, `\2` to turn it on, `\3` to toggle, `\4` is a no-op.

All commands are answered with the state after (so the last one is effectively a
read): `\1` if led is off and `\2` if led is on.

Signals start at `\1` so that line noise (which reads as `\0`) never maps to a
valid command nor to a valid answer: a flood of `\0` bytes is silently ignored.
