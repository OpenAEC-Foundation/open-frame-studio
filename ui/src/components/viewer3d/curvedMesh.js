// The kernel keeps circles exact. Only the displayed mesh is tessellated.
// Limit the chord error in model millimetres and keep quadrant extrema as vertices.
const CHORD_ERROR = 0.08;
const EPS = 1e-7;
const TAU = 2 * Math.PI;

export function contourPoints(contour) {
  const points = [];
  for (const seg of contour?.segments || []) {
    points.push(seg.from);
    if (seg.kind !== "arc" || !seg.radius || !seg.sweepAngle) continue;
    const a = seg.startAngle, b = a + seg.sweepAngle;
    const cuts = [Math.min(a,b)];
    for (let k = Math.floor(cuts[0] / (Math.PI / 2)) + 1; k * Math.PI / 2 < Math.max(a,b) - EPS; k++) {
      cuts.push(k * Math.PI / 2);
    }
    cuts.push(Math.max(a,b));
    if (b < a) cuts.reverse();
    const step = 2 * Math.acos(Math.max(0, 1 - CHORD_ERROR / seg.radius));
    for (let i = 1; i < cuts.length; i++) {
      const n = Math.max(1, Math.ceil(Math.abs(cuts[i] - cuts[i-1]) / step));
      for (let j = i === 1 ? 1 : 0; j < n; j++) {
        const t = cuts[i-1] + (cuts[i] - cuts[i-1]) * j / n;
        points.push([seg.center[0] + seg.radius * Math.cos(t), seg.center[1] + seg.radius * Math.sin(t)]);
      }
    }
  }
  return points;
}

const cross = (ax, ay, bx, by) => ax * by - ay * bx;

function pointInLoop(points, x, y) {
  let inside = false;
  for (let i=0, j=points.length-1; i<points.length; j=i++) {
    const a = points[i], b = points[j];
    if ((a[1] > y) !== (b[1] > y) && x < a[0]+(y-a[1])*(b[0]-a[0])/(b[1]-a[1])) inside = !inside;
  }
  return inside;
}

export function timberPlan(members) {
  const pieces = members.map(piece => ({ piece, loop: contourPoints(piece.outline) }));
  const cuts = [];
  for (const piece of members) {
    for (const cut of [piece.startCut, piece.endCut]) {
      if (!cut || cuts.some(c => c.every((p,i) => Math.hypot(p[0]-cut[i][0],p[1]-cut[i][1]) < EPS))) continue;
      cuts.push(cut);
    }
  }
  return { cuts, at: (x,y) => pieces.find(p => pointInLoop(p.loop,x,y))?.piece,
    nearest: (x,y) => {
      let nearest, best=Infinity;
      for (const {piece} of pieces) {
        const distance=contourSide(piece.outline,x,y).distance;
        if (distance<best) { nearest=piece; best=distance; }
      }
      return nearest;
    } };
}

// Position on the physical saw face. The chord rendering tolerance also
// applies at endpoints where a sampled arc meets that exact face.
function cutPosition(points, cut) {
  const [[ax,ay],[bx,by]] = cut, dx = bx-ax, dy = by-ay;
  const tolerance = 2*CHORD_ERROR / Math.max(1,Math.hypot(dx,dy));
  for (let i=0; i<points.length; i++) {
    const p = points[i], q = points[(i+1)%points.length];
    const ex = q[0]-p[0], ey = q[1]-p[1], den = cross(dx,dy,ex,ey);
    if (Math.abs(den) < EPS) continue;
    const t = cross(p[0]-ax,p[1]-ay,ex,ey)/den;
    const u = cross(p[0]-ax,p[1]-ay,dx,dy)/den;
    if (t >= -tolerance && t <= 1+tolerance && u >= -EPS && u <= 1+EPS) return t;
  }
  return null;
}
function signedArea(points) {
  return points.reduce((sum, p, i) => {
    const q = points[(i + 1) % points.length];
    return sum + cross(p[0], p[1], q[0], q[1]);
  }, 0) / 2;
}

export function clipByLine(points, nx, ny, limit) {
  const result = [];
  for (let i = 0; i < points.length; i++) {
    const p = points[i], q = points[(i+1) % points.length];
    const a = nx*p[0] + ny*p[1] - limit, b = nx*q[0] + ny*q[1] - limit;
    if (a <= 0) result.push(p);
    if ((a < 0 && b > 0) || (a > 0 && b < 0)) {
      const t = a / (a-b);
      result.push(p.map((v,axis) => v + (q[axis]-v)*t));
    }
  }
  return result;
}

function rectPlanes(r) {
  return [[-1,0,-r.x], [1,0,r.x+r.width], [0,-1,-r.y], [0,1,r.y+r.height]];
}

export function clipToRect(points, r) {
  return rectPlanes(r).reduce((p,plane) => clipByLine(p,...plane), points);
}

// A masonry return can interrupt both the curved stock and the central web.
// Keep the part of each triangle outside the return. The masonry slab closes
// the exposed faces at the interface and uses the same analytic outer plan.
export function trimAtMasonry(THREE, mesh, rect) {
  const old = mesh.geometry;
  old.computeBoundingBox();
  const { min, max } = old.boundingBox;
  if (min.x >= rect.x+rect.width || max.x <= rect.x || -max.y >= rect.y+rect.height || -min.y <= rect.y) return;
  const positions = old.getAttribute("position"), indices = old.index;
  const count = indices?.count ?? positions.count;
  const planes = rectPlanes(rect), xyz = [];
  function append(polygon) {
    for (let j = 1; j+1 < polygon.length; j++) {
      for (const [x,y,z] of [polygon[0],polygon[j],polygon[j+1]]) xyz.push(x,-y,z);
    }
  }
  for (let i = 0; i < count; i+=3) {
    let polygon = [i,i+1,i+2].map(k => {
      const n = indices ? indices.getX(k) : k;
      return [positions.getX(n), -positions.getY(n), positions.getZ(n)];
    });
    for (const [nx,ny,limit] of planes) {
      if (polygon.some(([x,y]) => nx*x+ny*y > limit)) {
        append(clipByLine(polygon, -nx,-ny,-limit));
        polygon = clipByLine(polygon, nx,ny,limit);
        if (!polygon.length) break;
      }
    }
  }
  old.dispose();
  if (!xyz.length) { mesh.removeFromParent(); return; }
  mesh.geometry = new THREE.BufferGeometry();
  mesh.geometry.setAttribute("position", new THREE.Float32BufferAttribute(xyz,3));
  mesh.geometry.computeVertexNormals();
}

// Offset a convex (circle-/arch-clipped) opening for a gasket or a stock datum.
// The supporting half-planes also work when a very short edge vanishes.
export function insetPolygon(points, inward) {
  if (points.length < 3 || Math.abs(inward) < EPS) return points;
  const xs = points.map(p => p[0]), ys = points.map(p => p[1]);
  const margin = Math.abs(inward) * 4 + 1;
  const x0 = Math.min(...xs)-margin, y0 = Math.min(...ys)-margin;
  const x1 = Math.max(...xs)+margin, y1 = Math.max(...ys)+margin;
  let poly = [[x0,y0],[x1,y0],[x1,y1],[x0,y1]];
  for (let i = 0; i < points.length && poly.length; i++) {
    const p = points[i], q = points[(i+1) % points.length];
    const dx = q[0]-p[0], dy = q[1]-p[1], len = Math.hypot(dx, dy);
    if (len <= EPS) continue;
    const nx = dy/len, ny = -dx/len;
    poly = clipByLine(poly, nx, ny, nx*p[0] + ny*p[1] - inward);
  }
  return poly;
}

export function contourSlab(THREE, boundary, holes, zMin, depth, material) {
  if (boundary.length < 3 || !(depth > 0)) return null;
  const vectors = pts => pts.map(([x,y]) => new THREE.Vector2(x,-y));
  const shape = new THREE.Shape(vectors(boundary));
  shape.holes = holes.filter(h => h.length >= 3).map(h => new THREE.Path(vectors(h)));
  const geometry = new THREE.ExtrudeGeometry(shape, { depth, bevelEnabled: false });
  const mesh = new THREE.Mesh(geometry, material);
  mesh.position.z = zMin;
  mesh.castShadow = mesh.receiveShadow = true;
  return mesh;
}

// A common angular coordinate lets an oblique joint change topology between
// the glass edge and the back of the rebate. Each section datum follows its
// own normal offset, even when the centre of the cell is off the arch axis.
function polarLoop(points, cx, cy) {
  return points.map(p => ({ p, a: Math.atan2(p[1]-cy,p[0]-cx) })).sort((a,b) => a.a-b.a);
}
function sampleLoop(loop, angles, cx, cy) {
  const side = loop.points;
  return angles.map(station => {
    const a = loop.angleAt(station);
    let lo = 0, i = side.length;
    while (lo < i) {
      const mid = (lo+i) >>> 1;
      if (side[mid].a < a) lo = mid+1; else i = mid;
    }
    const q = side[i % side.length].p;
    if (i < side.length && side[i].a === a) return q;
    const p = side[(i + side.length - 1) % side.length].p;
    const dx = Math.cos(a), dy = Math.sin(a), ex = q[0]-p[0], ey = q[1]-p[1];
    const den = cross(dx,dy,ex,ey);
    if (Math.abs(den) < EPS) return p;
    const t = cross(p[0]-cx, p[1]-cy, ex, ey) / den;
    return [cx+t*dx,cy+t*dy];
  });
}

const turn = a => ((a % TAU) + TAU) % TAU;
const principal = a => turn(a + Math.PI) - Math.PI;

// Link the cut stations on all radii of a profile. A ray from the centre of
// the opening would imply a radial mitre even at a square shoulder.
function transferAngle(angle, from, to) {
  if (!from.length) return angle;
  if (from.length === 1) return principal(angle - from[0] + to[0]);
  const span = (a,b) => Math.abs(principal(b-a)) < 1e-10 ? 0 : turn(b-a);
  const exact = from.findIndex(a => Math.abs(principal(angle-a)) < 1e-10);
  if (exact >= 0) return to[exact];
  const i = from.findIndex((a,i) => turn(angle-a) < span(a,from[(i+1)%from.length]));
  if (i < 0) return to[0];
  const j = (i+1)%from.length;
  const fraction = turn(angle-from[i]) / span(from[i],from[j]);
  return principal(to[i] + fraction * span(to[i],to[j]));
}

function sawLoop(points, cx, cy, anchors) {
  const path = polarLoop(points,cx,cy);
  const reference = [], current = [];
  for (const { cut, angle } of anchors) {
    const at = cutPosition(points,cut);
    if (at == null) continue;
    const [a,b] = cut;
    reference.push(angle);
    current.push(Math.atan2(a[1]+(b[1]-a[1])*at-cy, a[0]+(b[0]-a[0])*at-cx));
  }
  return { points: path, angleAt: t => transferAngle(t,reference,current),
    stations: [...reference, ...path.map(p => transferAngle(p.a,current,reference))] };
}

// Sweep one actual (u,v) stock cross-section around a closed contour.
// outer and inner are the paths at u=0 and u=stock.w respectively. A change
// of stock or of glazing side at a joint terminates and caps each profile.
export function sweepRing(THREE, outer, inner, sectionsAt, zInner, material, role, pathAtDepth, cuts = [], seamMaterial) {
  if (outer.length < 3 || inner.length < 3 || Math.abs(signedArea(inner)) < EPS) return [];
  const cx = inner.reduce((s,p) => s+p[0], 0) / inner.length;
  const cy = inner.reduce((s,p) => s+p[1], 0) / inner.length;
  const anchors = [];
  for (const cut of cuts) {
    // Adjacent pieces can taper to the very same corner at the day. Their
    // distinct outer cut positions still delimit the entire piece of stock.
    const at = cutPosition(outer,cut);
    if (at == null || cutPosition(inner,cut) == null) continue;
    const [a,b] = cut;
    anchors.push({ cut, angle: Math.atan2(a[1]+at*(b[1]-a[1])-cy, a[0]+at*(b[0]-a[0])-cx) });
  }
  anchors.sort((a,b) => a.angle-b.angle);
  const outside = sawLoop(outer,cx,cy,anchors), inside = sawLoop(inner,cx,cy,anchors);
  const loops = new Map();
  const directions = new Set([...outside.stations,...inside.stations]);
  const sectionBetween = (p,q) => sectionsAt((p[0]+q[0])/2,(p[1]+q[1])/2);
  const loopAt = depth => {
    if (!loops.has(depth)) {
      const boundary = pathAtDepth?.(depth) || insetPolygon(inner,-depth);
      loops.set(depth, sawLoop(boundary,cx,cy,anchors));
    }
    return loops.get(depth);
  };
  // Include corners of the intermediate cuts as stations. At an acute joint
  // the spring of the rebate can fall between the outer and day springs.
  for (let i = 0; i < inner.length; i++) {
    for (const section of sectionBetween(inner[i],inner[(i+1)%inner.length])) {
      for (const [u] of section.pts) {
        const depth = section.minU + section.w - u;
        if (depth > EPS && depth < section.w-EPS && !loops.has(depth)) {
          for (const angle of loopAt(depth).stations) directions.add(angle);
        }
      }
    }
  }
  const angles = [...directions].sort((a,b) => a-b).filter((a,i,sorted) => !i || a-sorted[i-1] > 1e-11);
  const outerSamples = sampleLoop(outside, angles, cx, cy), innerSamples = sampleLoop(inside, angles, cx, cy);
  const pieces = innerSamples.map((p, i) => {
    const next = (i+1) % angles.length, q = innerSamples[next];
    const a = outerSamples[i], b = outerSamples[next];
    return sectionsAt((p[0]+q[0])/2,(p[1]+q[1])/2,(p[0]+q[0]+a[0]+b[0])/4,(p[1]+q[1]+a[1]+b[1])/4);
  });
  const buffers = new Map();
  const offsets = new Map();
  function vertices(s, i, points) {
    return points.map(([u,v]) => {
      const depth = s.minU + s.w - u;
      let positions = innerSamples;
      if (depth >= s.w-EPS) positions = outerSamples;
      else if (depth > EPS) {
        if (!offsets.has(depth)) {
          offsets.set(depth, sampleLoop(loopAt(depth), angles,cx,cy));
        }
        positions = offsets.get(depth);
      }
      const [x,y] = positions[i];
      return [x, -y, zInner-s.d+v-s.minV];
    });
  }
  for (let i=0; i<angles.length; i++) {
    const next = (i+1) % angles.length, prev = (i+angles.length-1) % angles.length;
    for (const s of pieces[i]) {
      if (s.pts.length < 3 || !(s.w > 0)) continue;
      let buffer = buffers.get(s.id);
      if (!buffer) {
        const pts = signedArea(s.pts) >= 0 ? s.pts : [...s.pts].reverse();
        const caps = THREE.ShapeUtils.triangulateShape(pts.map(([u,v]) => new THREE.Vector2(u,v)), []);
        buffer = { pts, caps, xyz: [], seams: [], piece: s.timber, leaf: s.leaf, role: s.meshRole || role };
        buffers.set(s.id, buffer);
      }
      const { pts, caps, xyz } = buffer;
      const a = vertices(s, i, pts), b = vertices(s, next, pts);
      for (let j=0; j<pts.length; j++) {
        const k=(j+1)%pts.length;
        xyz.push(...a[j],...b[j],...b[k],...a[j],...b[k],...a[k]);
      }
      if (!pieces[prev].some(other => other.id === s.id)) {
        for (const [j,k,l] of caps) xyz.push(...a[j],...a[k],...a[l]);
        if (s.timber && pieces[prev].some(other => other.timber && other.timber !== s.timber)) {
          for (let j=0; j<a.length; j++) buffer.seams.push(...a[j], ...a[(j+1)%a.length]);
        }
      }
      if (!pieces[next].some(other => other.id === s.id)) {
        for (const [j,k,l] of caps) xyz.push(...b[l],...b[k],...b[j]);
      }
    }
  }
  return [...buffers.values()].map(({ xyz, role, piece, leaf, seams }) => {
    const geo = new THREE.BufferGeometry();
    geo.setAttribute("position",new THREE.Float32BufferAttribute(xyz,3));
    geo.computeVertexNormals();
    const mesh = new THREE.Mesh(geo,material);
    mesh.userData.role = role;
    if (piece) mesh.userData.timber = { role: piece.role, index: piece.index };
    if (leaf) mesh.userData.leaf = leaf;
    if (seams.length && seamMaterial) mesh.add(jointLines(THREE,seams,seamMaterial));
    mesh.castShadow = mesh.receiveShadow = true;
    return mesh;
  });
}

function jointLines(THREE, vertices, material) {
  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute("position",new THREE.Float32BufferAttribute(vertices,3));
  const line = new THREE.LineSegments(geometry,material);
  line.userData.role = "timber-joint";
  line.renderOrder = 1;
  return line;
}

// The solid between neighbouring rebate channels includes both the outer
// stock and the core of the mullions. Partition its existing faceted surfaces
// at the daylight boundary and the same saw planes used by the profile sweeps.
export function splitTimberWeb(THREE, outer, holes, day, z, depth, material, seamMaterial, plan, memberDepth, frameOnlySlab) {
  const slab = frameOnlySlab || contourSlab(THREE,outer,holes,z,depth,material);
  if (!slab) return [];
  const buffers = new Map();
  function emit(poly, owner) {
    if (poly.length < 3) return;
    if (!buffers.has(owner)) buffers.set(owner, { xyz:[], seams:[] });
    const { xyz } = buffers.get(owner);
    const scale = owner ? memberDepth(owner)/depth : 1;
    const innerFace = z+depth;
    for (let j=1; j+1<poly.length; j++) {
      for (const [x,y,h] of [poly[0],poly[j],poly[j+1]]) xyz.push(x,-y,innerFace+(h-innerFace)*scale);
    }
  }
  function ownerAt(poly) {
    const n = poly.length;
    const x=poly.reduce((sum,p) => sum+p[0],0)/n, y=poly.reduce((sum,p) => sum+p[1],0)/n;
    return plan.at(x,y) || (frameOnlySlab && plan.nearest(x,y));
  }
  function plane(a,b) {
    const nx = b[1]-a[1], ny = a[0]-b[0];
    return [nx,ny,nx*a[0]+ny*a[1]];
  }
  function halves(poly,[nx,ny,c]) {
    const d = poly.map(([x,y]) => nx*x+ny*y-c);
    if (Math.max(...d) <= EPS) return [poly,[]];
    if (Math.min(...d) >= -EPS) return [[],poly];
    return [clipByLine(poly,nx,ny,c),clipByLine(poly,-nx,-ny,-c)];
  }
  const sawPlanes = plan.cuts.map(([a,b]) => plane(a,b));
  const dayPlanes = day.map((a,i) => plane(a,day[(i+1)%day.length]));
  const positions = slab.geometry.getAttribute("position");
  for (let i=0; i<positions.count; i+=3) {
    const triangle = [i,i+1,i+2].map(j => [positions.getX(j),-positions.getY(j),slab.position.z+positions.getZ(j)]);
    // Reject a wholly exterior triangle before decomposing a convex ring
    // into strips. Most triangles at the perimeter satisfy this cheap test.
    const exterior = [];
    let inner = triangle;
    if (frameOnlySlab || dayPlanes.some(([nx,ny,c]) => triangle.every(([x,y]) => nx*x+ny*y >= c-EPS))) {
      exterior.push(triangle);
    } else {
      for (const edge of dayPlanes) {
        const [kept,tail] = halves(inner,edge);
        if (tail.length) exterior.push(tail);
        inner = kept;
        if (!inner.length) break;
      }
      emit(inner,null);
    }
    for (const poly of exterior) {
      let blocks = [poly];
      for (const edge of sawPlanes) blocks = blocks.flatMap(p => halves(p,edge).filter(part => part.length));
      for (const block of blocks) emit(block,ownerAt(block));
    }
  }
  slab.geometry.dispose();
  // Where the flat core meets the same cut in the machined bands, continue
  // its end-grain faces and the two visible joint lines right to their edges.
  for (const cut of plan.cuts) {
    const [a,b] = cut, dx=b[0]-a[0], dy=b[1]-a[1], len=Math.hypot(dx,dy);
    if (len < EPS) continue;
    const at = t => [a[0]+t*dx,a[1]+t*dy];
    const stops = [0,1,...[outer,...holes].map(loop => cutPosition(loop,cut)).filter(t => t != null && t>0 && t<1)].sort((u,v) => u-v);
    for (let i=1; i<stops.length; i++) {
      if (stops[i]-stops[i-1] < EPS) continue;
      const [x,y] = at((stops[i]+stops[i-1])/2);
      if (!pointInLoop(outer,x,y) || holes.some(h => pointInLoop(h,x,y))) continue;
      const p = at(stops[i-1]), q = at(stops[i]);
      const cap = [[...p,z],[...q,z],[...q,z+depth],[...p,z+depth]];
      const nx = -dy/len*0.2, ny = dx/len*0.2;
      const first = plan.at(x+nx,y+ny), second = plan.at(x-nx,y-ny);
      if (!first && !second) continue;
      // Model y is inverted in the emitted triangles.
      if (first) emit(cap.slice().reverse(),first);
      if (second) emit(cap,second);
      const owner = first || second, { seams } = buffers.get(owner);
      for (const h of [z+depth-memberDepth(owner),z+depth]) seams.push(p[0],-p[1],h,q[0],-q[1],h);
    }
  }
  return [...buffers.entries()].map(([piece,{ xyz,seams }]) => {
    const geo = new THREE.BufferGeometry();
    geo.setAttribute("position",new THREE.Float32BufferAttribute(xyz,3));
    geo.computeVertexNormals();
    const mesh = new THREE.Mesh(geo,material);
    mesh.userData.role = piece ? "frame-member" : "divider";
    if (piece) mesh.userData.timber = { role:piece.role, index:piece.index };
    if (seams.length) mesh.add(jointLines(THREE,seams,seamMaterial));
    mesh.castShadow = mesh.receiveShadow = true;
    return mesh;
  });
}

export function contourSide(contour, x, y) {
  let nearest = null, distance = Infinity;
  for (const seg of contour.segments) {
    const [a,b] = seg.from, [c,d] = seg.to;
    let dist;
    if (seg.kind === "arc") {
      const dx = x-seg.center[0], dy = y-seg.center[1];
      const angle = turn(Math.sign(seg.sweepAngle)*(Math.atan2(dy,dx)-seg.startAngle));
      dist = angle <= Math.abs(seg.sweepAngle) + EPS ? Math.abs(Math.hypot(dx,dy)-seg.radius) :
        Math.min(Math.hypot(x-a,y-b),Math.hypot(x-c,y-d));
    } else {
      const dx = c-a, dy = d-b;
      const t = Math.max(0,Math.min(1,((x-a)*dx+(y-b)*dy)/(dx*dx+dy*dy || 1)));
      dist = Math.hypot(x-a-t*dx,y-b-t*dy);
    }
    if (dist < distance) { nearest = seg; distance = dist; }
  }
  if (!nearest || nearest.kind === "arc") return { side:"top", curved:true, distance };
  const dx = nearest.to[0]-nearest.from[0], dy = nearest.to[1]-nearest.from[1];
  const side = Math.abs(dx)>Math.abs(dy) ? dx>0 ? "top" : "bottom" : dy>0 ? "right" : "left";
  return { side, curved:false, distance };
}
