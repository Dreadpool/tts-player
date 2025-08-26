import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { CharacterCounter } from '../CharacterCounter';
import { server } from '../../test/mocks/server';
import { http, HttpResponse } from 'msw';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

import { invoke } from '@tauri-apps/api/core';
const mockInvoke = vi.mocked(invoke);

describe('CharacterCounter Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('displays character count', () => {
    render(<CharacterCounter text="Hello world" />);
    
    expect(screen.getByText(/11 \/ 5,000/)).toBeInTheDocument();
  });

  it('shows warning when near limit', () => {
    const longText = 'a'.repeat(4600); // 92% of 5000
    render(<CharacterCounter text={longText} showWarnings={true} />);
    
    expect(screen.getByText('⚠️ Near limit')).toBeInTheDocument();
  });

  it('shows error when exceeding limit', () => {
    const tooLongText = 'a'.repeat(5001);
    render(<CharacterCounter text={tooLongText} showWarnings={true} />);
    
    expect(screen.getByText(/Text exceeds maximum length/)).toBeInTheDocument();
  });

  it('displays user quota information when available', async () => {
    mockInvoke.mockResolvedValueOnce({
      subscription_tier: 'creator',
      character_limit: 100000,
      character_used: 25000,
      characters_remaining: 75000,
      reset_date: '2024-12-01T00:00:00Z',
      last_updated: '2024-11-15T10:00:00Z'
    });

    render(<CharacterCounter text="Hello world" />);
    
    await waitFor(() => {
      expect(screen.getByText('Creator')).toBeInTheDocument();
      expect(screen.getByText(/75,000 remaining this month/)).toBeInTheDocument();
    });
  });

  it('warns when request would exceed quota', async () => {
    mockInvoke.mockResolvedValueOnce({
      subscription_tier: 'starter',
      character_limit: 10000,
      character_used: 9800,
      characters_remaining: 200,
      reset_date: '2024-12-01T00:00:00Z',
      last_updated: '2024-11-15T10:00:00Z'
    });

    const text = 'a'.repeat(500); // Exceeds remaining quota
    render(<CharacterCounter text={text} showWarnings={true} />);
    
    await waitFor(() => {
      expect(screen.getByText(/would exceed your monthly quota/)).toBeInTheDocument();
    });
  });

  it('shows estimated cost for paid tiers', async () => {
    mockInvoke.mockResolvedValueOnce({
      subscription_tier: 'creator',
      character_limit: 100000,
      character_used: 25000,
      characters_remaining: 75000,
      reset_date: '2024-12-01T00:00:00Z',
      last_updated: '2024-11-15T10:00:00Z'
    });

    const text = 'a'.repeat(1000); // 1000 characters
    render(<CharacterCounter text={text} />);
    
    await waitFor(() => {
      expect(screen.getByText(/\$0\.0300 estimated cost/)).toBeInTheDocument();
    });
  });

  it('handles failed user info fetch gracefully', async () => {
    mockInvoke.mockRejectedValueOnce(new Error('API key invalid'));

    render(<CharacterCounter text="Hello world" />);
    
    // Should still show character count even if user info fails
    expect(screen.getByText(/11 \/ 5,000/)).toBeInTheDocument();
    
    // Should not show quota information
    await waitFor(() => {
      expect(screen.queryByText(/remaining this month/)).not.toBeInTheDocument();
    });
  });

  it('updates progress bar color based on usage', () => {
    const { rerender } = render(<CharacterCounter text="Hello" />);
    
    // Should show blue for normal usage
    let progressBar = document.querySelector('.bg-blue-500');
    expect(progressBar).toBeInTheDocument();
    
    // Should show yellow for high usage
    rerender(<CharacterCounter text={'a'.repeat(4500)} />);
    progressBar = document.querySelector('.bg-yellow-500');
    expect(progressBar).toBeInTheDocument();
    
    // Should show red for over limit
    rerender(<CharacterCounter text={'a'.repeat(5001)} />);
    progressBar = document.querySelector('.bg-red-500');
    expect(progressBar).toBeInTheDocument();
  });
});