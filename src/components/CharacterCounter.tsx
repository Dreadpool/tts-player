import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface CharacterCounterProps {
  text: string;
  maxLength?: number;
  showWarnings?: boolean;
  minimal?: boolean;
}

interface UserInfo {
  subscription_tier: string;
  character_limit: number;
  character_used: number;
  characters_remaining: number;
  reset_date: string;
  last_updated: string;
}

export function CharacterCounter({ 
  text, 
  maxLength = 5000, 
  showWarnings = true,
  minimal = false 
}: CharacterCounterProps) {
  const [userInfo, setUserInfo] = useState<UserInfo | null>(null);

  useEffect(() => {
    loadUserInfo();
  }, []);

  const loadUserInfo = async () => {
    try {
      const info = await invoke<UserInfo>('get_user_info');
      setUserInfo(info);
    } catch (error) {
      console.warn('Failed to load user info:', error);
    }
  };

  const getCountColor = () => {
    const percentage = (text.length / maxLength) * 100;
    if (percentage >= 90) return 'text-error';
    if (percentage >= 75) return 'text-warning';
    return 'text-text-tertiary';
  };

  const willExceedQuota = () => {
    return userInfo && text.length > userInfo.characters_remaining;
  };

  // Minimal version for inline display - Ive design system
  if (minimal) {
    return (
      <div className="flex items-center gap-2">
        <span className={`text-xs font-medium tabular-nums ${getCountColor()}`}>
          {text.length.toLocaleString()}
        </span>
        {userInfo && (
          <>
            <span className="text-xs text-gray-300 font-light">/</span>
            <span className="text-xs text-text-tertiary font-medium tabular-nums">
              {userInfo.characters_remaining.toLocaleString()}
            </span>
          </>
        )}
        {willExceedQuota() && (
          <span className="text-xs text-error font-medium">!</span>
        )}
      </div>
    );
  }

  // Full version - Ive design system
  return (
    <div className="space-y-4">
      {/* Character Count Bar */}
      <div className="space-y-2">
        <div className="flex justify-between items-center">
          <span className={`text-sm font-medium tabular-nums ${getCountColor()}`}>
            {text.length.toLocaleString()} characters
          </span>
          {userInfo && (
            <span className="text-xs text-text-secondary font-medium tabular-nums">
              {userInfo.characters_remaining.toLocaleString()} remaining
            </span>
          )}
        </div>
        
        {/* Progress Bar - Refined design */}
        <div className="relative h-1.5 bg-gray-200 rounded-full overflow-hidden">
          <div
            className={`
              absolute left-0 top-0 h-full rounded-full transition-all duration-[250ms] ease-out
              ${text.length > maxLength 
                ? 'bg-error' 
                : text.length / maxLength > 0.9 
                  ? 'bg-warning' 
                  : text.length / maxLength > 0.75
                    ? 'bg-warning'
                    : 'bg-text-primary'}
            `}
            style={{ width: `${Math.min((text.length / maxLength) * 100, 100)}%` }}
          />
        </div>
      </div>

      {/* Warnings - System feedback */}
      {showWarnings && willExceedQuota() && (
        <div className="bg-error/10 text-error text-xs px-4 py-3 rounded-2xl font-medium">
          This will exceed your monthly quota
        </div>
      )}

      {showWarnings && text.length > maxLength && (
        <div className="bg-warning/10 text-warning text-xs px-4 py-3 rounded-2xl font-medium">
          Text exceeds {maxLength.toLocaleString()} character limit
        </div>
      )}
    </div>
  );
}