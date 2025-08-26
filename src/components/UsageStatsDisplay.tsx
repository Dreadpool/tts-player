import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Activity, BarChart3, Clock, Zap } from 'lucide-react';

interface UserInfo {
  subscription_tier: string;
  character_limit: number;
  character_used: number;
  characters_remaining: number;
  reset_date: string;
  last_updated: string;
}

interface UsageStats {
  total_requests: number;
  total_characters: number;
  successful_requests: number;
  failed_requests: number;
  most_used_voice: string;
  daily_usage: Array<{
    date: string;
    character_count: number;
    request_count: number;
  }>;
}

interface UsageStatsDisplayProps {
  isVisible: boolean;
  onClose: () => void;
}

export function UsageStatsDisplay({ isVisible, onClose }: UsageStatsDisplayProps) {
  const [userInfo, setUserInfo] = useState<UserInfo | null>(null);
  const [usageStats, setUsageStats] = useState<UsageStats | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>('');

  useEffect(() => {
    if (isVisible) {
      loadData();
    }
  }, [isVisible]);

  const loadData = async () => {
    setLoading(true);
    setError('');

    try {
      const [userInfoData, statsData] = await Promise.all([
        invoke<UserInfo>('get_user_info'),
        invoke<UsageStats>('get_usage_stats', { days: 30 })
      ]);

      setUserInfo(userInfoData);
      setUsageStats(statsData);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  const getUsagePercentage = () => {
    if (!userInfo || userInfo.character_limit === 0) return 0;
    return (userInfo.character_used / userInfo.character_limit) * 100;
  };

  const getUsageColor = () => {
    const percentage = getUsagePercentage();
    if (percentage >= 90) return 'bg-red-500';
    if (percentage >= 75) return 'bg-yellow-500';
    return 'bg-green-500';
  };

  if (!isVisible) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
      <div className="bg-white rounded-lg shadow-xl max-w-2xl w-full max-h-[80vh] overflow-y-auto">
        <div className="p-6">
          <div className="flex items-center justify-between mb-6">
            <h2 className="text-2xl font-bold text-gray-800 flex items-center">
              <BarChart3 size={24} className="mr-2" />
              Usage Statistics
            </h2>
            <button
              onClick={onClose}
              className="text-gray-500 hover:text-gray-700 text-2xl"
            >
              ×
            </button>
          </div>

          {loading && (
            <div className="text-center py-8">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto"></div>
              <p className="mt-2 text-gray-600">Loading usage data...</p>
            </div>
          )}

          {error && (
            <div className="p-4 bg-red-50 border border-red-200 rounded-md mb-6">
              <p className="text-sm text-red-800">{error}</p>
              <button
                onClick={loadData}
                className="mt-2 text-sm text-red-600 hover:text-red-800 underline"
              >
                Retry
              </button>
            </div>
          )}

          {userInfo && !loading && (
            <div className="space-y-6">
              {/* Subscription Overview */}
              <div className="bg-gray-50 rounded-lg p-4">
                <h3 className="text-lg font-semibold text-gray-800 mb-3 flex items-center">
                  <Zap size={20} className="mr-2" />
                  Subscription: {userInfo.subscription_tier.charAt(0).toUpperCase() + userInfo.subscription_tier.slice(1)}
                </h3>
                
                <div className="space-y-4">
                  <div>
                    <div className="flex justify-between text-sm text-gray-600 mb-1">
                      <span>Characters Used</span>
                      <span>{userInfo.character_used.toLocaleString()} / {userInfo.character_limit.toLocaleString()}</span>
                    </div>
                    <div className="w-full bg-gray-200 rounded-full h-3">
                      <div
                        className={`h-3 rounded-full transition-all duration-300 ${getUsageColor()}`}
                        style={{ width: `${Math.min(getUsagePercentage(), 100)}%` }}
                      ></div>
                    </div>
                    <p className="text-xs text-gray-500 mt-1">
                      {userInfo.characters_remaining.toLocaleString()} characters remaining
                    </p>
                  </div>
                </div>
              </div>

              {/* Usage Statistics */}
              {usageStats && (
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div className="bg-blue-50 rounded-lg p-4">
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="text-sm text-blue-600">Total Requests</p>
                        <p className="text-2xl font-bold text-blue-800">{usageStats.total_requests}</p>
                      </div>
                      <Activity size={32} className="text-blue-500" />
                    </div>
                  </div>

                  <div className="bg-green-50 rounded-lg p-4">
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="text-sm text-green-600">Success Rate</p>
                        <p className="text-2xl font-bold text-green-800">
                          {usageStats.total_requests > 0 
                            ? Math.round((usageStats.successful_requests / usageStats.total_requests) * 100)
                            : 0}%
                        </p>
                      </div>
                      <div className="w-8 h-8 rounded-full bg-green-500 flex items-center justify-center">
                        <span className="text-white text-sm font-bold">✓</span>
                      </div>
                    </div>
                  </div>

                  <div className="bg-purple-50 rounded-lg p-4">
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="text-sm text-purple-600">Characters Generated</p>
                        <p className="text-2xl font-bold text-purple-800">
                          {usageStats.total_characters.toLocaleString()}
                        </p>
                      </div>
                      <div className="text-purple-500 text-2xl">🔤</div>
                    </div>
                  </div>

                  <div className="bg-orange-50 rounded-lg p-4">
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="text-sm text-orange-600">Favorite Voice</p>
                        <p className="text-2xl font-bold text-orange-800 capitalize">
                          {usageStats.most_used_voice}
                        </p>
                      </div>
                      <div className="text-orange-500 text-2xl">🎤</div>
                    </div>
                  </div>
                </div>
              )}

              {/* Recent Activity */}
              {usageStats && usageStats.daily_usage.length > 0 && (
                <div>
                  <h3 className="text-lg font-semibold text-gray-800 mb-3 flex items-center">
                    <Clock size={20} className="mr-2" />
                    Recent Activity (Last 7 Days)
                  </h3>
                  <div className="space-y-2">
                    {usageStats.daily_usage.slice(0, 7).map((day, index) => (
                      <div key={day.date} className="flex items-center justify-between p-3 bg-gray-50 rounded">
                        <div className="flex items-center space-x-3">
                          <div className="w-2 h-2 bg-blue-500 rounded-full"></div>
                          <span className="text-sm text-gray-700">
                            {new Date(day.date).toLocaleDateString()}
                          </span>
                        </div>
                        <div className="text-right">
                          <div className="text-sm font-medium text-gray-800">
                            {day.character_count.toLocaleString()} chars
                          </div>
                          <div className="text-xs text-gray-500">
                            {day.request_count} requests
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* Refresh Data */}
              <div className="flex justify-center pt-4 border-t">
                <button
                  onClick={loadData}
                  disabled={loading}
                  className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 transition-colors"
                >
                  Refresh Data
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}