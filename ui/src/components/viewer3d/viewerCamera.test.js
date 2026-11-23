import test from 'node:test';
import assert from 'node:assert/strict';
import { OrthographicCamera, PerspectiveCamera, Vector3 } from 'three';
import { VIEW_FOV, HOME_DIRECTION, anglesForDirection, orbitOrientation, dragOrbit, snapOrbit, fitDistance, applyViewerCamera } from './viewerCamera.js';

const pivot = [50, 725, -12];
const home = anglesForDirection(HOME_DIRECTION);
const near = (value, expected) => assert.ok(Math.abs(value - expected) < 1e-8, `${value} ≠ ${expected}`);

test('all 26 cube directions preserve the orbit centre, distance and right-handed screen axes', () => {
  const camera = new PerspectiveCamera(VIEW_FOV);
  for (const x of [-1, 0, 1]) for (const y of [-1, 0, 1]) for (const z of [-1, 0, 1]) {
    if (!x && !y && !z) continue;
    const expected = new Vector3(x, y, z).normalize();
    const angles = anglesForDirection([x, y, z]);
    for (const t of [0.3, 1]) {
      const orientation = orbitOrientation(snapOrbit(home, angles, t));
      applyViewerCamera(camera, orientation, pivot, 1400, 1.5, 1100);
      const center = new Vector3(...pivot);
      near(center.distanceTo(camera.position), 1400);
      const projected = center.project(camera);
      near(projected.x, 0);
      near(projected.y, 0);
      const screenRight = new Vector3(...orientation.right);
      const eye = screenRight.clone().cross(new Vector3(...orientation.up));
      near(eye.distanceTo(new Vector3(...orientation.eye)), 0);
      near(screenRight.distanceTo(new Vector3().setFromMatrixColumn(camera.matrixWorld, 0)), 0);
      if (t === 1) near(eye.distanceTo(expected), 0);
    }
  }
});

test('top and bottom have +X to the right and a free drag leaves either pole smoothly', () => {
  for (const sign of [-1, 1]) {
    const pole = anglesForDirection([0, sign, 0]);
    const orientation = orbitOrientation(pole);
    near(orientation.right[0], 1);
    near(orientation.up[2], -sign);
    const dragged = orbitOrientation(dragOrbit(pole, 0, -sign * 2));
    near(dragged.right[0], 1);
    assert.ok(dragged.eye[2] > 0 && dragged.eye[2] < 0.011);
    near(dragged.eye[1], sign * Math.cos(0.01));
  }
  const mid = snapOrbit({ yaw: Math.PI - 0.1, pitch: 0 }, { yaw: -Math.PI + 0.1, pitch: 0 }, 0.5);
  near(mid.yaw, Math.PI);
});

test('switching projection preserves scale in the target plane at different zooms and aspect ratios', () => {
  const orientation = orbitOrientation(home);
  const planePoint = new Vector3(...pivot).addScaledVector(new Vector3(...orientation.right), 160)
    .addScaledVector(new Vector3(...orientation.up), 70);
  for (const aspect of [0.45, 1.8]) for (const distance of [150, 6000]) {
    const perspective = new PerspectiveCamera(VIEW_FOV);
    const orthographic = new OrthographicCamera();
    for (const camera of [perspective, orthographic]) applyViewerCamera(camera, orientation, pivot, distance, aspect, 1400);
    const p = planePoint.clone().project(perspective), o = planePoint.clone().project(orthographic);
    near(p.x, o.x);
    near(p.y, o.y);
    const further = planePoint.clone().addScaledVector(new Vector3(...orientation.eye), -50).project(orthographic);
    near(further.x, o.x);
    near(further.y, o.y);
  }
});

test('home frames all corners of a wide model even in a narrow pane', () => {
  const dimensions = [9000, 4300, 240];
  const bound = Math.hypot(...dimensions) / 2;
  for (const aspect of [0.32, 2.2]) for (const camera of [new PerspectiveCamera(VIEW_FOV), new OrthographicCamera()]) {
    applyViewerCamera(camera, orbitOrientation(home), pivot, fitDistance(bound, aspect), aspect, bound);
    for (const x of [-1, 1]) for (const y of [-1, 1]) for (const z of [-1, 1]) {
      const p = new Vector3(x * dimensions[0] / 2, y * dimensions[1] / 2, z * dimensions[2] / 2)
        .add(new Vector3(...pivot)).project(camera);
      assert.ok(Math.max(Math.abs(p.x), Math.abs(p.y), Math.abs(p.z)) < 1);
    }
  }
});
