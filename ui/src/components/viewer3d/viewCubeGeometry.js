// The model uses X to the right, Y up and negative Z towards the outside.
// Each bevel is an actual pickable surface. A corner belongs to one
// isometric direction even when several adjacent faces are visible.
const BEVEL = 0.69;
const SCALE = 38;
const DOT_EPSILON = 1e-6;
const dot = (a, b) => a.reduce((sum, v, i) => sum + v * b[i], 0);
const cross = (a, b) => [
  a[1] * b[2] - a[2] * b[1],
  a[2] * b[0] - a[0] * b[2],
  a[0] * b[1] - a[1] * b[0],
];

const faces = [
  { direction: [0, 0, -1], label: "outside" },
  { direction: [0, 0, 1], label: "inside" },
  { direction: [-1, 0, 0], label: "left" },
  { direction: [1, 0, 0], label: "right" },
  { direction: [0, 1, 0], label: "top" },
  { direction: [0, -1, 0], label: "bottom" },
];

function cubeSurface(direction) {
  const axes = [0, 1, 2].filter(axis => direction[axis]);
  const base = direction.map(value => value * BEVEL);
  const outerPoint = axis => base.map((value, i) => i === axis ? direction[i] : value);
  let vertices;
  if (axes.length === 1) {
    const axis = axes[0];
    vertices = [[-1,-1], [1,-1], [1,1], [-1,1]].map(([s,t]) => {
      const point = outerPoint(axis);
      point[(axis + 1) % 3] = s * BEVEL;
      point[(axis + 2) % 3] = t * BEVEL;
      return point;
    });
  } else if (axes.length === 2) {
    const freeAxis = direction.indexOf(0);
    vertices = [[axes[0],-1], [axes[1],-1], [axes[1],1], [axes[0],1]].map(([axis,sign]) => {
      const point = outerPoint(axis);
      point[freeAxis] = sign * BEVEL;
      return point;
    });
  } else {
    vertices = axes.map(outerPoint);
  }
  const length = Math.hypot(...direction);
  return {
    key: direction.join(","), direction, vertices,
    normal: direction.map(value => value / length),
    labels: faces.filter(face => dot(face.direction, direction) > 0).map(face => face.label),
  };
}

const directions = faces.map(face => face.direction);
for (const x of [-1, 0, 1]) {
  for (const y of [-1, 0, 1]) {
    for (const z of [-1, 0, 1]) {
      if (Math.abs(x) + Math.abs(y) + Math.abs(z) > 1) directions.push([x,y,z]);
    }
  }
}
export const cubeSurfaces = directions.map(cubeSurface);

export function projectViewCube({right, up, eye}) {
  const project = point => [64 + SCALE * dot(point, right), 58 - SCALE * dot(point, up)];
  return cubeSurfaces.filter(surface => dot(surface.normal, eye) > DOT_EPSILON).map(surface => {
    const facing = dot(surface.normal, eye);
    let textTransform;
    if (surface.labels.length === 1 && facing > 0.28) {
      const normal = surface.normal;
      const readRight = normal[1] ? [1,0,0] : cross([0,1,0], normal);
      const readUp = cross(normal, readRight);
      // A cap is seen from either side of the frame. Keep its lettering
      // readable when orbiting from the inside to the outside.
      const sign = dot(readUp, up) < -DOT_EPSILON ? -1 : 1;
      const [x,y] = project(normal);
      textTransform = `matrix(${sign * dot(readRight,right)} ${-sign * dot(readRight,up)} ${-sign * dot(readUp,right)} ${sign * dot(readUp,up)} ${x} ${y})`;
    }
    return {
      ...surface, textTransform, aligned: facing > 1 - DOT_EPSILON,
      points: surface.vertices.map(point => project(point).join(",")).join(" "),
    };
  });
}
