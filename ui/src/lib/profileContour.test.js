import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { profileEditor } from "../stores/profileEditor.js";
import { woodCrossSection, woodSectionFits } from "./profileContour.js";

const frame = { width: 67, depth: 114, applicableAs: ["frame"] };
const rebate = { width: 51, depth: 17, position: "binnen" };

test("one rebate at the opening edge, reversed at the exterior and repeated at a mullion", () => {
  assert.deepEqual(woodCrossSection({ ...frame, sponning: rebate }),
    [[0, 0], [67, 0], [67, 63], [50, 63], [50, 114], [0, 114]]);
  const spec = { ...frame, applicableAs: ["divider"], sponning: { ...rebate, position: "buiten" } };
  assert.deepEqual(woodCrossSection(spec),
    [[17, 0], [50, 0], [50, 51], [67, 51], [67, 114], [0, 114], [0, 51], [17, 51]]);
  assert.equal(woodSectionFits({ ...spec, width: 34 }), false);
});

test("rebates on both faces must leave timber between the pockets", () => {
  const sp = { ...rebate, position: "midden", sponningType: "dubbele_sponning", secondWidth: 29, secondDepth: 12 };
  assert.deepEqual(woodCrossSection({ ...frame, sponning: sp }),
    [[0, 0], [55, 0], [55, 29], [67, 29], [67, 63], [50, 63], [50, 114], [0, 114]]);
  assert.equal(woodSectionFits({ ...frame, depth: 79, sponning: sp }), false);
});

test("catalogue tilt-turn sash has a frame-side lip and an open 51 × 17 glass pocket", () => {
  const profiles = JSON.parse(readFileSync(new URL("../../../profiles/wood/raamhout.json", import.meta.url)));
  const sash = profiles.find(p => p.id === "raamhout-69x90-opdek");
  const contour = woodCrossSection(sash);
  assert.deepEqual(contour, sash.crossSection);
  assert.deepEqual(contour.slice(0, 6), [[13, 0], [69, 0], [69, 39], [52, 39], [52, 90], [0, 90]]);
});

test("changing rebate parameters updates geometry, units, history and serialized profile", () => {
  profileEditor.newProfile();
  assert.equal(profileEditor.updateSponning({ ...rebate, width: 57, depth: 20 }), true);
  assert.equal(profileEditor.scaleToSize(78, 139), true);
  let exported = profileEditor.getExportData();
  assert.equal(exported.glazingRebate, 57);
  assert.equal(exported.sightline, 58);
  assert.deepEqual(exported.crossSection, [[0, 0], [78, 0], [78, 82], [58, 82], [58, 139], [0, 139]]);
  profileEditor.undo();
  profileEditor.undo();
  assert.equal(profileEditor.getExportData().glazingRebate, 51);
  profileEditor.redo();
  profileEditor.redo();
  profileEditor.loadProfile(JSON.parse(JSON.stringify(exported)));
  profileEditor.mirrorV();
  assert.equal(profileEditor.getExportData().sponning.position, "buiten");
  profileEditor.updateSponning({ ...profileEditor.getExportData().sponning, width: 45 });
  exported = profileEditor.getExportData();
  assert.deepEqual(exported.crossSection, [[0, 0], [58, 0], [58, 45], [78, 45], [78, 139], [0, 139]]);
});

test("hand-edited and imported outlines survive updates to their rebate description", () => {
  profileEditor.newProfile();
  profileEditor.moveVertex(0, 2, 0);
  const outline = profileEditor.getExportData().crossSection;
  profileEditor.updateSponning({ ...rebate, width: 45 });
  assert.deepEqual(profileEditor.getExportData().crossSection, outline);
  profileEditor.loadProfile({ ...frame, material: "wood", crossSection: outline, glazingRebate: 39, sightline: 61 });
  assert.equal(profileEditor.getExportData().glazingRebate, 39);
  assert.equal(profileEditor.getExportData().sponning, null);
});
