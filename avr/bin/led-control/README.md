# `led-control`

Control the debug led via serial, with single byte commands.

Send `\0` to turn it off, `\1` to turn it on, `\2` to toggle, `\4` is a no-op.

All commands are answered with the state after (so the last one is effectively a
read): `\0` if led is off and `\1` if led is on.
