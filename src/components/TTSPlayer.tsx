import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { CompactMediaPlayer } from './CompactMediaPlayer';
import { CharacterCounter } from './CharacterCounter';
import { UsageStatsDisplay } from './UsageStatsDisplay';

interface TTSError {
  type: 'auth' | 'rate_limit' | 'network' | 'unknown';
  message: string;
  retryAfter?: number;
}

function parseError(error: unknown): TTSError {
  const errorMessage = error instanceof Error ? error.message : String(error);
  
  if (errorMessage.includes('401') || errorMessage.includes('API key')) {
    return {
      type: 'auth',
      message: 'Authentication failed. Please check your API key.'
    };
  }
  
  if (errorMessage.includes('429') || errorMessage.includes('rate limit')) {
    const retryMatch = errorMessage.match(/retry.*?(\d+)/i);
    const seconds = retryMatch ? parseInt(retryMatch[1], 10) : 60;
    return {
      type: 'rate_limit',
      message: `Rate limit reached. Try again in ${seconds} seconds.`,
      retryAfter: seconds
    };
  }
  
  if (errorMessage.includes('network') || errorMessage.includes('fetch')) {
    return {
      type: 'network',
      message: 'Network error. Please check your internet connection.'
    };
  }
  
  return {
    type: 'unknown',
    message: errorMessage
  };
}

interface TTSPlayerProps {
  initialText?: string;
  initialVoice?: string;
}

export function TTSPlayer({ initialText = '', initialVoice = 'nova' }: TTSPlayerProps) {
  const [text, setText] = useState(initialText);
  const [voice, setVoice] = useState(initialVoice);
  const [isGenerating, setIsGenerating] = useState(false);
  const [audioSrc, setAudioSrc] = useState<string>('');
  const [error, setError] = useState<string>('');
  const [showUsageStats, setShowUsageStats] = useState(false);
  const [isFocused, setIsFocused] = useState(false);
  const [shouldAutoplay, setShouldAutoplay] = useState(false);
  const [showVoiceSelector, setShowVoiceSelector] = useState(false);

  const availableVoices = [
    { id: 'nova', name: 'Nova', description: 'Natural female voice' },
    { id: 'shimmer', name: 'Shimmer', description: 'Expressive female' },
    { id: 'alloy', name: 'Alloy', description: 'Neutral, versatile' },
    { id: 'echo', name: 'Echo', description: 'Male voice' },
    { id: 'fable', name: 'Fable', description: 'British accent' },
    { id: 'onyx', name: 'Onyx', description: 'Deep male voice' },
  ];

  const currentVoiceName = availableVoices.find(v => v.id === voice)?.name || 'Nova';

  useEffect(() => {
    setText(initialText);
    // Automatically generate speech when text is loaded from clipboard/CLI
    if (initialText && initialText.trim()) {
      // Call the generation function directly with the initial text
      generateSpeechAuto(initialText.trim(), voice);
    }
  }, [initialText, voice]);

  useEffect(() => {
    setVoice(initialVoice);
  }, [initialVoice]);

  // Separate function for auto-generation
  const generateSpeechAuto = useCallback(async (textToSpeak: string, voiceId: string) => {
    setIsGenerating(true);
    setError('');
    setShouldAutoplay(true); // Enable autoplay for auto-generated speech

    try {
      const audioPath: string = await invoke('generate_speech', {
        text: textToSpeak,
        voiceId: voiceId,
      });
      
      setAudioSrc(audioPath);
      // Keep text for manual editing/regeneration instead of nuclear clear
    } catch (err) {
      const parsedError = parseError(err);
      setError(parsedError.message);
    } finally {
      setIsGenerating(false);
    }
  }, []);

  const handleGenerate = async () => {
    if (!text.trim()) return;

    setIsGenerating(true);
    setError('');
    setShouldAutoplay(false); // Don't autoplay for manual generation

    try {
      const audioPath: string = await invoke('generate_speech', {
        text: text.trim(),
        voiceId: voice,
      });
      
      setAudioSrc(audioPath);
      // Keep text for manual editing/regeneration instead of nuclear clear
    } catch (err) {
      const parsedError = parseError(err);
      setError(parsedError.message);
    } finally {
      setIsGenerating(false);
    }
  };

  const isGenerateDisabled = !text.trim() || isGenerating;

  return (
    <div className="space-y-8">
      {/* Loading Overlay - Refined with Ive principles */}
      {isGenerating && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-bg-primary/95 backdrop-blur-sm animate-fade-in">
          <div className="text-center space-y-6">
            <div className="relative">
              {/* Outer ring - Precise sizing */}
              <div className="w-16 h-16 rounded-full border-2 border-gray-200"></div>
              {/* Spinning ring - Refined timing */}
              <div className="absolute inset-0 w-16 h-16 rounded-full border-2 border-text-primary border-t-transparent animate-spin"></div>
            </div>
            <p className="text-text-secondary font-light text-sm tracking-wide">Generating speech...</p>
          </div>
        </div>
      )}

      {/* Header - Essential content (80% visual priority) */}
      <header className="text-center mb-8">
        <h1 className="text-3xl font-light text-text-primary tracking-tight leading-tight">
          read to me
        </h1>
      </header>

      {/* Audio Player - Always at top when available */}
      {audioSrc && (
        <div className="mb-6 animate-fade-in">
          <CompactMediaPlayer 
            audioSrc={audioSrc} 
            autoplay={shouldAutoplay} 
          />
        </div>
      )}

      {/* Text Input - Always visible */}
      <div className="space-y-4">
        {/* Conditional label - only show when empty */}
        {!text && (
          <label htmlFor="text-input" className="text-sm text-text-secondary font-medium transition-opacity duration-200">
            What would you like me to say?
          </label>
        )}
        
        {/* Text Input Container - Clean scrollable area */}
        <div className={`
          bg-bg-secondary rounded-3xl transition-all duration-[250ms] ease-out overflow-hidden
          ${isFocused ? 'ring-1 ring-accent/30 bg-bg-primary shadow-xl shadow-gray-200/50' : ''}
        `}>
          <div className="max-h-48 overflow-y-auto overflow-x-hidden">
            <textarea
              id="text-input"
              value={text}
              onChange={(e) => setText(e.target.value)}
              onFocus={() => setIsFocused(true)}
              onBlur={() => setIsFocused(false)}
              className={`
                w-full bg-transparent px-6 py-6 pb-8 rounded-t-3xl
                resize-none outline-none text-text-primary text-base
                transition-all duration-[250ms] ease-out
                leading-relaxed min-h-[8rem]
              `}
              placeholder="Enter your text here..."
            />
          </div>
        </div>
        
        {/* Voice Selector - Completely separate */}
        <div className="bg-white rounded-2xl shadow-sm border border-gray-200 px-6 py-4">
          <div className="flex items-center justify-between">
            <div className="relative">
              <button
                onClick={() => setShowVoiceSelector(!showVoiceSelector)}
                className="flex items-center gap-2 px-3 py-1.5 text-sm text-text-tertiary hover:text-text-secondary transition-colors rounded-lg hover:bg-gray-50"
              >
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z" />
                </svg>
                <span>{currentVoiceName}</span>
                <svg className={`w-3 h-3 transition-transform ${showVoiceSelector ? 'rotate-180' : ''}`} fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                </svg>
              </button>
              
              {showVoiceSelector && (
                <div className="absolute bottom-full left-0 mb-2 bg-white rounded-xl shadow-xl border border-gray-200 overflow-hidden z-10 min-w-48">
                  {availableVoices.map((voiceOption) => (
                    <button
                      key={voiceOption.id}
                      onClick={() => {
                        setVoice(voiceOption.id);
                        setShowVoiceSelector(false);
                      }}
                      className={`w-full px-4 py-3 text-left hover:bg-gray-50 transition-colors ${
                        voice === voiceOption.id ? 'bg-accent/10 text-accent' : 'text-text-primary'
                      }`}
                    >
                      <div className="font-medium text-sm">{voiceOption.name}</div>
                      <div className="text-xs text-text-tertiary mt-0.5">{voiceOption.description}</div>
                    </button>
                  ))}
                </div>
              )}
            </div>
            
            <CharacterCounter text={text} minimal={true} />
          </div>
        </div>
      </div>

      {/* Error Message - System feedback */}
      {error && (
        <div className="animate-fade-in">
          <div className="bg-error/10 text-error px-6 py-4 rounded-2xl text-sm font-medium leading-relaxed">
            {error}
          </div>
        </div>
      )}

      {/* Primary Action Button - Always in same location (Essential interaction) */}
      <div className="flex justify-center pt-2">
        {/* Generate Speech Button - Always available when text exists */}
          <button
            onClick={handleGenerate}
            disabled={isGenerateDisabled}
            className={`
              min-h-[44px] px-8 py-3 rounded-full font-medium
              transition-all duration-[250ms] cubic-bezier(0.25, 0.46, 0.45, 0.94)
              transform-gpu tracking-wide
              ${isGenerateDisabled
                ? 'bg-gray-200 text-text-tertiary cursor-not-allowed'
                : 'bg-text-primary text-bg-primary hover:bg-gray-800 hover:scale-[1.02] active:scale-[0.98] shadow-lg shadow-gray-900/20'}
            `}
          >
            <span className="flex items-center justify-center gap-3">
              {isGenerating ? (
                <>
                  <svg className="animate-spin h-4 w-4" viewBox="0 0 24 24">
                    <circle className="opacity-30" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="3" fill="none" />
                    <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                  </svg>
                  <span className="text-sm">Generating...</span>
                </>
              ) : (
                <span className="text-base">Generate Speech</span>
              )}
            </span>
          </button>
      </div>

      {/* Usage Stats - Subtle indicator */}
      <button
        onClick={() => setShowUsageStats(true)}
        className="fixed top-6 right-6 p-2 rounded-full hover:bg-gray-100 transition-colors"
        title="View Usage"
      >
        <svg className="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
        </svg>
      </button>

      {/* Usage Statistics Modal */}
      <UsageStatsDisplay 
        isVisible={showUsageStats}
        onClose={() => setShowUsageStats(false)}
      />
    </div>
  );
}