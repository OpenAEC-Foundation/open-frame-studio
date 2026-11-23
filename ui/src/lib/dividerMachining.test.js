import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { dividerToolReference } from "./dividerMachining.js";
import { systemProfileSolids } from "./systemProfileSolids.js";

test("the VEKA MD tool reference applies only at an identified straight square frame joint", () => {
  const square = {
    machiningStatus: "veka_md_glm", sawAllowanceMm: 25, angleDeg: 90,
    segments: [{kind: "line", from: [0, 0], to: [94, 0]}],
  };
  assert.match(dividerToolReference(square), /L 19 -26 V -15 H 43\.5 V -0\.9 H 61\.5 V -4\.1 H 77\.5 V 0/);
  for (const other of [
    {machiningStatus: "veka_md_glm_at_t"}, {machiningStatus: "drawing_required"},
    {sawAllowanceMm: 0}, {radiusMm: 527}, {angleDeg: 89}, {angleDeg: undefined},
    {segments: []}, {segments: [{kind: "arc"}]},
  ]) assert.equal(dividerToolReference({...square, ...other}), null);
});

test("VEKA 102.310 has its own chambers and the specified slotted steel reinforcement", () => {
  const [profile] = JSON.parse(readFileSync(new URL("../../../profiles/pvc/veka-divider.json", import.meta.url)));
  assert.equal(profile.articleNumber, "102.310");
  assert.deepEqual([profile.width, profile.depth], [94, 82]);
  const solids = systemProfileSolids(profile);
  // A horizontal line through a split convex strip intersects its left and right edges.
  function materialAt(x, y) {
    return solids.find(({points}) => {
      const hits = [];
      for (let i = 0; i < points.length; i++) {
        const [a, b] = points[i], [c, d] = points[(i + 1) % points.length];
        if ((b > y) !== (d > y)) hits.push(a + (y - b) * (c - a) / (d - b));
      }
      return hits.length === 2 && x > Math.min(...hits) && x < Math.max(...hits);
    })?.material;
  }
  assert.equal(materialAt(47, 1), "pvc");
  assert.equal(materialAt(47, 5), undefined);
  assert.equal(materialAt(32.7, 45), "reinforcement");
  assert.equal(materialAt(47, 45), undefined);
  assert.equal(materialAt(61.3, 44.6), undefined);
});
