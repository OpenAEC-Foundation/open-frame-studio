import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { resolveProfileSnapshot } from "./profileSnapshot.js";

function catalogProfile(file, id) {
  const records = JSON.parse(readFileSync(new URL(`../../../profiles/${file}`, import.meta.url)));
  return records.find(p => p.id === id);
}

test("PVC system dimensions preserve the combined sightline and closing clearances", () => {
  const veka = resolveProfileSnapshot(catalogProfile("pvc/veka.json", "veka-softline82"));
  assert.equal(veka.aanzichtbreedte + veka.sashFaceWidth - veka.sashOverlap, 124);
  assert.equal(veka.sashFaceWidth, 84);
  assert.equal(veka.sashClearance, 12);
  assert.equal(veka.meetingGap, 8);
  assert.equal(veka.sealingLevels, 3);
  assert.equal(veka.glaslatHoogte, 25);
});

test("aluminium sash uses the combined sightline, shadow joint and explicit overlap", () => {
  const ml8 = resolveProfileSnapshot(catalogProfile("aluminum/reynaers.json", "reynaers-masterline8"));
  assert.equal(ml8.sashOverlap, 6);
  assert.equal(ml8.aanzichtbreedte + ml8.sashFaceWidth - ml8.sashOverlap, 97);
  assert.equal(ml8.sashBuildDepth, 87);
  assert.equal(ml8.sponningDiepte, null);
});

test("an incomplete system profile keeps unknown dimensions and seal count open", () => {
  const snapshot = resolveProfileSnapshot({ material: "pvc", width: 73, sealingLevels: 8 });
  for (const key of ["sealingLevels", "sashClearance", "sashOverlap", "meetingGap", "sashFaceWidth"]) {
    assert.equal(snapshot[key], null);
  }
});
