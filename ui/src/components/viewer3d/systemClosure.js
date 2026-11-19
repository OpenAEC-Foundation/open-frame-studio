import { clipByLine } from "./curvedMesh.js";

// Elevation coordinate n runs from the frame day towards the glass. z runs
// from outside to inside. All distances and section coordinates are in mm.
function edgeAt(points, z, nearest) {
  const hits = [];
  for (let i = 0; i < points.length; i++) {
    const [x,y] = points[i], [u,v] = points[(i+1)%points.length];
    if ((y <= z && z < v) || (v <= z && z < y)) hits.push(x+(u-x)*(z-y)/(v-y));
  }
  return hits.length ? nearest(...hits) : null;
}

export function systemClosure(frame, leaf, frameInner, sashInner, overlap, spec) {
  if (!spec?.hinged || !frame || !leaf) return null;
  if (!(overlap > 0 || spec.sashClearanceMm > 0 || spec.sealingLevels > 0)) return null;
  const clearance = spec.sashClearanceMm ?? 2;
  const depthGap = 2;
  const framePoints = frame.pts.map(([u,v]) => [u-frame.minU-frame.w,frameInner-frame.d+v-frame.minV]);
  const sashOrigin = sashInner-leaf.d;
  const profile = leaf.pts.map(([u,v]) => [u-leaf.minU-overlap,sashOrigin+v-leaf.minV]);
  const epsilon = 1e-5;
  const frameLevels = [...new Set(framePoints.map(p=>p[1]))];
  const levels = [...new Set([sashOrigin,sashInner,...frameLevels.flatMap(z=>[z-depthGap,z,z+depthGap])
    .filter(z=>z>sashOrigin && z<sashInner)])].sort((a,b)=>a-b);
  function sashLimit(z) {
    // The sash return clears every point of the manufactured frame nose,
    // including its bevel and the transition onto a deeper interior lip.
    const samples = [z-depthGap,z+depthGap,...frameLevels.filter(v=>v>z-depthGap && v<z+depthGap)];
    let limit = -overlap;
    for (const v of samples) for (const probe of [v-epsilon/4,v+epsilon/4]) {
      const edge = edgeAt(framePoints,probe,Math.max);
      if (edge != null) limit = Math.max(limit,edge+clearance);
    }
    return limit;
  }
  const parts = [], sections = [];
  for (let i=1; i<levels.length; i++) {
    const low=levels[i-1], high=levels[i];
    if (high-low <= epsilon*2) continue;
    const a=sashLimit(low+epsilon), b=sashLimit(high-epsilon), slope=(b-a)/(high-low);
    const cut = ring => clipByLine(clipByLine(clipByLine(ring,0,-1,-low),0,1,high),-1,slope,slope*low-a);
    const inSection = ([n,z]) => [n+leaf.minU+overlap,z-sashOrigin+leaf.minV];
    const intoModel = ring => ring.map(([u,v])=>[u-leaf.minU-overlap,sashOrigin+v-leaf.minV]);
    const section=cut(profile);
    if (section.length<3) continue;
    sections.push(section);
    parts.push({...leaf,id:[leaf.id,"system-return",overlap,clearance,a,b,low,high].join("|"),
      pts:section.map(inSection),innerWalls:leaf.innerWalls?.map(r=>cut(intoModel(r))).filter(r=>r.length>=3)
        .map(r=>r.map(inSection))});
  }

  const seals = [];
  const lo = Math.max(frameInner-frame.d,sashOrigin), hi = Math.min(frameInner,sashInner);
  const zones = spec.sealingLevels === 3 ? [["outer",0.16],["middle",0.50],["inner",0.84]]
    : spec.sealingLevels === 2 ? [["outer",0.16],["inner",0.84]] : spec.sealingLevels === 1 ? [["outer",0.16]] : [];
  // On a section without manufacturer groove coordinates the section is a
  // schematic compression lip. Its endpoints are measured on the two solids.
  for (const [level,fraction] of zones) {
    const target=lo+(hi-lo)*fraction;
    const breaks = [...new Set([lo,hi,...levels,...frameLevels,...profile.map(p=>p[1])].filter(v=>v>=lo && v<=hi))]
      .sort((a,b)=>a-b);
    const spans = breaks.slice(1).map((v,i)=>[breaks[i],v]).filter(([l,h])=>h-l>=1)
      .sort((a,b)=>Math.max(0,a[0]-target,target-a[1])-Math.max(0,b[0]-target,target-b[1]));
    for (const [l,h] of spans) {
      const half=Math.min(2,(h-l)/2-0.01), z=Math.min(h-half-0.005,Math.max(l+half+0.005,target));
      const at = zz => {
        const frameEdge=edgeAt(framePoints,zz,Math.max);
        const sashEdges=sections.map(s=>edgeAt(s,zz,Math.min)).filter(v=>v!=null);
        return frameEdge==null || !sashEdges.length ? null : [frameEdge,Math.min(...sashEdges)];
      };
      const back=at(z-half), front=at(z+half);
      const nose=at(z);
      if (!back || !front || !nose || Math.min(back[1]-back[0],front[1]-front[0])<0.2) continue;
      // Thick systems use a matching stop to carry the elastic lip across a
      // wide rebate. The seal compresses against the sash at its narrow nose.
      const land = edges=>Math.max(edges[0],edges[1]-Math.min(8,clearance+2));
      const stop = Math.max(land(back)-back[0],land(front)-front[0])>0.2 ?
        [[back[0],z-half],[land(back),z-half],[land(front),z+half],[front[0],z+half]] : null;
      const rubber = [[land(back),z-half],[nose[1],z],[land(front),z+half]];
      seals.push({level,stop,rubber});
      break;
    }
  }
  return {parts,seals};
}
