#!/bin/bash
# verify-minimax-music-api.sh
# Quick smoke test: verify token works for music-2.6 endpoint

set -e

TOKEN="${1:-$MINIMAX_API_KEY}"
ENDPOINT="https://api.minimax.io/v1/music_generation"

if [ -z "$TOKEN" ]; then
  echo "ERROR: Pass token as arg1 or set MINIMAX_API_KEY env var"
  exit 1
fi

echo "Testing MiniMax music-2.6 API..."
response=$(curl -s -X POST "$ENDPOINT" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "model": "music-2.6",
    "prompt": "test music generation",
    "lyrics": "[Intro]\ntest",
    "audio_setting": {"sample_rate": 44100, "bitrate": 256000, "format": "mp3"},
    "output_format": "url"
  }')

status=$(echo "$response" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('base_resp',{}).get('status_code',''))")
msg=$(echo "$response" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('base_resp',{}).get('status_msg',''))")

if [ "$status" = "0" ]; then
  echo "✅ Token valid — music API accessible"
  echo "Response: $msg"
  exit 0
elif [ "$status" = "1004" ]; then
  echo "❌ Auth failed — check token (status_code 1004: login fail)"
  exit 2
elif [ "$status" = "2013" ]; then
  echo "⚠️  Auth OK but lyrics required error — token works, add lyrics to payload"
  exit 0
else
  echo "⚠️  Unexpected response: status=$status msg=$msg"
  echo "$response"
  exit 3
fi
