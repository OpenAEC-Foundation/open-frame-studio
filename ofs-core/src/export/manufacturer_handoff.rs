//! Vendor-neutral, indicative handoff: IFC parts plus an exact ID crosswalk.
//! A producer must still map profiles, joints and machine operations.

use crate::export::ifc::generate_ifc_parts_project;
use crate::import::ifc_import::parse_ifc_file;
use crate::kozijn::Project;
use crate::production::compute_production_data;
use serde::Serialize;
use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HandoffManifest {
    schema_version: u32,
    status: &'static str,
    project_id: String,
    project_number: String,
    ifc_file: &'static str,
    part_count: usize,
    parts: Vec<HandoffPart>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HandoffPart {
    kozijn_id: String,
    kozijn_mark: String,
    assembly_global_id: String,
    part_global_id: String,
    ifc_class: String,
    part_tag: String,
    role: String,
    list_kind: &'static str,
    list_id: String,
    material: String,
    description: String,
    net_length_mm: Option<f64>,
    gross_length_mm: Option<f64>,
    miter_left_deg: Option<f64>,
    miter_right_deg: Option<f64>,
    width_mm: Option<f64>,
    height_mm: Option<f64>,
    thickness_mm: Option<f64>,
    quantity: u32,
}

/// Write a new child folder inside `parent_dir`; never overwrite an earlier handoff.
/// The output is an exchange proposal, not approved fabrication data.
pub fn generate_manufacturer_handoff(
    project: &Project,
    parent_dir: &Path,
) -> Result<PathBuf, String> {
    if !parent_dir.is_dir() {
        return Err("Kies een bestaande uitvoermap voor de producentenoverdracht".into());
    }
    let nonce = uuid::Uuid::new_v4();
    let final_dir = parent_dir.join(format!("OFS-overdracht-{nonce}"));
    let staging = parent_dir.join(format!(".OFS-overdracht-{nonce}.partial"));
    std::fs::create_dir(&staging).map_err(|e| format!("Kan overdrachtsmap niet maken: {e}"))?;
    let result = write_bundle(project, &staging).and_then(|_| {
        std::fs::rename(&staging, &final_dir)
            .map_err(|e| format!("Kan overdracht niet afronden: {e}"))
    });
    if result.is_err() {
        let _ = std::fs::remove_dir_all(&staging);
    }
    result.map(|_| final_dir)
}

fn write_bundle(project: &Project, dir: &Path) -> Result<(), String> {
    let ifc_path = dir.join("onderdelen.ifc");
    generate_ifc_parts_project(project, ifc_path.to_str().ok_or("Ongeldig IFC-uitvoerpad")?)?;
    // Read back what was actually written, so every crosswalk ID is proven to
    // occur in the IFC, rather than predicted from a duplicate numbering rule.
    let imported = parse_ifc_file(ifc_path.to_str().ok_or("Ongeldig IFC-uitvoerpad")?)?;
    if imported.assemblies.len() != project.kozijnen.len() {
        return Err("IFC-overdracht: aantal samenstellingen komt niet overeen".into());
    }
    let mut rows = Vec::new();
    let mut used_list_ids = HashSet::new();
    for assembly in imported.assemblies {
        let kozijn_id = crate::ifc_guid::ifc_guid_to_uuid(&assembly.guid)
            .ok_or("IFC-overdracht: ongeldig kozijn-GlobalId")?;
        let kozijn = project
            .kozijnen
            .iter()
            .find(|k| k.id == kozijn_id)
            .ok_or("IFC-overdracht: samenstelling ontbreekt in bronproject")?;
        let production = compute_production_data(kozijn);
        for part in assembly.parts {
            let part_kozijn_id = part
                .properties
                .get("OFS_Onderdeel.KozijncombinatieID")
                .ok_or("IFC-overdracht: onderdeel mist kozijn-ID")?;
            if part_kozijn_id != &kozijn.id.to_string() {
                return Err("IFC-overdracht: onderdeel hoort bij ander kozijn".into());
            }
            let role = part
                .properties
                .get("OFS_Onderdeel.Onderdeel")
                .ok_or("IFC-overdracht: onderdeel mist rol")?
                .clone();
            let (
                list_kind,
                list_id,
                material,
                description,
                net,
                gross,
                left,
                right,
                width,
                height,
                thickness,
                quantity,
            ) = match part.entity_type.as_str() {
                "IFCMEMBER" => {
                    let id = part
                        .properties
                        .get("OFS_CalculatieIndicatief.ZaaglijstID")
                        .ok_or("IFC-overdracht: profiel mist zaaglijst-ID")?;
                    let matches: Vec<_> = production
                        .cut_list
                        .iter()
                        .filter(|x| &x.piece_id == id)
                        .collect();
                    if matches.len() != 1 {
                        return Err("IFC-overdracht: zaaglijst-ID niet uniek of afwezig".into());
                    }
                    let x = matches[0];
                    (
                        "zaaglijst",
                        id.clone(),
                        x.material.clone(),
                        x.profile_name.clone(),
                        Some(x.net_length_mm),
                        Some(x.gross_length_mm),
                        Some(x.miter_left_deg),
                        Some(x.miter_right_deg),
                        None,
                        None,
                        None,
                        x.quantity,
                    )
                }
                "IFCPLATE" => {
                    let id = part
                        .properties
                        .get("OFS_CalculatieIndicatief.GlaslijstID")
                        .ok_or("IFC-overdracht: ruit mist glaslijst-ID")?;
                    let matches: Vec<_> = production
                        .glass_list
                        .iter()
                        .filter(|x| &x.piece_id == id)
                        .collect();
                    if matches.len() != 1 {
                        return Err("IFC-overdracht: glaslijst-ID niet uniek of afwezig".into());
                    }
                    let x = matches[0];
                    (
                        "glaslijst",
                        id.clone(),
                        "Glas".into(),
                        x.glass_type.clone(),
                        None,
                        None,
                        None,
                        None,
                        Some(x.width_mm),
                        Some(x.height_mm),
                        Some(x.thickness_mm),
                        x.quantity,
                    )
                }
                _ => return Err("IFC-overdracht: onbekende onderdeelklasse".into()),
            };
            if !used_list_ids.insert((kozijn.id, list_kind, list_id.clone())) {
                return Err(
                    "IFC-overdracht: calculatieregel is aan meerdere onderdelen gekoppeld".into(),
                );
            }
            rows.push(HandoffPart {
                kozijn_id: kozijn.id.to_string(),
                kozijn_mark: kozijn.mark.clone(),
                assembly_global_id: assembly.guid.clone(),
                part_global_id: part.guid,
                ifc_class: part.entity_type,
                part_tag: part.tag,
                role,
                list_kind,
                list_id,
                material,
                description,
                net_length_mm: net,
                gross_length_mm: gross,
                miter_left_deg: left,
                miter_right_deg: right,
                width_mm: width,
                height_mm: height,
                thickness_mm: thickness,
                quantity,
            });
        }
    }
    rows.sort_by(|a, b| (&a.kozijn_id, &a.part_tag).cmp(&(&b.kozijn_id, &b.part_tag)));
    let manifest = HandoffManifest {
        schema_version: 1,
        status: "indicatief; niet voor directe productieaansturing",
        project_id: project.id.to_string(),
        project_number: project.project_info.number.clone(),
        ifc_file: "onderdelen.ifc",
        part_count: rows.len(),
        parts: rows,
    };
    let json = serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("overdracht.json"), json).map_err(|e| e.to_string())?;
    write_csv(&manifest, &dir.join("onderdelen.csv"))?;
    Ok(())
}

fn write_csv(manifest: &HandoffManifest, path: &Path) -> Result<(), String> {
    let mut out = std::io::BufWriter::new(std::fs::File::create(path).map_err(|e| e.to_string())?);
    out.write_all(b"\xEF\xBB\xBF").map_err(|e| e.to_string())?;
    out.write_all(b"KozijnUUID;KozijnMerk;SamenstellingGlobalId;OnderdeelGlobalId;IFCKlasse;OnderdeelMerk;Rol;Lijst;LijstID;Materiaal;Omschrijving;Netto_mm;Bruto_mm;HoekLinks_graad;HoekRechts_graad;Breedte_mm;Hoogte_mm;Dikte_mm;Aantal;Status\r\n")
        .map_err(|e| e.to_string())?;
    for p in &manifest.parts {
        let fields = [
            p.kozijn_id.clone(),
            p.kozijn_mark.clone(),
            p.assembly_global_id.clone(),
            p.part_global_id.clone(),
            p.ifc_class.clone(),
            p.part_tag.clone(),
            p.role.clone(),
            p.list_kind.to_string(),
            p.list_id.clone(),
            p.material.clone(),
            p.description.clone(),
            optional_number(p.net_length_mm),
            optional_number(p.gross_length_mm),
            optional_number(p.miter_left_deg),
            optional_number(p.miter_right_deg),
            optional_number(p.width_mm),
            optional_number(p.height_mm),
            optional_number(p.thickness_mm),
            p.quantity.to_string(),
            manifest.status.to_string(),
        ];
        let line = fields
            .iter()
            .map(|s| csv_field(s))
            .collect::<Vec<_>>()
            .join(";");
        writeln!(out, "{line}\r").map_err(|e| e.to_string())?;
    }
    out.flush().map_err(|e| e.to_string())
}

fn optional_number(value: Option<f64>) -> String {
    value.map(|n| n.to_string()).unwrap_or_default()
}

fn csv_field(value: &str) -> String {
    // CSV may be opened in a spreadsheet. Keep arbitrary project names from
    // becoming formulas while preserving the exact value in the JSON file.
    let safe = if value.trim_start().starts_with(['=', '+', '-', '@']) {
        format!("'{value}")
    } else {
        value.to_string()
    };
    format!("\"{}\"", safe.replace('"', "\"\""))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kozijn::Kozijn;

    #[test]
    fn handoff_matches_written_ifc_part_ids() {
        let mut project = Project::new("Proef", "Q-1");
        let mut kozijn = Kozijn::new("Voorgevel", "K-01", 1200.0, 1500.0);
        kozijn.add_column(500.0);
        project.kozijnen.push(kozijn);
        let parent =
            std::env::temp_dir().join(format!("ofs-handoff-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&parent).unwrap();
        let dir = generate_manufacturer_handoff(&project, &parent).unwrap();
        let manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(dir.join("overdracht.json")).unwrap()).unwrap();
        let parsed = parse_ifc_file(dir.join("onderdelen.ifc").to_str().unwrap()).unwrap();
        assert_eq!(manifest["partCount"], 7);
        assert_eq!(
            manifest["parts"].as_array().unwrap().len(),
            parsed.assemblies[0].parts.len()
        );
        for row in manifest["parts"].as_array().unwrap() {
            assert!(parsed.assemblies[0]
                .parts
                .iter()
                .any(|p| p.guid == row["partGlobalId"].as_str().unwrap()));
            assert!(row["listId"].as_str().unwrap().starts_with("K-01"));
        }
        assert!(std::fs::read(dir.join("onderdelen.csv"))
            .unwrap()
            .starts_with(b"\xEF\xBB\xBF"));
        std::fs::remove_dir_all(&parent).unwrap();
    }

    #[test]
    fn csv_quotes_delimiters_and_blocks_formulas() {
        assert_eq!(csv_field("A;\"B\""), "\"A;\"\"B\"\"\"");
        assert_eq!(csv_field("=1+1"), "\"'=1+1\"");
    }

    #[test]
    fn failed_handoff_does_not_publish_a_bundle() {
        let project = Project::new("Leeg", "Q-0");
        let parent =
            std::env::temp_dir().join(format!("ofs-handoff-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&parent).unwrap();
        assert!(generate_manufacturer_handoff(&project, &parent).is_err());
        assert_eq!(std::fs::read_dir(&parent).unwrap().count(), 0);
        std::fs::remove_dir(&parent).unwrap();
    }
}
