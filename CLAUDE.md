# TTS Player Project - Development Guide

## Project Overview
Tauri-based text-to-speech application using OpenAI's TTS API with Raycast integration for clipboard reading.

## Critical Lessons Learned

### 1. Application Deployment & Testing
**Problem**: After building updates, the old version may still be running.
**Solution**: 
```bash
# WRONG - creates nested directory structure
cp -R "path/to/new.app" "/Applications/TTS Player.app"

# CORRECT - remove old first, then copy
rm -rf "/Applications/TTS Player.app"
cp -R "path/to/new.app" "/Applications/"
```

**Always verify binary dates after deployment:**
```bash
ls -la "/Applications/TTS Player.app/Contents/MacOS/tts-player"
```

### 2. Audio Concatenation for Long Text
**Problem**: OpenAI TTS has a 4096 character limit. Simply concatenating MP3 bytes creates invalid audio.
**Solution**: Use FFmpeg for proper MP3 concatenation:
- Split text at sentence boundaries (keeping chunks under 4000 chars)
- Generate audio for each chunk separately
- Save chunks as temp files
- Use FFmpeg's concat demuxer to merge properly
- Clean up temp files

**Implementation location**: `src-tauri/src/tts.rs::generate_speech_tracked()`

### 3. Validation Chain
**Problem**: Text validation happens at multiple points in the call chain.
**Key locations to check**:
- `src-tauri/src/main.rs` - API endpoint validation (lines 22, 48)
- `src-tauri/src/tts.rs::validate_text()` - Core validation logic
- Frontend components may show warnings but shouldn't block submission

### 4. Raycast Integration
**Script location**: `/Users/bradyprice/Tools/scripts/tts-speak-hotkey.sh`
**App paths**:
- Production: `/Applications/TTS Player.app`
- Development: `/Users/bradyprice/workspace/personal/tts-player/src-tauri/target/release/bundle/macos/TTS Player.app`

## Build & Deploy Workflow

### Full rebuild and deployment:
```bash
cd /Users/bradyprice/workspace/personal/tts-player
npm run tauri build

# Deploy to Applications
rm -rf "/Applications/TTS Player.app"
cp -R "src-tauri/target/release/bundle/macos/TTS Player.app" "/Applications/"

# Verify deployment
ls -la "/Applications/TTS Player.app/Contents/MacOS/tts-player"
```

### Quick backend-only rebuild:
```bash
cd src-tauri
cargo build --release
```

## Technical Architecture

### Key Dependencies
- **FFmpeg**: Required for audio concatenation (must be installed: `brew install ffmpeg`)
- **tempfile**: Rust crate for temporary file handling during audio processing
- **OpenAI API**: TTS-1-HD model, $30 per 1M characters

### File Structure
```
src-tauri/
  src/
    main.rs         # Tauri command handlers
    tts.rs          # TTS service with chunking logic
    database.rs     # Usage tracking
    file_manager.rs # File operations
src/
  components/
    TTSPlayer.tsx   # Main UI component
    CharacterCounter.tsx # Shows character count (doesn't block >4096)
```

## Common Issues & Solutions

### "Text too long (max 4096 characters)"
1. Check you're running the latest build (see binary date)
2. Verify `validate_text()` has no max length check
3. Ensure proper app installation (no nested directories)

### "Network error: error decoding response body"
- Indicates MP3 concatenation issue
- Solution: Use FFmpeg-based concatenation, not byte concatenation

### Raycast not using updated version
1. Check script path: `cat /Users/bradyprice/Tools/scripts/tts-speak-hotkey.sh | grep APP_PATH`
2. Ensure it points to `/Applications/TTS Player.app`
3. Verify latest build is deployed there

## Development Tips

1. **Always test with long text** (>4096 chars) after making changes
2. **Use version checking**: Add version number to UI to verify which build is running
3. **Log chunking operations**: Add debug logging to see chunk boundaries
4. **Monitor temp files**: Ensure cleanup happens even on errors

## Future Improvements

- [ ] Add version display in UI for easier debugging
- [ ] Implement progress indicator for multi-chunk processing
- [ ] Add chunk count display when processing long text
- [ ] Consider using WAV format for easier concatenation
- [ ] Add retry logic for individual chunk failures