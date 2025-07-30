#!/bin/bash
set -e

echo "=== Niri Wayland Environment Test ==="
echo "Mesa version: $(glxinfo | grep "OpenGL version" 2>/dev/null || echo "N/A")"
echo "Waypipe version: $(waypipe --version)"
echo "Niri version: $(cd ~/repos/niri && ./target/release/niri --version)"

echo "=== Starting Weston ==="
weston --width=1200 --height=800 &
WESTON_PID=$!
sleep 5

echo "=== Testing Niri ==="
export WAYLAND_DISPLAY=wayland-1
cd ~/repos/niri
timeout 10s ./target/release/niri --help > /dev/null && echo "✓ Niri help works"
echo "=== Attempting to start Niri (15 second timeout) ==="
timeout 15s ./target/release/niri 2>&1 | head -10

echo "=== Cleanup ==="
kill $WESTON_PID 2>/dev/null || true
wait $WESTON_PID 2>/dev/null || true
echo "Test completed"
