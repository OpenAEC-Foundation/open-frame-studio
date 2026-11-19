import { clipByLine, clipToRect, contourPoints, contourSide } from "./curvedMesh.js";

const EPS = 0.00001;

// Each finished divider is a convex portion of the frame day. Use the very
// same faceted boundary for the two mating solids, including the pocket walls.
export function dividerFootprint(member) {
  const loop=contourPoints(member.outline);
  return {member, ...footprint(loop, [...member.start.segments,...member.end.segments])};
}

// A swept cell band is already bounded by the profile and the frame day.
// Separate the surrounding stock along its actual straight joint planes.
// A half-ring contributes its diameter; a headed frame contributes its
// shoulders and jamb faces. The curved inner edge is never a cutting plane.
export function frameBandFootprint(member) {
  const loop=contourPoints(member.outline), xs=loop.map(p=>p[0]), ys=loop.map(p=>p[1]);
  const planes=[];
  for (const edge of member.outline.segments) {
    if (edge.kind !== "line") continue;
    const [x,y]=edge.from, [u,v]=edge.to, len=Math.hypot(u-x,v-y);
    if (len < EPS) continue;
    let nx=(v-y)/len, ny=(x-u)/len;
    const sides=loop.map(p=>nx*(p[0]-x)+ny*(p[1]-y));
    if (Math.max(...sides)>EPS) {
      if (Math.min(...sides)<-EPS) continue;
      nx=-nx; ny=-ny;
    }
    const t0=-ny*x+nx*y, t1=-ny*u+nx*v;
    planes.push({nx,ny,c:nx*x+ny*y,t0:Math.min(t0,t1),t1:Math.max(t0,t1)});
  }
  return {member,planes,box:[Math.min(...xs),Math.min(...ys),Math.max(...xs),Math.max(...ys)]};
}

function footprint(loop, ends=[]) {
  const xs = loop.map(p => p[0]), ys = loop.map(p => p[1]);
  const winding = Math.sign(loop.reduce((s, p, i) => {
    const q = loop[(i+1) % loop.length];
    return s+p[0]*q[1]-q[0]*p[1];
  }, 0));
  const planes = [];
  for (let i=0; i<loop.length; i++) {
    const [x,y] = loop[i], [u,v] = loop[(i+1)%loop.length];
    const length = Math.hypot(u-x,v-y);
    if (length < EPS) continue;
    const nx = winding*(v-y)/length, ny = winding*(x-u)/length;
    const t=-ny*x+nx*y, s=-ny*u+nx*v;
    planes.push({nx, ny, c:nx*x+ny*y, t0:Math.min(t,s), t1:Math.max(t,s),
      end:contourSide({segments:ends}, (x+u)/2, (y+v)/2).distance < 0.09});
  }
  // Cull the long parallel sides first, before walking the sampled end arcs.
  planes.sort((a,b) => Number(a.end)-Number(b.end));
  return {planes, box:[Math.min(...xs),Math.min(...ys),Math.max(...xs),Math.max(...ys)]};
}

// Between two masonry leaves the nominal splitter can be wholly or partly
// absent. The drawing, casting and timber list must all leave that span free.
export function masonrySpans(day, members, verticalRects=[], horizontalRects=[]) {
  const gaps=[];
  for (const vertical of [true,false]) {
    const rects=vertical ? verticalRects : horizontalRects, axis=vertical ? 1 : 0;
    for (const [index,rect] of rects.entries()) {
      const blank=clipToRect(day,rect);
      if (blank.length<3) continue;
      const extent=blank.map(p=>p[axis]);
      const pieces=members.filter(p=>p.member.vertical===vertical && p.member.sourceIndex===index)
        .map(({box})=>[box[axis],box[axis+2]]).sort((a,b)=>a[0]-b[0]);
      let start=Math.min(...extent);
      for (const [end,next] of [...pieces,[Math.max(...extent),Infinity]]) {
        if (end-start>0.2) {
          const cut=vertical ? {...rect,y:start,height:end-start} : {...rect,x:start,width:end-start};
          const loop=clipToRect(blank,cut);
          if (loop.length>=3) gaps.push({loop,...footprint(loop)});
        }
        start=next;
      }
    }
  }
  return gaps;
}

export function touchesPocket(footprint, pocket, bounds) {
  const [x0,y0,x1,y1]=footprint.box;
  if (x0>=bounds.x+bounds.width || x1<=bounds.x || y0>=bounds.y+bounds.height || y1<=bounds.y) return false;
  let intersection=pocket;
  for (const {nx,ny,c} of footprint.planes) {
    intersection=clipByLine(intersection,nx,ny,c);
    if (intersection.length<3) return false;
  }
  return true;
}

// The swept pocket, flat web and end-grain all use model coordinates (y-down).
export function meshFaces(mesh) {
  const pos = mesh.geometry.getAttribute("position"), indices = mesh.geometry.index;
  const faces = [];
  for (let i=0; i<(indices?.count ?? pos.count); i+=3) {
    faces.push([i,i+1,i+2].map(j => {
      const k = indices ? indices.getX(j) : j;
      return [pos.getX(k),-pos.getY(k),pos.getZ(k)+mesh.position.z];
    }));
  }
  return faces;
}

function triangulate(faces) {
  const xyz = [];
  for (const p of faces) for (let i=1; i+1<p.length; i++) {
    for (const [x,y,z] of [p[0],p[i],p[i+1]]) xyz.push(x,-y,z);
  }
  return xyz;
}

export function stockMesh(THREE, faces, material, role) {
  if (!faces.length) return null;
  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute("position", new THREE.Float32BufferAttribute(triangulate(faces),3));
  geometry.computeVertexNormals();
  const mesh = new THREE.Mesh(geometry, material);
  mesh.userData.role = role;
  mesh.castShadow = mesh.receiveShadow = true;
  return mesh;
}

// Intersections with a vertical cutting plane are straight in (t,z), where t
// runs along the facet. Fill only material between successive surface hits.
// This preserves an open rebate, a projecting lip and disconnected sections.
function sectionCaps(edges, {nx,ny,c,t0,t1}) {
  const stops = [...new Set([t0,t1,...edges.flatMap(e => [e[0],e[2]]).filter(t=>t>t0 && t<t1)])].sort((a,b) => a-b);
  const faces = [];
  const point = (t,z) => [nx*c-ny*t, ny*c+nx*t, z];
  for (let i=1; i<stops.length; i++) {
    const a = stops[i-1], b = stops[i];
    if (b-a < EPS) continue;
    const mid = (a+b)/2;
    const walls = edges.filter(([l,,r]) => l<mid && mid<r).map(([l,z,r,w]) => {
      const slope=(w-z)/(r-l);
      return [z+(a-l)*slope, z+(b-l)*slope];
    }).sort((e,f) => e[0]+e[1]-f[0]-f[1]);
    for (let j=1; j<walls.length; j+=2) {
      const [za,zb]=walls[j-1], [wa,wb]=walls[j];
      if (wa+wb-za-zb > EPS) faces.push([point(a,za),point(a,wa),point(b,wb),point(b,zb)]);
    }
  }
  return faces;
}

function cutCaps(faces, plane) {
  const {nx,ny,c,t0,t1} = plane, edges = [];
  let crosses=false;
  for (const p of faces) {
    const distance = p.map(([x,y]) => nx*x+ny*y-c);
    const lo = Math.min(...distance), hi = Math.max(...distance);
    if (hi < -EPS || lo > EPS || hi-lo < EPS) continue;
    const contacts = [];
    for (let i=0; i<p.length; i++) {
      const j=(i+1)%p.length, a=distance[i], b=distance[j];
      if (Math.abs(a)<=EPS) contacts.push(p[i]);
      if (a*b<0 && Math.abs(a)>EPS && Math.abs(b)>EPS) {
        contacts.push(p[i].map((v,k) => v+(p[j][k]-v)*a/(a-b)));
      }
    }
    const ts = contacts.map(([x,y,z]) => [-ny*x+nx*y,z]).sort((a,b) => a[0]-b[0]);
    if (ts.length>=2 && ts[0][0]<t1 && ts.at(-1)[0]>t0 && ts.at(-1)[0]-ts[0][0]>EPS) {
      // The sweep often has a station exactly on the saw plane. Its two
      // adjacent triangles touch the cut without straddling it. One supplies
      // the section edge; the other confirms there was stock on the far side.
      if (lo < -EPS) edges.push([...ts[0],...ts.at(-1)]);
      if (hi>EPS) crosses=true;
    }
  }
  return crosses ? sectionCaps(edges,plane) : [];
}

export function clipStock(faces, footprint, keepInside) {
  const [x0,y0,x1,y1]=footprint.box, outside=[];
  let inside=[];
  for (const face of faces) {
    if (face.every(p=>p[0]<x0-EPS) || face.every(p=>p[0]>x1+EPS) ||
        face.every(p=>p[1]<y0-EPS) || face.every(p=>p[1]>y1+EPS)) {
      if (!keepInside) outside.push(face);
    } else inside.push(face);
  }
  if (!inside.length) return outside;
  const candidates=inside;
  const caps=[];
  for (const plane of footprint.planes) {
    const {nx,ny,c}=plane, next=[];
    for (const face of inside) {
      const d=face.map(([x,y])=>nx*x+ny*y-c);
      if (Math.max(...d)<=EPS) next.push(face);
      else if (Math.min(...d)>=-EPS) { if (!keepInside) outside.push(face); }
      else {
        next.push(clipByLine(face,nx,ny,c));
        if (!keepInside) outside.push(clipByLine(face,-nx,-ny,-c));
      }
    }
    if (!next.length) return keepInside ? [] : faces;
    caps.push(...cutCaps(candidates,plane));
    inside=next;
  }
  return keepInside ? inside.concat(caps) : outside.concat(caps.map(p=>p.reverse()));
}

export function dividerMesh(THREE, faces, footprint, material, seamMaterial) {
  const mesh = stockMesh(THREE,faces,material,"divider");
  if (!mesh) return null;
  const {vertical,sourceIndex,pieceIndex} = footprint.member;
  mesh.userData.divider = {vertical,sourceIndex,pieceIndex};
  const seams=[];
  for (const {nx,ny,c,end} of footprint.planes) {
    if (!end) continue;
    for (const p of faces) {
      const on=p.map(([x,y])=>Math.abs(nx*x+ny*y-c)<0.0005);
      if (on.every(Boolean)) continue;
      for (let i=0; i<p.length; i++) {
        const j=(i+1)%p.length;
        if (on[i] && on[j] && Math.hypot(p[j][0]-p[i][0],p[j][1]-p[i][1])>EPS) {
          for (const [x,y,z] of [p[i],p[j]]) seams.push(x,-y,z);
        }
      }
    }
  }
  if (seams.length) {
    const geo=new THREE.BufferGeometry();
    geo.setAttribute("position",new THREE.Float32BufferAttribute(seams,3));
    const lines=new THREE.LineSegments(geo,seamMaterial);
    lines.userData.role="timber-joint";
    lines.renderOrder=1;
    mesh.add(lines);
  }
  return mesh;
}
