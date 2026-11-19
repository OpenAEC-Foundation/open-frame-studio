import test from "node:test";
import assert from "node:assert/strict";
import { archJointLabel, cutRows, jointPaths, ringSeamCorners, selectedJoints } from "./dividerJoinery.js";

test("left and right ring joints stay on the correct ends of opposite half-rings", () => {
  const upper = { role: "arc", index: 0, profileSide: "top", startCorner: 0, endCorner: 1 };
  const lower = { role: "arc", index: 1, profileSide: "bottom", startCorner: 1, endCorner: 0 };
  const geometry = { curvedTimber: [upper, lower] };
  assert.deepEqual(ringSeamCorners(geometry), [0, 1]);
  assert.deepEqual(ringSeamCorners({ curvedTimber: [upper, { ...lower, endCorner: null }] }), [1]);
  const seam = (corner, end, receiverEnd, length) => ({
    kind: "ring_seam", pieceId: "R-BO2", receiverPieceId: "R-BO1", source: lower, receiver: upper,
    corner, end, receiverEnd, tenon: { tenonLengthMm: length, mortiseDepthMm: length + 2 },
  });
  const archJoints = [seam(0, "end", "start", 22), seam(1, "start", "end", 31)];
  const [top, bottom] = cutRows({ cutList: [{ pieceId: "R-BO1" }, { pieceId: "R-BO2" }], archJoints });
  assert.equal(top.startArchJoint.tenon.mortiseDepthMm, 24);
  assert.equal(top.endArchJoint.tenon.mortiseDepthMm, 33);
  assert.equal(bottom.startArchJoint.tenon.tenonLengthMm, 31);
  assert.equal(bottom.endArchJoint.tenon.tenonLengthMm, 22);
  assert.deepEqual(top.mortises.map(m => archJointLabel(key => key, m.corner, m.kind)), ["ringJoint.left", "ringJoint.right"]);
  const selected = selectedJoints({ ...geometry, archJoints }, { type: "frame_top", index: 0 });
  assert.deepEqual(selected.map(j => [j.corner, j.received]), [[0, true], [1, true]]);
});

test("a springing on a split arch is selectable from either member without dividers", () => {
  const arch = { role: "arc", index: 1, profileSide: "top" };
  const jamb = { role: "right", index: 0, profileSide: "right" };
  const tenon = {
    shoulder: [[70, 38], [90, 30]], tip: [[76, 48], [96, 40]],
    bottom: [[77, 50], [97, 42]], zFromInsideMm: [[35, 47]],
  };
  const joint = { corner: 1, source: arch, receiver: jamb, tenon };
  const geometry = { curvedTimber: [{ ...arch, index: 0 }, arch, jamb], archJoints: [joint] };
  assert.deepEqual(selectedJoints(geometry, { type: "frame_top", index: 0 }), []);
  const [source] = selectedJoints(geometry, { type: "frame_top", index: 1 });
  const [receiver] = selectedJoints(geometry, { type: "frame_right", index: 0 });
  assert.equal(source.received, false);
  assert.equal(receiver.received, true);
  assert.deepEqual(jointPaths(source), jointPaths(receiver));
  assert.deepEqual(jointPaths(source), {
    tenon: "M70,38 L90,30 L96,40 L76,48 Z", socket: "M76,48 L77,50 L97,42 L96,40",
  });
});

test("an unresolved springing identifies both ends and invalidates the donor blank only", () => {
  const joint = { pieceId: "B-BO2", receiverPieceId: "B-SR", end: "end", receiverEnd: "start",
    corner: 1, issue: "insufficient_stock" };
  const [arch, jamb] = cutRows({ cutList: ["B-BO2", "B-SR"].map(pieceId => ({ pieceId })), archJoints: [joint] });
  assert.equal(arch.endArchJoint.mate, "B-SR");
  assert.equal(arch.unknownGross, true);
  assert.equal(jamb.startArchJoint.mate, "B-BO2");
  assert.equal(jamb.unknownGross, undefined);
  assert.equal(jamb.mortises.length, 0);
  assert.equal(jointPaths({ tenon: null }), null);
});

test("a selected half-ring shows only the mortise it receives", () => {
  const upper = { role: "arc", index: 0, profileSide: "top" };
  const lower = { role: "arc", index: 1, profileSide: "bottom" };
  const tenon = { receiver: { kind: "frame", role: "arc", index: 1 }, crossRangeMm: [40,65],
    shoulder: [{ kind: "line", from: [40,70], to: [65,73] }], tipAxisMm: 99, bottomAxisMm: 101 };
  const piece = { vertical: true, sourceIndex: 0, pieceIndex: 1, end: { tenon } };
  const geometry = { curvedTimber: [upper, lower], curvedDividerTimber: [piece] };
  const received = selectedJoints(geometry, {type: "frame_bottom", index: 1});
  assert.equal(received.length, 1);
  assert.equal(received[0].received, true);
  assert.deepEqual(jointPaths(received[0]), {
    tenon: "M40,70 L65,73 L65,99 L40,99 Z", socket: "M40,99 L40,101 L65,101 L65,99",
  });
  assert.equal(selectedJoints(geometry, {type: "frame_top", index: 0}).length, 0);
  assert.equal(selectedJoints(geometry, {type: "divider_v", index: 0, pieceIndex: 0}).length, 0);
});

test("a receiving transom retains the positions of both donor ends", () => {
  const cuts = ["A-TS1.1","A-TS1.2","A-TD1"].map(pieceId => ({
    pieceId, memberType: pieceId.includes("TS") ? "divider_v" : "divider_h",
  }));
  const joints = cuts.slice(0,2).map((c, i) => ({
    pieceId: c.pieceId, receiverPieceId: "A-TD1", end: i === 0 ? "end" : "start", tenon: {},
  }));
  const rows = cutRows({ kozijnMark: "A", cutList: cuts, dividerJoints: joints });
  assert.equal(rows[0].endReceiver, "A-TD1");
  assert.equal(rows[1].startReceiver, "A-TD1");
  assert.deepEqual(rows[2].mortises.map(m => [m.id, m.side]), [["A-TS1.1","end"], ["A-TS1.2","start"]]);
});
