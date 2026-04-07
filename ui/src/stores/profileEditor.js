/**
 * Profile Editor store.
 * Beheert de state van de interactieve profieleditor.
 */
import { writable, derived, get } from "svelte/store";

const MAX_UNDO = 40;

function createProfileEditorStore() {
  const { subscribe, set, update } = writable({
    profile: null,
    vertices: [],
    selectedVertex: -1,
    tool: "select",
    snap: true,
    gridSize: 1,
    zoom: 4,
    pan: { x: 0, y: 0 },
    undoStack: [],
    redoStack: [],
    isDirty: false,
    contextProfile: null,
    sponning: { width: 12, depth: 17, position: "buiten" },
    referenceLines: [],
  });

  function pushUndo(state) {
    update((s) => {
      const snapshot = JSON.stringify(s.vertices);
      const stack = [...s.undoStack, snapshot].slice(-MAX_UNDO);
      return { ...s, undoStack: stack, redoStack: [] };
    });
  }

  return {
    subscribe,
    set,
    update,

    loadProfile(profile) {
      const verts = (profile.crossSection || []).map(([x, y]) => ({ x, y }));
      set({
        profile: { ...profile },
        vertices: verts,
        selectedVertex: -1,
        tool: "select",
        snap: true,
        gridSize: 1,
        zoom: 4,
        pan: { x: 0, y: 0 },
        undoStack: [],
        redoStack: [],
        isDirty: false,
        contextProfile: null,
        sponning: profile.sponning
          ? { ...profile.sponning }
          : { width: 12, depth: 17, position: "buiten" },
        referenceLines: [],
      });
    },

    newProfile() {
      const defaultVerts = [
        { x: 0, y: 0 },
        { x: 67, y: 0 },
        { x: 67, y: 97 },
        { x: 55, y: 97 },
        { x: 55, y: 114 },
        { x: 12, y: 114 },
        { x: 12, y: 97 },
        { x: 0, y: 97 },
      ];
      set({
        profile: {
          id: null,
          name: "",
          material: "wood",
          series: "",
          manufacturer: "",
          width: 67,
          depth: 114,
          sightline: 55,
          glazingRebate: 17,
          ufValue: 1.8,
          applicableAs: ["frame"],
        },
        vertices: defaultVerts,
        selectedVertex: -1,
        tool: "select",
        snap: true,
        gridSize: 1,
        zoom: 4,
        pan: { x: 0, y: 0 },
        undoStack: [],
        redoStack: [],
        isDirty: false,
        contextProfile: null,
        sponning: { width: 12, depth: 17, position: "buiten" },
        referenceLines: [],
      });
    },

    selectVertex(index) {
      update((s) => ({ ...s, selectedVertex: index }));
    },

    deselectAll() {
      update((s) => ({ ...s, selectedVertex: -1 }));
    },

    moveVertex(index, x, y) {
      update((s) => {
        const verts = [...s.vertices];
        verts[index] = { x, y };
        return { ...s, vertices: verts, isDirty: true };
      });
    },

    beginDrag() {
      const s = get({ subscribe });
      pushUndo(s);
    },

    addVertex(afterIndex, x, y) {
      pushUndo();
      update((s) => {
        const verts = [...s.vertices];
        verts.splice(afterIndex + 1, 0, { x, y });
        return {
          ...s,
          vertices: verts,
          selectedVertex: afterIndex + 1,
          isDirty: true,
        };
      });
    },

    removeVertex(index) {
      const s = get({ subscribe });
      if (s.vertices.length <= 3) return;
      pushUndo();
      update((s) => {
        const verts = s.vertices.filter((_, i) => i !== index);
        return {
          ...s,
          vertices: verts,
          selectedVertex: -1,
          isDirty: true,
        };
      });
    },

    setTool(tool) {
      update((s) => ({ ...s, tool, selectedVertex: -1 }));
    },

    toggleSnap() {
      update((s) => ({ ...s, snap: !s.snap }));
    },

    mirrorH() {
      pushUndo();
      update((s) => {
        const maxX = Math.max(...s.vertices.map((v) => v.x));
        const verts = s.vertices.map((v) => ({ x: maxX - v.x, y: v.y }));
        return { ...s, vertices: verts, isDirty: true };
      });
    },

    mirrorV() {
      pushUndo();
      update((s) => {
        const maxY = Math.max(...s.vertices.map((v) => v.y));
        const verts = s.vertices.map((v) => ({ x: v.x, y: maxY - v.y }));
        return { ...s, vertices: verts, isDirty: true };
      });
    },

    scaleToSize(newWidth, newHeight) {
      const s = get({ subscribe });
      if (s.vertices.length === 0) return;
      const xs = s.vertices.map((v) => v.x);
      const ys = s.vertices.map((v) => v.y);
      const oldW = Math.max(...xs) - Math.min(...xs);
      const oldH = Math.max(...ys) - Math.min(...ys);
      if (oldW === 0 || oldH === 0) return;
      pushUndo();
      const sx = newWidth / oldW;
      const sy = newHeight / oldH;
      const minX = Math.min(...xs);
      const minY = Math.min(...ys);
      update((s) => ({
        ...s,
        vertices: s.vertices.map((v) => ({
          x: Math.round((minX + (v.x - minX) * sx) * 10) / 10,
          y: Math.round((minY + (v.y - minY) * sy) * 10) / 10,
        })),
        isDirty: true,
      }));
    },

    undo() {
      update((s) => {
        if (s.undoStack.length === 0) return s;
        const stack = [...s.undoStack];
        const prev = stack.pop();
        const redo = [...s.redoStack, JSON.stringify(s.vertices)];
        return {
          ...s,
          vertices: JSON.parse(prev),
          undoStack: stack,
          redoStack: redo,
          selectedVertex: -1,
          isDirty: true,
        };
      });
    },

    redo() {
      update((s) => {
        if (s.redoStack.length === 0) return s;
        const stack = [...s.redoStack];
        const next = stack.pop();
        const undo = [...s.undoStack, JSON.stringify(s.vertices)];
        return {
          ...s,
          vertices: JSON.parse(next),
          undoStack: undo,
          redoStack: stack,
          selectedVertex: -1,
          isDirty: true,
        };
      });
    },

    updateSponning(sponning) {
      update((s) => ({ ...s, sponning: { ...sponning }, isDirty: true }));
    },

    updateProfile(partial) {
      update((s) => ({
        ...s,
        profile: { ...s.profile, ...partial },
        isDirty: true,
      }));
    },

    getExportData() {
      const s = get({ subscribe });
      const xs = s.vertices.map((v) => v.x);
      const ys = s.vertices.map((v) => v.y);
      const width = Math.max(...xs) - Math.min(...xs);
      const depth = Math.max(...ys) - Math.min(...ys);
      return {
        ...s.profile,
        width: Math.round(width * 10) / 10,
        depth: Math.round(depth * 10) / 10,
        sightline: Math.round((width - s.sponning.width) * 10) / 10,
        glazingRebate: s.sponning.depth,
        crossSection: s.vertices.map((v) => [v.x, v.y]),
        sponning: { ...s.sponning },
      };
    },
  };
}

export const profileEditor = createProfileEditorStore();

// Derived stores for convenience
export const editorVertices = derived(profileEditor, ($s) => $s.vertices);
export const editorZoom = derived(profileEditor, ($s) => $s.zoom);
export const editorPan = derived(profileEditor, ($s) => $s.pan);
export const editorTool = derived(profileEditor, ($s) => $s.tool);
export const editorSnap = derived(profileEditor, ($s) => $s.snap);
export const editorSelectedVertex = derived(profileEditor, ($s) => $s.selectedVertex);
export const editorIsDirty = derived(profileEditor, ($s) => $s.isDirty);
export const editorProfile = derived(profileEditor, ($s) => $s.profile);

export const editorBounds = derived(profileEditor, ($s) => {
  if ($s.vertices.length === 0) return { minX: 0, minY: 0, maxX: 100, maxY: 100, width: 100, height: 100 };
  const xs = $s.vertices.map((v) => v.x);
  const ys = $s.vertices.map((v) => v.y);
  const minX = Math.min(...xs);
  const minY = Math.min(...ys);
  const maxX = Math.max(...xs);
  const maxY = Math.max(...ys);
  return { minX, minY, maxX, maxY, width: maxX - minX, height: maxY - minY };
});
