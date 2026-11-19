import { fittingDrawing } from "../../lib/hardwareDrawing.js";

export function addHardware(THREE, group, jobs, frameInner, frameDepth) {
  if (!jobs.some(job => job.hardwareLeaves?.some(leaf => leaf.components?.length))) return;
  const metal = new THREE.MeshStandardMaterial({ color: 0x9da2a6, metalness: 0.7, roughness: 0.32 });
  const trim = new THREE.MeshStandardMaterial({ color: 0x4f5356, metalness: 0.45, roughness: 0.45 });
  const fixtures = new Map();
  const sash = group.children.filter(mesh => mesh.userData.role === "sash");
  const frame = group.children.filter(mesh => ["frame-member","divider"].includes(mesh.userData.role));
  group.updateMatrixWorld(true);
  const ray = new THREE.Raycaster();
  for (const job of jobs) for (const leaf of job.hardwareLeaves || []) {
    if (leaf.issue) continue;
    for (const part of leaf.components || []) {
      const drawing = fittingDrawing(part,leaf.set);
      if (!drawing) continue;
      const [x,y] = part.position, [tx,ty] = part.tangent;
      const fixed = part.mount === "frame";
      const inside = fixed ? frameInner : job.sashInner;
      const depth = fixed ? frameDepth : job.sash?.d || frameDepth;
      const signs = drawing.bothFaces ? [1,-1] : [drawing.face === "opening" && leaf.opensOutward ? -1 : 1];
      for (const sign of signs) {
        let z;
        if (drawing.face === "rebate") {
          z = (Math.max(frameInner-frameDepth,job.sashInner-(job.sash?.d || frameDepth)) +
            Math.min(frameInner,job.sashInner)) / 2;
        } else {
          ray.set(new THREE.Vector3(x,-y,sign*(frameDepth+depth+100)),new THREE.Vector3(0,0,-sign));
          z = ray.intersectObjects(fixed ? frame : sash,false)[0]?.point.z ?? (sign<0 ? inside-depth : inside);
        }
        for (const [along,inward,offset,length,width,height,shape] of drawing.boxes) {
          const key = [length,width,height,shape].join("|");
          if (!fixtures.has(key)) {
            let geometry;
            if (shape==="barrel") geometry=new THREE.CylinderGeometry(width/2,width/2,length,12).rotateZ(Math.PI/2);
            else if (shape==="disc") geometry=new THREE.CylinderGeometry(width/2,width/2,height,16).rotateX(Math.PI/2);
            else geometry=new THREE.BoxGeometry(length,width,height);
            fixtures.set(key,geometry);
          }
          const mesh = new THREE.Mesh(fixtures.get(key),drawing.face === "rebate" ? trim : metal);
          mesh.position.set(x+along*tx-inward*ty,-(y+along*ty+inward*tx),z+sign*(offset+height/2));
          mesh.rotation.z = -Math.atan2(ty,tx);
          mesh.castShadow = true;
          mesh.userData = {role:"hardware", hardware:{cellIndex:job.cellIndex,leaf:leaf.leaf,...part}};
          group.add(mesh);
        }
      }
    }
  }
}
