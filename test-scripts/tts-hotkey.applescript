#!/usr/bin/osascript

# Save the current clipboard
set oldClipboard to the clipboard

# Copy selected text
tell application "System Events"
    keystroke "c" using command down
    delay 0.1
end tell

# Get the copied text
set selectedText to the clipboard as string

# Restore old clipboard
set the clipboard to oldClipboard

# URL encode the text
set encodedText to do shell script "python3 -c \"import sys, urllib.parse; print(urllib.parse.quote(sys.argv[1]))\" " & quoted form of selectedText

# Launch TTS Player with the text
set appPath to "/Users/bradyprice/workspace/personal/tts-player/src-tauri/target/release/bundle/macos/TTS Player.app"
do shell script "open -a " & quoted form of appPath & " --args --text=\"" & encodedText & "\""

return "Speaking selected text..."