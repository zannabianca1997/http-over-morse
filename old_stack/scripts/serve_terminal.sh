#!/usr/bin/env bash
#
# serve_terminal.sh — flash the `wiretap` firmware and serve a login terminal
# over the morse wire, throttled so we don't overrun the micro.
#
#   1. Builds `bins/wiretap` with the `write` + `base32` features and flashes it
#      to the Nano on $PORT at compile-time baud $BAUD (via ravedude).
#   2. Stands up a virtual serial port ($VPORT) bridged to $PORT through a
#      trickle: host -> Arduino is capped to $RATE bytes/sec (the firmware can't
#      empty the serial buffer fast enough otherwise); Arduino -> host runs full
#      speed.
#   3. Attaches `agetty` to $VPORT so a real login session runs over the link.
#   4. Keeps everything running until you Ctrl-C, then tears it all down.
#
# Requires: socat, pv (assumed installed). agetty needs sudo.

set -euo pipefail

# agetty needs root; grab (and cache) credentials now, on the real terminal,
# so the later backgrounded `sudo` can't silently fail on a password prompt.
sudo -v

PORT=/dev/ttyUSB1
BAUD=9600
RATE=5                      # bytes/sec, host -> Arduino (the throttled direction)
VPORT=/tmp/wiretap-tty      # virtual port agetty logs in on
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# ---------------------------------------------------------------------------
# 1. Compile + flash the firmware
# ---------------------------------------------------------------------------
echo ">> Building + flashing wiretap (write,base32) to $PORT @ ${BAUD} baud..."
cd "$ROOT"
BAUDRATE="$BAUD" RAVEDUDE_PORT="$PORT" \
  cargo run --release -p wiretap --no-default-features --features write,base32

# ---------------------------------------------------------------------------
# Cleanup (registered before we spawn anything long-lived)
# ---------------------------------------------------------------------------
BRIDGE=""
SOCAT_PID=""
AGETTY_PID=""
cleanup() {
  [ -n "$AGETTY_PID" ] && sudo kill "$AGETTY_PID" 2>/dev/null || true
  [ -n "$SOCAT_PID" ]  && kill "$SOCAT_PID"        2>/dev/null || true
  [ -n "$BRIDGE" ]     && rm -f "$BRIDGE"
  rm -f "$VPORT"
}
trap cleanup EXIT INT TERM

# ---------------------------------------------------------------------------
# 2. Trickle bridge: agetty <-> VPORT(pty) <-> socat <-> [throttle] <-> PORT
#
#   agetty ── VPORT ── socat ──┬─ pv -qL N ─► PORT   (TX, throttled to $RATE B/s)
#                              └─ cat       ◄─ PORT   (RX, full speed)
#
# `cat` opens PORT read-only and `pv` write-only: one fd per direction, so the
# two never fight over a shared read/write fd.
# ---------------------------------------------------------------------------
echo ">> Starting trickle bridge ($RATE B/s host->Arduino) on $VPORT..."
BRIDGE="$(mktemp)"
cat >"$BRIDGE" <<EOF
#!/bin/sh
stty -F "$PORT" "$BAUD" raw -echo -crtscts
cat "$PORT" &                       # RX: Arduino -> agetty, full speed
trap 'kill \$!' EXIT
exec pv -qL "$RATE" -B 1 > "$PORT"  # TX: agetty -> Arduino, throttled
EOF
chmod +x "$BRIDGE"

socat -d -d PTY,link="$VPORT",raw,echo=0 "EXEC:$BRIDGE" &
SOCAT_PID=$!

# Wait for the pty to materialize before handing it to agetty.
while [ ! -e "$VPORT" ]; do sleep 0.1; done

# ---------------------------------------------------------------------------
# 3. Attach agetty to the virtual port
# ---------------------------------------------------------------------------
# agetty blindly prepends "/dev/" to its tty argument, so an absolute /tmp path
# turns into the bogus "/dev//tmp/wiretap-tty". Resolve the pty symlink to its
# real /dev/pts/N node and hand agetty the part after "/dev/" (e.g. "pts/6").
PTS="$(readlink -f "$VPORT")"          # e.g. /dev/pts/6
echo ">> Attaching agetty to $PTS @ ${BAUD} baud..."
sudo setsid agetty -L "$BAUD" "${PTS#/dev/}" vt100 &
AGETTY_PID=$!

# ---------------------------------------------------------------------------
# 4. Keep everything open until interrupted
# ---------------------------------------------------------------------------
echo ">> Terminal is live. Press Ctrl-C to tear everything down."
wait
