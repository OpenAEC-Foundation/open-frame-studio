import { isPlatformBuild, isTauri } from "./tauri.js";
import { confirmUnsavedChanges } from "./project-actions.js";

export const repositoryUrl = "https://github.com/OpenAEC-Foundation/Open-Frame-Studio";

export async function openProjectLink(path = "") {
  if (isPlatformBuild) return;
  const url = repositoryUrl + path;
  if (isTauri) {
    try {
      const { open } = await import("@tauri-apps/plugin-shell");
      await open(url);
      return;
    } catch {
      // Fall back to the browser if the native URL handler is unavailable.
    }
  }
  window.open(url, "_blank", "noopener,noreferrer");
}

export async function closeApplication() {
  if (!isTauri || !(await confirmUnsavedChanges())) return;
  try {
    const { exit } = await import("@tauri-apps/plugin-process");
    await exit(0);
  } catch {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    const appWindow = getCurrentWindow();
    try {
      await appWindow.destroy();
    } catch {
      try { await appWindow.close(); } catch { window.close(); }
    }
  }
}
