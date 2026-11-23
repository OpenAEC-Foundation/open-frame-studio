import { cutCaps, stockMesh } from "./dividerMesh.js";

// The inspector uses the actual swept solids. A flat cap preserves the space
// between glass, rubber and extrusion when looking down onto the cross-cut.
export function sectionCapsAt(THREE, group, y) {
  const caps = new THREE.Group();
  for (const mesh of group.children) {
    if (!mesh.isMesh) continue;
    mesh.updateMatrix();
    const geometry = mesh.geometry;
    if (!geometry.boundingBox) geometry.computeBoundingBox();
    const bounds = geometry.boundingBox.clone().applyMatrix4(mesh.matrix);
    if (-y <= bounds.min.y || -y >= bounds.max.y) continue;
    // Only a few stations of a curved extrusion touch this plane. Keep
    // the full mesh in its existing coordinates and transform those faces.
    const positions = geometry.getAttribute("position"), indices=geometry.index;
    const matrix=mesh.matrix.elements, point=new THREE.Vector3();
    const worldY=i=>matrix[1]*positions.getX(i)+matrix[5]*positions.getY(i)+matrix[9]*positions.getZ(i)+matrix[13];
    const modelPoint=i=>{
      point.fromBufferAttribute(positions,i).applyMatrix4(mesh.matrix);
      return [point.x,-point.y,point.z];
    };
    const faces=[];
    for (let i=0;i<(indices?.count ?? positions.count);i+=3) {
      const a=indices ? indices.getX(i) : i;
      const b=indices ? indices.getX(i+1) : i+1;
      const c=indices ? indices.getX(i+2) : i+2;
      const ya=worldY(a), yb=worldY(b), yc=worldY(c);
      if (Math.min(ya,yb,yc)>-y+0.00001 || Math.max(ya,yb,yc)<-y-0.00001) continue;
      faces.push([modelPoint(a),modelPoint(b),modelPoint(c)]);
    }
    const cuts = cutCaps(faces,{nx:0,ny:-1,c:-y,t0:bounds.min.x-1,t1:bounds.max.x+1});
    const cap = stockMesh(THREE,cuts,mesh.material,"section");
    if (cap) caps.add(cap);
  }
  return caps;
}
