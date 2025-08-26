#!/bin/bash

# TTS Player with Clipboard - Alternative launcher
# This script copies text to clipboard and launches the TTS Player
# The app will automatically read from clipboard if CLI args fail

if [ $# -eq 0 ]; then
    echo "Usage: $0 \"text to speak\""
    echo "Alternative: Copy text to clipboard and run: $0 --clipboard"
    exit 1
fi

if [ "$1" = "--clipboard" ]; then
    echo "Launching TTS Player - it will read from clipboard"
else
    # Copy the text to clipboard
    echo -n "$*" | pbcopy
    echo "Text copied to clipboard. Launching TTS Player..."
fi

# Launch the app - it will try CLI args first, then fall back to clipboard
exec ./src-tauri/target/release/tts-player --text "$*"