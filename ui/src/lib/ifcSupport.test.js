import test from "node:test";
import assert from "node:assert/strict";
import { ifcPreflight, ifcStructuralBlocker, needsLod200 } from "./ifcSupport.js";

const fixed = { mark: "K01", frame: { shape: { shapeType: "rectangular" } }, cells: [{ panelType: "fixed_glass" }] };
const turn = { mark: "K02", frame: { shape: { shapeType: "rectangular" } }, cells: [{ panelType: "turn_tilt" }] };
const free = { mark: "K03", frame: { shape: { shapeType: "rectangular" } }, cells: [], layout: { kind: "leaf" } };
const arched = { mark: "K04", frame: { shape: { shapeType: "arched" } }, cells: [{ panelType: "fixed_glass" }] };

test("fixed glass passes at LOD 300 unchanged", () => {
  const r = ifcPreflight([fixed], "300");
  assert.deepEqual([r.ok, r.lod, r.downgraded, r.message], [true, "300", false, null]);
});

test("vrije indeling is blocked before the dialog and names the way out", () => {
  const r = ifcPreflight([fixed, free], "200");
  assert.equal(r.ok, false);
  assert.match(r.message, /K03/);
  assert.match(r.message, /Terug naar raster/);
});

test("non-rectangular shape is blocked", () => {
  assert.ok(ifcStructuralBlocker(arched));
  assert.equal(ifcPreflight([arched], "200").ok, false);
});

test("LOD 300 with operable parts falls back to 200 with a notice when allowed", () => {
  assert.ok(needsLod200(turn));
  const r = ifcPreflight([turn], "300", { allowLodFallback: true });
  assert.deepEqual([r.ok, r.lod, r.downgraded], [true, "200", true]);
  assert.match(r.message, /LOD 200/);
});

test("LOD 300 with operable parts is refused with advice when fallback is off", () => {
  const r = ifcPreflight([turn], "300");
  assert.equal(r.ok, false);
  assert.match(r.message, /kies LOD 200/);
});

test("LOD 200 never needs a fallback", () => {
  assert.equal(ifcPreflight([turn], "200").downgraded, false);
});
