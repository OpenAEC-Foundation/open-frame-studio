import { segmentPath } from "./contour2d.js";
import { dividerPieceLabel, memberLabel, timberMemberType } from "./labels.js";

export function endLabel(t, vertical, side) {
  if (vertical) return t(side === "start" ? "divider.top" : "divider.bottom");
  return t(side === "start" ? "divider.left" : "divider.right");
}

export function archJointLabel(t, corner, kind) {
  const group = kind === "ring_seam" ? "ringJoint" : "archJoint";
  return t(group + (corner === 0 ? ".left" : ".right"));
}

/** A cut against masonry can remove either half of an original radial seam. */
export function ringSeamCorners(geometry) {
  const halves = geometry?.curvedTimber?.filter(p => p.role === "arc") ?? [];
  const hasEnd = (side, corner) => halves.some(p => p.profileSide === side &&
    (p.startCorner === corner || p.endCorner === corner));
  return [0, 1].filter(corner => hasEnd("top", corner) && hasEnd("bottom", corner));
}

export function jointMemberLabel(t, geometry, member) {
  if (!member) return "";
  if (member.role) return memberLabel(t, member.arc ? "frame_arc" : timberMemberType(member), member.profileSide);
  return dividerPieceLabel(t, member, geometry.curvedDividerTimber);
}

export function cutRows(production) {
  const rows = (production.cutList ?? []).map(cut => ({ ...cut, mark: production.kozijnMark, mortises: [] }));
  const byId = new Map(rows.map(row => [row.pieceId, row]));
  for (const joint of production.dividerJoints ?? []) {
    const source = byId.get(joint.pieceId);
    const target = byId.get(joint.receiverPieceId);
    if (source && target && joint.tenon) {
      source[joint.end === "start" ? "startReceiver" : "endReceiver"] = target.pieceId;
      target.mortises.push({ id: source.pieceId, vertical: source.memberType === "divider_v", side: joint.end, tenon: joint.tenon });
    }
  }
  for (const joint of production.archJoints ?? []) {
    const donor = byId.get(joint.pieceId);
    const host = byId.get(joint.receiverPieceId);
    if (donor) {
      donor[joint.end + "ArchJoint"] = { ...joint, received: false, mate: joint.receiverPieceId };
      if (joint.issue) donor.unknownGross = true;
    }
    if (host) {
      host[joint.receiverEnd + "ArchJoint"] = { ...joint, received: true, mate: joint.pieceId };
      if (joint.tenon) host.mortises.push({ id: joint.pieceId, corner: joint.corner, kind: joint.kind, tenon: joint.tenon });
    }
  }
  return rows;
}

function receiverMember(geometry, receiver) {
  if (receiver?.kind === "frame") {
    return geometry.curvedTimber?.find(p => p.role === receiver.role && p.index === receiver.index);
  }
  if (receiver?.kind === "divider") {
    return geometry.curvedDividerTimber?.find(p => p.vertical === receiver.vertical &&
      p.sourceIndex === receiver.sourceIndex && p.pieceIndex === receiver.pieceIndex);
  }
}

/** Resolve the physical pieces so selecting either side shows the same joint. */
export function selectedJoints(geometry, selected) {
  if (!geometry || !selected) return [];
  const selectedPiece = selected.type === "divider_v" || selected.type === "divider_h"
    ? receiverMember(geometry, { kind: "divider", vertical: selected.type === "divider_v",
        sourceIndex: selected.index, pieceIndex: selected.pieceIndex ?? 0 })
    : geometry.curvedTimber?.find(p => timberMemberType(p) === selected.type && p.index === selected.index);
  if (!selectedPiece) return [];
  const joints = [];
  for (const piece of geometry.curvedDividerTimber ?? []) {
    for (const side of ["start", "end"]) {
      const tenon = piece[side]?.tenon;
      if (!tenon) continue;
      const receiver = receiverMember(geometry, tenon.receiver);
      if (piece === selectedPiece || receiver === selectedPiece) {
        joints.push({ piece, side, tenon, receiver, received: receiver === selectedPiece });
      }
    }
  }
  for (const joint of geometry.archJoints ?? []) {
    const findMember = ref => geometry.curvedTimber?.find(p => p.role === ref.role && p.index === ref.index);
    const piece = findMember(joint.source);
    const receiver = findMember(joint.receiver);
    if (selectedPiece === piece || selectedPiece === receiver) {
      joints.push({ ...joint, piece, receiver, received: receiver === selectedPiece });
    }
  }
  return joints;
}

export function receiverLabel(t, geometry, joint) {
  return jointMemberLabel(t, geometry, joint?.receiver);
}

// Elevation, in model mm. All tenons in one depth stack share this footprint.
export function jointPaths({ tenon, piece }) {
  const edge = tenon?.shoulder;
  if (!edge?.length) return null;
  if (tenon.tip) {
    const [a, b] = tenon.tip;
    const [c, d] = tenon.bottom;
    return {
      tenon: `M${edge[0]} L${edge[1]} L${b} L${a} Z`,
      socket: `M${a} L${c} L${d} L${b}`,
    };
  }
  const project = (x, axis) => piece.vertical ? [x, axis] : [axis, x];
  const [a, b] = tenon.crossRangeMm;
  const tip = tenon.tipAxisMm, bottom = tenon.bottomAxisMm;
  return {
    tenon: segmentPath(edge) + ` L${project(b, tip)} L${project(a, tip)} Z`,
    socket: `M${project(a, tip)} L${project(a, bottom)} L${project(b, bottom)} L${project(b, tip)}`,
  };
}
