/**
 * Profile Editor store.
 * Beheert de state van de interactieve profieleditor.
 */
import { writable, derived, get } from "svelte/store";

import { profileRebate, woodCrossSection, woodSectionFits } from "../lib/profileContour.js";

const standardRebate = { width: 51, depth: 17, position: "binnen", sponningType: "binnensponning" };
const roundMm = (value) => Math.round(value * 10) / 10;
const verticesOf = (points) => points.map(([x, y]) => ({ x, y, radius: 0 }));

function snapshot(s) {
  return JSON.stringify({ vertices: s.vertices, innerWalls: s.innerWalls, sponning: s.sponning, profile: s.profile });
}

function boundsOf(vertices) {
  if (!vertices.length) return { minX: 0, minY: 0, maxX: 100, maxY: 100, width: 100, height: 100 };
  const xs = vertices.map(v => v.x), ys = vertices.map(v => v.y);
  const minX = Math.min(...xs), maxX = Math.max(...xs);
  const minY = Math.min(...ys), maxY = Math.max(...ys);
  return { minX, minY, maxX, maxY, width: maxX - minX, height: maxY - minY };
}

function woodSpec(s) {
  const { width, height } = boundsOf(s.vertices);
  return { ...s.profile, width, depth: height, sponning: s.sponning };
}

// The same outline can start at any vertex and run clockwise or anticlockwise.
function isGeneratedWood(s) {
  if (s.profile?.material !== "wood" || !s.vertices.length) return false;
  const points = woodCrossSection(woodSpec(s));
  const n = points.length;
  if (n !== s.vertices.length) return false;
  const start = s.vertices.findIndex(({ x, y }) => x === points[0][0] && y === points[0][1]);
  if (start < 0) return false;
  return [1, -1].some(step => points.every(([u, v], i) => {
    const vertex = s.vertices[(start + i * step + n) % n];
    return u === vertex.x && v === vertex.y && !vertex.radius;
  }));
}

function dimensionsFor(s) {
  const { width } = boundsOf(s.vertices);
  const timber = ["wood", "wood_aluminum"].includes(s.profile?.material);
  const sightline = timber && s.sponning.depth ? roundMm(width - s.sponning.depth) : s.profile?.sightline ?? width;
  const glazingRebate = timber && (s.sponning.width || s.profile?.sponning)
    ? s.sponning.width : s.profile?.glazingRebate ?? s.sponning.depth;
  return { sightline, glazingRebate };
}

const MAX_UNDO = 40;

function createProfileEditorStore() {
  const { subscribe, set, update } = writable({
    profile: null,
    vertices: [],
    innerWalls: [],
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
    sponning: { ...standardRebate },
    referenceLines: [],
  });

  function cloneWalls(walls) {
    return (walls || []).map((poly) => poly.map(([x, y]) => [x, y]));
  }

  function pushUndo() {
    update((s) => {
      const stack = [...s.undoStack, snapshot(s)].slice(-MAX_UNDO);
      return { ...s, undoStack: stack, redoStack: [] };
    });
  }

  return {
    subscribe,
    set,
    update,

    loadProfile(profile) {
      const verts = verticesOf(profile.crossSection || []);
      set({
        profile: { ...profile },
        vertices: verts,
        // Binnenstructuur (kamers/staal/isolatoren) — decoratieve tekenlaag
        innerWalls: cloneWalls(profile.innerWalls),
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
        sponning: profileRebate(profile.sponning),
        referenceLines: [],
      });
    },

    newProfile() {
      const defaultVerts = verticesOf(woodCrossSection({ width: 67, depth: 114, sponning: standardRebate }));
      set({
        profile: {
          id: null,
          name: "",
          material: "wood",
          series: "",
          manufacturer: "",
          width: 67,
          depth: 114,
          sightline: 50,
          glazingRebate: 51,
          ufValue: 1.8,
          applicableAs: ["frame"],
        },
        vertices: defaultVerts,
        innerWalls: [],
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
        sponning: { ...standardRebate },
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
      pushUndo();
    },

    addVertex(afterIndex, x, y) {
      pushUndo();
      update((s) => {
        const verts = [...s.vertices];
        verts.splice(afterIndex + 1, 0, { x, y, radius: 0 });
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

    setVertexRadius(index, radius) {
      pushUndo();
      update((s) => {
        const verts = [...s.vertices];
        if (verts[index]) {
          verts[index] = { ...verts[index], radius: Math.max(0, radius) };
        }
        return { ...s, vertices: verts, isDirty: true };
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
        const walls = (s.innerWalls || []).map((poly) => poly.map(([x, y]) => [maxX - x, y]));
        return { ...s, vertices: verts, innerWalls: walls, isDirty: true };
      });
    },

    mirrorV() {
      pushUndo();
      update((s) => {
        const maxY = Math.max(...s.vertices.map((v) => v.y));
        const verts = s.vertices.map((v) => ({ x: v.x, y: maxY - v.y }));
        const walls = (s.innerWalls || []).map((poly) => poly.map(([x, y]) => [x, maxY - y]));
        const position = { binnen: "buiten", buiten: "binnen" }[s.sponning.position] ?? s.sponning.position;
        const sponning = { ...s.sponning, position };
        if (["binnensponning", "buitensponning"].includes(sponning.sponningType) && ["binnen", "buiten"].includes(position)) {
          sponning.sponningType = position === "binnen" ? "binnensponning" : "buitensponning";
        }
        if (position === "dubbel") {
          sponning.width = s.sponning.secondWidth ?? s.sponning.width;
          sponning.depth = s.sponning.secondDepth ?? s.sponning.depth;
          sponning.secondWidth = s.sponning.width;
          sponning.secondDepth = s.sponning.depth;
        }
        return { ...s, vertices: verts, innerWalls: walls, sponning, isDirty: true };
      });
    },

    scaleToSize(newWidth, newHeight) {
      const s = get({ subscribe });
      if (s.vertices.length === 0) return;
      const xs = s.vertices.map((v) => v.x);
      const ys = s.vertices.map((v) => v.y);
      const oldW = Math.max(...xs) - Math.min(...xs);
      const oldH = Math.max(...ys) - Math.min(...ys);
      if (oldW === 0 || oldH === 0) return false;
      if (isGeneratedWood(s)) {
        const resized = { ...woodSpec(s), width: newWidth, depth: newHeight };
        if (!woodSectionFits(resized)) return false;
        pushUndo();
        update(s => ({ ...s, vertices: verticesOf(woodCrossSection(resized)), selectedVertex: -1, isDirty: true }));
        return true;
      }
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
        profile: { ...s.profile, sightline: roundMm((s.profile.sightline ?? oldW) * sx),
          glazingRebate: roundMm((s.profile.glazingRebate || 0) * (["wood", "wood_aluminum"].includes(s.profile.material) ? sy : sx)) },
        sponning: profileRebate({ ...s.sponning,
          width: roundMm(s.sponning.width * sy), depth: roundMm(s.sponning.depth * sx),
          secondWidth: s.sponning.secondWidth == null ? undefined : roundMm(s.sponning.secondWidth * sy),
          secondDepth: s.sponning.secondDepth == null ? undefined : roundMm(s.sponning.secondDepth * sx),
        }),
        innerWalls: (s.innerWalls || []).map((poly) =>
          poly.map(([x, y]) => [
            Math.round((minX + (x - minX) * sx) * 10) / 10,
            Math.round((minY + (y - minY) * sy) * 10) / 10,
          ])
        ),
        isDirty: true,
      }));
      return true;
    },

    undo() {
      update((s) => {
        if (s.undoStack.length === 0) return s;
        const stack = [...s.undoStack];
        const prev = JSON.parse(stack.pop());
        const redo = [...s.redoStack, snapshot(s)];
        return {
          ...s,
          ...prev,
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
        const next = JSON.parse(stack.pop());
        const undo = [...s.undoStack, snapshot(s)];
        return {
          ...s,
          ...next,
          undoStack: undo,
          redoStack: stack,
          selectedVertex: -1,
          isDirty: true,
        };
      });
    },

    updateSponning(sponning) {
      const current = get({ subscribe });
      if (isGeneratedWood(current) && !woodSectionFits({ ...woodSpec(current), sponning })) return false;
      pushUndo();
      update((s) => {
        const generated = isGeneratedWood(s);
        const next = profileRebate(sponning);
        const vertices = generated
          ? verticesOf(woodCrossSection({ ...woodSpec(s), sponning: next }))
          : s.vertices;
        return { ...s, sponning: next, vertices, selectedVertex: -1, isDirty: true };
      });
      return true;
    },

    updateProfile(partial) {
      update((s) => {
        const profile = { ...s.profile, ...partial };
        const next = { ...s, profile, isDirty: true };
        if (partial.applicableAs && isGeneratedWood(s) && woodSectionFits(woodSpec(next))) {
          next.vertices = verticesOf(woodCrossSection(woodSpec(next)));
          next.selectedVertex = -1;
        }
        return next;
      });
    },

    getExportData() {
      const s = get({ subscribe });
      const { width, height: depth } = boundsOf(s.vertices);
      return {
        ...s.profile,
        width: Math.round(width * 10) / 10,
        depth: Math.round(depth * 10) / 10,
        ...dimensionsFor(s),
        crossSection: s.vertices.map((v) => [v.x, v.y]),
        // Meegespiegeld/-geschaald met de contour; bij losse punt-edits
        // blijft de binnenstructuur als decoratieve laag ongewijzigd.
        innerWalls: cloneWalls(s.innerWalls),
        sponning: !s.profile.sponning && !s.sponning.depth && !s.sponning.width ? null : { ...s.sponning },
      };
    },
  };
}

export const profileEditor = createProfileEditorStore();

// Derived stores for convenience
export const editorVertices = derived(profileEditor, ($s) => $s.vertices);
export const editorInnerWalls = derived(profileEditor, ($s) => $s.innerWalls || []);
export const editorTool = derived(profileEditor, ($s) => $s.tool);
export const editorSnap = derived(profileEditor, ($s) => $s.snap);
export const editorSelectedVertex = derived(profileEditor, ($s) => $s.selectedVertex);
export const editorIsDirty = derived(profileEditor, ($s) => $s.isDirty);
export const editorProfile = derived(profileEditor, ($s) => $s.profile);

export const editorBounds = derived(profileEditor, ($s) => boundsOf($s.vertices));
export const editorDimensions = derived(profileEditor, dimensionsFor);
export const editorParametric = derived(profileEditor, isGeneratedWood);
