/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        background: "#121212",
        surface: "#1e1e1e",
        "surface-2": "#252525",
        "surface-3": "#2e2e2e",
        border: "#333333",
        accent: "#e53935",
        "accent-hover": "#ef5350",
        "accent-muted": "#b71c1c",
        text: "#e0e0e0",
        "text-muted": "#9e9e9e",
        "text-dim": "#616161",
        success: "#4caf50",
        warning: "#ff9800",
        error: "#f44336",
        info: "#2196f3",
      },
      fontFamily: {
        sans: ["Inter", "system-ui", "sans-serif"],
        mono: ["JetBrains Mono", "Fira Code", "monospace"],
      },
    },
  },
  plugins: [],
};
