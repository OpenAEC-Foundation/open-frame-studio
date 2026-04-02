/**
 * Project file actions — New, Open, Save, Save As.
 * Includes unsaved changes protection.
 */
import { get } from "svelte/store";
import { _ } from "svelte-i18n";
import { newProject, openProject, saveProject, projectPath, isDirty } from "../stores/project.js";
import { clearHistory } from "../stores/history.js";
import { isTauri } from "./tauri.js";
import { toast } from "../stores/toast.js";

async function getDialogs() {
  if (isTauri) return await import("@tauri-apps/plugin-dialog");
  return {
    open: async () => prompt("File path:"),
    save: async () => prompt("Save path:", "project.ofs"),
  };
}

/**
 * Check for unsaved changes and prompt user.
 * Returns true if it's safe to proceed, false if user cancelled.
 */
export async function confirmUnsavedChanges() {
  if (!get(isDirty)) return true;

  if (isTauri) {
    try {
      const { ask } = await import("@tauri-apps/plugin-dialog");
      const save = await ask(
        get(_)("dialog.unsavedMessage"),
        {
          title: get(_)("dialog.unsavedTitle"),
          kind: "warning",
          okLabel: get(_)("dialog.save"),
          cancelLabel: get(_)("dialog.discard"),
        }
      );
      if (save) {
        return await fileSave();
      }
      return true; // user chose discard
    } catch {
      // fallback to confirm
    }
  }

  const save = confirm(get(_)("dialog.unsavedMessage"));
  if (save) {
    return await fileSave();
  }
  return true;
}

export async function fileNew() {
  if (!(await confirmUnsavedChanges())) return false;
  await newProject("New", "");
  clearHistory();
  return true;
}

export async function fileOpen() {
  if (!(await confirmUnsavedChanges())) return false;
  const { open } = await getDialogs();
  const path = await open({
    filters: [{ name: "Open Frame Studio", extensions: ["ofs"] }],
    multiple: false,
  });
  if (path) {
    await openProject(path);
    clearHistory();
    return true;
  }
  return false;
}

export async function fileSave() {
  let path = get(projectPath);
  if (!path) {
    const { save } = await getDialogs();
    path = await save({
      filters: [{ name: "Open Frame Studio", extensions: ["ofs"] }],
      defaultPath: "project.ofs",
    });
  }
  if (path) {
    await saveProject(path);
    toast.success(get(_)("alert.saved"));
    return true;
  }
  return false;
}

export async function fileSaveAs() {
  const { save } = await getDialogs();
  const path = await save({
    filters: [{ name: "Open Frame Studio", extensions: ["ofs"] }],
    defaultPath: "project.ofs",
  });
  if (path) {
    await saveProject(path);
    toast.success(get(_)("alert.saved"));
    return true;
  }
  return false;
}
