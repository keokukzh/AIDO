import { defineConfig } from "vite";

export default defineConfig({
  clearScreen: false,
  build: {
    rollupOptions: {
      input: { preview: "index.html", companion: "companion.html", chat: "chat.html", setup: "setup.html", tools: "tools.html" }
    }
  },
  server: {
    port: 1420,
    strictPort: true,
    host: "127.0.0.1"
  }
});
