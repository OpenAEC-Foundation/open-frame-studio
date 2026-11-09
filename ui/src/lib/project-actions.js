/**
 * Project file actions — New, Open, Save, Save As.
 * Includes unsaved changes protection.
 */
import { get } from "svelte/store";
import { _ } from "svelte-i18n";
import {
  newProject, openProject, openProjectJson, saveProject,
  projectPath, hostFileRef, isDirty, projectSaveToken, isProjectTokenCurrent, finishHostSave,
} from "../stores/project.js";
import { clearHistory } from "../stores/history.js";
import {
  currentKozijn, currentGeometry, selectedCellIndex, selectedMember, selectedLayoutLeafId,
} from "../stores/kozijn.js";
import { invoke, isTauri } from "./tauri.js";
import { getOpenAecHost, listHostProjects, openHostProjectIfCurrent, writeHostProject } from "./platformHost.js";
import { toast } from "../stores/toast.js";

function clearKozijnSelection() {
  currentKozijn.set(null);
  currentGeometry.set(null);
  selectedCellIndex.set(null);
  selectedMember.set(null);
  selectedLayoutLeafId.set(null);
}

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
        return (await fileSave()) && !get(isDirty);
      }
      return true; // user chose discard
    } catch {
      // fallback to confirm
    }
  }

  const save = confirm(get(_)("dialog.unsavedMessage"));
  if (save) {
    return (await fileSave()) && !get(isDirty);
  }
  return true;
}

export async function fileNew() {
  if (!(await confirmUnsavedChanges())) return false;
  await newProject("New", "");
  clearHistory();
  clearKozijnSelection();
  return true;
}

export async function fileOpen() {
  if (!(await confirmUnsavedChanges())) return false;
  const host = getOpenAecHost();
  if (host) {
    try {
      const files = await listHostProjects(host);
      if (files.length === 0) {
        toast.error("Geen .ofs-projecten in deze Docs-map gevonden");
        return false;
      }
      const choices = files.map((file, index) => `${index + 1}. ${file.name}`).join("\n");
      const answer = window.prompt(`Kies een projectnummer uit Docs:\n${choices}`, "1");
      if (answer === null) return false;
      const index = Number(answer) - 1;
      if (!Number.isInteger(index) || index < 0 || index >= files.length) {
        toast.error("Kies een geldig projectnummer");
        return false;
      }
      return await openHostFileById(files[index].id);
    } catch (error) {
      toast.error(`Docs openen mislukt: ${error?.message || error}`);
      return false;
    }
  }
  if (!isTauri) {
    toast.error("Projecten openen is in deze browser alleen via de SuperCloud Docs-koppeling beschikbaar");
    return false;
  }
  const { open } = await getDialogs();
  const path = await open({
    filters: [{ name: "Open Frame Studio", extensions: ["ofs"] }],
    multiple: false,
  });
  if (path) {
    await openProject(path);
    clearHistory();
    clearKozijnSelection();
    return true;
  }
  return false;
}

/** Called at startup for host.initialFileId and from the Docs file picker. */
export async function openHostFileById(id, version) {
  const host = getOpenAecHost();
  if (!host) return false;
  try {
    const opened = await openHostProjectIfCurrent(host, id, version, {
      captureState: async () => ({ token: projectSaveToken(), json: await invoke("save_project") }),
      isStateCurrent: async ({ token, json }) =>
        isProjectTokenCurrent(token) && (await invoke("save_project")) === json,
      apply: ({ json, ref }) => openProjectJson(json, ref),
    });
    if (!opened) {
      toast.error("Docs-project niet geopend: het huidige project is intussen gewijzigd");
      return false;
    }
    clearHistory();
    clearKozijnSelection();
    return true;
  } catch (error) {
    toast.error(`Docs-project openen mislukt: ${error?.message || error}`);
    return false;
  }
}

export async function fileSave() {
  const host = getOpenAecHost();
  if (host) {
    const current = get(hostFileRef);
    if (!current) return fileSaveAs();
    try {
      const token = projectSaveToken();
      const json = await invoke("save_project");
      const saved = await writeHostProject(host, json, { current });
      const stillOpen = finishHostSave(token, saved);
      if (!stillOpen) {
        toast.success(`Docs-kopie opgeslagen: ${saved.name}; intussen is een ander project geopend`);
        return true;
      }
      toast.success(`Opgeslagen in Docs: ${saved.name}`);
      return true;
    } catch (error) {
      toast.error(`Docs opslaan mislukt: ${error?.message || error}`);
      return false;
    }
  }
  if (!isTauri) {
    toast.error("Projecten opslaan is in deze browser alleen via de SuperCloud Docs-koppeling beschikbaar");
    return false;
  }
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
  const host = getOpenAecHost();
  if (host) {
    const name = window.prompt("Naam voor het nieuwe Docs-project:", get(hostFileRef)?.name || "project.ofs");
    if (name === null) return false;
    try {
      const token = projectSaveToken();
      const json = await invoke("save_project");
      const saved = await writeHostProject(host, json, { name });
      const stillOpen = finishHostSave(token, saved);
      if (!stillOpen) {
        toast.success(`Docs-kopie opgeslagen: ${saved.name}; intussen is een ander project geopend`);
        return true;
      }
      toast.success(`Opgeslagen in Docs: ${saved.name}`);
      return true;
    } catch (error) {
      toast.error(`Docs Opslaan als mislukt: ${error?.message || error}`);
      return false;
    }
  }
  if (!isTauri) {
    toast.error("Projecten opslaan is in deze browser alleen via de SuperCloud Docs-koppeling beschikbaar");
    return false;
  }
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
