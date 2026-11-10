import { defineConfig } from "vite";

// In development the UI calls /api, which is the corral API with the
// prefix dropped.
export default defineConfig({
  server: {
    proxy: {
      "/api": {
        target: "http://127.0.0.1:8080",
        rewrite: (path) => path.replace(/^\/api/, ""),
      },
    },
  },
});
