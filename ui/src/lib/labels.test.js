import { test } from "node:test";
import assert from "node:assert/strict";
import nl from "../locales/nl.json" with { type: "json" };
import de from "../locales/de.json" with { type: "json" };
import en from "../locales/en.json" with { type: "json" };
import { cutMemberLabel } from "./labels.js";

test("kortlijst benoemt vleugelprofielen naar materiaal en bewaart de boogzijde", () => {
  const positions = ["sash_top", "sash_bottom", "sash_left", "sash_right"];
  for (const material of ["Kunststof", "Aluminium"]) {
    const keys = positions.map(memberType => cutMemberLabel(key => key, { memberType, material }));
    assert.deepEqual(keys.map(key => nl[key]), [
      "Bovenregel vleugel", "Onderregel vleugel", "Vleugelstijl links", "Vleugelstijl rechts",
    ]);
    for (const key of keys) assert.ok(de[key] && en[key]);
    assert.equal(cutMemberLabel(key => nl[key], { memberType: "sash_arc", profileSide: "top", material }), "Vleugelboog boven");
  }
  for (const material of ["Meranti", "Hout-aluminium", undefined]) {
    assert.equal(cutMemberLabel(key => nl[key], { memberType: "sash_top", material }), "Raamhout boven");
  }
});
