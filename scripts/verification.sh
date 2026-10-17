#!/usr/bin/env bash
set -euo pipefail

# Directory for temporary artifacts
TMPDIR="/tmp/filmcraft_verif_$$"
mkdir -p "$TMPDIR"

# 1. GPU vs CPU render parity test using the built‑in test harness.
# The GPU test writes a PNG to target/debug/gpu_parity.png and the CPU reference
# is written to target/debug/cpu_parity.png by the same test suite. We run the test
# and capture the RMSE metric printed by the test harness.
echo "Running GPU/CPU parity test..."
# Run the parity test; it prints the RMSE metric to stdout.
PARITY_OUTPUT=$(cargo test -p filmcraft-gpu --release --quiet -- --nocapture || true)
# Extract the metric line (looks like "RMSE: <value>")
RMSE_LINE=$(echo "$PARITY_OUTPUT" | grep -i "rmse" || true)
echo "$RMSE_LINE" > "$TMPDIR/compare.txt"

# 2. Wayland control latency test (no wmctrl).
# Start FilmCraft with a control server on a free port (9876).
CTRL_PORT=9876
filmcraft --control $CTRL_PORT &
FC_PID=$!
# Give the server a moment to start.
sleep 2
# Baseline request latency.
START=$(date +%s%3N)
filmcraft-cli --bridge 127.0.0.1:$CTRL_PORT exec ui.inspect > /dev/null
END=$(date +%s%3N)
LAT1=$((END-START))
# Simulate occlusion by opening a temporary Wayland window (foot terminal).
foot -e "sleep 5" &
sleep 1
# Request latency while occluded.
START2=$(date +%s%3N)
filmcraft-cli --bridge 127.0.0.1:$CTRL_PORT exec ui.inspect > /dev/null
END2=$(date +%s%3N)
LAT2=$((END2-START2))
# Wake the app without stealing focus using the UI focus command.
filmcraft-cli --bridge 127.0.0.1:$CTRL_PORT exec ui.focus > /dev/null
# Request latency after explicit focus.
START3=$(date +%s%3N)
filmcraft-cli --bridge 127.0.0.1:$CTRL_PORT exec ui.inspect > /dev/null
END3=$(date +%s%3N)
LAT3=$((END3-START3))
# Clean up.
kill $FC_PID || true

echo "Latency baseline: ${LAT1}ms"
echo "Latency while occluded: ${LAT2}ms"
echo "Latency after ui.focus: ${LAT3}ms"

# 3. Audio format test – capture the format reported by CPAL on startup.
# Generate a short sine‑wave WAV.
AUDIO_WAV="$TMPDIR/sine.wav"
ffmpeg -f lavfi -i sine=frequency=440:duration=2 -c:a pcm_s16le "$AUDIO_WAV" -y > /dev/null 2>&1
# Import the wav into a demo project.
filmcraft-cli --demo import "$AUDIO_WAV" > "$TMPDIR/audio_import.log" 2>&1 || true
# Export to /dev/null; the audio subsystem will log the chosen format.
filmcraft-cli --demo export /dev/null > "$TMPDIR/audio_export.log" 2>&1 || true

echo "Audio format test logs:" && cat "$TMPDIR/audio_export.log" | grep -i "format" || true

# 4. Caption ingestion edge cases.
cat <<'EOF' > "$TMPDIR/edge.srt"
1
00:00:01,000 --> 00:00:02,000
Hello <b>World</b> &amp; <font color="#00ff00">Green</font>
2
00:00:03,000 --> 00:00:04,000
<i>Italic <b>Bold</b></i> < & > literal
EOF
filmcraft-cli --demo import "$TMPDIR/edge.srt" > /dev/null 2>&1
filmcraft-cli --demo exec captions.list > "$TMPDIR/captions_output.txt"

echo "Caption ingestion output:"
cat "$TMPDIR/captions_output.txt"

# 5. Caption UI reminder – manual verification.
echo "Please manually launch FilmCraft UI and verify caption style controls (stroke, background, margin, scroll)."
