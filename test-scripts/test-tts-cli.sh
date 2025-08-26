#!/bin/bash

# Test script to verify CLI argument passing to TTS Player

echo "Testing TTS Player CLI argument passing..."
echo "============================================"

# Test text
TEST_TEXT="Hello, this is a test of the TTS player CLI arguments."
echo "Test text: $TEST_TEXT"

# URL encode the text
ENCODED_TEXT=$(python3 -c "import sys, urllib.parse; print(urllib.parse.quote(sys.argv[1]))" "$TEST_TEXT")
echo "Encoded text: $ENCODED_TEXT"

# Path to binary
BINARY="/Users/bradyprice/workspace/personal/tts-player/src-tauri/target/release/bundle/macos/TTS Player.app/Contents/MacOS/tts-player"

if [ ! -f "$BINARY" ]; then
    echo "Error: Binary not found at $BINARY"
    echo "Trying debug build..."
    BINARY="/Users/bradyprice/workspace/personal/tts-player/src-tauri/target/debug/bundle/macos/TTS Player.app/Contents/MacOS/tts-player"
fi

if [ ! -f "$BINARY" ]; then
    echo "Error: No binary found. Please build the app first."
    exit 1
fi

echo ""
echo "Executing command:"
echo "$BINARY --text \"$ENCODED_TEXT\""
echo ""

# Kill any existing instances first
pkill -f "tts-player" 2>/dev/null
sleep 0.2

# Execute the binary with arguments
"$BINARY" --text "$ENCODED_TEXT"

echo ""
echo "Test complete. Check if the app opened with text filled in."