import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getMatches } from '@tauri-apps/plugin-cli';
import { readText } from '@tauri-apps/plugin-clipboard-manager';
import { TTSPlayer } from './components/TTSPlayer';

interface CliMatches {
  args: {
    text?: { value: string };
    voice?: { value: string };
    file?: { value: string };
  };
}

function App() {
  const [initialText, setInitialText] = useState<string>('');
  const [initialVoice, setInitialVoice] = useState<string>('onwK4e9ZLuTAKqWW03F9');

  useEffect(() => {
    const loadInitialText = async () => {
      try {
        // First try CLI arguments
        const matches = await getMatches() as CliMatches;
        console.log('CLI matches:', matches);
        
        if (matches?.args?.text?.value) {
          console.log('Raw text arg:', matches.args.text.value);
          // Decode URL-encoded text
          const decodedText = decodeURIComponent(matches.args.text.value);
          console.log('Decoded text:', decodedText);
          setInitialText(decodedText);
          
          if (matches?.args?.voice?.value) {
            setInitialVoice(matches.args.voice.value);
          }
          return; // CLI args worked, don't try other methods
        }
        
        // Try file argument
        if (matches?.args?.file?.value) {
          console.log('File argument found:', matches.args.file.value);
          try {
            const fileContent: string = await invoke('read_text_file', { filePath: matches.args.file.value });
            console.log('File content loaded:', fileContent);
            setInitialText(fileContent);
            
            if (matches?.args?.voice?.value) {
              setInitialVoice(matches.args.voice.value);
            }
            return; // File loaded successfully
          } catch (fileError) {
            console.error('Error reading file:', fileError);
            // Continue to clipboard fallback
          }
        }
      } catch (error) {
        console.error('Error loading CLI args:', error);
      }
      
      // If CLI args didn't work or weren't provided, try clipboard
      try {
        console.log('CLI args not found, trying clipboard...');
        const clipboardText = await readText();
        console.log('Clipboard content:', clipboardText);
        
        if (clipboardText && clipboardText.trim()) {
          console.log('Using clipboard text:', clipboardText);
          setInitialText(clipboardText.trim());
        } else {
          console.log('Clipboard was empty or could not be read');
        }
      } catch (clipboardError) {
        console.error('Error reading clipboard:', clipboardError);
        // This is fine - app can still be used manually
      }
    };

    loadInitialText();
  }, []);

  return (
    <div className="min-h-screen bg-white">
      <div className="max-w-2xl mx-auto px-6 py-12">
        <TTSPlayer 
          initialText={initialText}
          initialVoice={initialVoice}
        />
      </div>
    </div>
  );
}

export default App;