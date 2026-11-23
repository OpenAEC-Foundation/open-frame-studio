import test from "node:test";
import assert from "node:assert/strict";
import { circleDiameter, contourPath, openingPath, sawAllowancePath } from "./contour2d.js";

const right = [70, 50], left = [30, 50], center = [50, 50];
const half = { kind: "arc", center, radius: 20, from: right, to: left, sweepAngle: Math.PI };
const circle = { segments: [half, { ...half, from: left, to: right }] };

test("the drawing extends a verified blank beyond the day line at the qualified end only", () => {
  const rail = {vertical:false, start:{segments:[{kind:"line",from:[73,600],to:[73,694]}],sawAllowanceMm:25}, end:{segments:[half]}};
  assert.equal(sawAllowancePath(rail,"start"), "M73,600 L48,600 L48,694 L73,694");
  assert.equal(sawAllowancePath(rail,"end"), "");
  const post = {vertical:true, end:{segments:[{kind:"line",from:[99,1427],to:[193,1427]}],sawAllowanceMm:25}};
  assert.equal(sawAllowancePath(post,"end"), "M99,1427 L99,1452 L193,1452 L193,1427");
});

test("full circles render as two SVG arcs and have a diameter", () => {
  assert.equal(contourPath(circle), "M70,50 A20 20 0 0 1 30,50 A20 20 0 0 1 70,50 Z");
  assert.equal(circleDiameter(circle), 40);
});

test("partially clipped circles retain the major arc and straight cut edge", () => {
  const edge = { ...half, sweepAngle: 1.5 * Math.PI, to: [50, 30] };
  const cut = { segments: [edge, { kind: "line", from: edge.to, to: right }] };
  assert.equal(contourPath(cut), "M70,50 A20 20 0 1 1 50,30 L70,50 Z");
  assert.equal(circleDiameter(cut), null);
});

test("a cell outside the contour stays empty and a legacy rectangle is drawable", () => {
  const rect = { x: 3, y: 5, width: 17, height: 19 };
  assert.equal(openingPath({ segments: [] }, rect), "");
  assert.equal(openingPath(undefined, rect), "M3,5h17v19h-17Z");
});
