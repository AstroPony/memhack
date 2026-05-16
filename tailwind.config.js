/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        mono: ['"JetBrains Mono"', '"Fira Code"', "monospace"],
        sans: ['"IBM Plex Sans"', "system-ui", "sans-serif"],
      },
      colors: {
        surface: {
          0: "#0a0a0f",
          1: "#12121a",
          2: "#1a1a26",
          3: "#242434",
        },
        accent: {
          DEFAULT: "#00ff88",
          dim: "#00cc6a",
          glow: "#00ff8822",
        },
        warn: "#ff6644",
        frozen: "#44aaff",
      },
    },
  },
  plugins: [],
};
