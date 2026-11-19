import { cutCaps, meshFaces, stockMesh } from "./dividerMesh.js";

// The inspector uses the actual swept solids. A flat cap preserves the space
// between glass, rubber and extrusion when looking down onto the cross-cut.
export function sectionCapsAt(THREE, group, y) {
  const caps = new THREE.Group();
  for (const mesh of group.children) {
    if (!mesh.isMesh) continue;
    mesh.updateMatrix();
    const geometry = mesh.geometry;
    geometry.computeBoundingBox();
    const bounds = geometry.boundingBox.clone().applyMatrix4(mesh.matrix);
    if (-y <= bounds.min.y || -y >= bounds.max.y) continue;
    const transformed = geometry.clone().applyMatrix4(mesh.matrix);
    const faces = meshFaces({geometry:transformed,position:{z:0}});
    transformed.dispose();
    const cuts = cutCaps(faces,{nx:0,ny:-1,c:-y,t0:bounds.min.x-1,t1:bounds.max.x+1});
    const cap = stockMesh(THREE,cuts,mesh.material,"section");
    if (cap) caps.add(cap);
  }
  return caps;
}
