/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'bg-primary': '#0a0e1a',
        'bg-secondary': '#0f1726',
        'bg-tertiary': '#1a2332',
        'bg-chat': '#0d1420',
        'text-primary': '#e8eaed',
        'text-secondary': '#9ca3af',
        'text-muted': '#6b7280',
        'border': '#1f2937',
        'border-subtle': '#161e2e',
        'm3': '#f38ba8',
        'light': '#89dceb',
        'dark': '#89b4fa',
        'accent': '#cba6f7',
        'success': '#a6e3a1',
        'warning': '#f9e2af',
        'error': '#f38ba8',
        'focus': '#3b82f6',
        'hover': '#1e293b',
        'grey': '#6b7280',
        'agent-explorer': '#89dceb',
        'agent-builder': '#a6e3a1',
        'agent-analyzer': '#cba6f7',
        'agent-director': '#f9e2af',
      },
      fontFamily: {
        'mono': ['JetBrains Mono', 'Fira Code', 'Courier New', 'monospace'],
      },
      animation: {
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'breathing': 'breathing 4s ease-in-out infinite',
        'flash': 'flash 0.5s ease-in-out',
      },
      keyframes: {
        breathing: {
          '0%, 100%': { opacity: '0.7', transform: 'scale(1)' },
          '50%': { opacity: '1', transform: 'scale(1.05)' },
        },
        flash: {
          '0%, 100%': { opacity: '0.7' },
          '50%': { opacity: '1' },
        },
      },
    },
  },
  plugins: [],
}