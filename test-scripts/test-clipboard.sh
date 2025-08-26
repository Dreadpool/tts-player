#!/bin/bash

echo "Testing TTS Player with clipboard approach..."
echo "============================================"

# Test text
TEST_TEXT="Hello! This is a test of the clipboard-based TTS Player."
echo "Test text: $TEST_TEXT"

# Copy to clipboard
echo "$TEST_TEXT" | pbcopy
echo "Text copied to clipboard"

# Kill any existing instances
pkill -f "tts-player" 2>/dev/null
sleep 0.1

# Launch the app
APP_PATH="/Users/bradyprice/workspace/personal/tts-player/src-tauri/target/release/bundle/macos/TTS Player.app"

echo ""
echo "Launching app..."
open -a "$APP_PATH"

echo ""
echo "✓ App should now open with the text from clipboard"
echo "Check if the text appears in the text box"