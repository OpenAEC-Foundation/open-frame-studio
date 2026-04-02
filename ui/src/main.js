import { loadSettings } from "./lib/settings.js";
import { initWasm } from "./lib/tauri.js";

// Load settings, init WASM (web mode), then i18n, then mount
async function boot() {
  await loadSettings();
  await initWasm();
  await import("./lib/i18n.js");
  const { mount } = await import("svelte");
  const App = (await import("./App.svelte")).default;
  mount(App, { target: document.getElementById("app") });
}

boot();
