#!/bin/bash

# TTS Player - Raycast Integration Script
# Usage: tts-player.sh "Text to speak"

# Configuration
TTS_PLAYER_PATH="/Users/bradyprice/workspace/personal/tts-player/src-tauri/target/release/tts-player"
DEFAULT_VOICE="rachel"

# Get text from argument
TEXT="$1"
VOICE="${2:-$DEFAULT_VOICE}"

# Validate input
if [ -z "$TEXT" ]; then
    echo "Usage: tts-player.sh \"Text to speak\" [voice_id]"
    echo "Available voices: rachel, adam, bella"
    exit 1
fi

# URL encode the text to handle special characters
ENCODED_TEXT=$(python3 -c "import urllib.parse; print(urllib.parse.quote('$TEXT'))")

# Check if TTS Player binary exists
if [ ! -f "$TTS_PLAYER_PATH" ]; then
    echo "Error: TTS Player not found at $TTS_PLAYER_PATH"
    echo "Please build the application first: cd ~/workspace/personal/tts-player && npm run tauri build"
    exit 1
fi

# Launch TTS Player with text parameter
echo "Launching TTS Player with text: $TEXT"
"$TTS_PLAYER_PATH" --text "$ENCODED_TEXT" --voice "$VOICE" &

# Optional: Wait a moment to see if app launches successfully
sleep 1

# Check if the process is running
if pgrep -f "tts-player.*$ENCODED_TEXT" > /dev/null; then
    echo "✓ TTS Player launched successfully"
else
    echo "⚠ Warning: TTS Player may not have launched correctly"
fi