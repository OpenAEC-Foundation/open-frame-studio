import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { readFileSync } from "node:fs";

export default defineConfig(({ mode }) => ({
  plugins: [
    svelte(),
    ...(mode === "platform" ? [{
      name: "supercloud-platform-assets",
      apply: "build",
      transformIndexHtml(html) {
        // Opaque iframe origins cannot load external SVG <use> references, even with CORS.
        const sprite = readFileSync(new URL("./public/blueprint/frame.svg", import.meta.url), "utf8");
        return html.replace("<body>", () => "<body>\n" + sprite);
      },
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
