# TTS Player

A beautifully minimal text-to-speech application built with **Jony Ive design principles** - crafted for macOS with seamless Raycast integration.

## Features

### 🎯 **Core Functionality**
- **Instant Text-to-Speech** - Convert any text to high-quality speech using ElevenLabs API
- **Smart Clipboard Integration** - Automatically processes text from clipboard on launch
- **Morphing Interface** - Text input elegantly transforms into compact media player controls
- **Autoplay Support** - Generated speech plays automatically when launched via hotkey

### ⚡ **Raycast Integration**
- **One-Click Workflow** - Highlight text → Press hotkey → Instant speech
- **No Manual Copying** - Automatically copies highlighted text when triggered
- **Background Processing** - Works seamlessly without disrupting your workflow
- **Focus Management** - Preserves text selection while launching

### 🎨 **Design Philosophy**
- **Jony Ive Inspired** - Following Apple's design principles of simplicity and inevitability
- **Spatial Efficiency** - Interface morphs rather than expanding, no scrolling needed
- **Consistent Actions** - Primary button always in the same location
- **Progressive Disclosure** - Shows exactly what's needed, when it's needed
- **Material Honesty** - Respects platform conventions and user expectations

### 📊 **Advanced Features**
- **Usage Tracking** - Monitor character usage and API costs
- **Character Counter** - Real-time feedback with quota awareness
- **Error Recovery** - Graceful handling of API limits and network issues
- **Keyboard Accessibility** - Complete keyboard navigation support

## Quick Start

### Prerequisites
- Node.js 18+
- Rust 1.70+
- ElevenLabs API key

### Installation

1. **Clone and setup:**
   ```bash
   cd ~/workspace/personal/tts-player
   npm install
   ```

2. **Configure ElevenLabs API:**
   ```bash
   export ELEVENLABS_API_KEY="your-api-key"
   ```

3. **Development:**
   ```bash
   npm run tauri dev
   ```

4. **Build:**
   ```bash
   npm run tauri build
   ```

### Raycast Setup

1. **Copy the Raycast script**
   The script is located at `/Users/bradyprice/Tools/scripts/tts-speak-hotkey.sh`

2. **Grant Accessibility Permissions**
   - Open **System Settings** → **Privacy & Security** → **Accessibility**
   - Add **Raycast** and **Terminal** to the allowed apps
   - Ensure both are enabled

3. **Set up Raycast hotkey**
   - Open Raycast preferences
   - Find "Speak (Hotkey)" command
   - Assign your preferred hotkey (e.g., `⌘⇧S`)

### Usage

**Method 1: Raycast Hotkey (Recommended)**
1. Highlight text in any application
2. Press your assigned hotkey
3. TTS Player launches and speaks the text automatically

**Method 2: Manual Launch**
1. Copy text to clipboard (`⌘C`)
2. Launch TTS Player
3. Text auto-generates speech, or click "Generate Speech"

## Testing

We follow a comprehensive test-first development approach with multiple testing layers:

### Test Architecture
- **Unit Tests (60%)**: Fast, isolated component testing
- **Integration Tests (30%)**: API and system integration  
- **E2E Tests (10%)**: Complete user workflows

### Running Tests

```bash
# Run all tests
./run-tests.sh

# Individual test suites
npm test              # Frontend tests
cd src-tauri && cargo test  # Rust tests
bats tests/e2e/*.bats # Integration tests
```

### Test Coverage

- ✅ **CLI Arguments**: Text parsing, URL encoding, voice selection
- ✅ **ElevenLabs API**: Authentication, TTS generation, rate limiting
- ✅ **Audio Playback**: Play/pause/stop, seek, volume control
- ✅ **UI Components**: Loading states, error handling, voice selection
- ✅ **File Management**: Temp file creation/cleanup
- ✅ **Performance**: Startup time, generation speed, responsiveness
- ✅ **User Journeys**: Complete Raycast → TTS → Playback flow

## Project Structure

```
tts-player/
├── src/                    # React frontend
│   ├── components/         # UI components
│   │   └── __tests__/     # Component tests
│   └── test/              # Test setup and mocks
├── src-tauri/             # Rust backend
│   ├── src/               # Tauri commands
│   └── tests/             # Backend tests
├── tests/                 # Integration tests
│   ├── e2e/              # End-to-end tests
│   └── fixtures/         # Test data
└── Tools/scripts/         # Raycast integration
```

## Development Approach

This project was built using **Test-Driven Development (TDD)**:

1. **Explore**: Research modern Tauri testing frameworks and patterns
2. **Plan**: Design comprehensive test strategy with user journeys
3. **Test-First**: Write test cases before implementation
4. **Implement**: Build application to pass all tests
5. **Validate**: Run full test suite and performance benchmarks

## Configuration

### ElevenLabs Setup
```bash
export ELEVENLABS_API_KEY="your-api-key"
```

### Voice Options
- `rachel` (default)
- `adam` 
- `bella`

## Performance Requirements

- App startup: < 2 seconds
- TTS generation: < 5 seconds
- UI responsiveness: < 100ms
- Memory usage: < 100MB
- Audio latency: < 200ms

## Architecture Decisions

- **Tauri over Electron**: 10x smaller bundle size, faster startup
- **React + TypeScript**: Familiar, type-safe frontend development
- **MSW for API mocking**: Realistic network-level testing
- **Vitest over Jest**: Modern, fast testing framework
- **Tailwind CSS**: Rapid, consistent styling

## Contributing

1. **Write tests first** for any new functionality
2. **Run the full test suite** before submitting changes
3. **Follow existing patterns** for consistency
4. **Keep bundle size minimal** - justify any new dependencies

## License

Personal project - not for commercial use.