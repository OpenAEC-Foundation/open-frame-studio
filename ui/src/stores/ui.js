import { writable } from "svelte/store";
import { saveSetting, getSetting } from "../lib/settings.js";

export const activeRibbonTab = writable("home");
export const activeWorkspaceView = writable("editor");
export const showAppMenu = writable(false);
export const showSettings = writable(false);
export const showFeedback = writable(false);
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

function applyTheme(id) {
  const blueprintTheme = {
    light: "light",
    blue: "blueprint",
    highContrast: "high-contrast-dark",
  }[id] || "dark";
  document.documentElement.dataset.theme = id;
  document.documentElement.dataset.oaTheme = blueprintTheme;
}

const savedTheme = getSetting("theme") || "light";
applyTheme(savedTheme);
export const theme = writable(savedTheme);

export function setTheme(id) {
  theme.set(id);
  applyTheme(id);
  saveSetting("theme", id);
}
