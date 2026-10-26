#!/usr/bin/env bash
# One .wav per narration line into assets/vo/, skipping lines already rendered.
# Re-run build_index.py afterwards: index.html hardcodes each line's measured length.
set -uo pipefail
HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
VO=$(dirname "$HERE")/assets/vo

# Environment, not repo layout. HYPERFRAMES_PYTHON must be an interpreter with kokoro_onnx
# installed; the pin matches package.json so the voice re-renders on the recorded version.
export HYPERFRAMES_PYTHON=${HYPERFRAMES_PYTHON:-$HOME/.venv/bin/python}
HF_PIN=${HYPERFRAMES_PIN:-hyperframes@0.8.186}

mkdir -p "$VO"
LINES=$(mktemp) && trap 'rm -f "$LINES"' EXIT
python3 -c "
import json, sys
for key, _scene, _anchor, text in json.load(open(sys.argv[1])):
    print(key + '\t' + text)
" "$HERE/narration.json" > "$LINES"

while IFS=$'\t' read -r id text; do
  out="$VO/$id.wav"
  [ -s "$out" ] && { echo "skip $id"; continue; }
  # keep kokoro_onnx resident so the CLI's 10s import probe does not time out under swap
  "$HYPERFRAMES_PYTHON" -c "import kokoro_onnx" >/dev/null 2>&1
  echo "=== $id: $text"
  npx --yes "$HF_PIN" tts "$text" --voice af_heart --speed 1.0 --output "$out" --json 2>&1 | tail -2
done < "$LINES"
echo "DONE"
ls -la "$VO" | tail -40
