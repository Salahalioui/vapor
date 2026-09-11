/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{svelte,js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        vapor: {
          50: '#f8fafc',
          100: '#f1f5f9',
          800: '#1e293b',
          900: '#0f172a',
          950: '#090d16',
          accent: '#06b6d4',
          accentHover: '#0891b2',
          danger: '#ef4444',
          warning: '#f59e0b',
          success: '#10b981',
          card: '#131b2e',
          cardBorder: '#1e293b'
        }
      },
      fontFamily: {
        sans: ['Segoe UI Variable', 'Segoe UI', 'Inter', '-apple-system', 'BlinkMacSystemFont', 'sans-serif'],
        mono: ['Cascadia Code', 'Consolas', 'monospace']
      }
    },
  },
  plugins: [],
}
