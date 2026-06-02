# Direct to wire communication

This binary will read the content of the wire to the serial port, and optionally
write the content of the serial port to the wire.

To configure the port baud rate, set the `BAUDRATE` environment variable. The
default is 9600. Remember that changing the baudrate will make the `ravedude`
console break, unless you change also the `Ravedude.toml` in the root.

To configure the byte encoding set the appropriate feature flag at build time.
Possible flags are `base32` and `discard`. If both are set `base32` will take
precedence. If none set `discard` is the default.

To also write set the `write` feature flag. This has some caveats (see next
section).

# Caveats

Even at the slowest possible baud rate of 250 the encoding is yet not fast
enough to empty the serial buffer in time. Do not pipe things straight to this
serial port.

In `base32` mode spaces are used to flush between writes. The effective encoding
will depend on the timings of the keystrokes.
