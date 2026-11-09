import { writable, derived, get } from "svelte/store";
import { invoke, setHostReadOnly } from "../lib/tauri.js";
import { refreshCustomProfiles } from "./profiles.js";

export const project = writable(null);
export const projectPath = writable(null);
export const hostFileRef = writable(null);
export const isDirty = writable(false);
let projectEpoch = 0;
let projectRevision = 0;

/** Capture the project and edit generation before an asynchronous Docs save. */
export function projectSaveToken() {
  return { epoch: projectEpoch, revision: projectRevision };
}

export function isProjectTokenCurrent(token) {
  return token?.epoch === projectEpoch && token?.revision === projectRevision;
}

/** A completed save may belong to a project that is no longer open. */
export function finishHostSave(token, ref) {
  if (token.epoch !== projectEpoch) return false;
  setHostFileRef(ref);
  if (token.revision === projectRevision) isDirty.set(false);
  return true;
}

export function setHostFileRef(ref) {
  hostFileRef.set(ref);
  projectPath.set(ref?.name ?? null);
  setHostReadOnly(ref?.readOnly ?? false);
}

export const kozijnen = derived(project, ($project) =>
  $project ? $project.kozijnen : []
);

/** Mark the project as modified. Call after any mutation. */
export function markDirty() {
  projectRevision += 1;
  isDirty.set(true);
}

export async function loadProject() {
  try {
    const p = await invoke("get_project");
    project.set(p);
    return p;
  } catch (e) {
    console.error("Laden project mislukt:", e);
    return null;
  }
}

export async function newProject(name, number) {
  const p = await invoke("new_project", { name, number });
  projectEpoch += 1;
  projectRevision += 1;
  project.set(p);
  projectPath.set(null);
  setHostFileRef(null);
  isDirty.set(false);
  await refreshCustomProfiles();
  return p;
}

export async function openProject(filePath) {
  const p = await invoke("open_project", { filePath });
  projectEpoch += 1;
  projectRevision += 1;
  project.set(p);
  projectPath.set(filePath);
  hostFileRef.set(null);
  setHostReadOnly(false);
  isDirty.set(false);
  await refreshCustomProfiles();
  return p;
}

/** Restore a Docs-hosted .ofs snapshot through the same WASM parser as web mode. */
export async function openProjectJson(json, ref) {
  const p = await invoke("open_project", { json });
  projectEpoch += 1;
  projectRevision += 1;
  project.set(p);
  setHostFileRef(ref);
  isDirty.set(false);
  await refreshCustomProfiles();
  return p;
}

export async function saveProject(filePath) {
  await invoke("save_project", { filePath });
  projectPath.set(filePath);
  isDirty.set(false);
}

export async function refreshProject() {
  return loadProject();
}
