#!/usr/bin/env bats

setup() {
    export TEST_MODE=true
    export TTS_PLAYER_PATH="/Users/bradyprice/workspace/personal/tts-player/src-tauri/target/debug/tts-player"
}

teardown() {
    # Kill any running instances
    pkill -f "tts-player" || true
}

@test "raycast script launches app with text parameter" {
    skip_if_binary_missing
    
    run timeout 10 "$TTS_PLAYER_PATH" --text "Hello world"
    [ "$status" -eq 0 ]
}

@test "app handles URL encoded text" {
    skip_if_binary_missing
    
    run timeout 10 "$TTS_PLAYER_PATH" --text "Hello%20world%21"
    [ "$status" -eq 0 ]
}

@test "app handles empty text parameter" {
    skip_if_binary_missing
    
    run timeout 10 "$TTS_PLAYER_PATH" --text ""
    [ "$status" -eq 0 ]
}

@test "app handles special characters" {
    skip_if_binary_missing
    
    run timeout 10 "$TTS_PLAYER_PATH" --text "Hello! 你好! Émojis 🎵"
    [ "$status" -eq 0 ]
}

@test "app handles voice parameter" {
    skip_if_binary_missing
    
    run timeout 10 "$TTS_PLAYER_PATH" --text "Hello world" --voice "rachel"
    [ "$status" -eq 0 ]
}

@test "app shows help with invalid arguments" {
    skip_if_binary_missing
    
    run "$TTS_PLAYER_PATH" --invalid-flag
    [ "$status" -ne 0 ]
    [[ "$output" == *"help"* ]] || [[ "$output" == *"usage"* ]]
}

@test "raycast script handles long text" {
    skip_if_binary_missing
    
    local long_text=$(printf 'a%.0s' {1..500})
    run timeout 10 "$TTS_PLAYER_PATH" --text "$long_text"
    [ "$status" -eq 0 ]
}

# Helper functions
skip_if_binary_missing() {
    if [ ! -f "$TTS_PLAYER_PATH" ]; then
        skip "Binary not built yet: $TTS_PLAYER_PATH"
    fi
}

skip_if_no_display() {
    if [ -z "$DISPLAY" ] && [ -z "$WAYLAND_DISPLAY" ]; then
        skip "No display available for GUI testing"
    fi
}