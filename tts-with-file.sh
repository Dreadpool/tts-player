#!/bin/bash

# TTS Player with File - Alternative launcher
# This script creates a temporary file with the text and uses file argument

if [ $# -eq 0 ]; then
    echo "Usage: $0 \"text to speak\""
    exit 1
fi

# Create temporary file
TMP_FILE=$(mktemp)
echo -n "$*" > "$TMP_FILE"

echo "Created temp file: $TMP_FILE"
echo "Launching TTS Player with file argument..."

# Launch the app with file argument
exec ./src-tauri/target/release/tts-player --file "$TMP_FILE"