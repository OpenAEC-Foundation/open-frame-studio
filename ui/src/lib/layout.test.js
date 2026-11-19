import test from "node:test";
import assert from "node:assert/strict";
import { gridToLayout, layoutToRects } from "./layout.js";

test("existing arched transoms keep their elevation when converting a weighted grid", () => {
  const frame = { frameWidth: 67, outerHeight: 1500 };
  const grid = { columns: [{ size: 1066 }], rows: [{ size: 649.5 }, { size: 649.5 }] };
  const inner = { x: 67, y: 67, width: 1066, height: 1366 };
  const geometry = { curvedFrame: {}, hDividers: [{ x: 67, y: 833, width: 1066, height: 67 }] };
  const tree = gridToLayout({ grid, frame }, geometry);
  const drawing = layoutToRects(tree, inner, 67);
  assert.deepEqual(drawing.dividers[0].rect, geometry.hDividers[0]);
});
