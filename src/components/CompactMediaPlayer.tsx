import { useState, useRef, useEffect } from 'react';
import { Play, Pause } from 'lucide-react';

interface CompactMediaPlayerProps {
  audioSrc: string;
  text: string;
  autoplay?: boolean;
}

export function CompactMediaPlayer({ audioSrc, text, autoplay = false }: CompactMediaPlayerProps) {
  const audioRef = useRef<HTMLAudioElement>(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);

  useEffect(() => {
    const audio = audioRef.current;
    if (!audio || !audioSrc) return;

    const handleTimeUpdate = () => setCurrentTime(audio.currentTime);
    const handleDurationChange = () => setDuration(audio.duration);
    const handleEnded = () => setIsPlaying(false);
    const handlePlay = () => setIsPlaying(true);
    const handlePause = () => setIsPlaying(false);

    audio.addEventListener('timeupdate', handleTimeUpdate);
    audio.addEventListener('durationchange', handleDurationChange);
    audio.addEventListener('ended', handleEnded);
    audio.addEventListener('play', handlePlay);
    audio.addEventListener('pause', handlePause);

    // Autoplay handling
    let hasAutoplayed = false;
    const handleCanPlay = async () => {
      if (autoplay && !hasAutoplayed) {
        hasAutoplayed = true;
        setTimeout(async () => {
          try {
            await audio.play();
            setIsPlaying(true);
          } catch (error) {
            console.error('Error autoplaying audio:', error);
            setIsPlaying(false);
          }
        }, 100);
      }
    };

    audio.addEventListener('canplaythrough', handleCanPlay);

    // Reset states when audio source changes
    setIsPlaying(false);
    setCurrentTime(0);
    setDuration(0);

    return () => {
      audio.removeEventListener('timeupdate', handleTimeUpdate);
      audio.removeEventListener('durationchange', handleDurationChange);
      audio.removeEventListener('ended', handleEnded);
      audio.removeEventListener('play', handlePlay);
      audio.removeEventListener('pause', handlePause);
      audio.removeEventListener('canplaythrough', handleCanPlay);
    };
  }, [audioSrc, autoplay]);

  const togglePlayPause = async () => {
    const audio = audioRef.current;
    if (!audio) return;

    try {
      if (isPlaying) {
        audio.pause();
      } else {
        await audio.play();
      }
    } catch (error) {
      console.error('Error toggling playback:', error);
    }
  };

  const handleSeek = (e: React.MouseEvent<HTMLDivElement>) => {
    const audio = audioRef.current;
    if (!audio || !duration) return;

    const rect = e.currentTarget.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const percentage = x / rect.width;
    const seekTime = percentage * duration;
    
    audio.currentTime = seekTime;
    setCurrentTime(seekTime);
  };

  const formatTime = (time: number) => {
    if (isNaN(time)) return '0:00';
    const minutes = Math.floor(time / 60);
    const seconds = Math.floor(time % 60);
    return `${minutes}:${seconds.toString().padStart(2, '0')}`;
  };

  const progressPercentage = duration ? (currentTime / duration) * 100 : 0;

  return (
    <div className="bg-bg-secondary rounded-3xl p-6">
      <audio ref={audioRef} src={audioSrc} preload="metadata" />
      
      {/* Compact Media Controls */}
      <div className="flex items-center gap-4">
        {/* Play/Pause Button - Always accessible */}
        <button
          onClick={togglePlayPause}
          aria-label={isPlaying ? 'Pause' : 'Play'}
          className="w-12 h-12 rounded-full bg-text-primary text-bg-primary
                    flex items-center justify-center
                    hover:bg-gray-800 transition-all duration-[250ms] ease-out
                    transform-gpu hover:scale-[1.03] active:scale-[0.97]
                    shadow-lg shadow-gray-900/20
                    focus:outline-none focus:ring-2 focus:ring-accent/50"
        >
          {isPlaying ? (
            <Pause size={18} strokeWidth={2.5} />
          ) : (
            <Play size={18} strokeWidth={2.5} fill="currentColor" />
          )}
        </button>

        {/* Progress Area - Takes remaining space */}
        <div className="flex-1 space-y-2">
          {/* Text Preview */}
          <div className="text-sm text-text-secondary font-medium leading-relaxed truncate">
            {text.length > 80 ? `${text.substring(0, 80)}...` : text}
          </div>
          
          {/* Integrated Progress Bar */}
          <div 
            className="relative h-1.5 bg-gray-200 rounded-full overflow-hidden cursor-pointer group"
            onClick={handleSeek}
          >
            <div
              className="absolute left-0 top-0 h-full bg-text-primary rounded-full transition-all duration-[150ms] ease-out"
              style={{ width: `${progressPercentage}%` }}
            />
            
            {/* Hover Effect */}
            <div className="absolute inset-0 bg-text-primary opacity-0 group-hover:opacity-8 transition-opacity duration-[250ms] ease-out" />
            
            {/* Progress Thumb - Appears on hover */}
            <div 
              className="absolute top-1/2 -translate-y-1/2 w-3 h-3 bg-text-primary rounded-full opacity-0 group-hover:opacity-100 transition-opacity duration-[250ms] ease-out shadow-sm"
              style={{ left: `calc(${progressPercentage}% - 6px)` }}
            />
          </div>
          
          {/* Time Display */}
          <div className="flex justify-between text-xs text-text-secondary font-medium tabular-nums">
            <span>{formatTime(currentTime)}</span>
            <span>{formatTime(duration)}</span>
          </div>
        </div>
      </div>
    </div>
  );
}