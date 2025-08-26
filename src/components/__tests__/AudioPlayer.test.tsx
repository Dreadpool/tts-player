import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { AudioPlayer } from '../AudioPlayer';

describe('AudioPlayer Component', () => {
  let mockAudio: any;

  beforeEach(() => {
    mockAudio = {
      play: vi.fn(() => Promise.resolve()),
      pause: vi.fn(),
      load: vi.fn(),
      currentTime: 0,
      duration: 120,
      paused: true,
      volume: 1,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      src: '',
    };
    
    vi.mocked(global.Audio).mockImplementation(() => mockAudio);
  });

  it('renders audio controls', () => {
    render(<AudioPlayer />);
    
    expect(screen.getByRole('button', { name: /play/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /stop/i })).toBeInTheDocument();
    expect(screen.getByRole('slider', { name: /volume/i })).toBeInTheDocument();
  });

  it('play button starts audio playback', async () => {
    render(<AudioPlayer audioSrc="test-audio.mp3" />);
    
    const playButton = screen.getByRole('button', { name: /play/i });
    fireEvent.click(playButton);
    
    await waitFor(() => {
      expect(mockAudio.play).toHaveBeenCalled();
    });
  });

  it('pause button pauses audio', async () => {
    render(<AudioPlayer audioSrc="test-audio.mp3" />);
    
    // Start playing first
    const playButton = screen.getByRole('button', { name: /play/i });
    fireEvent.click(playButton);
    
    // Mock audio as playing
    mockAudio.paused = false;
    
    // Now pause
    const pauseButton = screen.getByRole('button', { name: /pause/i });
    fireEvent.click(pauseButton);
    
    expect(mockAudio.pause).toHaveBeenCalled();
  });

  it('stop button resets audio position', () => {
    render(<AudioPlayer audioSrc="test-audio.mp3" />);
    
    const stopButton = screen.getByRole('button', { name: /stop/i });
    fireEvent.click(stopButton);
    
    expect(mockAudio.currentTime).toBe(0);
    expect(mockAudio.pause).toHaveBeenCalled();
  });

  it('volume control updates audio volume', () => {
    render(<AudioPlayer audioSrc="test-audio.mp3" />);
    
    const volumeSlider = screen.getByRole('slider', { name: /volume/i });
    fireEvent.change(volumeSlider, { target: { value: '0.8' } });
    
    expect(mockAudio.volume).toBe(0.8);
  });

  it('seek functionality updates currentTime', () => {
    render(<AudioPlayer audioSrc="test-audio.mp3" />);
    
    // Mock duration for seek calculation
    mockAudio.duration = 120;
    
    const progressBar = screen.getByRole('slider', { name: /progress/i });
    fireEvent.change(progressBar, { target: { value: '50' } });
    
    expect(mockAudio.currentTime).toBe(60); // 50% of 120 seconds
  });

  it('shows loading state when no audio source', () => {
    render(<AudioPlayer />);
    
    expect(screen.getByRole('button', { name: /play/i })).toBeDisabled();
    expect(screen.getByText(/no audio loaded/i)).toBeInTheDocument();
  });

  it('enables controls after audio loads', async () => {
    render(<AudioPlayer />);
    
    // Initially disabled
    expect(screen.getByRole('button', { name: /play/i })).toBeDisabled();
    
    // Simulate audio loading
    render(<AudioPlayer audioSrc="test-audio.mp3" />);
    
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /play/i })).not.toBeDisabled();
    });
  });
});