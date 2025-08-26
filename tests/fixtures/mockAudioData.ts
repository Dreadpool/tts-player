// Mock audio data for testing
export const mockAudioResponses = {
  shortPhrase: new ArrayBuffer(512),
  mediumText: new ArrayBuffer(1024),
  longText: new ArrayBuffer(4096),
  emptyResponse: new ArrayBuffer(0),
};

// Base64 minimal MP3 data for HTML5 audio element testing
export const testAudioDataUrl = 'data:audio/mpeg;base64,SUQzBAAAAAAAI1RTU0UAAAAPAAADAE1QM1JPQU1JTkcgU1RSRUE=';

// Mock voice configurations
export const mockVoices = [
  { voice_id: 'rachel', name: 'Rachel', category: 'premade' },
  { voice_id: 'adam', name: 'Adam', category: 'premade' },
  { voice_id: 'bella', name: 'Bella', category: 'premade' },
];