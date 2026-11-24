import { loadSettings } from "./lib/settings.js";
import { initWasm, invoke, isPlatformBuild, isTauri } from "./lib/tauri.js";

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
  if (isTauri) {
    const container = await invoke("get_container_startup");
    if (container) {
      const { setDialogWorkspace } = await import("./lib/desktopDialogs.js");
      setDialogWorkspace(container.workspace);
      if (container.project) {
        try {
          const { openProject } = await import("./stores/project.js");
          await openProject(container.project);
        } catch (error) {
          const { toast } = await import("./stores/toast.js");
          toast.error(String(error));
        }
      }
    }
  }
  const { mount } = await import("svelte");
  const App = (await import("./App.svelte")).default;
  mount(App, { target: document.getElementById("app") });
}

bootReady = boot();
