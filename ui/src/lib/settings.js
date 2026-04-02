/**
 * Persistent app settings — stored as JSON in OS app config directory.
 * Falls back to localStorage when not running in Tauri.
 */
import { invoke, isTauri } from "./tauri.js";

let cache = null;

const DEFAULTS = {
  theme: "default",
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
    } else {
      const stored = localStorage.getItem("ofs-settings");
      cache = stored ? JSON.parse(stored) : { ...DEFAULTS };
    }
  } catch {
    cache = { ...DEFAULTS };
  }
  return cache;
}

export async function saveSetting(key, value) {
  if (!cache) cache = { ...DEFAULTS };
  cache[key] = value;

  try {
    if (isTauri) {
      await invoke("save_settings", { settingsJson: JSON.stringify(cache) });
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
