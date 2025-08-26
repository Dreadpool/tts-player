import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { UsageStatsDisplay } from '../UsageStatsDisplay';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

import { invoke } from '@tauri-apps/api/core';
const mockInvoke = vi.mocked(invoke);

const mockUserInfo = {
  subscription_tier: 'creator',
  character_limit: 100000,
  character_used: 35000,
  characters_remaining: 65000,
  reset_date: '2024-12-01T00:00:00Z',
  last_updated: '2024-11-15T10:00:00Z'
};

const mockUsageStats = {
  total_requests: 150,
  total_characters: 35000,
  successful_requests: 145,
  failed_requests: 5,
  most_used_voice: 'rachel',
  daily_usage: [
    { date: '2024-11-15', character_count: 2500, request_count: 10 },
    { date: '2024-11-14', character_count: 1800, request_count: 8 },
    { date: '2024-11-13', character_count: 3200, request_count: 12 },
  ]
};

describe('UsageStatsDisplay Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('does not render when not visible', () => {
    render(<UsageStatsDisplay isVisible={false} onClose={() => {}} />);
    
    expect(screen.queryByText('Usage Statistics')).not.toBeInTheDocument();
  });

  it('renders loading state initially', () => {
    mockInvoke.mockImplementation(() => new Promise(() => {})); // Never resolves
    
    render(<UsageStatsDisplay isVisible={true} onClose={() => {}} />);
    
    expect(screen.getByText('Loading usage data...')).toBeInTheDocument();
    expect(screen.getByRole('progressbar', { hidden: true })).toBeInTheDocument();
  });

  it('displays user subscription information', async () => {
    mockInvoke
      .mockResolvedValueOnce(mockUserInfo)
      .mockResolvedValueOnce(mockUsageStats);

    render(<UsageStatsDisplay isVisible={true} onClose={() => {}} />);
    
    await waitFor(() => {
      expect(screen.getByText('Subscription: Creator')).toBeInTheDocument();
      expect(screen.getByText('35,000 / 100,000')).toBeInTheDocument();
      expect(screen.getByText('65,000 characters remaining')).toBeInTheDocument();
    });
  });

  it('displays usage statistics correctly', async () => {
    mockInvoke
      .mockResolvedValueOnce(mockUserInfo)
      .mockResolvedValueOnce(mockUsageStats);

    render(<UsageStatsDisplay isVisible={true} onClose={() => {}} />);
    
    await waitFor(() => {
      expect(screen.getByText('150')).toBeInTheDocument(); // Total requests
      expect(screen.getByText('97%')).toBeInTheDocument(); // Success rate (145/150)
      expect(screen.getByText('35,000')).toBeInTheDocument(); // Total characters
      expect(screen.getByText('Rachel')).toBeInTheDocument(); // Most used voice
    });
  });

  it('displays daily usage history', async () => {
    mockInvoke
      .mockResolvedValueOnce(mockUserInfo)
      .mockResolvedValueOnce(mockUsageStats);

    render(<UsageStatsDisplay isVisible={true} onClose={() => {}} />);
    
    await waitFor(() => {
      expect(screen.getByText('Recent Activity (Last 7 Days)')).toBeInTheDocument();
      expect(screen.getByText('2,500 chars')).toBeInTheDocument();
      expect(screen.getByText('10 requests')).toBeInTheDocument();
    });
  });

  it('shows correct usage progress bar color', async () => {
    // Test high usage (90%+)
    const highUsageUserInfo = {
      ...mockUserInfo,
      character_used: 95000,
      characters_remaining: 5000
    };
    
    mockInvoke
      .mockResolvedValueOnce(highUsageUserInfo)
      .mockResolvedValueOnce(mockUsageStats);

    render(<UsageStatsDisplay isVisible={true} onClose={() => {}} />);
    
    await waitFor(() => {
      expect(document.querySelector('.bg-red-500')).toBeInTheDocument();
    });
  });

  it('handles API errors gracefully', async () => {
    mockInvoke.mockRejectedValueOnce(new Error('API key invalid'));

    render(<UsageStatsDisplay isVisible={true} onClose={() => {}} />);
    
    await waitFor(() => {
      expect(screen.getByText('API key invalid')).toBeInTheDocument();
      expect(screen.getByText('Retry')).toBeInTheDocument();
    });
  });

  it('allows retrying after error', async () => {
    mockInvoke
      .mockRejectedValueOnce(new Error('Network error'))
      .mockResolvedValueOnce(mockUserInfo)
      .mockResolvedValueOnce(mockUsageStats);

    render(<UsageStatsDisplay isVisible={true} onClose={() => {}} />);
    
    await waitFor(() => {
      expect(screen.getByText('Retry')).toBeInTheDocument();
    });
    
    fireEvent.click(screen.getByText('Retry'));
    
    await waitFor(() => {
      expect(screen.getByText('Subscription: Creator')).toBeInTheDocument();
    });
  });

  it('closes modal when close button is clicked', () => {
    const mockOnClose = vi.fn();
    
    mockInvoke
      .mockResolvedValueOnce(mockUserInfo)
      .mockResolvedValueOnce(mockUsageStats);

    render(<UsageStatsDisplay isVisible={true} onClose={mockOnClose} />);
    
    const closeButton = screen.getByText('×');
    fireEvent.click(closeButton);
    
    expect(mockOnClose).toHaveBeenCalled();
  });

  it('refreshes data when refresh button is clicked', async () => {
    mockInvoke
      .mockResolvedValueOnce(mockUserInfo)
      .mockResolvedValueOnce(mockUsageStats)
      .mockResolvedValueOnce(mockUserInfo)
      .mockResolvedValueOnce(mockUsageStats);

    render(<UsageStatsDisplay isVisible={true} onClose={() => {}} />);
    
    await waitFor(() => {
      expect(screen.getByText('Refresh Data')).toBeInTheDocument();
    });
    
    fireEvent.click(screen.getByText('Refresh Data'));
    
    expect(mockInvoke).toHaveBeenCalledTimes(4); // 2 initial + 2 refresh calls
  });

  it('calculates success rate correctly with zero requests', async () => {
    const noRequestsStats = {
      ...mockUsageStats,
      total_requests: 0,
      successful_requests: 0,
      failed_requests: 0
    };
    
    mockInvoke
      .mockResolvedValueOnce(mockUserInfo)
      .mockResolvedValueOnce(noRequestsStats);

    render(<UsageStatsDisplay isVisible={true} onClose={() => {}} />);
    
    await waitFor(() => {
      expect(screen.getByText('0%')).toBeInTheDocument(); // Success rate with 0 requests
    });
  });

  it('formats large numbers with commas', async () => {
    const largeUsageUserInfo = {
      ...mockUserInfo,
      character_limit: 1000000,
      character_used: 350000,
      characters_remaining: 650000
    };
    
    const largeUsageStats = {
      ...mockUsageStats,
      total_characters: 350000
    };
    
    mockInvoke
      .mockResolvedValueOnce(largeUsageUserInfo)
      .mockResolvedValueOnce(largeUsageStats);

    render(<UsageStatsDisplay isVisible={true} onClose={() => {}} />);
    
    await waitFor(() => {
      expect(screen.getByText('350,000 / 1,000,000')).toBeInTheDocument();
      expect(screen.getByText('650,000 characters remaining')).toBeInTheDocument();
    });
  });
});