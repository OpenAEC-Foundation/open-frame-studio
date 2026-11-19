use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

fn read_json(path: &Path) -> Value {
    println!("cargo:rerun-if-changed={}", path.display());
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn mm(value: &Value) -> Option<f64> {
    value.as_f64().filter(|v| *v > 0.0)
}

fn main() {
    // Compile only assembly dimensions into the core. Projects saved by an
    // older version and presets often refer to a profile by its ID alone.
    let root = Path::new("../profiles");
    let index = read_json(&root.join("index.json"));
    let mut rows = BTreeMap::new();
    let mut stock = BTreeMap::new();
    for category in index["categories"].as_array().unwrap() {
        for file in category["files"].as_array().unwrap() {
            let records = read_json(&root.join(file.as_str().unwrap()));
            for profile in records.as_array().unwrap() {
                let id = profile["id"].as_str().unwrap();
                let material = profile["material"].as_str().unwrap();
                if let Some(depth) = mm(&profile["depth"]) {
                    let shape = if matches!(material, "wood" | "wood_aluminum") {
                        profile["crossSection"].as_array().into_iter().flatten().filter_map(|p|
                            Some([p.get(0)?.as_f64()?,p.get(1)?.as_f64()?])).collect::<Vec<_>>()
                    } else { Vec::new() };
                    stock.insert(id.to_owned(), format!("({id:?}, {material:?}, {depth:?}, &{shape:?})"));
                }
                let rebate = &profile["sponning"];
                let depth = if matches!(material, "wood" | "wood_aluminum") {
                    mm(&rebate["width"]).or_else(|| mm(&profile["glazingRebate"]))
                } else {
                    None
                };
                let overlap = [
                    &rebate["opdekWidth"],
                    &rebate["opdek_width"],
                    &profile["overslagHoogte"],
                    &profile["ventOverlap"],
                ]
                .iter()
                .find_map(|v| mm(v));
                let side = match rebate["position"].as_str().or(rebate["type"].as_str()) {
                    Some("binnensponning" | "binnen") => "Some(GlaslatPosition::Binnen)",
                    Some("buitensponning" | "buiten") => "Some(GlaslatPosition::Buiten)",
                    _ => "None",
                };
                rows.insert(id.to_string(), format!(
                    "({id:?}, {material:?}, ProfileSnapshot {{ sponning_diepte: {depth:?}, \
                    sponning_hoogte: {:?}, glasinval: {:?}, glaslat_breedte: {:?}, glaslat_hoogte: {:?}, \
                    aanzichtbreedte: {:?}, bead_width_base: {:?}, sash_overlap: {overlap:?}, bead_position: {side}, \
                    sash_clearance: {:?}, sash_seat_depth: {:?} }})",
                    mm(&rebate["depth"]), mm(&profile["glasinval"]), mm(&profile["glaslatWidth"]),
                    mm(&profile["glaslatHeight"]), mm(&profile["sightline"]), mm(&profile["glazingBead"]["widthBase"]),
                    mm(&profile["aanslagSponning"]["clearance"]), mm(&profile["aanslagSponning"]["depth"]),
                ));
            }
        }
    }
    let table = rows.into_values().collect::<Vec<_>>().join(",\n");
    let stock_path = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("joinery_stock.rs");
    fs::write(stock_path, format!("&[{}]", stock.into_values().collect::<Vec<_>>().join(",\n"))).unwrap();
    let path = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("profile_snapshots.rs");
    fs::write(path, format!("&[\n{table}\n]")).unwrap();
}
