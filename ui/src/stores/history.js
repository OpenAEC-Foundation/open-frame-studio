/**
 * Undo/redo history system for kozijn edits.
 *
 * Stores snapshots of kozijn state before each mutation.
 * Ctrl+Z = undo, Ctrl+Y / Ctrl+Shift+Z = redo.
 */
import { writable, get } from "svelte/store";
import { currentKozijn, currentGeometry } from "./kozijn.js";
import { invoke } from "../lib/tauri.js";
import { refreshProject, markDirty } from "./project.js";

const MAX_HISTORY = 50;

// Internal state
let undoStack = [];
let redoStack = [];

export const canUndo = writable(false);
export const canRedo = writable(false);

function updateFlags() {
  canUndo.set(undoStack.length > 0);
  canRedo.set(redoStack.length > 0);
}

/**
 * Deep-clone a kozijn snapshot. structuredClone is fast but throws on values it
 * can't clone (e.g. a Svelte $state proxy that reached `kozijn.layout`); the
 * JSON fallback reads through proxies and always yields plain data, so taking a
 * snapshot never crashes a mutation.
 */
function snapshot(k) {
  try {
    return structuredClone(k);
  } catch {
    return JSON.parse(JSON.stringify(k));
  }
}

/**
 * Push a snapshot of the current kozijn onto the undo stack.
 * Call this BEFORE making a mutation.
 */
export function pushSnapshot() {
  const k = get(currentKozijn);
  if (!k) return;

  undoStack.push(snapshot(k));
  if (undoStack.length > MAX_HISTORY) {
    undoStack.shift();
  }

  // Clear redo stack on new action
  redoStack = [];
  updateFlags();
  markDirty();
}

/**
 * Undo: restore the previous state.
 */
export async function undo() {
  if (undoStack.length === 0) return;

  const k = get(currentKozijn);
  if (k) {
    redoStack.push(snapshot(k));
  }

  const prev = undoStack.pop();
  await restoreKozijn(prev);
  updateFlags();
}

/**
 * Redo: re-apply the undone state.
 */
export async function redo() {
  if (redoStack.length === 0) return;

  const k = get(currentKozijn);
  if (k) {
    undoStack.push(snapshot(k));
  }

  const next = redoStack.pop();
  await restoreKozijn(next);
  updateFlags();
}

/**
 * Restore a kozijn snapshot to the backend and update stores.
 */
async function restoreKozijn(snapshot) {
  try {
    // Update dimensions (this also triggers cell rebuild in backend)
    const updated = await invoke("update_kozijn_dimensions", {
      id: snapshot.id,
      width: snapshot.frame.outerWidth,
      height: snapshot.frame.outerHeight,
    });

    // Restore cell types
    for (let i = 0; i < snapshot.cells.length; i++) {
      const cell = snapshot.cells[i];
      await invoke("update_cell_type", {
        id: snapshot.id,
        cellIndex: i,
        panelType: cell.panelType,
        openingDirection: cell.openingDirection || null,
      });
    }

    // Restore the free-subdivision layout tree ("null" clears it, so undoing
    // the very first split also works)
    await invoke("update_kozijn_layout", {
      id: snapshot.id,
      layoutJson: JSON.stringify(snapshot.layout ?? null),
    });

    // Re-fetch the current state. Keep the snapshot's own layout tree rather
    // than the backend echo: older backends strip vulling extras (glaslat/
    // opensOutward/hardware) on the roundtrip, which would make undo lossy.
    const final = await invoke("get_kozijn", { id: snapshot.id });
    currentKozijn.set({ ...final, layout: snapshot.layout ?? null });

    const geom = await invoke("get_kozijn_geometry", { id: snapshot.id });
    currentGeometry.set(geom);

    await refreshProject();
  } catch (e) {
    console.error("Undo/redo mislukt:", e);
  }
}

/**
 * Clear history (e.g., when switching kozijnen or creating new project).
 */
export function clearHistory() {
  undoStack = [];
  redoStack = [];
  updateFlags();
}

/**
 * Register keyboard shortcuts. Call once on mount.
 */
export function registerUndoRedoShortcuts() {
  function handler(e) {
    if ((e.ctrlKey || e.metaKey) && e.key === "z" && !e.shiftKey) {
      e.preventDefault();
      undo();
    } else if (
      ((e.ctrlKey || e.metaKey) && e.key === "y") ||
      ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === "z") ||
      ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === "Z")
    ) {
      e.preventDefault();
      redo();
    }
  }

  window.addEventListener("keydown", handler);
  return () => window.removeEventListener("keydown", handler);
}
