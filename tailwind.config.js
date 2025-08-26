/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      // Jony Ive Design System - Precise timing and easing
      animation: {
        'fade-in': 'fadeIn 250ms ease-out',
        'slide-up': 'slideUp 400ms cubic-bezier(0.25, 0.46, 0.45, 0.94)',
        'pulse-subtle': 'pulseSubtle 2s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'button-press': 'buttonPress 150ms ease-out',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        slideUp: {
          '0%': { 
            opacity: '0',
            transform: 'translateY(10px)',
          },
          '100%': { 
            opacity: '1',
            transform: 'translateY(0)',
          },
        },
        pulseSubtle: {
          '0%, 100%': { opacity: '1' },
          '50%': { opacity: '0.8' },
        },
        buttonPress: {
          '0%': { transform: 'scale(1)' },
          '50%': { transform: 'scale(0.98)' },
          '100%': { transform: 'scale(1)' },
        },
      },
      fontFamily: {
        'sans': [
          '-apple-system',
          'BlinkMacSystemFont',
          'Inter',
          'system-ui',
          'Segoe UI',
          'Roboto',
          'Helvetica Neue',
          'Arial',
          'sans-serif',
        ],
      },
      // Ive-approved spacing scale (8px grid system)
      spacing: {
        '1': '4px',   // micro spacing
        '2': '8px',   // small spacing
        '3': '12px',
        '4': '16px',  // medium spacing
        '5': '20px',
        '6': '24px',  // large spacing
        '7': '28px',
        '8': '32px',  // xl spacing
        '9': '36px',
        '10': '40px',
        '11': '44px', // minimum touch target
        '12': '48px',
        '16': '64px',
      },
      // Ive Design System Colors - Precise palette
      colors: {
        // Primary text colors
        text: {
          primary: '#000000',   // Pure black for primary text
          secondary: '#666666', // Secondary text
          tertiary: '#999999',  // Tertiary/disabled text
        },
        // Background colors
        bg: {
          primary: '#FFFFFF',   // Pure white
          secondary: '#F8F9FA', // Light gray background
        },
        // Accent color (minimal usage)
        accent: '#007AFF',      // iOS blue - only for primary actions
        // System colors
        success: '#28A745',
        warning: '#FFC107', 
        error: '#DC3545',
        // Refined gray scale
        gray: {
          50: '#FAFAFA',
          100: '#F5F5F5',
          200: '#E5E5E5',
          300: '#D4D4D4',
          400: '#A3A3A3',
          500: '#737373',
          600: '#525252',
          700: '#404040',
          800: '#262626',
          900: '#171717',
        },
      },
    },
  },
  plugins: [],
}