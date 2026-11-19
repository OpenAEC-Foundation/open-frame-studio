import test from "node:test";
import assert from "node:assert/strict";
import * as THREE from "three";
import { buildFrameGroup } from "../components/viewer3d/frameGroup.js";
import { insetOpening } from "./glazingFit.js";
import { rebateChannels, sectionFromPoints } from "./frame3d.js";
import { woodCrossSection } from "./profileContour.js";
import { contourPoints, sweepRing, timberPlan } from "../components/viewer3d/curvedMesh.js";
import { inspectDividerJoints, machineDividerJoints } from "../components/viewer3d/tenonMesh.js";

const profile = (id, width, depth) => ({
  id, width, depth, material: "wood", applicableAs: ["frame", "sash"],
  sponning: { depth: 17, width: 51, position: "binnen" },
  crossSection: [[0,0], [width,0], [width,depth-51], [width-17,depth-51], [width-17,depth], [0,depth]],
});
const jamb = profile("jamb", 67, 114);
const sash = profile("sash", 69, 90);
const left = { x: 67, y: 67, width: 400, height: 500 };
const right = { ...left, x: 534 };
const frame = { profile: jamb, material: { wood: "meranti" }, frameWidth: 67,
  frameDepth: 114, outerWidth: 1001, outerHeight: 634 };

function fit(opening, operable, outside) {
  const sashOuter = operable ? insetOpening(opening, -14) : null;
  const day = operable ? insetOpening(sashOuter, sash.width) : opening;
  return { sashOuter, day, rebate: insetOpening(day, -17), glass: insetOpening(day, -12),
    beadHeight: 17, beadWidth: 15, beadMitered: true, beadPosition: outside ? "buiten" : "binnen" };
}

function model({ layout, operable = false, stock = jamb, vent = sash, firstBead = "binnen" }) {
  const cells = [
    { panelType: operable ? "turn" : "fixed_glass", sashProfile: vent, sashDepth: 90,
      glazing: { thicknessMm: 28 }, glaslat: { position: firstBead } },
    { panelType: "fixed_glass", glaslat: { position: "buiten" }, glazing: { thicknessMm: 24 } },
  ];
  const fill = [{ type: operable ? "raam" : "glas" }, { type: "glas", glaslat: "buiten" }];
  const kozijn = { frame: { ...frame, profile: stock }, cells, layout: layout ? { kind: "split", direction: "row",
    children: fill.map(vulling => ({ node: { kind: "leaf", vulling } })) } : undefined };
  const geometry = { outerRect: { x: 0, y: 0, width: 1001, height: 634 },
    frameRects: [{ x: 0, y: 0, width: 1001, height: 67 }, { x: 0, y: 567, width: 1001, height: 67 },
      { x: 0, y: 67, width: 67, height: 500 }, { x: 934, y: 67, width: 67, height: 500 }],
    vDividers: [{ x: 467, y: 67, width: 67, height: 500 }],
    cellRects: [left, right].map((rect, i) => ({ cellIndex: i, rect,
      ...(layout ? { vulling: fill[i] } : {}), fit: fit(rect, operable && i === 0, i === 1) })),
  };
  const group = buildFrameGroup(THREE, kozijn, geometry, [stock, vent]);
  group.updateMatrixWorld();
  return group.children;
}

// Intersect the actual extruded triangles, including their orientation, from
// outside or inside. Bboxes alone can't distinguish a rebate from solid wood.
const byRole = (meshes, role) => meshes.filter(m => m.userData.role === role);
function hit(meshes, x, y, inside = true) {
  const dir = inside ? -1 : 1;
  const ray = new THREE.Raycaster(new THREE.Vector3(x, -y, -dir * 500), new THREE.Vector3(0, 0, dir));
  return ray.intersectObjects(meshes).find(result => result.object.isMesh)?.point.z;
}
const close = (actual, expected) => assert.ok(Math.abs(actual - expected) < 0.001, `${actual} != ${expected}`);

for (const layout of [false, true]) {
  test(`opposing rebates seat glass on both sides of one divider (layout=${layout})`, () => {
    const meshes = model({ layout });
    const glass = byRole(meshes, "glass"), divider = byRole(meshes, "divider");
    assert.equal(glass.length, 2);
    close(hit(divider, 472, 280), 6);
    close(hit(glass, 472, 280, false), 8);
    close(hit(divider, 529, 280, false), -6);
    close(hit(glass, 529, 280), -8);
    close(hit(glass, 100, 280), 36);
    assert.equal(byRole(meshes, "bead").length, 8);
    assert.equal(hit(byRole(meshes, "wall"), 225, 300), undefined);
  });
}

test("movable sash overlaps the stop with a compressed seal between the solids", () => {
  const meshes = model({ operable: true });
  const frame = byRole(meshes, "frame-member"), sash = byRole(meshes, "sash");
  close(hit(frame, 64, 300), -27);
  close(hit(sash, 64, 300, false), -25);
  close(hit(byRole(meshes, "sash-seal"), 64, 300), -25);
  close(hit(sash, 64, 300), 65);
  assert.equal(hit(sash, 48, 300), undefined);
  const bounds = new THREE.Box3().setFromObject(byRole(meshes, "glass")[0]);
  close(bounds.min.x, fit(left, true).glass.x);
});

test("a sloped bed and upstand constrain the usable pocket depth", () => {
  const sill = sectionFromPoints("sill", [[0,0],[67,0],[67,61],[42,61],[50,106],[65,106],[65,114],[0,114]]);
  assert.deepEqual(rebateChannels(sill, 12), [[61,106]]);
  const [channel] = rebateChannels(sill, 21);
  close(channel[0], 61);
  close(channel[1], 83.5);
});

test("a drip groove on the outer face survives local timber sash machining", () => {
  const stock = { ...jamb, crossSection: [[0,0],[12,0],[12,5],[18,5],[18,0], ...jamb.crossSection.slice(1)] };
  const members = byRole(model({ operable: true, stock }), "frame-member");
  close(hit(members, 15, 300, false), -52);
  close(hit(members, 24, 300, false), -57);
});

test("the projecting sash lip contacts a shallow second stop", () => {
  const vent = { ...sash, sponning: { ...sash.sponning, type: "draaikiep", opdek_width: 13 } };
  vent.crossSection = woodCrossSection(vent);
  const meshes = model({ operable: true, vent });
  const frame = byRole(meshes, "frame-member");
  close(hit(frame, 55, 300), 50);
  close(hit(byRole(meshes, "sash"), 55, 300, false), 52);
  close(hit(byRole(meshes, "sash-seal"), 55, 300), 52);
  close(hit(frame, 66, 300), -27);

  const exteriorGlazing = model({ operable: true, vent, firstBead: "buiten" });
  close(hit(byRole(exteriorGlazing, "sash"), 55, 300, false), 52);
  close(hit(byRole(exteriorGlazing, "sash-seal"), 55, 300), 52);
});

test("a system profile retains its stop and interior cut lines beside a sash", () => {
  const stock = { ...jamb, material: "pvc", innerWalls: [[[2,2],[9,2],[9,15],[2,15]]] };
  const meshes = model({ operable: true, stock });
  const frame = byRole(meshes, "frame-member");
  close(hit(frame, 64, 300), 6);
  close(hit(byRole(meshes, "sash"), 64, 300, false), 8);
  assert.ok(frame.some(m => m.children.some(detail => detail.isLineSegments)));
});

test("stopped timber rebates close the head over jambs and between dissimilar cells", () => {
  const meshes = model({ operable: true });
  const frame = byRole(meshes, "frame-member");
  // Immediately above and below the butt between through rail and jamb.
  for (const y of [66.8, 67.2]) {
    close(hit(frame, 22, y), 57);
    close(hit(frame, 977, y, false), -57);
  }
  // Through head above the meeting of a movable and outside-glazed cell.
  close(hit(frame, 500, 66.8), 57);
  close(hit(frame, 500, 66.8, false), -57);
  const railClearance = hit(frame, 56, 57);
  const movingLeaf = hit(byRole(meshes, "sash"), 56, 57, false);
  close(movingLeaf - railClearance, 2);

  // The glass-side rebate on the through sash stile also stops short of
  // its ends. The jamb remains solid up to the top rail's glazing pocket.
  close(hit(byRole(meshes, "sash"), 119, 54), 65);
  const fixed = model({ operable: false });
  // Inside the actual glass corner both profiles leave all 28 mm free.
  close(hit(byRole(fixed, "frame-member"), 55.4, 55.4), 6);
  close(hit(byRole(fixed, "glass"), 55.4, 55.4, false), 8);
});

const line = (from, to) => ({kind: "line", from, to});
function arc(center, radius, startAngle, sweepAngle) {
  const at = a => [center[0] + radius*Math.cos(a), center[1] + radius*Math.sin(a)];
  return {kind: "arc", center, radius, startAngle, sweepAngle, from: at(startAngle), to: at(startAngle+sweepAngle)};
}
function circle(inset) {
  return { segments: [arc([500,500], 500-inset, 0, Math.PI), arc([500,500], 500-inset, Math.PI, Math.PI)] };
}
function segmental(inset, column) {
  const x0 = column === 1 ? 601+inset : inset;
  const x1 = column === 0 ? 668-inset : 1000-inset;
  const radius = 625-inset;
  const a = -Math.acos((x0-500)/radius), b = -Math.acos((x1-500)/radius);
  const head = arc([500,625], radius, a, b-a);
  const bottomLeft = [x0,1400-inset], bottomRight = [x1,1400-inset];
  return { segments: [head, line(head.to,bottomRight), line(bottomRight,bottomLeft), line(bottomLeft,head.from)] };
}
function bounds(contour) {
  const p = contourPoints(contour);
  const x = p.map(v => v[0]), y = p.map(v => v[1]);
  return { x: Math.min(...x), y: Math.min(...y), width: Math.max(...x)-Math.min(...x), height: Math.max(...y)-Math.min(...y) };
}
function curvedModel(path, divided, operable = false, members, overrides = {}, configureJob, dividerGeometry) {
  const outer = path(0), day = path(67);
  const jobs = (divided ? [0,1] : [undefined]).map((column, i) => {
    const make = inset => path(inset,column);
    const d = operable ? 122 : 67;
    const contours = { day: make(d), rebate: make(d-17), glass: make(d-12), beadInner: make(d),
      rebateHeight: 17, edgeClearance: 5, sashWidth: operable ? 69 : 0, sashOverlap: operable ? 14 : 0 };
    if (operable) contours.sashOuter = make(53);
    const openingContour = make(67), side = i === 1 ? "buiten" : "binnen";
    const fit = { contours, day: bounds(contours.day), rebate: bounds(contours.rebate), glass: bounds(contours.glass),
      sashOuter: contours.sashOuter && bounds(contours.sashOuter), beadWidth: 15, beadHeight: 17, glassThickness: 24, beadPosition: side };
    const job = { rect: bounds(openingContour), openingContour, fit, cellIndex: i,
      cell: { panelType: operable ? "turn" : "fixed_glass", glaslat: { position: side }, sashProfile: sash } };
    configureJob?.(job, make);
    return job;
  });
  const dividerStock = dividerGeometry?.stock;
  const group = buildFrameGroup(THREE, { frame:{...frame,...overrides}, cells: jobs.map(j => j.cell),
    grid: {columns:[{}, {dividerProfile: dividerStock}],rows:[{}, {dividerProfile: dividerStock}]} },
    { curvedFrame: { outer, day }, curvedTimber:members, archJoints:dividerGeometry?.archJoints,
      cellRects: jobs, outerRect: bounds(outer),
      curvedDividerTimber:dividerGeometry?.members,
      vDividers: divided && !dividerGeometry?.transom ? [{ x: 601, y: 67, width: 67, height: 1266 }] : [],
      hDividers: dividerGeometry?.transom ? [dividerGeometry.transom] : [] }, [jamb,...jobs.map(j=>j.cell.sashProfile),overrides.bottomProfile,overrides.sillProfile,dividerStock].filter(Boolean));
  group.updateMatrixWorld();
  return group.children;
}

function arcTimber(outer, inner, index = 0) {
  return { role:"arc", index, profileSide:index ? "bottom" : "top", outerEdge:outer,
    startCut:[outer.from,inner.from], endCut:[outer.to,inner.to], outline:{ segments:[outer,line(outer.to,inner.to),
      arc(inner.center,inner.radius,inner.startAngle+inner.sweepAngle,-inner.sweepAngle),line(inner.from,outer.from)] } };
}
function straightTimber(role, startCut, endCut, points) {
  return { role, profileSide:role, index:0, startCut,endCut, outline:{segments:points.map((a,i) => line(a,points[(i+1)%points.length]))} };
}
function archStock() {
  const head = arcTimber(segmental(0).segments[0],segmental(67).segments[0]);
  const left = [[0,1333],[67,1333]], right = [[1000,1333],[933,1333]];
  return [head,straightTimber("left",head.startCut,left,[head.startCut[0],head.startCut[1],left[1],left[0]]),
    straightTimber("right",head.endCut,right,[head.endCut[0],right[0],right[1],head.endCut[1]]),
    straightTimber("bottom",left,right,[left[0],left[1],right[1],right[0],[1000,1400],[0,1400]])];
}

test("both arch feet fit their flat shoulders, including an oblique tenon from the arch into the right jamb", () => {
  const members=archStock(), arch=members[0];
  const section=(p,q,t) => p.map((v,i) => v+t*(q[i]-v));
  const shift=(p,axis,distance) => p.map((v,i) => v+distance*axis[i]);
  const stock=(meshes,part) => meshes.filter(m=>m.userData.timber?.role===part.role && m.userData.timber.index===part.index);
  const cast=(meshes,p,z,direction) => {
    const ray=new THREE.Raycaster(new THREE.Vector3(p[0],-p[1],z),new THREE.Vector3(direction[0],-direction[1],0));
    const hit=ray.intersectObjects(meshes,false)[0];
    return hit && [hit.point.x,-hit.point.y];
  };
  const verify=(actual,expected) => {
    assert.ok(actual);
    close(actual[0],expected[0]); close(actual[1],expected[1]);
  };
  const archJoints=[arch.startCut,arch.endCut].map((cut,corner) => {
    const head={role:"arc",index:0}, jamb={role:corner ? "right" : "left",index:0};
    const direction=corner ? [0.6,0.8] : [0,-1];
    const shoulder=[0.24,0.53].map(t=>section(...cut,t)), center=section(...shoulder,0.5);
    const plane=depth => shoulder.map(p=>shift(p,direction,depth-(p[0]-center[0])*direction[0]-(p[1]-center[1])*direction[1]));
    return {corner,source:corner ? head : jamb,receiver:corner ? jamb : head,
      tenon:{shoulder,direction,tip:plane(18),bottom:plane(20),zFromInsideMm:[[22,34],[69,81]]}};
  });
  const plain=curvedModel(segmental,false,false,members);
  const finished=curvedModel(segmental,false,false,members,{},null,{archJoints});
  for (const {source,receiver,tenon:q} of archJoints) {
    const backwards=q.direction.map(n=>-n);
    for (const t of [0.05,0.5,0.95]) {
      const mouth=section(...q.shoulder,t), tip=section(...q.tip,t), bottom=section(...q.bottom,t);
      for (const z of [29,-18]) {
        verify(cast(stock(plain,receiver),shift(mouth,backwards,1),z,q.direction),mouth);
        verify(cast(stock(finished,receiver),shift(mouth,backwards,1),z,q.direction),bottom);
        verify(cast(stock(finished,source),shift(bottom,q.direction,0.5),z,backwards),tip);
      }
      verify(cast(stock(finished,receiver),shift(mouth,backwards,1),0,q.direction),mouth);
      verify(cast(stock(finished,source),shift(mouth,q.direction,1),0,backwards),mouth);
    }
  }
  const group=finished[0].parent;
  inspectDividerJoints(group,true,114);
  const head=stock(finished,{role:"arc",index:0}), left=stock(finished,{role:"left",index:0});
  assert.ok(head.every(m=>m.position.y>=114 && m.position.z < -114));
  assert.ok(left.every(m=>m.position.y===0 && m.position.z===0));
  inspectDividerJoints(group,false,114);
  assert.ok(head.every(m=>m.position.y===0 && m.position.z===0));
});

test("the upper and lower semicircular timbers meet at two square saw faces and select their own stock", () => {
  const outer = circle(0).segments, inner = circle(67).segments;
  const members = [arcTimber(outer[1],inner[1],0), arcTimber(outer[0],inner[0],1)];
  const sillProfile = profile("normal-sill",67,140);
  const stock = (group,i) => group.filter(m => m.userData.timber?.role === "arc" && m.userData.timber.index === i);
  for (const explicitBottom of [false,true]) {
    const overrides = { sillProfile, ...(explicitBottom ? { bottomProfile:profile("lower-half",67,128) } : {}) };
    const meshes = curvedModel(circle,false,false,members,overrides);
    for (const x of [23,977]) {
      close(hit(stock(meshes,0),x,499.6,false),-57);
      assert.equal(hit(stock(meshes,0),x,500.4),undefined);
      close(hit(stock(meshes,1),x,500.4,false),explicitBottom ? -71 : -57);
      assert.equal(hit(stock(meshes,1),x,499.6),undefined);
    }
    assert.equal(stock(meshes,0).length,1);
    assert.equal(stock(meshes,1).length,1);
    for (const mesh of [...stock(meshes,0),...stock(meshes,1)]) {
      const seam = mesh.children.find(child => child.userData.role === "timber-joint");
      assert.ok(seam);
      const coords = seam.geometry.getAttribute("position");
      for (let i=0; i<coords.count; i++) close(coords.getY(i),-500);
    }
  }
});

test("opposed tenons on a round frame close in a single motion and retain square shoulders", () => {
  const outer=circle(0).segments, inner=circle(67).segments;
  const upper=arcTimber(outer[1],inner[1],4), lower=arcTimber(outer[0],inner[0],7);
  upper.profileSide="top";
  const member=index=>({role:"arc",index});
  const archJoints=[0,1].map(corner=>{
    const x=corner ? 966.5 : 33.5, direction=corner ? 1 : -1;
    const side=depth=>[[x-12.5,500+direction*depth],[x+12.5,500+direction*depth]];
    return {kind:"ring_seam",corner,source:member(corner ? upper.index : lower.index),
      receiver:member(corner ? lower.index : upper.index),tenon:{direction:[0,direction],
        shoulder:side(0),tip:side(24),bottom:side(26),tenonLengthMm:24,zFromInsideMm:[[24,36],[72,84]]}};
  });
  const meshes=curvedModel(circle,false,false,[upper,lower],{bottomProfile:profile("round-lower",67,128)},null,{archJoints});
  const half=index=>meshes.filter(m=>m.userData.role==="frame-member" && m.userData.timber?.index===index);
  const cutY=(stock,x,y,z,sign)=>{
    const ray=new THREE.Raycaster(new THREE.Vector3(x,-y,z),new THREE.Vector3(0,-sign,0));
    const hit=ray.intersectObjects(stock,false)[0]; assert.ok(hit); return -hit.point.y;
  };
  for (const {source,receiver,tenon} of archJoints) {
    const sign=tenon.direction[1], sourceStock=half(source.index), receiverStock=half(receiver.index);
    const [left,right]=tenon.shoulder.map(p=>p[0]);
    for (const z of [27,-21]) {
      for (const x of [left+0.5,(left+right)/2,right-0.5]) {
        close(cutY(receiverStock,x,500-sign,z,sign),500+26*sign);
        close(cutY(sourceStock,x,500+28*sign,z,-sign),500+24*sign);
      }
      // Two spaced probes along each remaining shoulder measure the saw face.
      for (const x of [left-2,right+2]) {
        close(cutY(receiverStock,x,500-sign,z,sign),500);
        close(cutY(sourceStock,x,500+sign,z,-sign),500);
      }
    }
  }
  const group=meshes[0].parent, original=meshes.map(m=>[m.position.clone(),m.visible]);
  inspectDividerJoints(group,true,114);
  assert.ok(half(upper.index).every(m=>m.position.y>100));
  assert.ok(half(lower.index).every(m=>m.position.y < -100));
  assert.ok([...half(upper.index),...half(lower.index)].every(m=>m.position.z===0));
  inspectDividerJoints(group,false,114);
  meshes.forEach((m,i)=>{assert.deepEqual(m.position,original[i][0]);assert.equal(m.visible,original[i][1]);});
});

for (const divided of [false,true]) {
  test(`sawn shoulders and through sill are disjoint at all rebate depths (mullion=${divided})`, () => {
    const members = archStock();
    const meshes = curvedModel(segmental,divided,false,members,{sillProfile:profile("deep-sill",67,140)});
    const member = role => meshes.filter(m => m.userData.timber?.role === role);
    const [outer,inner] = members[0].startCut;
    const dx = inner[0]-outer[0], dy = inner[1]-outer[1], length = Math.hypot(dx,dy);
    // Two probes straddle the same flat saw at the exterior edge, in the
    // unmachined core and at the heel of the glazing pocket.
    for (const t of [0.08,0.33,0.51,0.68,0.89]) {
      const x=outer[0]+t*dx, y=outer[1]+t*dy;
      const high = [x+dy/length*0.4,y-dx/length*0.4], low = [x-dy/length*0.4,y+dx/length*0.4];
      close(hit(member("arc"),...high,false),-57);
      assert.equal(hit(member("left"),...high,false),undefined);
      close(hit(member("left"),...low,false),-57);
      assert.equal(hit(member("arc"),...low,false),undefined);
    }
    close(hit(member("left"),33,1332.6),57);
    assert.equal(hit(member("bottom"),33,1332.6),undefined);
    close(hit(member("bottom"),33,1333.4),57);
    assert.equal(hit(member("left"),33,1333.4),undefined);
    // The core between the machined bands follows the through sill's depth.
    close(hit(member("bottom"),350,1367,false),-83);
  });
}

test("round stock has a radial pocket, glazing heel, bevelled bead and compression gaskets", () => {
  const meshes = curvedModel(circle,false);
  for (const angle of [0.31,1.9,3.48,5.35]) {
    const probe = (role, radius, inside = true) => hit(byRole(meshes,role), 500+radius*Math.cos(angle), 500+radius*Math.sin(angle), inside);
    close(probe("frame-member",452), 57);
    close(probe("frame-member",440), 6);
    close(probe("glass",444,false), 8);
    assert.equal(probe("glass",446), undefined);
    close(probe("bead",440,false), 34);
    close(probe("bead",440), 49);
    close(probe("seal",435,false), 6);
    assert.equal(probe("bead",431), undefined);
  }
  assert.equal(hit(byRole(meshes,"wall"),500,500), undefined);
  close(hit(byRole(meshes,"wall"),75,75), -57);
  assert.equal(hit(byRole(meshes,"frame-member"),75,75), undefined);
});

test("segmental head and offset mullion form separate pockets for opposite glazing sides", () => {
  const meshes = curvedModel(segmental,true);
  for (const [x,inside] of [[355,true],[734,false]]) {
    const y = radius => 625-Math.sqrt(radius*radius-(x-500)**2);
    const probe = (role,radius,lookInside) => hit(byRole(meshes,role),x,y(radius),lookInside);
    close(probe("frame-member",573,inside), inside ? 6 : -6);
    close(probe("glass",569,!inside), inside ? 8 : -8);
    assert.equal(probe("glass",571,!inside), undefined);
    close(probe("bead",564,!inside), inside ? 34 : -34);
  }
  close(hit(byRole(meshes,"divider"),607,200), 6);
  close(hit(byRole(meshes,"divider"),662,200,false), -6);
  const timber = meshes.filter(m => ["divider","frame-member"].includes(m.userData.role));
  // At the acute junction the heel of the rebate meets the stile at a
  // different angle from either face of the stock. Sample both sides of it.
  const head = segmental(50,1).segments[0];
  for (const t of [0.02,0.04,0.07,0.96]) {
    const angle = head.startAngle + head.sweepAngle*t;
    const atRadius = r => [500+r*Math.cos(angle),625+r*Math.sin(angle)];
    close(hit(timber,...atRadius(head.radius-0.45),false), -6);
    close(hit(timber,...atRadius(head.radius+0.45),false), -57);
  }
  close(hit(timber,634,120),57);
  assert.equal(hit(byRole(meshes,"glass"),634,120), undefined);
  close(hit(byRole(meshes,"wall"),925,45), -57);
});

test("the offset mullion ends at the frame day through the core and both rebates", () => {
  const x0=601, x1=668, radius=558, cy=625;
  const a=-Math.acos((x0-500)/radius), b=-Math.acos((x1-500)/radius);
  const head=arc([500,cy],radius,a,b-a);
  const sill=line([x1,1333],[x0,1333]);
  const outline={segments:[head,line(head.to,sill.from),sill,line(sill.to,head.from)]};
  const piece={vertical:true,sourceIndex:0,pieceIndex:0,outline,start:{segments:[head]},end:{segments:[sill]}};
  const meshes=curvedModel(segmental,true,false,archStock(),{},null,{members:[piece],stock:profile("mullion-94",67,94)});
  const divider=byRole(meshes,"divider"), fixed=byRole(meshes,"frame-member");
  assert.equal(divider.length,1);
  assert.deepEqual(divider[0].userData.divider,{vertical:true,sourceIndex:0,pieceIndex:0});
  assert.ok(divider[0].children.some(c=>c.userData.role==="timber-joint"));
  for (const x of [605,623,634,652,664]) {
    const y=cy-Math.sqrt(radius**2-(x-500)**2);
    assert.equal(hit(divider,x,y-.3),undefined);
    assert.equal(hit(fixed,x,y+.3),undefined);
    close(hit(divider,x,y+.3,x!==664),x===605 ? 6 : x===664 ? 14 : 57);
    for (const z of x>=x1-17 ? [20,45] : [-25,0]) {
      const ray=new THREE.Raycaster(new THREE.Vector3(x,-y+10,z),new THREE.Vector3(0,-1,0));
      const end=ray.intersectObjects(divider,false)[0]?.point;
      assert.ok(end && Math.abs(-end.y-y)<0.11,JSON.stringify({x,z,y,end}));
    }
  }
  close(hit(divider,634,400,false),-37);
  close(hit(fixed,634,70,false),-57);
  // Both portions of the glazing pocket stop on the physical through sill.
  // These points lie between the divider's centre web and its side rebates.
  const throughSill=fixed.filter(m=>m.userData.timber?.role==="bottom");
  for (const x of [622,647]) {
    const entry=new THREE.Raycaster(new THREE.Vector3(x,-1330,-11),new THREE.Vector3(0,-1,0));
    close(-entry.intersectObjects(throughSill,false)[0].point.y,1333);
  }
  // The profile sweep has vertices on this saw plane. Its two halves still
  // need end grain in the narrow band between the central web and glass rebate.
  const [outside,inside]=archStock()[0].startCut;
  const x=44.5, atCut=outside[1]+(inside[1]-outside[1])*(x-outside[0])/(inside[0]-outside[0]);
  for (const [role,sign] of [["left",1],["arc",-1]]) {
    const ray=new THREE.Raycaster(new THREE.Vector3(x,-atCut+sign,19),new THREE.Vector3(0,-sign,0));
    const end=ray.intersectObjects(fixed.filter(m=>m.userData.timber?.role===role),false)[0];
    assert.ok(end); close(-end.point.y,atCut);
  }
});

test("a double tenon projects beyond the curved shoulder and both blind sockets leave bottom clearance", () => {
  const r=558, x0=601, x1=668, center=[500,625];
  const dayY=x=>625-Math.sqrt(r*r-(x-500)**2);
  const left=-Math.acos((x0-500)/r), right=-Math.acos((x1-500)/r);
  const head=arc(center,r,left,right-left), base=line([x1,1333],[x0,1333]);
  const x=(x1+x0)/2, root=dayY(x), tip=root-25, bottom=tip-2;
  const tenon={receiver:{kind:"frame",role:"arc",index:0},
    crossRangeMm:[x-12.5,x+12.5],zFromInsideMm:[[24,36],[74,86]],
    tipAxisMm:tip,bottomAxisMm:bottom};
  const piece={vertical:true,sourceIndex:0,pieceIndex:0,
    outline:{segments:[head,line(head.to,base.from),base,line(base.to,head.from)]},
    start:{segments:[head],tenon},end:{segments:[base]}};
  const meshes=curvedModel(segmental,true,false,archStock(),{},null,{members:[piece]});
  const divider=byRole(meshes,"divider"),fixed=byRole(meshes,"frame-member");
  const cast=(pieces,from,to)=>new THREE.Raycaster(new THREE.Vector3(...from),new THREE.Vector3(...to)).intersectObjects(pieces,false)[0]?.point;
  for (const z of [27,-23]) {
    const face=cast(divider,[x,-bottom+1,z],[0,-1,0]);
    const floor=cast(fixed,[x,-root-3,z],[0,1,0]);
    close(-face.y,tip); close(-floor.y,bottom); close(floor.y-face.y,2);
    const wall=cast(fixed,[x,-tip-7,z],[1,0,0]);
    close(wall.x,x+12.5);
    const shoulder=cast(fixed,[x+14,-root-15,z],[0,1,0]);
    assert.ok(Math.abs(-shoulder.y-dayY(x+14))<0.12);
  }
  // Solid wood between the mortises and across the outside of the arch.
  assert.ok(Math.abs(-cast(fixed,[x,-root-8,0],[0,1,0]).y-root)<0.12);
  const outsideY=625-Math.sqrt(625**2-(x-500)**2);
  assert.ok(Math.abs(-cast(fixed,[x,-outsideY+4,27],[0,-1,0]).y-outsideY)<0.12);
  const group=meshes[0].parent;
  inspectDividerJoints(group,true,114);
  assert.ok(divider[0].position.z>200); assert.equal(byRole(meshes,"glass")[0].visible,false);
  inspectDividerJoints(group,false,114);
  assert.equal(divider[0].position.z,0); assert.equal(byRole(meshes,"glass")[0].visible,true);
});

test("opposed blind sockets stop separately in the same through transom", () => {
  const group=new THREE.Group(), material=new THREE.MeshBasicMaterial();
  const receiver={kind:"divider",vertical:false,sourceIndex:0,pieceIndex:0};
  function stock(x0,x1,y0,y1,divider) {
    const g=new THREE.BoxGeometry(x1-x0,y1-y0,94);
    g.translate((x0+x1)/2,-(y0+y1)/2,10);
    const m=new THREE.Mesh(g,material);
    m.userData={role:"divider",divider};group.add(m);
    return {mesh:m, member:{...divider,start:{},end:{},outline:{segments:[
      line([x0,y0],[x1,y0]),line([x1,y0],[x1,y1]),line([x1,y1],[x0,y1]),line([x0,y1],[x0,y0]) ]}}};
  }
  const rail=stock(0,500,300,340,receiver).mesh;
  const top=stock(170,237,50,300,{vertical:true,sourceIndex:0,pieceIndex:0});
  const lower=stock(170,237,340,600,{vertical:true,sourceIndex:0,pieceIndex:1});
  const common={receiver,crossRangeMm:[191,216],zFromInsideMm:[[41,53]]};
  top.member.end.tenon={...common,tipAxisMm:316,bottomAxisMm:318};
  lower.member.start.tenon={...common,tipAxisMm:324,bottomAxisMm:322};
  machineDividerJoints(THREE,group,[top.member,lower.member],57);
  group.updateMatrixWorld();
  const yHit=(mesh,y,dir,z=10)=>-new THREE.Raycaster(new THREE.Vector3(203,-y,z),
    new THREE.Vector3(0,-dir,0)).intersectObject(mesh,false)[0].point.y;
  close(yHit(rail,299,1),318); close(yHit(rail,341,-1),322);
  close(yHit(top.mesh,319,-1),316); close(yHit(lower.mesh,321,1),324);
  close(yHit(rail,299,1,25),300); close(yHit(rail,341,-1,25),340);
});

test("an offset transom meets both sides of a round frame on the same fitted radius", () => {
  const top=350, bottom=417, center=[500,500], radius=433;
  function opening(inset,row) {
    if (row==null) return circle(inset);
    const r=500-inset, y=row ? top+inset : bottom-inset, angle=Math.asin((y-500)/r);
    const bow=row ? arc(center,r,angle,Math.PI-2*angle) : arc(center,r,Math.PI-angle,Math.PI+2*angle);
    return {segments:[bow,line(bow.to,bow.from)]};
  }
  const a=Math.asin((top-500)/radius), b=Math.asin((bottom-500)/radius);
  const right=arc(center,radius,a,b-a), left=arc(center,radius,Math.PI-b,b-a);
  const outline={segments:[line(left.to,right.from),right,line(right.to,left.from),left]};
  const piece={vertical:false,sourceIndex:0,pieceIndex:0,outline,start:{segments:[left]},end:{segments:[right]}};
  const members=[arcTimber(circle(0).segments[1],circle(67).segments[1]),arcTimber(circle(0).segments[0],circle(67).segments[0],1)];
  const transom={x:67,y:top,width:866,height:67};
  const meshes=curvedModel(opening,true,false,members,{},null,{members:[piece],transom});
  const rail=byRole(meshes,"divider"), frameStock=byRole(meshes,"frame-member");
  assert.equal(rail.length,1);
  for (const y of [354,374,392,413]) for (const sign of [-1,1]) {
    const x=500+sign*Math.sqrt(radius**2-(y-500)**2);
    assert.equal(hit(rail,x+sign*.3,y),undefined);
    assert.equal(hit(frameStock,x-sign*.3,y),undefined);
    const z=y>400 ? 40 : -25;
    const ray=new THREE.Raycaster(new THREE.Vector3(x+5*sign,-y,z),new THREE.Vector3(-sign,0,0));
    const end=ray.intersectObjects(rail,false)[0]?.point;
    assert.ok(end && Math.abs(end.x-x)<0.1,JSON.stringify({x,y,z,end}));
    const upper=frameStock.filter(m=>m.userData.timber?.index===0);
    const fromRail=new THREE.Raycaster(new THREE.Vector3(x-3*sign,-y,z),new THREE.Vector3(sign,0,0));
    const stockFace=fromRail.intersectObjects(upper,false)[0]?.point;
    assert.ok(stockFace && Math.abs(stockFace.x-x)<0.2,JSON.stringify({x,y,z,stockFace}));
  }
});

test("an operable round sash seats on the curved stop and carries its own glazing pocket", () => {
  const meshes = curvedModel(circle,false,true);
  const probe = (role,radius,inside = true) => hit(byRole(meshes,role),500 + radius*0.6,500 + radius*0.8,inside);
  close(probe("frame-member",436),-27);
  close(probe("sash",436,false),-25);
  close(probe("sash-seal",436),-25);
  close(probe("sash",385),14);
  close(probe("glass",385,false),16);
  assert.equal(probe("glass",391),undefined);
});

function fitSash(job, make, opensOut, vent = sash) {
  const p = job.fit.contours;
  p.sashSeat = make(47);
  p.sashSeatDepth = 29;
  p.sashClearance = 6;
  p.sashOpensOutward = opensOut;
  job.cell.sashProfile = vent;
  const outer = p.sashOuter.segments, inner = p.day.segments;
  if (outer.length === 2) {
    p.sashTimber = [arcTimber(outer[1],inner[1],0), arcTimber(outer[0],inner[0],1)];
    return;
  }
  // The stiles run past the bottom rail, whose end cuts are vertical.
  const head = arcTimber(outer[0],inner[0]);
  const foot = i => [[inner[i].from[0],outer[i].from[1]],inner[i].from];
  const left = foot(3), right = foot(2);
  p.sashTimber = [head,straightTimber("left",head.startCut,left,
    [outer[0].from,inner[0].from,left[1],left[0],outer[3].from]),
    straightTimber("right",head.endCut,right,[outer[0].to,outer[2].from,right[0],right[1],inner[0].to]),
    straightTimber("bottom",left,right,[left[0],right[0],right[1],left[1]])];
}

for (const outward of [false,true]) {
  test(`curved wood sash closes on the specified shoulder, gap and seal (outward=${outward})`, () => {
    const meshes = curvedModel(circle,false,true,[],{},(job,make)=>fitSash(job,make,outward));
    const sign = outward ? -1 : 1;
    for (const a of [0.42,2.01,3.66,5.11]) {
      const probe = (role,r,front) => hit(byRole(meshes,role),500+r*Math.cos(a),500+r*Math.sin(a),front);
      close(probe("frame-member",436,!outward),sign*28); // frame face 57 minus 29
      close(probe("sash",436,outward),sign*30);           // 2 mm compressed seal
      close(probe("sash-seal",436,!outward),sign*30);
      close(probe("sash",426,outward),sign*(-25));       // full-depth body beyond its shoulder
      assert.equal(probe("sash",449,!outward),undefined);
      close(probe("frame-member",452.6,!outward),sign*28);
      close(probe("frame-member",453.4,!outward),sign*57);
    }
    const half = i => byRole(meshes,"sash").filter(m => m.userData.timber?.index === i);
    for (const i of [0,1]) {
      const y = i ? 500.4 : 499.6;
      close(hit(half(i),918,y,!outward),sign*65);
      assert.equal(hit(half(1-i),918,y,!outward),undefined);
      assert.ok(half(i).some(m=>m.children.some(c=>c.userData.role==="timber-joint")));
    }
  });

  test(`factory sash lip seats without filling its groove (outward=${outward})`, () => {
    const vent = { ...sash, id:"lip", sponning:{...sash.sponning,type:"draaikiep",opdek_width:13} };
    vent.crossSection = woodCrossSection(vent);
    const meshes = curvedModel(circle,false,true,[],{},(job,make)=>fitSash(job,make,outward,vent));
    const sign = outward ? -1 : 1;
    const probe = (role,r,front) => hit(byRole(meshes,role),500+r*0.6,500+r*0.8,front);
    close(probe("frame-member",446,!outward),sign*28);
    close(probe("sash",446,outward),sign*30);
    close(probe("sash-seal",446,!outward),sign*30);
    assert.ok(sign*probe("sash",441,outward)>30); // lip groove survives placement
    close(probe("sash",429,outward),sign*(-47));
  });
}

test("two hinged arch sections keep their own angled shoulders and opposing stops", () => {
  const members = [];
  const meshes = curvedModel(segmental,true,true,archStock(),{},(job,make)=>{
    fitSash(job,make,job.cellIndex===1);
    members.push(job.fit.contours.sashTimber);
  });
  for (const [leaf,parts] of members.entries()) {
    const timber = role => byRole(meshes,"sash").filter(m=>m.userData.leaf?.cellIndex===leaf && m.userData.timber?.role===role);
    const [a,b] = parts[0].startCut;
    const dx=b[0]-a[0], dy=b[1]-a[1], len=Math.hypot(dx,dy);
    for (const t of [0.12,0.45,0.86]) {
      const x=a[0]+dx*t, y=a[1]+dy*t;
      const high=[x+0.4*dy/len,y-0.4*dx/len], low=[x-0.4*dy/len,y+0.4*dx/len];
      assert.ok(Number.isFinite(hit(timber("arc"),...high)));
      assert.equal(hit(timber("left"),...high),undefined);
      assert.ok(Number.isFinite(hit(timber("left"),...low)));
      assert.equal(hit(timber("arc"),...low),undefined);
    }
  }
  close(hit(byRole(meshes,"divider"),604,600),28);
  close(hit(byRole(meshes,"divider"),665,600,false),-28);
});

test("a short head tapering to one inner corner keeps both saw faces through the rebate", () => {
  const outer = [[30,50],[60,10],[60,180],[2,180]];
  const day = [[50,92],[50,162],[30,162]];
  const inside = [day[0],day[0],day[1],day[2]];
  const cuts = outer.map((a,i) => [a,inside[i]]);
  const parts = ["arc","right","bottom","left"].map((role,i) => {
    const j = (i+1)%outer.length;
    return straightTimber(role,cuts[i],cuts[j],[outer[i],outer[j],inside[j],inside[i]]);
  });
  const plan = timberPlan(parts);
  const stock = sectionFromPoints("taper", [[0,2],[5,2],[5,0],[20,0],[20,8],[0,8]]);
  const select = (x,y,mx=x,my=y) => {
    const piece = plan.at(mx,my);
    return [{...stock,id:piece?.role || stock.id,timber:piece}];
  };
  const pathAt = depth => inside.map((p,i) => p.map((v,j) => v+(outer[i][j]-v)*depth/stock.w));
  const meshes = sweepRing(THREE,outer,day,select,8,new THREE.MeshBasicMaterial(),"sash",pathAt,cuts);
  const member = role => meshes.filter(m => m.userData.timber?.role === role);
  for (const [i,neighbor] of [[0,"left"],[1,"right"]]) {
    const [[x,y],[bx,by]] = cuts[i], dx=bx-x, dy=by-y, len=Math.hypot(dx,dy);
    for (const t of [0.22,0.52,0.81]) {
      const pair = [-1,1].map(sign => [x+t*dx+0.1*sign*dy/len,y+t*dy-0.1*sign*dx/len]);
      const head = pair.map(p => Number.isFinite(hit(member("arc"),...p)));
      const stile = pair.map(p => Number.isFinite(hit(member(neighbor),...p)));
      assert.notEqual(head[0],head[1]);
      assert.deepEqual(stile,head.map(h => !h));
    }
  }
});
