import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';

export const handlers = [
  // Mock ElevenLabs TTS API
  http.post('https://api.elevenlabs.io/v1/text-to-speech/:voiceId', async ({ request }) => {
    const apiKey = request.headers.get('xi-api-key');
    
    if (!apiKey) {
      return HttpResponse.json(
        { detail: 'Missing API key' },
        { status: 401 }
      );
    }

    // Simulate successful TTS generation
    const mockAudioBuffer = new ArrayBuffer(1024);
    return HttpResponse.arrayBuffer(mockAudioBuffer, {
      headers: {
        'Content-Type': 'audio/mpeg',
        'Content-Length': '1024'
      }
    });
  }),

  // Mock voices API
  http.get('https://api.elevenlabs.io/v1/voices', ({ request }) => {
    const apiKey = request.headers.get('xi-api-key');
    
    if (!apiKey) {
      return HttpResponse.json(
        { detail: 'Missing API key' },
        { status: 401 }
      );
    }

    return HttpResponse.json({
      voices: [
        {
          voice_id: 'rachel',
          name: 'Rachel',
          category: 'premade'
        },
        {
          voice_id: 'adam',
          name: 'Adam',
          category: 'premade'
        }
      ]
    });
  }),

  // Mock rate limiting
  http.post('https://api.elevenlabs.io/v1/text-to-speech/rate-limit-test', () => {
    return HttpResponse.json(
      { detail: 'Rate limit exceeded' },
      { 
        status: 429,
        headers: { 'Retry-After': '60' }
      }
    );
  }),
];

export const server = setupServer(...handlers);