# Encoding bytes to morse letters

This crate contains strategies to encode binaries to morse letters.

## Strategies available

### Ascii

The easiest strategy: map ascii bytes that have a corresponding morse letter in
that letter and viceversa. Discard all in both way that has not a direct
translation.

To make is more useful this will also translate all ascii spaces into simple
word spaces and uppercase all ascii letters.

### Base32

Morse code contains the alphabet for
[base32](https://en.wikipedia.org/wiki/Base32). This strategy employs it to
encode arbitrary octets in the stream.

Space are inserted when the underlying octet stream would block, and are read on
the decoding side as a signal to flush the buffer.
