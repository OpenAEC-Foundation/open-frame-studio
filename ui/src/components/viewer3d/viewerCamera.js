// Camera axes in frame coordinates: +Y is up, +Z inside, −Z outside.
export const VIEW_FOV = 45;
export const HOME_DIRECTION = [1, 1, 1];
const HALF_PI = Math.PI / 2;
const TAN_HALF_FOV = Math.tan(VIEW_FOV * Math.PI / 360);

export function anglesForDirection([x, y, z]) {
  const horizontal = Math.hypot(x, z);
  return { yaw: horizontal ? Math.atan2(x, z) : 0, pitch: Math.atan2(y, horizontal) };
}

export function orbitOrientation({ yaw, pitch }) {
  const s = Math.sin(yaw), c = Math.cos(yaw);
  const h = Math.sin(pitch), r = Math.cos(pitch);
  // An explicit screen-up vector keeps exact top and bottom views well defined.
  return { right: [c, 0, -s], up: [-h * s, r, -h * c], eye: [r * s, h, r * c] };
}

export function dragOrbit({ yaw, pitch }, dx, dy) {
  return { yaw: yaw - dx * 0.005, pitch: Math.max(-HALF_PI, Math.min(HALF_PI, pitch + dy * 0.005)) };
}

export function snapOrbit(from, to, progress) {
  if (progress >= 1) return to;
  const t = Math.max(0, progress);
  const blend = t * t * (3 - 2 * t);
  const difference = to.yaw - from.yaw;
  const shortest = Math.atan2(Math.sin(difference), Math.cos(difference));
  return { yaw: from.yaw + shortest * blend, pitch: from.pitch + (to.pitch - from.pitch) * blend };
}

export function fitDistance(boundRadius, aspect) {
  // A sphere around the frame fits every cube direction in either projection.
  const halfAngle = Math.atan(TAN_HALF_FOV * Math.min(1, aspect));
  return Math.max(1, boundRadius) * 1.08 / Math.sin(halfAngle);
}

export function applyViewerCamera(camera, orientation, pivot, distance, aspect, boundRadius) {
  if (camera.isOrthographicCamera) {
    // Match the perspective field at the pivot, including after wheel zoom.
    const halfHeight = distance * TAN_HALF_FOV;
    camera.top = halfHeight;
    camera.bottom = -halfHeight;
    camera.right = halfHeight * aspect;
    camera.left = -camera.right;
  } else {
    camera.aspect = aspect;
  }
  camera.near = Math.max(0.1, Math.min(10, distance / 1000));
  camera.far = distance + Math.max(5000, boundRadius * 3);
  camera.updateProjectionMatrix();
  const [x, y, z] = orientation.eye;
  camera.position.set(pivot[0] + distance * x, pivot[1] + distance * y, pivot[2] + distance * z);
  camera.up.fromArray(orientation.up);
  camera.lookAt(...pivot);
  camera.updateMatrixWorld();
}
