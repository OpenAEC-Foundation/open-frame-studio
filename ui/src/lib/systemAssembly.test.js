import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import * as THREE from "three";
import { buildFrameGroup } from "../components/viewer3d/frameGroup.js";
import { insetOpening } from "./glazingFit.js";
import { resolveProfileSnapshot } from "./profileSnapshot.js";
import { sectionCapsAt } from "../components/viewer3d/systemSection.js";

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
function assembly(profile,{round=false,outward=false,fixed=false,handle=false}={}) {
  const snapshot=resolveProfileSnapshot(profile), w=profile.width;
  const opening=insetOpening(bounds,w);
  const overlap=snapshot.sashOverlap || 0, face=snapshot.sashFaceWidth || w;
  const sashOuter = fixed ? null : insetOpening(opening,-overlap);
  const day = fixed ? opening : insetOpening(sashOuter,face);
  const systemSeals={hinged:!fixed,sealingLevels:snapshot.sealingLevels,sashClearanceMm:snapshot.sashClearance,
    sashOverlapMm:snapshot.sashOverlap};
  const h=snapshot.sponningHoogte, bead=snapshot.glaslatHoogte || h;
  const fit = {day,sashOuter,rebate:insetOpening(day,-h),glass:insetOpening(day,4-h),
    glassThickness:24,beadHeight:bead,beadWidth:15,beadPosition:"binnen",systemSeals};
  const frame={material:profile.material,profile,frameWidth:w,frameDepth:profile.depth,
    outerWidth:1200,outerHeight:1200};
  const part={kind:"handle",mount:"sash",side:"left",position:[fixed?35:sashOuter.x+face/2,710],tangent:[0,-1]};
  const leaves=handle?[{leaf:null,opensOutward:outward,components:[part]}]:[];
  const cr={cellIndex:0,rect:opening,fit,hardwareLeaves:leaves};
  const geometry={cellRects:[cr],outerRect:bounds,vDividers:[],hDividers:[],
    frameRects:[rect(0,0,1200,w),rect(0,1200-w,1200,w),rect(0,w,w,1200-2*w),rect(1200-w,w,w,1200-2*w)]};
  if (round) {
    const radius=r=>r.width/2;
    cr.openingContour=circle(radius(opening));
    geometry.curvedFrame={outer:circle(600),day:cr.openingContour};
    fit.contours={day:circle(radius(day)),glass:circle(radius(fit.glass)),rebate:circle(radius(fit.rebate)),
      beadInner:circle(radius(fit.rebate)-bead),rebateHeight:h,edgeClearance:4,
      sashOuter:sashOuter && circle(radius(sashOuter)),sashWidth:face,sashOverlap:overlap,sashOpensOutward:outward};
    geometry.perimeterStop={outer:circle(620),inner:circle(600),thicknessMm:4};
  }
  const group=buildFrameGroup(THREE,{frame,cells:[{panelType:fixed?"fixed_glass":"turn",
    openingDirection:outward?"outward":"inward"}]},geometry,profiles);
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
