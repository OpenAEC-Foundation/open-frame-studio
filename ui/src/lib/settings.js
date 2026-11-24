/**
 * Persistent app settings — stored as JSON in OS app config directory.
 * Falls back to localStorage when not running in Tauri.
 */
import { invoke, isTauri, isPlatformBuild } from "./tauri.js";
import { getOpenAecHost } from "./platformHost.js";
import { canStoreWorkspaceSetting, readWorkspaceSettings } from "./hostWorkspaceSettings.js";

let cache = null;
let connectedHost;
let loadingHost;
const pendingWrites = new Map();
const revisions = new Map();

const DEFAULTS = {
  theme: "light",
  locale: "nl",
  left_panel_width: 220,
  right_panel_width: 290,
  left_panel_open: true,
  right_panel_open: true,
};

export async function loadSettings() {
  try {
    if (isTauri) {
      const json = await invoke("load_settings");
      cache = JSON.parse(json);
    } else if (isPlatformBuild || getOpenAecHost()) {
      cache = { ...DEFAULTS };
      await connectHostSettings(getOpenAecHost());
    } else {
      const stored = localStorage.getItem("ofs-settings");
      cache = stored ? JSON.parse(stored) : { ...DEFAULTS };
    }
  } catch {
    cache = { ...DEFAULTS };
  }
  return cache;
}

export function connectHostSettings(host) {
  if (!host || host === connectedHost) return loadingHost;
  connectedHost = host;
  const originalRevisions = new Map(revisions);
  loadingHost = readWorkspaceSettings(host.settings).then((values) => {
    if (host !== connectedHost) return;
    for (const [key, value] of Object.entries(values)) {
      if (revisions.get(key) === originalRevisions.get(key)) cache[key] = value;
    }
  }).catch((error) => console.warn("Docs settings:", error));
  return loadingHost;
}

export async function saveSetting(key, value) {
  if (!cache) cache = { ...DEFAULTS };
  if (cache[key] === value) return;
  cache[key] = value;
  revisions.set(key, (revisions.get(key) || 0) + 1);

  try {
    if (isTauri) {
      await invoke("save_settings", { settingsJson: JSON.stringify(cache) });
    } else if (isPlatformBuild || getOpenAecHost()) {
      const host = getOpenAecHost();
      if (host?.settings?.set && canStoreWorkspaceSetting(key, value)) {
        const revision = revisions.get(key);
        const write = (pendingWrites.get(key) || Promise.resolve()).catch(() => {}).then(() => {
          if (revisions.get(key) === revision) return host.settings.set("frame." + key, value);
        });
        pendingWrites.set(key, write);
        try { await write; } finally { if (pendingWrites.get(key) === write) pendingWrites.delete(key); }
      }
    } else {
      localStorage.setItem("ofs-settings", JSON.stringify(cache));
    }
  } catch (e) {
    console.warn("Settings save failed:", e);
  }
}

export function getSetting(key) {
  return cache?.[key] ?? DEFAULTS[key];
}
