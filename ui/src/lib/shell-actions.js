import { isPlatformBuild, isTauri } from "./tauri.js";
import { confirmUnsavedChanges } from "./project-actions.js";

export const repositoryUrl = "https://github.com/OpenAEC-Foundation/Open-Frame-Studio";

/** Open a GitHub link (e.g. the issue the feedback service just created) in the system browser. */
export async function openGithubUrl(url) {
  if (isPlatformBuild || typeof url !== "string" || !url.startsWith("https://github.com/")) return;
  await openProjectLink(url, true);
}

export async function openProjectLink(path = "", absolute = false) {
  if (isPlatformBuild) return;
  const url = absolute ? path : repositoryUrl + path;
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
