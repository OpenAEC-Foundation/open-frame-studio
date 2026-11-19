// Schematic fitting bodies in mm. The origin and tangent come from the
// production model. Each box is [along, inward, depth, length, width, height],
// with depth measured from the mounting face to the base of the box.
export function fittingDrawing(part, set) {
  const along = part.tangent[1] < -0.01 ? -1 : 1;
  switch (part.kind) {
    case "handle": {
      const foot = [0,0,0,54,22,5];
      switch (set?.handle?.handleType) {
        case "knop": return {face:"inside",boxes:[foot,[0,0,5,17,17,16],[0,0,21,28,28,17,"disc"]]};
        case "inlaat_greep": return {face:"inside",boxes:[[0,0,0,71,23,2],[0,0,2,46,10,1]]};
        case "stangen_greep": return {face:"inside",boxes:[[-48,0,0,12,14,34],[48,0,0,12,14,34],[0,0,34,120,16,16,"barrel"]]};
        case "t_greep": return {face:"inside",boxes:[foot,[0,0,5,14,14,29],[0,0,34,80,14,14,"barrel"]]};
        default: return {face:"inside",bothFaces:set?.handle?.handleType==="kruk_kruk",
          boxes:[foot,[0,0,5,15,15,29],[along*32,0,34,79,14,12]]};
      }
    }
    case "hinge": {
      if (set?.hinges?.hingeType==="verdekt") return {face:"rebate",boxes:[[0,0,0,52,12,5],[-5,1,5,29,8,7]]};
      const knuckles=[[-14,-4,4,26,12,12,"barrel"],[14,-4,4,26,12,12,"barrel"]];
      return {face:"opening",boxes:set?.hinges?.hingeType==="inboor" ? knuckles : [[0,3,0,54,16,4],...knuckles]};
    }
    case "pivot": return { face: "opening", boxes: [[0,0,0,32,12,4],[0,-3,4,15,9,6]] };
    case "closer": return { face: "inside", boxes: [[0,0,0,100,23,22],[42,0,22,60,7,5]] };
    case "vent": return { face: "inside", boxes: [[0,0,0,140,18,7],[-22,0,7,38,4,1],[22,0,7,38,4,1]] };
    // The remaining parts are mounted in the rebate between sash and frame.
    case "lock": return { face: "rebate", boxes: [[0,1,0,48,9,12]] };
    case "locking_point": return { face: "rebate", boxes: [[0,0,0,19,7,3],[0,-2,3,8,10,5]] };
    case "keeper": return { face: "rebate", boxes: [[0,0,0,27,8,3],[-8,1,3,7,6,5],[8,1,3,7,6,5]] };
    case "stay": return { face: "rebate", boxes: [[0,0,0,57,6,4],[-23,2,4,11,10,3]] };
    case "roller": return { face: "rebate", boxes: [[0,1,0,36,12,8],[0,-4,2,18,8,6]] };
    case "flush_bolt": return { face: "rebate", boxes: [[0,0,0,14,30,5],[0,-14,2,8,23,5]] };
    default: return null;
  }
}
