/**
 * Keyboard shortcuts for Open Frame Studio.
 */
import { get } from "svelte/store";
import {
  currentKozijn,
  selectedCellIndex,
  selectedMember,
  createKozijn,
  addColumn,
  addRow,
  updateCellType,
} from "../stores/kozijn.js";
import { zoom } from "../stores/ui.js";

const PANEL_TYPE_KEYS = {
  "1": "fixed_glass",
  "2": "turn_tilt",
  "3": "turn",
  "4": "tilt",
  "5": "sliding",
  "6": "door",
  "7": "panel",
  "8": "ventilation",
};

// App shortcuts that override browser defaults
const APP_SHORTCUTS = new Set(["n", "o", "s", "d", "z", "y"]);

let registered = false;

/**
 * Block browser shortcuts, but allow app shortcuts through.
 */
function blockBrowserDefaults(e) {
  const ctrl = e.ctrlKey || e.metaKey;

  if (["F3", "F5", "F7"].includes(e.key)) {
    e.preventDefault();
    return;
  }

  if (!ctrl) return;

  // Let app shortcuts through — they're handled in the main handler
  if (APP_SHORTCUTS.has(e.key.toLowerCase()) && !e.altKey) return;

  // Block everything else with Ctrl
  const blocked = ["p", "f", "g", "u", "r", "h", "j", "l", "e", "k", "t", "w", "q"];
  if (blocked.includes(e.key.toLowerCase())) {
    e.preventDefault();
    return;
  }
  if (e.shiftKey && ["I", "J", "i", "j"].includes(e.key)) {
    e.preventDefault();
    return;
  }
  if (e.key >= "0" && e.key <= "9") {
    e.preventDefault();
    return;
  }
  if (e.key === "=" || e.key === "+" || e.key === "-") {
    e.preventDefault();
    return;
  }
}

export function registerShortcuts({ onDuplicate, onNew, onOpen, onSave, onSaveAs } = {}) {
  if (registered) return;
  registered = true;

  document.addEventListener("keydown", blockBrowserDefaults, true);
  document.addEventListener("contextmenu", (e) => e.preventDefault());

  document.addEventListener("keydown", (e) => {
    const tag = e.target.tagName;
    if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;

    const ctrl = e.ctrlKey || e.metaKey;
    const k = get(currentKozijn);
    const key = e.key.toLowerCase();

    // ── File shortcuts ──────────────────────────────────
    if (ctrl && key === "n") {
      e.preventDefault();
      if (onNew) onNew();
      return;
    }
    if (ctrl && key === "o") {
      e.preventDefault();
      if (onOpen) onOpen();
      return;
    }
    if (ctrl && e.shiftKey && key === "s") {
      e.preventDefault();
      if (onSaveAs) onSaveAs();
      return;
    }
    if (ctrl && key === "s") {
      e.preventDefault();
      if (onSave) onSave();
      return;
    }

    // ── Undo/Redo ───────────────────────────────────────
    // Ctrl+Z / Ctrl+Y handled by history.js registerUndoRedoShortcuts

    // ── Escape: deselect ────────────────────────────────
    if (e.key === "Escape") {
      selectedCellIndex.set(null);
      selectedMember.set(null);
      return;
    }

    // ── Tab: next/prev cell ─────────────────────────────
    if (e.key === "Tab" && k) {
      e.preventDefault();
      const current = get(selectedCellIndex);
      const total = k.cells.length;
      if (current === null) {
        selectedCellIndex.set(0);
      } else {
        selectedCellIndex.set((current + (e.shiftKey ? total - 1 : 1)) % total);
      }
      selectedMember.set(null);
      return;
    }

    // ── 1-8: quick panel type ───────────────────────────
    if (!ctrl && PANEL_TYPE_KEYS[e.key] && get(selectedCellIndex) !== null) {
      const panelType = PANEL_TYPE_KEYS[e.key];
      const dir = panelType === "turn_tilt" ? "left" : panelType === "door" ? "inward" : null;
      updateCellType(get(selectedCellIndex), panelType, dir);
      return;
    }

    // ── Ctrl+D: duplicate ───────────────────────────────
    if (ctrl && key === "d" && k) {
      e.preventDefault();
      if (onDuplicate) onDuplicate();
      return;
    }

    // ── Delete: reset cell ──────────────────────────────
    if (e.key === "Delete") {
      const cellIdx = get(selectedCellIndex);
      if (cellIdx !== null) {
        updateCellType(cellIdx, "fixed_glass", null);
      }
      if (get(selectedMember)) {
        selectedMember.set(null);
      }
      return;
    }

    // ── Ctrl+Shift+C: add column ────────────────────────
    if (e.key === "C" && ctrl && e.shiftKey && k) {
      e.preventDefault();
      addColumn((k.frame.outerWidth - 2 * k.frame.frameWidth) / 2);
      return;
    }

    // ── Ctrl+Shift+R: add row ───────────────────────────
    if (e.key === "R" && ctrl && e.shiftKey && k) {
      e.preventDefault();
      addRow((k.frame.outerHeight - 2 * k.frame.frameWidth) / 2);
      return;
    }

    // ── +/- zoom ────────────────────────────────────────
    if (!ctrl && (e.key === "=" || e.key === "+")) {
      zoom.update((z) => Math.min(2.0, z + 0.1));
      return;
    }
    if (!ctrl && e.key === "-") {
      zoom.update((z) => Math.max(0.05, z - 0.1));
      return;
    }
  });
}
