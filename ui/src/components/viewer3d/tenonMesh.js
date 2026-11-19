import { contourPoints } from "./curvedMesh.js";
import { meshFaces, stockMesh } from "./dividerMesh.js";

// Model coordinates have x right and y down. The core supplies the stock
// identities, insertion axes and depth bands for the finished connections.
const EPS = 1e-6;

function halfFace(face, axis, limit, sign) {
  const out = [];
  for (let i = 0; i < face.length; i++) {
    const a = face[i], b = face[(i + 1) % face.length];
    const da = sign * (a[axis] - limit), db = sign * (b[axis] - limit);
    if (da <= EPS) out.push(a);
    if ((da > EPS && db < -EPS) || (da < -EPS && db > EPS)) {
      out.push(a.map((v, k) => v + da / (da - db) * (b[k] - v)));
    }
  }
  return out;
}

function removeBox(faces, tool) {
  const {lo,hi,toLocal,toModel} = tool;
  const [min,max] = tool.bounds;
  const kept = [];
  for (const face of faces) {
    if (min.some((v, k) => face.every(p => p[k] < v) || face.every(p => p[k] > max[k]))) {
      kept.push(face);
      continue;
    }
    let interior = toLocal ? face.map(toLocal) : face;
    for (let axis = 0; axis < 3 && interior.length >= 3; axis++) {
      for (const [limit, sign] of [[lo[axis], -1], [hi[axis], 1]]) {
        if (!interior.some(p => sign * (p[axis] - limit) > EPS)) continue;
        const outside = halfFace(interior, axis, limit, -sign);
        if (outside.length >= 3) kept.push(toModel ? outside.map(toModel) : outside);
        interior = halfFace(interior, axis, limit, sign);
      }
    }
  }
  return kept;
}

// Use the faceting of the *whole* shoulder at the tenon root. Tessellating
// only the short arc between the cheeks could otherwise leave a fine crack.
function shoulderAt(loop, cross, atStart, [low, high]) {
  const axis = 1 - cross;
  const stations = [low, ...loop.map(p => p[cross]).filter(t => t > low && t < high), high];
  return [...new Set(stations)].sort((a,b) => a-b).map(t => {
    const hits = [];
    for (let i = 0; i < loop.length; i++) {
      const a = loop[i], b = loop[(i+1) % loop.length];
      if (Math.abs(a[cross] - t) < EPS) hits.push(a[axis]);
      if ((a[cross] - t) * (b[cross] - t) < 0) {
        hits.push(a[axis] + (b[axis]-a[axis])*(t-a[cross])/(b[cross]-a[cross]));
      }
    }
    return [t, atStart ? Math.min(...hits) : Math.max(...hits)];
  });
}

function endTool(shoulder, axis, stop, zRange, sign, cavity) {
  const lo = [], hi = [], faces = [];
  const cross = 1-axis, [first, last] = [shoulder[0], shoulder.at(-1)];
  const depths = shoulder.map(p => p[1]);
  // Enter the cutter from air. The slight extension clears the independently
  // sampled lip and core faces and never changes the measured bottom position.
  const mouth = sign < 0 ? Math.max(...depths)+0.2 : Math.min(...depths)-0.2;
  lo[axis] = Math.min(mouth, stop); hi[axis] = Math.max(mouth, stop);
  lo[cross] = first[0]; hi[cross] = last[0];
  [lo[2],hi[2]] = zRange;
  const xyz = (t,a,z) => axis === 1 ? [t,a,z] : [a,t,z];
  function face(points, normalAxis, direction) {
    const a = points[0], b = points[1], c = points[2];
    const i = (normalAxis+1)%3, j = (normalAxis+2)%3;
    const normal = (b[i]-a[i])*(c[j]-a[j])-(b[j]-a[j])*(c[i]-a[i]);
    // stockMesh reflects y, so reverse a positive model winding for exteriors.
    if (normal * direction * (cavity ? -1 : 1) > 0) points.reverse();
    faces.push(points);
  }
  for (const [z, direction] of [[lo[2],-1],[hi[2],1]]) {
    for (let i=1; i<shoulder.length; i++) {
      const [a,s] = shoulder[i-1], [b,t] = shoulder[i];
      face([xyz(a,s,z),xyz(b,t,z),xyz(b,stop,z),xyz(a,stop,z)],2,direction);
    }
  }
  for (const [[t,a], direction] of [[first,-1],[last,1]]) {
    face([xyz(t,a,lo[2]),xyz(t,stop,lo[2]),xyz(t,stop,hi[2]),xyz(t,a,hi[2])],cross,direction);
  }
  face([xyz(first[0],stop,lo[2]),xyz(last[0],stop,lo[2]),
        xyz(last[0],stop,hi[2]),xyz(first[0],stop,hi[2])],axis,sign);
  return {lo, hi, faces, bounds:[lo,hi]};
}

// An arch foot is machined along the specified insertion direction. With
// inclined jambs its cheeks, tip and bottom all rotate in elevation.
function orientedEndTool(shoulderFace, direction, stop, zRange, cavity) {
  const [dx,dy] = direction, [ox,oy] = shoulderFace[0];
  const toLocal = ([x,y,z]) => [dx*(x-ox)+dy*(y-oy), dx*(y-oy)-dy*(x-ox),z];
  const toModel = ([a,t,z]) => [ox+dx*a-dy*t, oy+dy*a+dx*t,z];
  const shoulder = shoulderFace.map(p => {
    const [a,t] = toLocal(p);
    return [t,a];
  }).sort((p,q) => p[0]-q[0]);
  const tool = endTool(shoulder,0,toLocal(stop)[0],zRange,1,cavity);
  const corners = [tool.lo[0],tool.hi[0]].flatMap(a =>
    [tool.lo[1],tool.hi[1]].map(t => toModel([a,t,0])));
  const xs = corners.map(p => p[0]), ys = corners.map(p => p[1]);
  return {...tool, toLocal, toModel, faces:tool.faces.map(f => f.map(toModel)),
    bounds:[[Math.min(...xs),Math.min(...ys),tool.lo[2]], [Math.max(...xs),Math.max(...ys),tool.hi[2]]]};
}

function ownedBy(mesh, part) {
  if (part.kind === "frame") {
    const timber = mesh.userData.timber;
    return mesh.userData.role === "frame-member" && timber?.role === part.role && timber.index === part.index;
  }
  const d = mesh.userData.divider;
  return d && d.vertical === part.vertical && d.sourceIndex === part.sourceIndex && d.pieceIndex === part.pieceIndex;
}

export function machineTimberJoints(THREE, group, members, frameInner, archJoints) {
  const stock = group.children.filter(m => m.isMesh && ["frame-member","divider"].includes(m.userData.role));
  const cuts = new Map();
  function cut(owner, tool, operation) {
    for (const mesh of owner) {
      if (!mesh.geometry.boundingBox) mesh.geometry.computeBoundingBox();
      const {min,max} = mesh.geometry.boundingBox, [lo,hi] = tool.bounds;
      if (max.x < lo[0] || min.x > hi[0] || -min.y < lo[1] || -max.y > hi[1]
        || max.z + mesh.position.z < lo[2] || min.z + mesh.position.z > hi[2]) continue;
      if (!cuts.has(mesh)) cuts.set(mesh, {boxes:[], walls:[]});
      cuts.get(mesh).boxes.push(tool);
    }
    const primary = owner.find(m => cuts.get(m)?.boxes.includes(tool));
    if (primary) {
      cuts.get(primary).walls.push(...tool.faces);
      (primary.userData.joinery ??= []).push(operation);
    }
  }
  for (const member of members ?? []) {
    const donor = stock.filter(m => ownedBy(m, member));
    if (!donor.length) continue;
    const axis = member.vertical ? 1 : 0;
    for (const end of ["start","end"]) {
      const joint = member[end]?.tenon;
      if (!joint) continue;
      const receiver = stock.filter(m => ownedBy(m, joint.receiver));
      if (!receiver.length) continue;
      const start = end === "start";
      const shoulder = shoulderAt(contourPoints(member.outline), 1-axis, start, joint.crossRangeMm);
      if (shoulder.some(p => !Number.isFinite(p[1]))) continue;
      const source = {vertical:member.vertical, sourceIndex:member.sourceIndex, pieceIndex:member.pieceIndex, end};
      for (const [near,far] of joint.zFromInsideMm) {
        const z = [frameInner-far,frameInner-near], sign = start ? -1 : 1;
        cut(receiver, endTool(shoulder,axis,joint.bottomAxisMm,z,sign,true), {...source,kind:"mortise"});
        cut(donor, endTool(shoulder,axis,joint.tipAxisMm,z,sign,false), {...source,kind:"tenon"});
      }
    }
  }
  for (const {kind,corner,source,receiver,tenon} of archJoints ?? []) {
    if (!tenon) continue;
    const donorStock = stock.filter(m => ownedBy(m, {kind:"frame",...source}));
    const receiverStock = stock.filter(m => ownedBy(m, {kind:"frame",...receiver}));
    if (!donorStock.length || !receiverStock.length) continue;
    const joint = {jointKind:kind,corner,source,receiver,direction:tenon.direction,tenonLengthMm:tenon.tenonLengthMm};
    for (const [near,far] of tenon.zFromInsideMm) {
      const z = [frameInner-far,frameInner-near];
      cut(donorStock, orientedEndTool(tenon.shoulder,tenon.direction,tenon.tip[0],z,false),
        {...joint,kind:"tenon"});
      cut(receiverStock, orientedEndTool(tenon.shoulder,tenon.direction,tenon.bottom[0],z,true),
        {...joint,kind:"mortise"});
    }
  }
  for (const [mesh,{boxes,walls}] of cuts) {
    let faces = meshFaces(mesh);
    for (const tool of boxes) faces = removeBox(faces,tool);
    const finished = stockMesh(THREE,faces.concat(walls),mesh.material,mesh.userData.role);
    mesh.geometry.dispose();
    if (!finished) {
      group.remove(mesh);
      continue;
    }
    mesh.geometry = finished.geometry;
    for (const child of mesh.children) child.position.z += mesh.position.z;
    mesh.position.z = 0;
  }
}

// Move every mesh of a jointed arch together, including the rebate and web.
// Dividers move to the viewer's side of the frame in two separate layers.
const assembledViews = new WeakMap();

export function inspectTimberJoints(group, enabled, frameDepth) {
  const original = assembledViews.get(group);
  if (!enabled) {
    if (original) {
      for (const [mesh,y,z,visible] of original) {
        mesh.position.y = y;
        mesh.position.z = z;
        mesh.visible = visible;
      }
      assembledViews.delete(group);
    }
    return;
  }
  if (original || !group.children.some(m => m.userData.joinery?.length)) return;
  let separation = frameDepth;
  let ringTravel = 0;
  const ringDirections = new Map();
  const arches = new Set();
  for (const mesh of group.children.filter(m => m.userData.divider || m.userData.timber)) {
    if (!mesh.geometry.boundingBox) mesh.geometry.computeBoundingBox();
    const {min,max} = mesh.geometry.boundingBox;
    separation = Math.max(separation, max.z-min.z);
    for (const op of mesh.userData.joinery ?? []) {
      if (op.jointKind === "ring_seam") {
        // A single vertical movement releases both seams, also when the
        // upper half has the left tenon and the lower half the right one.
        const sign = Math.sign(op.direction[1]);
        ringDirections.set(op.source.index,sign);
        ringDirections.set(op.receiver.index,-sign);
        ringTravel = Math.max(ringTravel,(op.tenonLengthMm ?? 0)/2+15);
      } else if (mesh.userData.timber?.role === "arc" && op.corner != null) {
        arches.add(mesh.userData.timber.index);
      }
    }
  }
  assembledViews.set(group, group.children.map(m => [m,m.position.y,m.position.z,m.visible]));
  for (const mesh of group.children) {
    const part = mesh.userData.divider;
    const timber = mesh.userData.timber;
    if (part) mesh.position.z += (separation+35) * (part.vertical ? 2 : 1);
    else if (timber?.role === "arc" && ringDirections.has(timber.index)) {
      mesh.position.y += ringDirections.get(timber.index) * Math.max(separation,ringTravel);
    } else if (timber?.role === "arc" && arches.has(timber.index)) {
      mesh.position.y += separation;
      mesh.position.z -= separation+35;
    }
    else if (mesh.userData.role !== "frame-member") mesh.visible = false;
  }
}

export { machineTimberJoints as machineDividerJoints, inspectTimberJoints as inspectDividerJoints };
