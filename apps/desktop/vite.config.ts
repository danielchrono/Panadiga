import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { resolve } from "path";
import tailwindcss from "tailwindcss";
import autoprefixer from "autoprefixer";

const tailwindConfig = {
  content: [
    resolve(import.meta.dirname, "./index.html"),
    resolve(import.meta.dirname, "./src/**/*.{ts,tsx}"),
  ],
  theme: {
    extend: {
      colors: {
        bench: {
          950: "#090d16",
          900: "#0f172a",
          850: "#131d33",
          800: "#1e293b",
        },
        diga: {
          dark: "#0b0f19",
          panel: "#111827",
          border: "#1f2937",
          accent: "#2563eb",
          accentHover: "#1d4ed8",
          emerald: "#10b981",
          amber: "#f59e0b",
          rose: "#ef4444",
          cyan: "#06b6d4",
        },
      },
      fontFamily: {
        mono: ["JetBrains Mono", "Fira Code", "Courier New", "monospace"],
      },
    },
  },
  plugins: [],
};

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  resolve: {
    dedupe: ["react", "react-dom"],
    alias: {
      "@": resolve(import.meta.dirname, "./src"),
    },
  },
  optimizeDeps: {
    include: [
      "react",
      "react-dom",
      "react-i18next",
      "i18next",
      "@radix-ui/react-dialog",
      "@radix-ui/react-slot",
      "@radix-ui/react-tabs",
      "framer-motion",
      "lucide-react",
    ],
  },
  css: {
    postcss: {
      plugins: [tailwindcss(tailwindConfig), autoprefixer()],
    },
  },
  build: {
    target: "es2020",
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes("node_modules")) {
            if (id.includes("lucide-react")) {
              return "vendor-icons";
            }
            if (id.includes("@tauri-apps")) {
              return "vendor-tauri";
            }
          }
        },
      },
    },
  },
});
