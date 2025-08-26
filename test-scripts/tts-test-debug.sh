#!/bin/bash

# Test script to debug CLI argument passing

# Test text
TEXT="Hello, this is a test"

# URL encode the text
ENCODED_TEXT=$(python3 -c "import sys, urllib.parse; print(urllib.parse.quote(sys.argv[1]))" "$TEXT")

echo "Original text: $TEXT"
echo "Encoded text: $ENCODED_TEXT"

# Path to the app
APP_PATH="/Users/bradyprice/workspace/personal/tts-player/src-tauri/target/release/bundle/macos/TTS Player.app"

# Show the exact command we're running
echo "Running command:"
echo "open -n -a \"$APP_PATH\" --args --text \"$ENCODED_TEXT\""

# Run it
open -n -a "$APP_PATH" --args --text "$ENCODED_TEXT"