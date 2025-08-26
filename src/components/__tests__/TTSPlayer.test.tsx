import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { TTSPlayer } from '../TTSPlayer';
import { server } from '../../test/mocks/server';
import { http, HttpResponse } from 'msw';

describe('TTSPlayer Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders text input and generate button', () => {
    render(<TTSPlayer />);
    
    expect(screen.getByRole('textbox', { name: /text to speak/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /generate speech/i })).toBeInTheDocument();
    expect(screen.getByRole('combobox', { name: /voice/i })).toBeInTheDocument();
  });

  it('shows initial text from props', () => {
    render(<TTSPlayer initialText="Hello world" />);
    
    const textInput = screen.getByRole('textbox', { name: /text to speak/i });
    expect(textInput).toHaveValue('Hello world');
  });

  it('shows loading state during generation', async () => {
    render(<TTSPlayer initialText="Hello world" />);
    
    const generateButton = screen.getByRole('button', { name: /generate speech/i });
    fireEvent.click(generateButton);
    
    expect(screen.getByText(/generating.../i)).toBeInTheDocument();
    expect(generateButton).toBeDisabled();
  });

  it('enables audio controls after successful generation', async () => {
    render(<TTSPlayer initialText="Hello world" />);
    
    const generateButton = screen.getByRole('button', { name: /generate speech/i });
    fireEvent.click(generateButton);
    
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /play/i })).not.toBeDisabled();
    });
  });

  it('handles API errors gracefully', async () => {
    // Mock API error
    server.use(
      http.post('https://api.elevenlabs.io/v1/text-to-speech/rachel', () => {
        return HttpResponse.json(
          { detail: 'API key invalid' },
          { status: 401 }
        );
      })
    );

    render(<TTSPlayer initialText="Hello world" />);
    
    const generateButton = screen.getByRole('button', { name: /generate speech/i });
    fireEvent.click(generateButton);
    
    await waitFor(() => {
      expect(screen.getByText(/error.*api key invalid/i)).toBeInTheDocument();
    });
  });

  it('voice selector updates selected voice', () => {
    render(<TTSPlayer />);
    
    const voiceSelect = screen.getByRole('combobox', { name: /voice/i });
    fireEvent.change(voiceSelect, { target: { value: 'adam' } });
    
    expect(voiceSelect).toHaveValue('adam');
  });

  it('handles empty text input', () => {
    render(<TTSPlayer />);
    
    const generateButton = screen.getByRole('button', { name: /generate speech/i });
    expect(generateButton).toBeDisabled();
  });

  it('handles special characters in text', async () => {
    const specialText = "Hello! 你好! Émojis 🎵";
    render(<TTSPlayer initialText={specialText} />);
    
    const textInput = screen.getByRole('textbox', { name: /text to speak/i });
    expect(textInput).toHaveValue(specialText);
    
    const generateButton = screen.getByRole('button', { name: /generate speech/i });
    fireEvent.click(generateButton);
    
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /play/i })).not.toBeDisabled();
    });
  });

  it('handles rate limiting errors', async () => {
    server.use(
      http.post('https://api.elevenlabs.io/v1/text-to-speech/rachel', () => {
        return HttpResponse.json(
          { detail: 'Rate limit exceeded' },
          { 
            status: 429,
            headers: { 'Retry-After': '60' }
          }
        );
      })
    );

    render(<TTSPlayer initialText="Hello world" />);
    
    const generateButton = screen.getByRole('button', { name: /generate speech/i });
    fireEvent.click(generateButton);
    
    await waitFor(() => {
      expect(screen.getByText(/rate limit.*try again in 60 seconds/i)).toBeInTheDocument();
    });
  });
});