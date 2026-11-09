import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { readFileSync } from "node:fs";

export default defineConfig(({ mode }) => ({
  plugins: [
    svelte(),
    ...(mode === "platform" ? [{
      name: "emit-platform-mcp-tools",
      apply: "build",
      generateBundle() {
        this.emitFile({
          type: "asset",
          fileName: "mcp-tools.json",
          source: readFileSync(new URL("./src/lib/mcp-tools.json", import.meta.url)),
        });
      },
    }] : []),
  ],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
}));
