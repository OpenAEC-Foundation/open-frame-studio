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
    let mut fitting_limits = BTreeMap::new();
    let mut system_ids = BTreeMap::new();
    let mut inward_window_frames = BTreeMap::new();
    let mut stock_articles = BTreeMap::new();
    let mut divider_articles = BTreeMap::new();
    for category in index["categories"].as_array().unwrap() {
        for file in category["files"].as_array().unwrap() {
            let records = read_json(&root.join(file.as_str().unwrap()));
            for profile in records.as_array().unwrap() {
                let id = profile["id"].as_str().unwrap();
                let material = profile["material"].as_str().unwrap();
                if let Some(frame) = profile["inwardWindowFrame"].as_str() {
                    let seals: Vec<String> = profile["closingSeals"].as_array().into_iter().flatten().map(|seal| {
                        let zone = match seal["position"].as_str() {
                            Some("inner") => "Inner",
                            Some("middle") => "Middle",
                            Some("outer") => "Outer",
                            _ => panic!("Unknown closing seal position on {id}"),
                        };
                        let article = seal["articleNumber"].as_str().expect("Missing closing seal article");
                        format!("(SealPosition::{zone}, {article:?})")
                    }).collect();
                    inward_window_frames.insert(id.to_owned(), format!("({id:?}, {frame:?}, &[{}])", seals.join(", ")));
                }
                if matches!(material, "aluminum" | "pvc") {
                    let article = profile["referenceFrameArticle"].as_str().map(|a| ("Frame", a))
                        .or_else(|| profile["inwardWindowFrame"].as_str()
                            .and_then(|_| profile["articleNumber"].as_str()).map(|a| ("Sash", a)))
                        .or_else(|| profile["dividerFrame"].as_str()
                            .and_then(|_| profile["articleNumber"].as_str()).map(|a| ("Divider", a)));
                    if let Some((role, article)) = article {
                        let width = mm(&profile["width"]).expect("Missing article width");
                        let depth = mm(&profile["depth"]).expect("Missing article depth");
                        let manufacturer = profile["manufacturer"].as_str().expect("Missing manufacturer");
                        let label = format!("{manufacturer} {article}");
                        let material = if material == "pvc" { "Pvc" } else { "Aluminum" };
                        if let Some(frame) = profile["dividerFrame"].as_str() {
                            let name = profile["name"].as_str().expect("Missing divider name");
                            divider_articles.insert(id.to_owned(), format!(
                                "DividerArticle {{ id: {id:?}, frame: {frame:?}, material: Material::{material}, name: {name:?}, width: {width:?}, depth: {depth:?} }}"
                            ));
                        }
                        stock_articles.insert(id.to_owned(), format!(
                            "({id:?}, Material::{material}, StockRole::{role}, {width:?}, {depth:?}, {label:?})"
                        ));
                    }
                    let sliding = profile["materialSubtype"] == "sliding";
                    let max_kg = profile["maxWeight"]
                        .as_f64()
                        .filter(|n| n.is_finite() && *n > 0.);
                    if sliding || max_kg.is_some() {
                        fitting_limits
                            .insert(id.to_owned(), format!("({id:?}, {sliding}, {max_kg:?})"));
                    }
                }
                if let Some(depth) = mm(&profile["depth"]) {
                    let shape = if matches!(material, "wood" | "wood_aluminum") {
                        profile["crossSection"]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .filter_map(|p| Some([p.get(0)?.as_f64()?, p.get(1)?.as_f64()?]))
                            .collect::<Vec<_>>()
                    } else {
                        Vec::new()
                    };
                    stock.insert(
                        id.to_owned(),
                        format!("({id:?}, {material:?}, {depth:?}, &{shape:?})"),
                    );
                }
                let rebate = &profile["sponning"];
                let depth = if matches!(material, "wood" | "wood_aluminum") {
                    mm(&rebate["width"]).or_else(|| mm(&profile["glazingRebate"]))
                } else {
                    None
                };
                let explicit_overlap = [
                    &rebate["opdekWidth"],
                    &rebate["opdek_width"],
                    &profile["overslagHoogte"],
                    &profile["ventOverlap"],
                ]
                .iter()
                .find_map(|v| mm(v));
                let frame_face = mm(&profile["sightline"]);
                let inward_article = profile["inwardWindowFrame"].is_string();
                let vent_face = if inward_article { mm(&profile["width"]) } else { mm(&profile["sightlineVent"]) };
                let vent_depth = if inward_article { mm(&profile["depth"]) } else { mm(&profile["ventDepth"]) };
                let combined = mm(&profile["sightlineCombination"]);
                let system = matches!(material, "aluminum" | "pvc");
                let overlap = explicit_overlap.or_else(|| {
                    let value = frame_face? + vent_face? - combined?;
                    (system && value > 0.0).then_some(value)
                });
                let sash_face = match (system, combined, frame_face, explicit_overlap) {
                    (true, Some(c), Some(f), Some(o)) if c - f + o > 0.0 => Some(c - f + o),
                    _ => vent_face,
                };
                let side = match rebate["position"]
                    .as_str()
                    .or(rebate["type"].as_str())
                    .or(rebate["sponningType"].as_str())
                {
                    Some("binnensponning" | "binnen") => "Some(GlaslatPosition::Binnen)",
                    Some("buitensponning" | "buiten") => "Some(GlaslatPosition::Buiten)",
                    _ => "None",
                };
                if system {
                    if let (Some(manufacturer), Some(series)) = (
                        profile["manufacturer"].as_str(),
                        profile["series"].as_str(),
                    ) {
                        let key = format!(
                            "{}::{}",
                            manufacturer.trim().to_lowercase(),
                            series.trim().to_lowercase()
                        );
                        system_ids.insert(id.to_owned(), format!("({id:?}, {key:?})"));
                    }
                }
                rows.insert(id.to_string(), format!(
                    "({id:?}, {material:?}, ProfileSnapshot {{ system_id: None, min_glass_thickness: {:?}, max_glass_thickness: {:?}, \
                    min_glass_thickness_fixed: {:?}, max_glass_thickness_fixed: {:?}, min_glass_thickness_vent: {:?}, max_glass_thickness_vent: {:?}, divider_face_width: {:?}, sponning_diepte: {depth:?}, \
                    sponning_hoogte: {:?}, glasinval: {:?}, glaslat_breedte: {:?}, glaslat_hoogte: {:?}, \
                    aanzichtbreedte: {:?}, bead_width_base: {:?}, sash_overlap: {overlap:?}, bead_position: {side}, \
                    sash_clearance: {:?}, sash_seat_depth: {:?}, sealing_levels: {:?}, meeting_gap: {:?}, sash_face_width: {:?}, sash_build_depth: {:?} }})",
                    mm(&profile["minGlassThickness"]), mm(&profile["maxGlassThickness"]),
                    mm(&profile["minGlassThicknessFixed"]), mm(&profile["maxGlassThicknessFixed"]),
                    mm(&profile["minGlassThicknessVent"]), mm(&profile["maxGlassThicknessVent"]),
                    mm(&profile["sightlineMullion"]).or_else(|| profile["dividerFrame"].as_str().and_then(|_| mm(&profile["width"]))),
                    mm(&rebate["depth"]), mm(&profile["glasinval"]), mm(&profile["glaslatWidth"]),
                    mm(&profile["glaslatHeight"]).or_else(|| mm(&profile["glazingBeadHeight"])).or_else(|| mm(&profile["glazingBead"]["clipHeight"])), mm(&profile["sightline"]), mm(&profile["glazingBead"]["widthBase"]),
                    mm(&profile["aanslagSponning"]["clearance"]).or_else(|| mm(&profile["falzluft"])),
                    mm(&profile["aanslagSponning"]["depth"]),
                    profile["sealingLevels"].as_u64().filter(|n| (1..=3).contains(n)),
                    mm(&profile["stulpNaad"]), sash_face, vent_depth,
                ));
            }
        }
    }
    let table = rows.into_values().collect::<Vec<_>>().join(",\n");
    let fitting_path =
        PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("fitting_limits.rs");
    fs::write(
        fitting_path,
        format!(
            "&[{}]",
            fitting_limits.into_values().collect::<Vec<_>>().join(",\n")
        ),
    )
    .unwrap();
    let systems_path = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("profile_system_ids.rs");
    let articles_path = systems_path.with_file_name("stock_articles.rs");
    fs::write(systems_path.with_file_name("divider_articles.rs"), format!("&[{}]", divider_articles.into_values().collect::<Vec<_>>().join(",\n"))).unwrap();
    fs::write(articles_path, format!("&[{}]", stock_articles.into_values().collect::<Vec<_>>().join(",\n"))).unwrap();
    let inward_path = systems_path.with_file_name("inward_window_frames.rs");
    fs::write(inward_path, format!("&[{}]", inward_window_frames.into_values().collect::<Vec<_>>().join(",\n"))).unwrap();
    fs::write(systems_path, format!("&[{}]", system_ids.into_values().collect::<Vec<_>>().join(",\n"))).unwrap();
    let stock_path = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("joinery_stock.rs");
    fs::write(
        stock_path,
        format!("&[{}]", stock.into_values().collect::<Vec<_>>().join(",\n")),
    )
    .unwrap();
    let path = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("profile_snapshots.rs");
    fs::write(path, format!("&[\n{table}\n]")).unwrap();
}
