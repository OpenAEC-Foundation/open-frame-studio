import test from "node:test";
import assert from "node:assert/strict";
import definitionsFile from "./mcp-tools.json" with { type: "json" };
import {
  frameToolDefinitions, frameToolNames, runFrameTool,
} from "./frameTools.js";

const makeKozijn = (id, mark, name, cells = []) => ({
  id, mark, name,
  frame: {
    outerWidth: 1200, outerHeight: 1500,
    material: { wood: "meranti" },
    profile: { id: "p-1", name: "Meranti 67x114" },
    shape: { shapeType: "rectangular" },
  },
  grid: { columns: [{ size: 1200 }], rows: [{ size: 1500 }] },
  cells,
  placement: { x: 10, y: 20, z: 30, orientation: 90 },
});

const project = {
  id: "project-1", formatVersion: "1.3",
  projectInfo: { name: "School", number: "P-42", client: "private client" },
  kozijnen: [
    makeKozijn("k-1", "K01", "Voorgevel", [{ panelType: "fixed_glass", glazing: { glassType: "HR++", thicknessMm: 24, ugValue: 1 } }]),
    makeKozijn("k-2", "K02", "Achtergevel"),
    makeKozijn("k-3", "K03", "Voorgevel verdieping"),
  ],
  vliesgevels: [{ id: "vg-1", mark: "VG01" }],
};

test("definitions and exported tool names come from the single JSON source", () => {
  assert.deepEqual(frameToolDefinitions(), definitionsFile);
  assert.deepEqual(frameToolNames(), ["frame_stand", "frame_kozijnen", "frame_kozijn"]);
  const definitions = frameToolDefinitions();
  definitions[0].name = "changed";
  assert.equal(frameToolNames()[0], "frame_stand");
});

test("frame_stand reports current model, Docs status and valid selection only", () => {
  const context = {
    project, selectedKozijnId: "k-1", dirty: true,
    hostFileRef: { id: "doc-1", name: "school.ofs", version: 7, readOnly: true },
  };
  const result = runFrameTool("frame_stand", {}, context);
  assert.deepEqual(result.counts, { kozijnen: 3, vliesgevels: 1 });
  assert.deepEqual(result.selection, { type: "kozijn", id: "k-1", mark: "K01" });
  assert.deepEqual(result.file, { id: "doc-1", name: "school.ofs", version: 7, readOnly: true });
  assert.equal(result.dirty, true);
  assert.equal(JSON.stringify(result).includes("private client"), false);
  assert.equal(runFrameTool("frame_stand", {}, { project, selectedKozijnId: "gone" }).selection, null);
});

test("frame_kozijnen filters and pages without exposing the full project", () => {
  const result = runFrameTool("frame_kozijnen", { query: "voorgevel", offset: 1, limit: 1 }, { project });
  assert.deepEqual({ total: result.total, offset: result.offset, limit: result.limit }, { total: 2, offset: 1, limit: 1 });
  assert.deepEqual(result.items.map(({ id }) => id), ["k-3"]);
  assert.equal(result.items[0].material, "wood:meranti");
  assert.equal(result.items[0].widthMm, 1200);
  assert.equal("cells" in result.items[0], false);
  assert.throws(() => runFrameTool("frame_kozijnen", { limit: 51 }, { project }), /ongeldige_argumenten/);
  assert.throws(() => runFrameTool("frame_kozijnen", { offset: -1 }, { project }), /ongeldige_argumenten/);
});

test("frame_kozijn uses selection or explicit id and bounds the cell preview", () => {
  const manyCells = Array.from({ length: 100 }, () => ({ panelType: "fixed_glass", glazing: { glassType: "HR++" } }));
  const model = { ...project, kozijnen: [makeKozijn("big", "K99", "A".repeat(100_000), manyCells)] };
  const result = runFrameTool("frame_kozijn", {}, { project: model, selectedKozijnId: "big" });
  assert.equal(result.name.length, 120);
  assert.equal(result.cells.total, 100);
  assert.equal(result.cells.first.length, 20);
  assert.equal(result.cells.omitted, 80);
  assert.equal(result.frame.profile.id, "p-1");
  assert.ok(JSON.stringify(result).length < 30_000);
  assert.equal(runFrameTool("frame_kozijn", { id: "k-2" }, { project }).id, "k-2");
  assert.throws(() => runFrameTool("frame_kozijn", {}, { project }), /geen_kozijn_geselecteerd/);
  assert.throws(() => runFrameTool("frame_kozijn", { id: "missing" }, { project }), /kozijn_niet_gevonden/);
});

test("unknown tools, extra arguments and absent projects fail explicitly", () => {
  assert.throws(() => runFrameTool("frame_delete", {}, { project }), /onbekende_tool/);
  assert.throws(() => runFrameTool("frame_stand", { mutate: true }, { project }), /ongeldige_argumenten/);
  assert.throws(() => runFrameTool("frame_stand", {}, { project: null }), /geen_project/);
});
