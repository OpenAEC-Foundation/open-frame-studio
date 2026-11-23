import test from "node:test";
import assert from "node:assert/strict";
import { Vector3 } from "three";
import { cubeSurfaces, projectViewCube } from "./viewCubeGeometry.js";

function basis(direction) {
  const eye = new Vector3(...direction).normalize();
  const upHint = new Vector3(0, 1, 0);
  if (Math.abs(eye.y) === 1) upHint.set(0, 0, -Math.sign(eye.y));
  const right = new Vector3().crossVectors(upHint, eye).normalize();
  const up = new Vector3().crossVectors(eye, right);
  return { eye: eye.toArray(), right: right.toArray(), up: up.toArray() };
}

test("the six faces, twelve edges and eight corners meet without gaps", () => {
  assert.deepEqual([1,2,3].map(n => cubeSurfaces.filter(s => s.labels.length === n).length), [6,12,8]);
  const uses = new Map();
  for (const face of cubeSurfaces) {
    face.vertices.forEach((point, i) => {
      const next = face.vertices[(i + 1) % face.vertices.length];
      const edge = [point.join(","), next.join(",")].sort().join("/");
      uses.set(edge, (uses.get(edge) || 0) + 1);
    });
  }
  assert.equal(uses.size, 48);
  assert.ok([...uses.values()].every(count => count === 2));
});

test("every snap points to exactly one visible matching cube region", () => {
  for (const surface of cubeSurfaces) {
    const projected = projectViewCube(basis(surface.direction));
    assert.deepEqual(projected.filter(s => s.aligned).map(s => s.key), [surface.key]);
    const opposite = surface.direction.map(v => -v).join(",");
    assert.ok(!projected.some(s => s.key === opposite));
    assert.ok(projected.every(s => !s.points.includes("NaN")));
    for (const label of projected.filter(s => s.textTransform)) {
      const [a,b,c,d] = label.textTransform.slice(7, -1).split(" ").map(Number);
      assert.ok(a*d - b*c > 0 && d >= -1e-6, "visible labels are upright and never mirrored");
    }
  }
});

test("outside is negative Z and stays readable looking straight at the frame", () => {
  const projected = projectViewCube(basis([0,0,-1]));
  const face = projected.find(surface => surface.aligned);
  assert.deepEqual(face.labels, ["outside"]);
  assert.equal(face.textTransform, "matrix(1 0 0 1 64 58)");
  assert.equal(projected.filter(surface => surface.labels.length === 2).length, 4);
  assert.equal(projected.filter(surface => surface.labels.length === 3).length, 4);
});
