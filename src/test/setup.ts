import '@testing-library/jest-dom';
import { vi } from 'vitest';
import { server } from './mocks/server';

// Create mock audio methods
const mockPlay = vi.fn(() => Promise.resolve());
const mockPause = vi.fn();
const mockLoad = vi.fn();
const mockAddEventListener = vi.fn();
const mockRemoveEventListener = vi.fn();

// Mock HTML5 Audio globally with proper implementation
const mockAudioElement = {
  play: mockPlay,
  pause: mockPause,
  load: mockLoad,
  addEventListener: mockAddEventListener,
  removeEventListener: mockRemoveEventListener,
  currentTime: 0,
  duration: NaN,
  paused: true,
  volume: 1,
  muted: false,
  readyState: 0,
  src: '',
  crossOrigin: null,
};

const mockAudio = vi.fn(() => mockAudioElement);

vi.stubGlobal('HTMLAudioElement', mockAudio);
vi.stubGlobal('Audio', mockAudio);

// Mock HTMLMediaElement methods for jsdom
Object.defineProperty(window.HTMLMediaElement.prototype, 'play', {
  writable: true,
  value: mockPlay,
});

Object.defineProperty(window.HTMLMediaElement.prototype, 'pause', {
  writable: true,
  value: mockPause,
});

Object.defineProperty(window.HTMLMediaElement.prototype, 'load', {
  writable: true,
  value: mockLoad,
});

// Mock AudioContext for Web Audio API
const mockAudioContext = vi.fn(() => ({
  createOscillator: vi.fn(),
  createGain: vi.fn(),
  destination: {},
  currentTime: 0,
  sampleRate: 44100,
  state: 'running',
  close: vi.fn(() => Promise.resolve()),
  resume: vi.fn(() => Promise.resolve()),
  suspend: vi.fn(() => Promise.resolve()),
}));

vi.stubGlobal('AudioContext', mockAudioContext);
vi.stubGlobal('webkitAudioContext', mockAudioContext);

// Setup MSW
beforeAll(() => server.listen({ onUnhandledRequest: 'error' }));
afterEach(() => server.resetHandlers());
afterAll(() => server.close());