#!/bin/bash

echo "Direct test of TTS Player"
echo "========================="

# Put test text in clipboard
TEST_TEXT="This is a direct test of the TTS Player clipboard functionality"
echo "$TEST_TEXT" | pbcopy
echo "Clipboard contains: $(pbpaste)"

# Kill any existing instances
pkill -f "tts-player" 2>/dev/null
sleep 0.2

# Launch the binary directly to see console output
BINARY="/Users/bradyprice/workspace/personal/tts-player/src-tauri/target/release/bundle/macos/TTS Player.app/Contents/MacOS/tts-player"

if [ ! -f "$BINARY" ]; then
    echo "Binary not found at release location"
    BINARY="/Users/bradyprice/workspace/personal/tts-player/src-tauri/target/debug/tts-player"
fi

if [ ! -f "$BINARY" ]; then
    echo "Error: Binary not found"
    exit 1
fi

echo ""
echo "Launching binary directly..."
echo "Check the console output and the app window"
echo ""

# Run directly to see output
"$BINARY"