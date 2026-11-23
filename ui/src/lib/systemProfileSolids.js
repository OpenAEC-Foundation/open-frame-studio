import { generateProfileGeometry } from "./profileContour.js";

// A library section contains chamber outlines as well as positive inserts.
// Keep the distinction when a profile is cut into half-mullions or sash returns.
// Convex strips can be clipped independently even when a cut opens a chamber.
export function systemProfileSolids(profile) {
  if (!["pvc", "aluminum"].includes(profile?.material) || !profile.innerWalls?.length) return null;
  const rings = [profile.crossSection, ...profile.innerWalls];
  if (!rings.every(r => Array.isArray(r) && r.length >= 3 &&
      r.every(p => Array.isArray(p) && p.length === 2 && p.every(Number.isFinite)))) return null;
  let types = profile.innerWallTypes;
  if (!types) {
    // Older catalogues store the generator's rings without their types.
    // Preserve arbitrary imported CAD linework unless its meaning is known.
    const generated = generateProfileGeometry(profile);
    if (JSON.stringify(generated.crossSection) !== JSON.stringify(profile.crossSection) ||
        JSON.stringify(generated.innerWalls) !== JSON.stringify(profile.innerWalls)) return null;
    types = generated.innerWallTypes;
  }
  if (!Array.isArray(types) || types.length !== profile.innerWalls.length ||
      types.some(t => !["chamber", "reinforcement", "insulator"].includes(t))) return null;
  const kinds = [profile.material, ...types];
  const bridge = rings.filter((_, i) => kinds[i] === "insulator").flat().map(p => p[1]);
  const bridgeLo = Math.min(...bridge);
  const bridgeHi = Math.max(...bridge);
  const ys = [...new Set(rings.flatMap(r => r.map(p => p[1])))].sort((a, b) => a - b);
  const parts = [];
  for (let j = 1; j < ys.length; j++) {
    const lo = ys[j - 1], hi = ys[j], middle = (lo + hi) / 2;
    const crossings = [];
    rings.forEach((ring, index) => {
      ring.forEach(([x, y], i) => {
        const [u, v] = ring[(i + 1) % ring.length];
        if ((y < middle) === (v < middle)) return;
        const at = level => x + (u - x) * (level - y) / (v - y);
        crossings.push({ index, lower: at(lo), upper: at(hi), mid: at(middle) });
      });
    });
    crossings.sort((a, b) => a.mid - b.mid);
    const inside = new Set();
    for (let i = 0; i + 1 < crossings.length; i++) {
      const left = crossings[i], right = crossings[i + 1];
      if (inside.has(left.index)) inside.delete(left.index);
      else inside.add(left.index);
      if (!inside.has(0) || right.mid - left.mid < 1e-7) continue;
      const layers = [...inside].map(k => kinds[k]);
      let material = profile.material;
      if (layers.includes("reinforcement")) material = "reinforcement";
      else if (layers.includes("insulator")) material = "insulator";
      else if (layers.includes("chamber") || (middle > bridgeLo && middle < bridgeHi)) continue;
      parts.push({ material, points: [[left.lower, lo], [right.lower, lo], [right.upper, hi], [left.upper, hi]] });
    }
  }
  return parts;
}

export function transformSectionSolids(solids, transformRing) {
  return solids?.map(part => ({ ...part, points: transformRing(part.points) })).filter(({ points }) => {
    if (points.length < 3) return false;
    const doubleArea = points.reduce((sum, [x,y], i) => {
      const [u,v] = points[(i + 1) % points.length];
      return sum + x * v - u * y;
    }, 0);
    return Math.abs(doubleArea) > 1e-7;
  });
}

// Only the perimeter needs sweeping. Adjacent convex strips can meet along
// part of a horizontal edge; cancel that interval without closing a chamber.
export function sectionSweepLayers(solids) {
  const layers = new Map();
  const snap = x => Math.round(x * 1e7) / 1e7;
  for (const {material, points} of solids) {
    if (!layers.has(material)) layers.set(material, {
      material, pts:[], caps:[], edges:[], indices:new Map(), sides:new Map(), levels:new Map()
    });
    const layer = layers.get(material);
    const positive = points.reduce((s,[x,y],i) => {
      const [u,v]=points[(i+1)%points.length];
      return s+x*v-u*y;
    },0) >= 0;
    const polygon = positive ? points : [...points].reverse();
    const ids = polygon.map(([x,y]) => vertex(layer,x,y));
    for (let i=1;i+1<ids.length;i++) {
      if (ids[0]!==ids[i] && ids[i]!==ids[i+1] && ids[0]!==ids[i+1]) layer.caps.push([ids[0],ids[i],ids[i+1]]);
    }
    for (let i=0; i<ids.length; i++) {
      const a=ids[i], b=ids[(i+1)%ids.length];
      if (a===b) continue;
      const [x,y]=layer.pts[a], [u,v]=layer.pts[b];
      if (y===v) {
        if (!layer.levels.has(y)) layer.levels.set(y,new Map());
        const changes=layer.levels.get(y);
        changes.set(x,(changes.get(x)||0)+1);
        changes.set(u,(changes.get(u)||0)-1);
      } else {
        if (!layer.sides.delete(b+","+a)) layer.sides.set(a+","+b,[a,b]);
      }
    }
  }
  function vertex(layer,x,y) {
    x=snap(x); y=snap(y);
    const key=x+","+y;
    if (!layer.indices.has(key)) {
      layer.indices.set(key,layer.pts.length);
      layer.pts.push([x,y]);
    }
    return layer.indices.get(key);
  }
  for (const layer of layers.values()) {
    layer.edges.push(...layer.sides.values());
    for (const [y,changes] of layer.levels) {
      let start, direction=0;
      for (const [x,delta] of [...changes].sort((a,b)=>a[0]-b[0])) {
        if (!delta) continue;
        if (direction) {
          const a=vertex(layer,start,y), b=vertex(layer,x,y);
          layer.edges.push(direction>0 ? [a,b] : [b,a]);
        }
        direction+=delta;
        start=x;
      }
    }
  }
  return [...layers.values()].map(({material,pts,edges,caps})=>{
    // Rejoin a wall subdivided by unrelated chambers at other heights.
    // A junction with multiple boundaries must keep its original topology.
    const incoming=new Map(), outgoing=new Map();
    for (const [a,b] of edges) {
      outgoing.set(a,outgoing.has(a) ? null : b);
      incoming.set(b,incoming.has(b) ? null : a);
    }
    const straight=i=>{
      const a=incoming.get(i), b=outgoing.get(i);
      if (a==null || b==null) return false;
      const [x,y]=pts[i], dx=x-pts[a][0], dy=y-pts[a][1], ex=pts[b][0]-x, ey=pts[b][1]-y;
      return Math.abs(dx*ey-dy*ex)<1e-7 && dx*ex+dy*ey>0;
    };
    const joined=[];
    for (const [a,b] of edges) {
      if (straight(a)) continue;
      let end=b;
      while (straight(end)) end=outgoing.get(end);
      joined.push([a,end]);
    }
    const used=[...new Set(joined.flat())], boundaryIds=new Map(used.map((v,i)=>[v,i]));
    return {material,pts:used.map(i=>pts[i]),edges:joined.map(e=>e.map(i=>boundaryIds.get(i))),capPts:pts,caps};
  });
}
