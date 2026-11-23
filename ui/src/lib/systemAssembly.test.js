import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import * as THREE from "three";
import { buildFrameGroup } from "../components/viewer3d/frameGroup.js";
import { insetOpening } from "./glazingFit.js";
import { resolveProfileSnapshot } from "./profileSnapshot.js";
import { systemProfileSolids } from "./systemProfileSolids.js";
import { sectionCapsAt } from "../components/viewer3d/systemSection.js";
import { clipToRect, contourPoints, insetPolygon } from "../components/viewer3d/curvedMesh.js";

const load = path => JSON.parse(readFileSync(new URL("../../../profiles/"+path,import.meta.url)));
const profiles = [...load("pvc/veka.json"),...load("aluminum/schuco.json")];
const md = profiles.find(p=>p.sealingLevels===3);
const ad = profiles.find(p=>p.sealingLevels===2);
const alu = profiles.find(p=>p.material==="aluminum" && p.sealingLevels===3);
const rect = (x,y,width,height) => ({x,y,width,height});
const bounds = rect(0,0,1200,1200);
function circle(radius) {
  const center=[600,600], left=[600-radius,600], right=[600+radius,600];
  return {segments:[[0,right,left],[Math.PI,left,right]].map(([startAngle,from,to])=>({
    kind:"arc",center,radius,startAngle,sweepAngle:Math.PI,from,to}))};
}
function arch(radius) {
  const left=[600-radius,600], right=[600+radius,600];
  const baseLeft=[left[0],600+radius], baseRight=[right[0],600+radius];
  return {segments:[{kind:"arc",center:[600,600],radius,startAngle:Math.PI,sweepAngle:Math.PI,from:left,to:right},
    ...[[right,baseRight],[baseRight,baseLeft],[baseLeft,left]].map(([from,to])=>({kind:"line",from,to}))]};
}
function assembly(profile,{round=false,arched=false,outward=false,fixed=false,handle=false,vent=null,glassThickness=24,beadWidth=15}={}) {
  const snapshot=resolveProfileSnapshot(profile), w=profile.width;
  const opening=insetOpening(bounds,w);
  const overlap=vent ? vent.width-vent.sightline-(vent.material==="aluminum"?5:0) : snapshot.sashOverlap || 0;
  const face=vent?.width || snapshot.sashFaceWidth || w;
  const sashOuter = fixed ? null : insetOpening(opening,-overlap);
  const day = fixed ? opening : insetOpening(sashOuter,face);
  const systemSeals={hinged:!fixed,sealingLevels:snapshot.sealingLevels,sashClearanceMm:snapshot.sashClearance,
    sashOverlapMm:snapshot.sashOverlap};
  if (!fixed && !outward && vent?.inwardWindowFrame === profile.id && vent.closingSeals) {
    systemSeals.closingSeals = vent.closingSeals;
    systemSeals.sealingLevels = vent.closingSeals.length;
    systemSeals.sashOverlapMm = overlap;
  }
  const h=snapshot.sponningHoogte, bead=snapshot.glaslatHoogte || h;
  const fit = {day,sashOuter,rebate:insetOpening(day,-h),glass:insetOpening(day,4-h),
    glassThickness,beadHeight:bead,beadWidth,beadPosition:"binnen",systemSeals};
  const frame={material:profile.material,profile,frameWidth:w,frameDepth:profile.depth,
    outerWidth:1200,outerHeight:1200};
  const part={kind:"handle",mount:"sash",side:"left",position:[fixed?35:sashOuter.x+face/2,710],tangent:[0,-1]};
  const leaves=handle?[{leaf:null,opensOutward:outward,components:[part]}]:[];
  const cr={cellIndex:0,rect:opening,fit,hardwareLeaves:leaves};
  const geometry={cellRects:[cr],outerRect:bounds,vDividers:[],hDividers:[],
    frameRects:[rect(0,0,1200,w),rect(0,1200-w,1200,w),rect(0,w,w,1200-2*w),rect(1200-w,w,w,1200-2*w)]};
  if (round || arched) {
    const radius=r=>r.width/2;
    const contour=arched ? arch : circle;
    cr.openingContour=contour(radius(opening));
    geometry.curvedFrame={outer:contour(600),day:cr.openingContour};
    fit.contours={day:contour(radius(day)),glass:contour(radius(fit.glass)),rebate:contour(radius(fit.rebate)),
      beadInner:contour(radius(fit.rebate)-bead),rebateHeight:h,edgeClearance:4,
      sashOuter:sashOuter && contour(radius(sashOuter)),sashWidth:face,sashOverlap:overlap,sashOpensOutward:outward};
    geometry.perimeterStop={outer:contour(620),inner:contour(600),thicknessMm:4};
  }
  const group=buildFrameGroup(THREE,{frame,cells:[{panelType:fixed?"fixed_glass":"turn",
    sashProfile:vent, sashDepth:vent?.depth,
    openingDirection:outward?"outward":"inward"}]},geometry,[profile,...(vent?[vent]:[]),...profiles]);
  group.updateMatrixWorld(true);
  return {group,part};
}
const role=(group,type)=>group.children.filter(m=>m.userData.role===type);
const box=meshes=>meshes.reduce((b,m)=>b.union(new THREE.Box3().setFromObject(m)),new THREE.Box3());
function edge(meshes,z,fromGlass) {
  const ray=new THREE.Raycaster(new THREE.Vector3(fromGlass?600:-100,-601,z),new THREE.Vector3(fromGlass?-1:1,0,0));
  return ray.intersectObjects(meshes,false)[0]?.point.x;
}
for (const [profile,outward,round] of [[md,false,false],[md,true,true],[ad,false,false],[alu,true,false]]) {
  test(`${profile.name} ${outward?"outward":"inward"} ${round?"round":"rectangular"} seals contact both profiles`,()=>{
    const {group}=assembly(profile,{outward,round});
    const rubber=role(group,"sash-seal"), carrier=role(group,"system-stop");
    assert.equal(new Set(rubber.map(m=>m.userData.systemSeal.level)).size,profile.sealingLevels);
    for (const seal of rubber) {
      const bounds=box([seal]), z=(bounds.max.z+bounds.min.z)/2+0.001;
      const support=role(group,"frame-member").concat(carrier.filter(m=>m.userData.systemSeal.level===seal.userData.systemSeal.level));
      assert.ok(Math.abs(edge(support,z,true)-edge([seal],z,false))<0.1,"seal root seats on the stop");
      assert.ok(Math.abs(edge([seal],z,true)-edge(role(group,"sash"),z,false))<0.1,"seal nose closes against the leaf");
    }
    const caps=sectionCapsAt(THREE,group,601);
    const cutRubbers=caps.children.filter(c=>c.material===rubber[0].material);
    assert.ok(cutRubbers.length>=profile.sealingLevels);
    assert.ok(cutRubbers.every(c=>Math.abs(box([c]).max.y+601)<0.001));
    if (round) {
      const band=box(role(group,"perimeter-stop"));
      assert.deepEqual(band.min.toArray().map(Math.round),[-20,-1220,-41]);
      assert.deepEqual(band.max.toArray().map(Math.round),[1220,20,-37]);
      assert.equal(edge(role(group,"perimeter-stop"),-39,true).toFixed(1),"0.0");
    }
  });
}
test("CS77 middle and inner closing seals contact the documented frame and sash at both glass thicknesses",()=>{
  const vent = load("aluminum/reynaers-sash.json").find(v=>v.id==="reynaers-0083102");
  const frame = load("aluminum/reynaers.json").find(f=>f.id===vent.inwardWindowFrame);
  for (const glassThickness of [24,60]) {
    const {group}=assembly(frame,{vent,glassThickness});
    const rubber=role(group,"sash-seal");
    assert.deepEqual([...new Set(rubber.map(m=>m.userData.systemSeal.level))].sort(),["inner","middle"]);
    assert.ok(role(group,"seal").length>0);
    const bands = {};
    for (const seal of rubber) {
      const level=seal.userData.systemSeal.level, b=box([seal]);
      const z=(b.min.z+b.max.z)/2+0.001;
      assert.equal(role(group,"system-stop").length,0,"CS77 seals seat directly on the two profiles");
      if (level==="middle") {
        assert.ok(Math.abs(edge(role(group,"frame-member"),z,true)-edge([seal],z,false))<0.1,"frame bridge contact");
        assert.ok(Math.abs(edge([seal],z,true)-edge(role(group,"sash"),z,false))<0.1,"sash bridge contact");
      } else {
        const x=frame.width-vent.ventOverlap+2.5+0.001;
        const depthEdge=(meshes,from,direction)=>new THREE.Raycaster(new THREE.Vector3(x,-601,from),
          new THREE.Vector3(0,0,direction)).intersectObjects(meshes,false)[0]?.point.z;
        assert.ok(Math.abs(depthEdge(role(group,"frame-member"),z,-1)-depthEdge([seal],b.min.z-1,1))<0.1,"frame interior contact");
        assert.ok(Math.abs(depthEdge(role(group,"sash"),z,1)-depthEdge([seal],b.max.z+1,-1))<0.1,"inside of sash lip");
      }
      bands[level]=b;
    }
    assert.ok(bands.middle.max.z<bands.inner.min.z,"the two closing lines have separate depth ranges");
  }
});

test("system fixed glazing has glass gaskets only, and a movable handle bears on the real sash face",()=>{
  const fixed=assembly(md,{fixed:true}).group;
  assert.equal(role(fixed,"sash-seal").length,0);
  assert.ok(role(fixed,"seal").length>0);
  const {group,part}=assembly(md,{handle:true});
  const hardware=role(group,"hardware");
  assert.equal(hardware.length,3);
  assert.deepEqual(hardware[0].userData.hardware.position,part.position);
  const [x,y]=part.position;
  const ray=new THREE.Raycaster(new THREE.Vector3(x,-y,500),new THREE.Vector3(0,0,-1));
  const surface=ray.intersectObjects(role(group,"sash"),false)[0].point.z;
  assert.ok(Math.abs(surface-box([hardware[0]]).min.z)<0.001);
});

// Interrogate the actual triangulated cut in the section inspector at mid-height.
function cutContains(caps, x, y, z) {
  const point = new THREE.Vector3(x,y,z);
  const triangle = new THREE.Triangle();
  return caps.children.some(mesh => {
    const vertices = mesh.geometry.getAttribute("position");
    for (let i=0; i<vertices.count; i+=3) {
      triangle.a.fromBufferAttribute(vertices,i);
      triangle.b.fromBufferAttribute(vertices,i+1);
      triangle.c.fromBufferAttribute(vertices,i+2);
      if (triangle.containsPoint(point)) return true;
    }
    return false;
  });
}

test("PVC inspection cuts through air chambers, webs and the separate steel reinforcement", () => {
  const {group} = assembly(md,{fixed:true});
  const caps = sectionCapsAt(THREE,group,601);
  const at = (u,v) => cutContains(caps,u,-601,v-md.depth/2);
  assert.equal(at(7.5,5),true);
  assert.equal(at(12,5),false);
  assert.equal(at(12.1,50),true);
  assert.equal(at(23,50),false);
  assert.ok(role(group,"frame-member").some(mesh=>mesh.userData.sectionMaterial==="reinforcement"));
});

test("a cut through the upper arc retains VEKA chambers and the separate reinforcement",()=>{
  const {group}=assembly(md,{round:true,fixed:true});
  const cutY=301;
  const caps=sectionCapsAt(THREE,{children:role(group,"frame-member")},cutY);
  const at=(u,v)=>cutContains(caps,600-Math.sqrt((600-u)**2-(600-cutY)**2),-cutY,v-md.depth/2);
  assert.equal(at(7.5,5),true,"PVC web");
  assert.equal(at(12,5),false,"air chamber at the outer face");
  assert.equal(at(12.2,50),true,"steel wall");
  assert.equal(at(23,50),false,"hollow reinforcement");
  assert.ok(role(group,"frame-member").some(mesh=>mesh.userData.sectionMaterial==="reinforcement"));
});

test("the arched CS 77 frame carries both insulating strips in the head and jamb",()=>{
  const profile=load("aluminum/reynaers.json").find(p=>p.id==="reynaers-cs77-standard");
  const {group}=assembly(profile,{arched:true,fixed:true});
  const members=role(group,"frame-member");
  const bridges=members.filter(mesh=>mesh.userData.sectionMaterial==="insulator");
  assert.ok(bridges.length);
  for (const y of [301,870]) {
    const caps=sectionCapsAt(THREE,{children:members},y);
    const polymer=sectionCapsAt(THREE,{children:bridges},y);
    const at=(section,u,v)=>cutContains(section,y>600 ? u : 600-Math.sqrt((600-u)**2-(600-y)**2),-y,v-profile.depth/2);
    assert.ok(at(polymer,7.1,30) && at(polymer,18.9,30),"both polyamide strips");
    assert.equal(at(caps,13,30),false,"clear gap between strips");
    assert.ok(at(caps,30,1),"metal retaining flange");
    assert.equal(at(caps,13,8),false,"outer metal shell is hollow");
  }
});

function cutTriangles(meshes) {
  const caps = sectionCapsAt(THREE,{children:meshes},601);
  const faces = [];
  for (const mesh of caps.children) {
    const p=mesh.geometry.getAttribute("position");
    for (let i=0;i<p.count;i+=3) {
      const points=[i,i+1,i+2].map(j=>[p.getX(j),p.getZ(j)]);
      if (Math.abs(area(points))>1e-7) faces.push(points);
    }
  }
  return faces;
}

function area(points) {
  return points.reduce((sum,[x,y],i)=>{
    const [u,v]=points[(i+1)%points.length];
    return sum+x*v-y*u;
  },0)/2;
}

function crosses(a,b) {
  for (const face of [a,b]) for (let i=0;i<3;i++) {
    const [x,y]=face[i], [u,v]=face[(i+1)%3], len=Math.hypot(u-x,v-y);
    const project=([p,q])=>((v-y)*p-(u-x)*q)/len;
    const ap=a.map(project), bp=b.map(project);
    if (Math.min(Math.max(...ap),Math.max(...bp))-Math.max(Math.min(...ap),Math.min(...bp)) < 0.0001) return false;
  }
  return true;
}

function meshOverlap(a,b) {
  const bounds=points=>({
    points, x0:Math.min(...points.map(p=>p[0])),x1:Math.max(...points.map(p=>p[0])),
    y0:Math.min(...points.map(p=>p[1])),y1:Math.max(...points.map(p=>p[1]))
  });
  const candidates=b.map(bounds);
  return a.map(bounds).some(f=>candidates.some(s=>f.x1>s.x0 && f.x0<s.x1 && f.y1>s.y0 && f.y0<s.y1 &&
    crosses(f.points,s.points)));
}

const csFrame = load("aluminum/reynaers.json").find(p => p.id === "reynaers-cs77-standard");
const csVent = load("aluminum/reynaers-sash.json")[0];

function dividedArch(profile, shape, reverseRight=false, article=null) {
  const fw=profile.width, width=profile.sightlineMullion;
  const outer=shape(600), day=shape(600-fw), loop=contourPoints(day);
  const polygon=points=>({segments:points.map((from,i)=>({kind:"line",from,to:points[(i+1)%points.length]}))});
  const stile=rect(752,fw,width,1200-2*fw), rail=rect(fw,293,1200-2*fw,width);
  const pieces=[];
  function member(vertical,pieceIndex,blank) {
    pieces.push({vertical,pieceIndex,sourceIndex:0,profile:article,outline:polygon(clipToRect(loop,blank)),
      start:{segments:[]},end:{segments:[]}});
  }
  member(true,0,stile);
  member(false,0,{...rail,width:stile.x-fw});
  member(false,1,{...rail,x:stile.x+width,width:1200-fw-stile.x-width});
  const cells=[];
  for (const [left,right] of [[fw,stile.x],[stile.x+width,1200-fw]]) {
    for (const [top,bottom] of [[fw,rail.y],[rail.y+width,1200-fw]]) {
      const opening=rect(left,top,right-left,bottom-top);
      const boundary=clipToRect(loop,opening), day=polygon(boundary);
      const h=profile.sponning.depth, rebate=insetOpening(opening,-h), glass=insetOpening(opening,4-h);
      const fit={day:opening,rebate,glass,beadHeight:h,beadWidth:15,glassThickness:24,beadPosition:"binnen",
        contours:{day,rebate:polygon(insetPolygon(boundary,-h)),glass:polygon(insetPolygon(boundary,4-h)),
          beadInner:day,rebateHeight:h,edgeClearance:4}};
      cells.push({cellIndex:cells.length,rect:opening,openingContour:day,fit});
    }
  }
  const geometry={outerRect:bounds,curvedFrame:{outer,day},vDividers:[stile],hDividers:[rail],
    curvedDividerTimber:pieces,cellRects:cells};
  const frame={profile,material:profile.material,frameWidth:fw,frameDepth:profile.depth,outerWidth:1200,outerHeight:1200};
  const group=buildFrameGroup(THREE,{frame,cells:cells.map(c=>({panelType:"fixed_glass",
    glaslat:{position:reverseRight && c.rect.x>stile.x ? "buiten" : "binnen"}}))},geometry,article ? [profile,article] : [profile]);
  group.updateMatrixWorld(true);
  return {group,width,stile,rail};
}

for (const [profile,insert,insertX,voidX,depth] of [
  [md,"reinforcement",41,32,50], [csFrame,"insulator",18.9,14,30],
]) for (const shape of [circle,arch]) {
  test(`four lights in ${shape.name} ${profile.id} retain hollow stock and independently fitted divider heads`,()=>{
    const {group,width,stile,rail}=dividedArch(profile,shape);
    const parts=role(group,"divider"), frame=role(group,"frame-member");
    const bar=(vertical,pieceIndex)=>parts.filter(m=>m.userData.divider.vertical===vertical && m.userData.divider.pieceIndex===pieceIndex);
    const mullion=bar(true,0), left=bar(false,0), right=bar(false,1);
    for (const piece of [mullion,left,right,frame]) {
      assert.ok(piece.some(m=>m.userData.sectionMaterial===insert),"insert continues into each separate member");
    }
    assert.equal(role(group,"glass").length,4);
    assert.ok(role(group,"bead").length>=4);
    const y=540.5, z=depth-profile.depth/2, h=profile.sponning.depth;
    const local=x=>h+x*(width-2*h)/(profile.width-h);
    const caps=sectionCapsAt(THREE,{children:mullion},y);
    for (const x of [u=>stile.x+local(u),u=>stile.x+width-local(u)]) {
      assert.ok(cutContains(caps,x(insertX),-y,z),"wall on each side of the carrier");
      assert.equal(cutContains(caps,x(voidX),-y,z),false,"unfilled centre chamber");
    }
    const r=600-profile.width, skin=0.5-profile.depth/2;
    const ray=(meshes,origin,direction)=>new THREE.Raycaster(new THREE.Vector3(...origin),new THREE.Vector3(...direction))
      .intersectObjects(meshes,false)[0]?.point;
    for (const x of [stile.x+width*0.31,stile.x+width*0.68]) {
      const head=600-Math.sqrt(r*r-(x-600)**2);
      const contact=ray(mullion,[x,0,skin],[0,-1,0]);
      assert.ok(contact && Math.abs(-contact.y-head)<0.15,"mullion follows the changing height of the curved head");
    }
    const probe=stile.x+local(voidX);
    assert.equal(ray(mullion,[probe,0,z],[0,-1,0]),undefined,"cut end opens into the same air chamber as the section");
    for (const y of [rail.y+width*0.31,rail.y+width*0.68]) {
      const dx=Math.sqrt(r*r-(y-600)**2);
      assert.ok(Math.abs(ray(left,[0,-y,skin],[1,0,0])?.x-(600-dx))<0.15,"outer end of left transom");
      assert.ok(Math.abs(ray(right,[1200,-y,skin],[-1,0,0])?.x-(600+dx))<0.15,"outer end of right transom");
      assert.ok(Math.abs(ray(left,[stile.x+1,-y,skin],[-1,0,0])?.x-stile.x)<0.01,"left transom butts to the continuous mullion");
      assert.ok(Math.abs(ray(right,[stile.x+width-1,-y,skin],[1,0,0])?.x-stile.x-width)<0.01,"right transom butt");
    }
  });
}

for (const [frame, article, insert, wall, opening] of [
  [md, load("pvc/veka-divider.json")[0], "reinforcement", [32.7,44.6], [61.3,44.6]],
  [csFrame, load("aluminum/reynaers-divider.json")[0], "insulator", [38,26.3], [38,30]],
]) {
  test(`the curved head of ${article.articleNumber} retains the unmirrored article section`, () => {
    const {group,stile} = dividedArch(frame,circle,false,article);
    const parts = role(group,"divider").filter(m => m.userData.divider.vertical);
    assert.ok(parts.length && parts.every(m => m.userData.profileId === article.id));
    const caps = sectionCapsAt(THREE,{children:parts.filter(m => m.userData.sectionMaterial === insert)},540);
    const occupied = ([u,v]) => cutContains(caps,stile.x + u,-540,v - article.depth/2);
    assert.equal(occupied(wall),true);
    assert.equal(occupied(opening),false);
    const x = stile.x + article.width/2;
    const ray = new THREE.Raycaster(new THREE.Vector3(x,0,1 - article.depth/2),new THREE.Vector3(0,-1,0));
    const point = ray.intersectObjects(parts,false)[0]?.point;
    const expected = 600 - Math.sqrt((600 - frame.width)**2 - (x-600)**2);
    assert.ok(point && Math.abs(-point.y-expected) < 0.15);
  });
}

test("a change of glazing side on the arched sill occurs underneath the mullion",()=>{
  const {group,stile,width}=dividedArch(csFrame,arch,true);
  const sill=role(group,"frame-member"), y=1200-csFrame.width+5;
  const depthAt = x => new THREE.Raycaster(new THREE.Vector3(x,-y,100),new THREE.Vector3(0,0,-1))
    .intersectObjects(sill,false)[0]?.point.z;
  assert.ok(depthAt(stile.x-1)<-20);
  assert.ok(depthAt(stile.x+width+1)>20);
});

for (const [frame, vent, slotFloor, slotDepth] of [
  [md, null, 41, 76], [md, profiles.find(p => p.id === "veka-103340"), 44, 75.5],
  [md, profiles.find(p => p.id === "veka-103341"), 54, 75.5],
  [csFrame, null, 22, 57], [csFrame, csVent, 38, 56],
]) for (const glassThickness of [24, 48]) {
  test(`clipped interior bead in ${(vent || frame).id} supports ${glassThickness} mm glass from its inner pocket`, () => {
    const beadWidth = frame.material === "pvc" ? frame.glazingBead.widthBase - glassThickness : 15;
    const {group} = assembly(frame, {fixed: !vent, vent, glassThickness, beadWidth});
    const left = type => role(group, type).filter(m => box([m]).max.x < 600);
    const caps = type => sectionCapsAt(THREE, {children: left(type)}, 601);
    const bead = caps("bead"), seal = caps("seal"), carrier = left(vent ? "sash" : "frame-member");
    const at = (section, x, z) => cutContains(section, x, -601, z);
    const beadFace = box(role(group, "glass")).max.z + 2;
    const glassEdge = box(role(group, "glass")).min.x;
    const beadParts = left("bead");
    const xmax = box(beadParts).max.x;
    assert.ok(glassEdge < xmax - 2);
    assert.ok(at(seal, xmax - 2, beadFace - 0.1), "gasket at the glass-side face");
    assert.ok(at(bead, xmax - 2, beadFace + 0.1), "solid bead nose supporting the gasket");
    const z = beadFace + beadWidth * 0.5;
    let runs = 0, filled = false;
    for (let x = box(beadParts).min.x; x <= xmax; x += 0.1) {
      const next = at(bead, x, z);
      if (next && !filled) runs++;
      filled = next;
    }
    assert.ok(runs >= 2, "two bead walls around an air cavity; a groove hook can also intersect this depth");
    const stock = vent || frame;
    const stockZ = frame.depth / 2 + (vent?.sashInsideOffset || 0) - stock.depth;
    const floor = box(carrier).min.x + slotFloor;
    assert.ok(at(bead, floor + 0.05, stockZ + slotDepth), "clip toe reaches the measured groove floor");
    assert.ok(at(caps(vent ? "sash" : "frame-member"), floor - 0.05, stockZ + slotDepth), "receiver supports the clip");
    assert.equal(meshOverlap(cutTriangles(carrier), cutTriangles(beadParts)), false);
    assert.equal(meshOverlap(cutTriangles(role(group, "glass")), cutTriangles(beadParts)), false);
    if (!vent) {
      const bearing = caps("frame-member");
      const outerFace = box(role(group, "glass")).min.z - 2;
      let contact = false;
      for (let x = glassEdge + 0.05; x < frame.width + 1; x += 0.1) {
        if (at(bearing, x, outerFace - 0.05) && at(seal, x, outerFace + 0.05)) { contact = true; break; }
      }
      assert.ok(contact, "exterior gasket bears on a real frame ledge under the glass edge");
    }
    if (stock.material === "aluminum") {
      const bridges = carrier.filter(m => m.userData.sectionMaterial === "insulator");
      assert.ok(bridges.length > 0);
      const innerShellStart = box(bridges).max.z;
      assert.ok(stockZ + slotDepth > innerShellStart, "clip bears on the interior aluminium shell");
      for (let depth = beadFace + 0.1; depth < innerShellStart; depth += 1) {
        const beadEdge = edge(beadParts, depth, false);
        const stockEdge = edge(carrier, depth, true);
        if (beadEdge !== undefined && stockEdge !== undefined) assert.ok(beadEdge > stockEdge + 0.05,
          "the bead must remain clear of the exterior shell and the thermal break");
      }
    }
  });
}

for (const vent of [...profiles.filter(p=>p.inwardWindowFrame),...load("aluminum/reynaers-sash.json")]) {
  test(`inward sash ${vent.articleNumber} keeps the complete article at its mounting depth`,()=>{
    const frame=[...profiles,...load("aluminum/reynaers.json")].find(p=>p.id===vent.inwardWindowFrame);
    const {group}=assembly(frame,{vent});
    const leftMembers=r=>role(group,r).filter(m=>box([m]).max.x<600);
    const jamb=cutTriangles(leftMembers("frame-member")), sash=cutTriangles(leftMembers("sash"));
    assert.ok(sash.length && jamb.length);
    const outerPlane=box(leftMembers("sash")).max.z;
    assert.ok(Math.abs(outerPlane-frame.depth/2-vent.sashInsideOffset)<0.001);
    const expected=systemProfileSolids(vent).reduce((sum,p)=>sum+Math.abs(area(p.points)),0);
    const actual=sash.reduce((sum,p)=>sum+Math.abs(area(p)),0);
    assert.ok(Math.abs(actual-expected)<0.05,`section area ${actual}, expected ${expected} mm²`);
    assert.ok(!meshOverlap(jamb,sash), "frame and sash extrusion overlap");
  });
}

test("insulated aluminium has two bridges and separate shells, plain aluminium has a continuous wall", () => {
  const [plain, insulated] = load("aluminum/generic.json");
  for (const [profile, thermal] of [[plain,false],[insulated,true]]) {
    // Include the generic stock in the library used by the production viewer.
    const {group} = assembly(profile,{fixed:true});
    const caps = sectionCapsAt(THREE,group,601);
    const at = (u,v) => cutContains(caps,u,-601,v-profile.depth/2);
    assert.equal(at(0.5,37),!thermal);
    assert.equal(at(2.3,37),thermal);
    assert.equal(at(10,37),false);
    assert.equal(at(0.5,5),true);
    assert.equal(at(10,5),false);
    assert.equal(role(group,"frame-member").some(m=>m.userData.sectionMaterial==="insulator"),thermal);
  }
});
