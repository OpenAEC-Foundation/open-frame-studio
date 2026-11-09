import { loadSettings } from "./lib/settings.js";
import { initWasm, isPlatformBuild } from "./lib/tauri.js";

let bootReady;
let siteToolsPromise;
function loadSiteTools() {
  return siteToolsPromise ??= import("./lib/siteAppTools.js");
}

if (isPlatformBuild) {
  // Register before the UI starts, but load the MCP layer only when the host
  // asks for a tool. run() waits for the WASM-backed app to finish booting.
  window.__OPDS_APP__ = {
    tools: async () => (await loadSiteTools()).tools(),
    definitions: async () => (await loadSiteTools()).definitions(),
    run: async (name, args) => {
      await bootReady;
      return (await loadSiteTools()).run(name, args);
    },
  };
}

// Load settings, init WASM (web mode), then i18n, then mount
async function boot() {
  await loadSettings();
  await initWasm();
  await import("./lib/i18n.js");
  const { mount } = await import("svelte");
  const App = (await import("./App.svelte")).default;
  mount(App, { target: document.getElementById("app") });
}

bootReady = boot();
