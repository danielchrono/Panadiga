/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        diga: {
          dark: "#0b0f19",
          panel: "#111827",
          border: "#1f2937",
          accent: "#2563eb",
          accentHover: "#1d4ed8",
          emerald: "#10b981",
          amber: "#f59e0b",
          rose: "#ef4444",
          cyan: "#06b6d4"
        }
      },
      fontFamily: {
        mono: ["JetBrains Mono", "Fira Code", "Courier New", "monospace"],
      }
    },
  },
  plugins: [],
}
