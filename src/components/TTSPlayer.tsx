import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { CompactMediaPlayer } from './CompactMediaPlayer';
import { CharacterCounter } from './CharacterCounter';
import { UsageStatsDisplay } from './UsageStatsDisplay';

interface TTSPlayerProps {
  initialText?: string;
  initialVoice?: string;
}

export function TTSPlayer({ initialText = '', initialVoice = 'onwK4e9ZLuTAKqWW03F9' }: TTSPlayerProps) {
  const [text, setText] = useState(initialText);
  const [voice, setVoice] = useState(initialVoice);
  const [isGenerating, setIsGenerating] = useState(false);
  const [audioSrc, setAudioSrc] = useState<string>('');
  const [error, setError] = useState<string>('');
  const [showUsageStats, setShowUsageStats] = useState(false);
  const [isFocused, setIsFocused] = useState(false);
  const [shouldAutoplay, setShouldAutoplay] = useState(false);

  useEffect(() => {
    setText(initialText);
    // Automatically generate speech when text is loaded from clipboard/CLI
    if (initialText && initialText.trim()) {
      // Call the generation function directly with the initial text
      generateSpeechAuto(initialText.trim(), voice);
    }
  }, [initialText]);

  useEffect(() => {
    setVoice(initialVoice);
  }, [initialVoice]);

  // Separate function for auto-generation
  const generateSpeechAuto = async (textToSpeak: string, voiceId: string) => {
    setIsGenerating(true);
    setError('');
    setShouldAutoplay(true); // Enable autoplay for auto-generated speech

    try {
      const audioPath: string = await invoke('generate_speech', {
        text: textToSpeak,
        voiceId: voiceId,
      });
      
      setAudioSrc(audioPath);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      
      if (errorMessage.includes('401') || errorMessage.includes('API key')) {
        setError('Authentication failed. Please check your API key.');
      } else if (errorMessage.includes('429')) {
        const retryMatch = errorMessage.match(/retry.*?(\d+)/i);
        const seconds = retryMatch ? retryMatch[1] : '60';
        setError(`Rate limit reached. Try again in ${seconds} seconds.`);
      } else {
        setError(errorMessage);
      }
    } finally {
      setIsGenerating(false);
    }
  };

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
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      
      if (errorMessage.includes('401') || errorMessage.includes('API key')) {
        setError('Authentication failed. Please check your API key.');
      } else if (errorMessage.includes('429')) {
        const retryMatch = errorMessage.match(/retry.*?(\d+)/i);
        const seconds = retryMatch ? retryMatch[1] : '60';
        setError(`Rate limit reached. Try again in ${seconds} seconds.`);
      } else {
        setError(errorMessage);
      }
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
          Speak
        </h1>
      </header>

      {/* Primary Content Area - Morphs between text input and media player */}
      <div className="relative">
        {audioSrc ? (
          /* Media Player State - Replaces text input */
          <div className="animate-fade-in">
            <CompactMediaPlayer 
              audioSrc={audioSrc} 
              text={text}
              autoplay={shouldAutoplay} 
            />
          </div>
        ) : (
          /* Text Input State */
          <div className={`
            relative bg-bg-secondary rounded-3xl transition-all duration-[250ms] ease-out
            ${isFocused ? 'ring-1 ring-accent/30 bg-bg-primary shadow-xl shadow-gray-200/50' : ''}
          `}>
            {/* Floating Label - Refined typography */}
            <label 
              htmlFor="text-input" 
              className={`
                absolute left-6 transition-all duration-[250ms] ease-out pointer-events-none
                ${text || isFocused 
                  ? 'top-4 text-xs text-text-tertiary font-medium tracking-wide' 
                  : 'top-6 text-base text-text-secondary'}
              `}
            >
              What would you like me to say?
            </label>
            
            <textarea
              id="text-input"
              value={text}
              onChange={(e) => setText(e.target.value)}
              onFocus={() => setIsFocused(true)}
              onBlur={() => setIsFocused(false)}
              className={`
                w-full bg-transparent px-6 py-6 pt-10 rounded-3xl
                resize-none outline-none text-text-primary text-base
                placeholder-transparent transition-all duration-[250ms] ease-out
                leading-relaxed
                ${isFocused ? 'min-h-[10rem]' : 'min-h-[8rem]'}
              `}
              placeholder="Enter text..."
            />
            
            {/* Character Counter - Secondary info (15% priority) */}
            <div className="absolute bottom-4 right-6">
              <CharacterCounter text={text} minimal={true} />
            </div>
          </div>
        )}
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
        {audioSrc ? (
          /* Clear & Start New Button */
          <button
            onClick={() => {
              setAudioSrc('');
              setText('');
              setShouldAutoplay(false);
            }}
            className="min-h-[44px] px-8 py-3 rounded-full font-medium
                       bg-gray-100 text-text-primary hover:bg-gray-200 
                       transition-all duration-[250ms] cubic-bezier(0.25, 0.46, 0.45, 0.94)
                       transform-gpu tracking-wide hover:scale-[1.02] active:scale-[0.98]"
          >
            <span className="text-base">Clear & Start New</span>
          </button>
        ) : (
          /* Generate Speech Button */
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
        )}
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