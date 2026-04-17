import { writable } from "svelte/store";
import { saveSetting, getSetting } from "../lib/settings.js";

export const activeRibbonTab = writable("home");
export const activeWorkspaceView = writable("editor");
export const showAppMenu = writable(false);
export const showSettings = writable(false);
export const zoom = writable(0.35);
export const editorPan = writable({ x: 40, y: 30 });

export const THEMES = [
  "default",
  "light",
  "dark",
  "blue",
  "amber-navy",
  "warm-ember",
  "highContrast",
];

const savedTheme = getSetting("theme") || "default";
document.documentElement.setAttribute("data-theme", savedTheme);
export const theme = writable(savedTheme);

export function setTheme(id) {
  theme.set(id);
  document.documentElement.setAttribute("data-theme", id);
  saveSetting("theme", id);
}
