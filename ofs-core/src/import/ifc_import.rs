//! Bounded IFC4 STEP metadata reader for kozijnen and their parts.
//! Geometry remains the responsibility of a full IFC engine.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IfcImportResult {
    pub windows: Vec<ImportedWindow>,
    pub doors: Vec<ImportedDoor>,
    pub openings: Vec<ImportedOpening>,
    #[serde(default)]
    pub assemblies: Vec<ImportedAssembly>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedWindow {
    pub guid: String,
    pub name: String,
    pub width_mm: Option<f64>,
    pub height_mm: Option<f64>,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedDoor {
    pub guid: String,
    pub name: String,
    pub width_mm: Option<f64>,
    pub height_mm: Option<f64>,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedOpening {
    pub guid: String,
    pub width_mm: Option<f64>,
    pub height_mm: Option<f64>,
    pub wall_guid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedAssembly {
    pub guid: String,
    pub name: String,
    pub mark: String,
    pub width_mm: Option<f64>,
    pub height_mm: Option<f64>,
    pub properties: HashMap<String, String>,
    pub parts: Vec<ImportedPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedPart {
    pub guid: String,
    pub entity_type: String,
    pub name: String,
    pub tag: String,
    pub properties: HashMap<String, String>,
}

#[derive(Debug)]
struct StepEntity {
    class: String,
    attrs: Vec<String>,
}

pub fn parse_ifc_file(filepath: &str) -> Result<IfcImportResult, String> {
    let content =
        std::fs::read_to_string(filepath).map_err(|e| format!("Kan IFC niet lezen: {}", e))?;
    parse_ifc_text(&content)
}

/// Unknown dimensions are None, never invented fallback values.
pub fn parse_ifc_text(content: &str) -> Result<IfcImportResult, String> {
    let schema = content
        .split("FILE_SCHEMA")
        .nth(1)
        .and_then(|rest| rest.split(';').next())
        .unwrap_or("");
    if !content.contains("ISO-10303-21;") || !schema.contains("'IFC4'") {
        return Err("IFC-import ondersteunt momenteel alleen IFC4 STEP-bestanden".into());
    }
    let entities = parse_entities(content)?;
    let length_to_mm = length_unit_to_mm(&entities)?;
    let properties = collect_properties(&entities);
    let mut result = IfcImportResult {
        windows: Vec::new(),
        doors: Vec::new(),
        openings: Vec::new(),
        assemblies: Vec::new(),
    };
    let mut ids: Vec<u32> = entities.keys().copied().collect();
    ids.sort_unstable();
    for id in ids {
        let entity = &entities[&id];
        let guid = attr_string(entity, 0).unwrap_or_default();
        if guid.is_empty() {
            continue;
        }
        let name = attr_string(entity, 2).unwrap_or_default();
        let object_props = properties.get(&id).cloned().unwrap_or_default();
        match entity.class.as_str() {
            "IFCWINDOW" | "IFCWINDOWSTANDARDCASE" => result.windows.push(ImportedWindow {
                guid,
                name,
                height_mm: attr_length(entity, 8, length_to_mm),
                width_mm: attr_length(entity, 9, length_to_mm),
                properties: object_props,
            }),
            "IFCDOOR" | "IFCDOORSTANDARDCASE" => result.doors.push(ImportedDoor {
                guid,
                name,
                height_mm: attr_length(entity, 8, length_to_mm),
                width_mm: attr_length(entity, 9, length_to_mm),
                properties: object_props,
            }),
            "IFCOPENINGELEMENT" => result.openings.push(ImportedOpening {
                guid,
                width_mm: None,
                height_mm: None,
                wall_guid: None,
            }),
            "IFCELEMENTASSEMBLY" => result.assemblies.push(ImportedAssembly {
                guid,
                name,
                mark: attr_string(entity, 7).unwrap_or_default(),
                width_mm: object_props
                    .get("OFS_KozijnAfmetingen.BuitenwerksBreedte_mm")
                    .and_then(|v| v.parse().ok()),
                height_mm: object_props
                    .get("OFS_KozijnAfmetingen.BuitenwerksHoogte_mm")
                    .and_then(|v| v.parse().ok()),
                properties: object_props,
                parts: assembly_parts(id, &entities, &properties),
            }),
            _ => {}
        }
    }
    let mut global_ids = std::collections::HashSet::new();
    for guid in result
        .windows
        .iter()
        .map(|e| e.guid.as_str())
        .chain(result.doors.iter().map(|e| e.guid.as_str()))
        .chain(result.assemblies.iter().map(|e| e.guid.as_str()))
        .chain(
            result
                .assemblies
                .iter()
                .flat_map(|e| e.parts.iter().map(|p| p.guid.as_str())),
        )
    {
        if !global_ids.insert(guid) {
            return Err("IFC-import: dubbel GlobalId bij kozijn of onderdeel".into());
        }
    }
    Ok(result)
}

fn parse_entities(content: &str) -> Result<HashMap<u32, StepEntity>, String> {
    let mut entities = HashMap::new();
    let mut record = String::new();
    let mut quoted = false;
    let mut chars = content.chars().peekable();
    while let Some(ch) = chars.next() {
        record.push(ch);
        if ch == '\'' {
            if quoted && chars.peek() == Some(&'\'') {
                record.push(chars.next().unwrap());
            } else {
                quoted = !quoted;
            }
        } else if ch == ';' && !quoted {
            let line = record.trim();
            if line.starts_with('#') {
                let (id_text, definition) = line
                    .split_once('=')
                    .ok_or("IFC-import: ongeldig STEP-record")?;
                let id = id_text[1..]
                    .trim()
                    .parse::<u32>()
                    .map_err(|_| "IFC-import: ongeldig STEP-ID")?;
                let open = definition
                    .find('(')
                    .ok_or("IFC-import: ongeldige entiteit")?;
                let close = definition
                    .rfind(')')
                    .ok_or("IFC-import: ongeldige entiteit")?;
                let class = definition[..open].trim().to_ascii_uppercase();
                let attrs = split_attrs(&definition[open + 1..close]);
                if entities.insert(id, StepEntity { class, attrs }).is_some() {
                    return Err("IFC-import: dubbel STEP-ID".into());
                }
            }
            record.clear();
        }
    }
    if entities.is_empty() {
        return Err("IFC-import: geen STEP-entiteiten gevonden".into());
    }
    Ok(entities)
}

fn split_attrs(input: &str) -> Vec<String> {
    let mut attrs = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    let mut quoted = false;
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\'' {
            current.push(ch);
            if quoted && chars.peek() == Some(&'\'') {
                current.push(chars.next().unwrap());
            } else {
                quoted = !quoted;
            }
            continue;
        }
        if !quoted {
            match ch {
                '(' => depth += 1,
                ')' => depth = depth.saturating_sub(1),
                ',' if depth == 0 => {
                    attrs.push(current.trim().to_string());
                    current.clear();
                    continue;
                }
                _ => {}
            }
        }
        current.push(ch);
    }
    attrs.push(current.trim().to_string());
    attrs
}

fn decode_step_string(value: &str) -> Option<String> {
    let inner = value.strip_prefix('\'')?.strip_suffix('\'')?;
    let mut out = String::new();
    let mut i = 0;
    while i < inner.len() {
        let tail = &inner[i..];
        if tail.starts_with("''") {
            out.push('\'');
            i += 2;
        } else if tail.starts_with("\\\\") {
            out.push('\\');
            i += 2;
        } else if tail.starts_with("\\X2\\") || tail.starts_with("\\X4\\") {
            let wide = tail.starts_with("\\X4\\");
            let encoded = &tail[4..];
            let end = encoded.find("\\X0\\")?;
            let hex = &encoded[..end];
            if wide {
                if hex.len() % 8 != 0 {
                    return None;
                }
                for chunk in hex.as_bytes().chunks(8) {
                    let code = u32::from_str_radix(std::str::from_utf8(chunk).ok()?, 16).ok()?;
                    out.push(char::from_u32(code)?);
                }
            } else {
                if hex.len() % 4 != 0 {
                    return None;
                }
                let mut units = Vec::new();
                for chunk in hex.as_bytes().chunks(4) {
                    units.push(u16::from_str_radix(std::str::from_utf8(chunk).ok()?, 16).ok()?);
                }
                out.push_str(&String::from_utf16(&units).ok()?);
            }
            i += 4 + end + 4;
        } else if tail.starts_with("\\X\\") && tail.len() >= 5 {
            let code = u8::from_str_radix(&tail[3..5], 16).ok()?;
            out.push(code as char);
            i += 5;
        } else {
            let ch = tail.chars().next()?;
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    Some(out)
}

fn attr_string(entity: &StepEntity, index: usize) -> Option<String> {
    decode_step_string(entity.attrs.get(index)?)
}

fn attr_length(entity: &StepEntity, index: usize, to_mm: f64) -> Option<f64> {
    let value = entity.attrs.get(index)?.parse::<f64>().ok()? * to_mm;
    (value.is_finite() && value > 0.0).then_some(value)
}

fn entity_ref(text: &str) -> Option<u32> {
    text.trim().strip_prefix('#')?.parse().ok()
}

fn entity_refs(text: &str) -> Vec<u32> {
    let Some(inner) = text
        .trim()
        .strip_prefix('(')
        .and_then(|v| v.strip_suffix(')'))
    else {
        return Vec::new();
    };
    split_attrs(inner)
        .iter()
        .filter_map(|s| entity_ref(s))
        .collect()
}

fn length_unit_to_mm(entities: &HashMap<u32, StepEntity>) -> Result<f64, String> {
    // Prefer the assignment actually referenced by IfcProject. Files may
    // contain unused units for presentation or other contexts.
    let project_unit_ids: Vec<u32> = entities
        .values()
        .filter(|e| e.class == "IFCPROJECT")
        .filter_map(|project| project.attrs.get(8).and_then(|v| entity_ref(v)))
        .flat_map(|id| {
            entities
                .get(&id)
                .filter(|e| e.class == "IFCUNITASSIGNMENT")
                .and_then(|e| e.attrs.first())
                .map(|v| entity_refs(v))
                .unwrap_or_default()
        })
        .collect();
    let mut found = None;
    for (id, entity) in entities.iter().filter(|(_, e)| e.class == "IFCSIUNIT") {
        if !project_unit_ids.is_empty() && !project_unit_ids.contains(id) {
            continue;
        }
        if entity.attrs.get(1).map(String::as_str) != Some(".LENGTHUNIT.") {
            continue;
        }
        if entity.attrs.get(3).map(String::as_str) != Some(".METRE.") {
            return Err("IFC-import: niet-ondersteunde lengteeenheid".into());
        }
        let factor = match entity.attrs.get(2).map(String::as_str) {
            Some("$") => 1000.0,
            Some(".MILLI.") => 1.0,
            Some(".CENTI.") => 10.0,
            Some(".DECI.") => 100.0,
            _ => return Err("IFC-import: niet-ondersteund SI-voorvoegsel".into()),
        };
        if found.replace(factor).is_some() {
            return Err("IFC-import: meerdere lengteeenheden".into());
        }
    }
    found.ok_or_else(|| "IFC-import: lengteeenheid ontbreekt".into())
}

fn typed_value(value: &str) -> Option<String> {
    let open = value.find('(')?;
    let inner = value[open + 1..].strip_suffix(')')?;
    decode_step_string(inner).or_else(|| Some(inner.to_string()))
}

fn collect_properties(
    entities: &HashMap<u32, StepEntity>,
) -> HashMap<u32, HashMap<String, String>> {
    let mut psets = HashMap::new();
    for (id, set) in entities.iter().filter(|(_, e)| e.class == "IFCPROPERTYSET") {
        let name = attr_string(set, 2).unwrap_or_default();
        let mut values = HashMap::new();
        if let Some(members) = set.attrs.get(4) {
            for prop_id in entity_refs(members) {
                let Some(prop) = entities
                    .get(&prop_id)
                    .filter(|e| e.class == "IFCPROPERTYSINGLEVALUE")
                else {
                    continue;
                };
                if let (Some(key), Some(value)) = (
                    attr_string(prop, 0),
                    prop.attrs.get(2).and_then(|v| typed_value(v)),
                ) {
                    values.insert(format!("{}.{}", name, key), value);
                }
            }
        }
        psets.insert(*id, values);
    }
    let mut objects: HashMap<u32, HashMap<String, String>> = HashMap::new();
    for rel in entities
        .values()
        .filter(|e| e.class == "IFCRELDEFINESBYPROPERTIES")
    {
        let Some(set_id) = rel.attrs.get(5).and_then(|v| entity_ref(v)) else {
            continue;
        };
        let Some(values) = psets.get(&set_id) else {
            continue;
        };
        for object_id in rel.attrs.get(4).map(|v| entity_refs(v)).unwrap_or_default() {
            objects.entry(object_id).or_default().extend(values.clone());
        }
    }
    objects
}

fn assembly_parts(
    assembly_id: u32,
    entities: &HashMap<u32, StepEntity>,
    properties: &HashMap<u32, HashMap<String, String>>,
) -> Vec<ImportedPart> {
    let mut part_ids = Vec::new();
    for rel in entities.values().filter(|e| e.class == "IFCRELAGGREGATES") {
        if rel.attrs.get(4).and_then(|v| entity_ref(v)) == Some(assembly_id) {
            part_ids.extend(rel.attrs.get(5).map(|v| entity_refs(v)).unwrap_or_default());
        }
    }
    part_ids
        .into_iter()
        .filter_map(|id| {
            let entity = entities.get(&id)?;
            if entity.class != "IFCMEMBER" && entity.class != "IFCPLATE" {
                return None;
            }
            Some(ImportedPart {
                guid: attr_string(entity, 0)?,
                entity_type: entity.class.clone(),
                name: attr_string(entity, 2).unwrap_or_default(),
                tag: attr_string(entity, 7).unwrap_or_default(),
                properties: properties.get(&id).cloned().unwrap_or_default(),
            })
        })
        .collect()
}

/// Recover only OFS's own rectangular, fixed-glazing parts export as an
/// editable basic project. Unsupported or missing fields fail closed.
/// Manufacturer profile snapshots, joints and machine details are not in IFC.
pub fn reconstruct_own_parts_project_file(filepath: &str) -> Result<crate::kozijn::Project, String> {
    let content = std::fs::read_to_string(filepath).map_err(|e| format!("Kan IFC niet lezen: {e}"))?;
    reconstruct_own_parts_project(&content)
}

pub fn reconstruct_own_parts_project(content: &str) -> Result<crate::kozijn::Project, String> {
    use crate::kozijn::{Cell, GridDivision, Kozijn, Material, Project, WoodType};
    use crate::profile::ProfileRef;

    let imported = parse_ifc_text(content)?;
    if imported.assemblies.is_empty() || !imported.windows.is_empty() || !imported.doors.is_empty() {
        return Err("IFC-herstel ondersteunt alleen een OFS-onderdelenproject met vaste beglazing".into());
    }
    let entities = parse_entities(content)?;
    let to_mm = length_unit_to_mm(&entities)?;
    let projects: Vec<_> = entities.iter().filter(|(_, e)| e.class == "IFCPROJECT").collect();
    if projects.len() != 1 {
        return Err("IFC-herstel verwacht precies één IfcProject".into());
    }
    let (source_project_id, source_project) = projects[0];
    let name = attr_string(source_project, 2).ok_or("IFC-herstel: projectnaam ontbreekt")?;
    let description = attr_string(source_project, 3).unwrap_or_default();
    let number = description.strip_prefix("Projectnummer: ").unwrap_or("");
    let mut project = Project::new(&name, number);
    if let Some(id) = collect_properties(&entities)
        .get(source_project_id)
        .and_then(|p| p.get("OFS_Project.ProjectUUID"))
    {
        project.id = id.parse().map_err(|_| "IFC-herstel: ongeldig OFS-project-ID")?;
    }
    for assembly in imported.assemblies {
        let p = &assembly.properties;
        let id = crate::ifc_guid::ifc_guid_to_uuid(&assembly.guid)
            .ok_or("IFC-herstel: ongeldige kozijn-GlobalId")?;
        let width = required_positive(p, "OFS_KozijnAfmetingen.BuitenwerksBreedte_mm")?;
        let height = required_positive(p, "OFS_KozijnAfmetingen.BuitenwerksHoogte_mm")?;
        let frame_width = required_positive(p, "OFS_KozijnAfmetingen.KozijnprofielBreedte_mm")?;
        let frame_depth = required_positive(p, "OFS_KozijnAfmetingen.KozijnprofielDiepte_mm")?;
        let cols = required_count(p, "OFS_KozijnAfmetingen.AantalKolommen")?;
        let rows = required_count(p, "OFS_KozijnAfmetingen.AantalRijen")?;
        if cols > 100 || rows > 100 || cols * rows > 1000 {
            return Err("IFC-herstel: vakindeling is te groot".into());
        }
        let mark = required_text(p, "OFS_KozijnAlgemeen.Merkteken")?;
        if mark != assembly.mark {
            return Err("IFC-herstel: kozijnmerk en IFC-tag verschillen".into());
        }
        let mut k = Kozijn::new(&assembly.name, mark, width, height);
        k.id = id;
        k.recovered_from_ifc = true;
        k.frame.frame_width = frame_width;
        k.frame.frame_depth = frame_depth;
        k.frame.material = match required_text(p, "OFS_KozijnMateriaal.Materiaal")? {
            "hout (meranti)" => Material::Wood(WoodType::Meranti),
            "hout (accoya)" => Material::Wood(WoodType::Accoya),
            "hout (vuren)" => Material::Wood(WoodType::Vuren),
            "hout (eiken)" => Material::Wood(WoodType::Eiken),
            "aluminium" => Material::Aluminum,
            "kunststof" => Material::Pvc,
            "hout-aluminium" => Material::WoodAluminum,
            _ => return Err("IFC-herstel: onbekend materiaal".into()),
        };
        k.frame.color_inside = required_text(p, "OFS_KozijnMateriaal.KleurBinnenzijde")?.into();
        k.frame.color_outside = required_text(p, "OFS_KozijnMateriaal.KleurBuitenzijde")?.into();
        let profile_id = required_text(p, "OFS_KozijnAlgemeen.ProfielSysteem")?;
        let profile_name = p.get("OFS_KozijnAlgemeen.ProfielSysteemNaam")
            .map(String::as_str).unwrap_or(profile_id);
        k.frame.profile = ProfileRef { id: profile_id.into(), name: profile_name.into() };
        // The IFC only carries the sill's display name, not a reliable
        // supplier profile ID; leave it unresolved instead of inventing one.
        k.frame.sill_profile = None;
        k.series_id = p.get("Pset_OFS_Kozijn.SeriesId").cloned();
        k.grid.columns = (0..cols).map(|i| {
            Ok(GridDivision {
                size: required_positive(p, &format!("OFS_KozijnAfmetingen.KolomBreedte_{}_mm", i + 1))?,
                divider_profile: (i > 0).then(|| ProfileRef {
                    id: "OFS-ONBEKEND-TUSSENPROFIEL".into(),
                    name: "Tussenprofiel controleren".into(),
                }),
            })
        }).collect::<Result<Vec<_>, String>>()?;
        k.grid.rows = (0..rows).map(|i| {
            Ok(GridDivision {
                size: required_positive(p, &format!("OFS_KozijnAfmetingen.RijHoogte_{}_mm", i + 1))?,
                divider_profile: (i > 0).then(|| ProfileRef {
                    id: "OFS-ONBEKEND-TUSSENPROFIEL".into(),
                    name: "Tussenprofiel controleren".into(),
                }),
            })
        }).collect::<Result<Vec<_>, String>>()?;
        let width_sum: f64 = k.grid.columns.iter().map(|d| d.size).sum::<f64>()
            + (cols - 1) as f64 * frame_width + 2.0 * frame_width;
        let height_sum: f64 = k.grid.rows.iter().map(|d| d.size).sum::<f64>()
            + (rows - 1) as f64 * frame_width + 2.0 * frame_width;
        if (width_sum - width).abs() > 1.0 || (height_sum - height).abs() > 1.0 {
            return Err("IFC-herstel: vakmaten sluiten niet aan op buitenmaat".into());
        }
        k.rebuild_cells();
        for index in 0..cols * rows {
            let n = index + 1;
            if required_text(p, &format!("OFS_KozijnOnderdelen.Cel_{n}_Type"))? != "vast" {
                return Err("IFC-herstel: beweegbaar of niet-beglaasd vak".into());
            }
            let cell: &mut Cell = &mut k.cells[index];
            cell.glazing.glass_type = required_text(p, &format!("OFS_KozijnBeglazing.Cel_{n}_GlasType"))?.into();
            cell.glazing.thickness_mm = required_positive(p, &format!("OFS_KozijnBeglazing.Cel_{n}_GlasDikte_mm"))?;
            cell.glazing.ug_value = required_positive(p, &format!("OFS_KozijnBeglazing.Cel_{n}_UgWaarde"))?;
            cell.glazing.panes.clear();
            cell.glazing.spacer_type = "onbekend".into();
        }
        let source_entity = entities.values().find(|e| {
            e.class == "IFCELEMENTASSEMBLY" && attr_string(e, 0).as_deref() == Some(&assembly.guid)
        }).ok_or("IFC-herstel: samenstelling ontbreekt")?;
        k.placement = own_placement(source_entity, &entities, to_mm)?;
        let mut vertical = vec![None; cols - 1];
        let mut horizontal = vec![None; rows - 1];
        let mut glass = vec![None; cols * rows];
        let mut frame_seen = [false; 4];
        for part in &assembly.parts {
            let part_id = crate::ifc_guid::ifc_guid_to_uuid(&part.guid)
                .ok_or("IFC-herstel: ongeldige onderdeel-GlobalId")?;
            let role = required_text(&part.properties, "OFS_Onderdeel.Onderdeel")?;
            let frame_index = ["Bovenregel", "Onderregel", "Linkerstijl", "Rechterstijl"]
                .iter().position(|name| *name == role);
            if let Some(index) = frame_index {
                if part.entity_type != "IFCMEMBER" || frame_seen[index] || part_id != k.frame_part_id(index) {
                    return Err("IFC-herstel: ongeldig buitenkaderonderdeel".into());
                }
                frame_seen[index] = true;
            } else if let Some(index) = role_index(role, "Tussenstijl ", vertical.len()) {
                if part.entity_type != "IFCMEMBER" || vertical[index].replace(part_id).is_some() {
                    return Err("IFC-herstel: dubbele of ongeldige tussenstijl".into());
                }
            } else if let Some(index) = role_index(role, "Tussenregel ", horizontal.len()) {
                if part.entity_type != "IFCMEMBER" || horizontal[index].replace(part_id).is_some() {
                    return Err("IFC-herstel: dubbele of ongeldige tussenregel".into());
                }
            } else if let Some(index) = role_index(role, "Ruit vak ", glass.len()) {
                if part.entity_type != "IFCPLATE" || glass[index].replace(part_id).is_some() {
                    return Err("IFC-herstel: dubbele of ongeldige ruit".into());
                }
            } else {
                return Err("IFC-herstel: onbekende onderdeelrol".into());
            }
        }
        if frame_seen.iter().any(|seen| !seen) {
            return Err("IFC-herstel: buitenkader is onvolledig".into());
        }
        let vertical = require_all_ids(vertical)?;
        let horizontal = require_all_ids(horizontal)?;
        let glass = require_all_ids(glass)?;
        k.restore_grid_part_ids(vertical, horizontal, glass)?;
        project.kozijnen.push(k);
    }
    Ok(project)
}

fn required_text<'a>(p: &'a HashMap<String, String>, key: &str) -> Result<&'a str, String> {
    p.get(key).map(String::as_str).filter(|v| !v.is_empty())
        .ok_or_else(|| format!("IFC-herstel: eigenschap {key} ontbreekt"))
}

fn required_positive(p: &HashMap<String, String>, key: &str) -> Result<f64, String> {
    let value = required_text(p, key)?.parse::<f64>()
        .map_err(|_| format!("IFC-herstel: ongeldige waarde voor {key}"))?;
    if !value.is_finite() || value <= 0.0 {
        return Err(format!("IFC-herstel: ongeldige waarde voor {key}"));
    }
    Ok(value)
}

fn required_count(p: &HashMap<String, String>, key: &str) -> Result<usize, String> {
    let count = required_text(p, key)?.parse::<usize>()
        .map_err(|_| format!("IFC-herstel: ongeldige waarde voor {key}"))?;
    if count == 0 { return Err(format!("IFC-herstel: ongeldige waarde voor {key}")); }
    Ok(count)
}

fn role_index(role: &str, prefix: &str, max: usize) -> Option<usize> {
    let one_based = role.strip_prefix(prefix)?.parse::<usize>().ok()?;
    (one_based > 0 && one_based <= max).then_some(one_based - 1)
}

fn require_all_ids(values: Vec<Option<uuid::Uuid>>) -> Result<Vec<uuid::Uuid>, String> {
    values.into_iter().map(|v| v.ok_or_else(|| "IFC-herstel: onderdeel ontbreekt".into())).collect()
}

fn own_placement(
    entity: &StepEntity,
    entities: &HashMap<u32, StepEntity>,
    to_mm: f64,
) -> Result<crate::kozijn::Placement, String> {
    let local_id = entity.attrs.get(5).and_then(|v| entity_ref(v))
        .ok_or("IFC-herstel: plaatsing ontbreekt")?;
    let local = entities.get(&local_id).filter(|e| e.class == "IFCLOCALPLACEMENT")
        .ok_or("IFC-herstel: ongeldige plaatsing")?;
    if local.attrs.first().map(String::as_str) != Some("$") {
        return Err("IFC-herstel: geneste plaatsing wordt niet ondersteund".into());
    }
    let axis_id = local.attrs.get(1).and_then(|v| entity_ref(v)).ok_or("IFC-herstel: as ontbreekt")?;
    let axis = entities.get(&axis_id).filter(|e| e.class == "IFCAXIS2PLACEMENT3D")
        .ok_or("IFC-herstel: ongeldige as")?;
    let point_id = axis.attrs.first().and_then(|v| entity_ref(v)).ok_or("IFC-herstel: oorsprong ontbreekt")?;
    let point = entities.get(&point_id).filter(|e| e.class == "IFCCARTESIANPOINT")
        .ok_or("IFC-herstel: ongeldige oorsprong")?;
    let xyz = numeric_tuple(point, 3)?;
    if let Some(up_id) = axis.attrs.get(1).and_then(|v| entity_ref(v)) {
        let up = entities.get(&up_id).filter(|e| e.class == "IFCDIRECTION")
            .ok_or("IFC-herstel: ongeldige hoogte-as")?;
        let v = numeric_tuple(up, 3)?;
        if v[0].abs() > 1e-6 || v[1].abs() > 1e-6 || (v[2] - 1.0).abs() > 1e-6 {
            return Err("IFC-herstel: gekantelde kozijnen worden niet ondersteund".into());
        }
    }
    let orientation = if let Some(dir_id) = axis.attrs.get(2).and_then(|v| entity_ref(v)) {
        let dir = entities.get(&dir_id).filter(|e| e.class == "IFCDIRECTION")
            .ok_or("IFC-herstel: ongeldige richting")?;
        let values = numeric_tuple(dir, 3)?;
        if values[2].abs() > 1e-6 || values[0].hypot(values[1]) < 1e-9 {
            return Err("IFC-herstel: ongeldige breedte-as".into());
        }
        values[1].atan2(values[0]).to_degrees()
    } else { 0.0 };
    let [x, y, z] = [xyz[0] * to_mm, xyz[1] * to_mm, xyz[2] * to_mm];
    if !x.is_finite() || !y.is_finite() || !z.is_finite() {
        return Err("IFC-herstel: ongeldige projectcoördinaten".into());
    }
    Ok(crate::kozijn::Placement {
        wall_id: None, x, y, z, orientation,
    })
}

fn numeric_tuple(entity: &StepEntity, len: usize) -> Result<Vec<f64>, String> {
    let raw = entity.attrs.first().ok_or("IFC-herstel: coördinaten ontbreken")?;
    let values = split_attrs(raw.trim().strip_prefix('(').and_then(|v| v.strip_suffix(')'))
        .ok_or("IFC-herstel: ongeldige coördinaten")?);
    if values.len() != len { return Err("IFC-herstel: onjuist aantal coördinaten".into()); }
    values.iter().map(|v| {
        let n = v.parse::<f64>().map_err(|_| "IFC-herstel: ongeldige coördinaat")?;
        if !n.is_finite() { return Err("IFC-herstel: ongeldige coördinaat".into()); }
        Ok(n)
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::ifc::{generate_ifc_parts_project, generate_project_ifc_with_lod, LodLevel};
    use crate::kozijn::{Kozijn, Project};

    #[test]
    fn restores_own_parts_as_editable_basic_project_with_placement_and_ids() {
        let mut project = Project::new("Proefproject", "QA-1");
        let mut first = Kozijn::new("Voorgevel", "K-01", 1200.0, 1500.0);
        first.add_column(500.0);
        first.placement.x = 1250.0;
        first.placement.y = -300.0;
        first.placement.z = 900.0;
        first.placement.orientation = 30.0;
        first.cells[1].glazing.glass_type = "Triple".into();
        first.cells[1].glazing.thickness_mm = 36.0;
        first.cells[1].glazing.ug_value = 0.7;
        let expected_ids = [first.glass_part_id(0), first.glass_part_id(1)];
        let original_id = first.id;
        project.kozijnen.push(first);
        project.kozijnen.push(Kozijn::new("Achtergevel", "K-02", 900.0, 1200.0));
        let path = std::env::temp_dir().join(format!("ofs_restore_{}.ifc", project.id));
        generate_ifc_parts_project(&project, &path.to_string_lossy()).unwrap();
        let restored = reconstruct_own_parts_project_file(&path.to_string_lossy()).unwrap();
        let replay_path = std::env::temp_dir().join(format!("ofs_restore_replay_{}.ifc", project.id));
        generate_ifc_parts_project(&restored, &replay_path.to_string_lossy()).unwrap();
        let replay = parse_ifc_file(&replay_path.to_string_lossy()).unwrap();
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&replay_path);
        assert_eq!(restored.project_info.name, "Proefproject");
        assert_eq!(restored.id, project.id);
        assert_eq!(restored.project_info.number, "QA-1");
        assert_eq!(restored.kozijnen.len(), 2);
        let k = restored.kozijnen.iter().find(|k| k.id == original_id).unwrap();
        assert!(k.recovered_from_ifc);
        assert_eq!(k.grid.columns.len(), 2);
        assert_eq!(k.cells[1].glazing.glass_type, "Triple");
        assert_eq!(k.cells[1].glazing.thickness_mm, 36.0);
        assert_eq!(k.cells[1].glazing.ug_value, 0.7);
        assert!((k.placement.x - 1250.0).abs() < 0.001);
        assert!((k.placement.y + 300.0).abs() < 0.001);
        assert!((k.placement.z - 900.0).abs() < 0.001);
        assert!((k.placement.orientation - 30.0).abs() < 0.001);
        assert_eq!([k.glass_part_id(0), k.glass_part_id(1)], expected_ids);
        assert_eq!(replay.assemblies.len(), 2);
        assert_eq!(replay.assemblies.iter().map(|a| a.parts.len()).sum::<usize>(), 12);
        assert!(replay.assemblies.iter().flat_map(|a| &a.parts).any(|p| p.guid == crate::ifc_guid::uuid_to_ifc_guid(&expected_ids[1])));
        let reopened: Project = serde_json::from_str(&serde_json::to_string(&restored).unwrap()).unwrap();
        assert_eq!(reopened.kozijnen.iter().find(|k| k.id == original_id).unwrap().glass_part_id(1), expected_ids[1]);
    }

    #[test]
    fn editable_recovery_rejects_ordinary_window_and_missing_ofs_metadata() {
        let mut project = Project::new("Proef", "P-1");
        project.kozijnen.push(Kozijn::new("Raam", "K-1", 900.0, 1200.0));
        let path = std::env::temp_dir().join(format!("ofs_recovery_reject_{}.ifc", project.id));
        generate_project_ifc_with_lod(&project, &path.to_string_lossy(), LodLevel::Lod300).unwrap();
        let window_text = std::fs::read_to_string(&path).unwrap();
        assert!(reconstruct_own_parts_project(&window_text).unwrap_err().contains("onderdelenproject"));
        generate_ifc_parts_project(&project, &path.to_string_lossy()).unwrap();
        let parts_text = std::fs::read_to_string(&path).unwrap().replace("'OFS_KozijnAfmetingen'", "'VerkeerdeEigenschappen'");
        assert!(reconstruct_own_parts_project(&parts_text).unwrap_err().contains("eigenschap"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn imports_own_window_and_assembly_exports_without_losing_properties() {
        let mut project = Project::new("Proefproject", "QA");
        let mut kozijn = Kozijn::new("René 😀", "K-É", 1200.0, 1500.0);
        kozijn.add_column(500.0);
        let expected_guid = crate::ifc_guid::uuid_to_ifc_guid(&kozijn.id);
        project.kozijnen.push(kozijn);
        let window_path =
            std::env::temp_dir().join(format!("ofs_import_window_{}.ifc", project.id));
        let parts_path = std::env::temp_dir().join(format!("ofs_import_parts_{}.ifc", project.id));
        generate_project_ifc_with_lod(&project, &window_path.to_string_lossy(), LodLevel::Lod300)
            .unwrap();
        generate_ifc_parts_project(&project, &parts_path.to_string_lossy()).unwrap();
        let window = parse_ifc_file(&window_path.to_string_lossy()).unwrap();
        let parts = parse_ifc_file(&parts_path.to_string_lossy()).unwrap();
        let _ = std::fs::remove_file(&window_path);
        let _ = std::fs::remove_file(&parts_path);

        assert_eq!(window.windows.len(), 1);
        assert_eq!(window.windows[0].guid, expected_guid);
        assert_eq!(window.windows[0].name, "René 😀");
        assert_eq!(window.windows[0].width_mm, Some(1200.0));
        assert_eq!(window.windows[0].height_mm, Some(1500.0));
        assert_eq!(
            window.windows[0].properties["OFS_KozijnAlgemeen.Merkteken"],
            "K-É"
        );
        assert_eq!(parts.assemblies.len(), 1);
        assert_eq!(parts.assemblies[0].guid, expected_guid);
        assert_eq!(parts.assemblies[0].mark, "K-É");
        assert_eq!(parts.assemblies[0].width_mm, Some(1200.0));
        assert_eq!(parts.assemblies[0].parts.len(), 7);
        assert_eq!(
            parts.assemblies[0]
                .parts
                .iter()
                .filter(|p| p.entity_type == "IFCPLATE")
                .count(),
            2
        );
        assert!(parts.assemblies[0]
            .parts
            .iter()
            .all(|p| p.properties.contains_key("OFS_Onderdeel.Onderdeel")));
        let top = parts.assemblies[0].parts.iter()
            .find(|p| p.name == "Bovenregel").unwrap();
        assert_eq!(top.properties["OFS_CalculatieIndicatief.ZaaglijstID"], "K-É-DB");
        assert!(top.properties.contains_key("OFS_CalculatieIndicatief.NettoLengte_mm"));
        let first_glass = parts.assemblies[0].parts.iter()
            .find(|p| p.name == "Ruit vak 1").unwrap();
        assert_eq!(first_glass.properties["OFS_CalculatieIndicatief.GlaslijstID"], "K-É-V1");
        assert!(first_glass.properties.contains_key("OFS_CalculatieIndicatief.BestelbreedteIndicatief_mm"));
    }

    #[test]
    fn honours_millimetre_units_and_missing_dimensions() {
        let text = "ISO-10303-21;HEADER;FILE_SCHEMA(('IFC4'));ENDSEC;DATA;\
            #1=IFCSIUNIT(*,.LENGTHUNIT.,.MILLI.,.METRE.);\
            #2=IFCWINDOW('0123456789012345678901',$,'K, 01',$,$,$,$,'M',1200.,900.,$,$,$);\
            #3=IFCDOOR('1123456789012345678901',$,'Onbekend',$,$,$,$,'D',$,$,$,$,$);\
            ENDSEC;END-ISO-10303-21;";
        let result = parse_ifc_text(text).unwrap();
        assert_eq!(result.windows[0].name, "K, 01");
        assert_eq!(result.windows[0].width_mm, Some(900.0));
        assert_eq!(result.windows[0].height_mm, Some(1200.0));
        assert_eq!(result.doors[0].width_mm, None);
        assert_eq!(result.doors[0].height_mm, None);
    }

    #[test]
    fn rejects_unknown_units_and_duplicate_step_ids() {
        let header = "ISO-10303-21;HEADER;FILE_SCHEMA(('IFC4'));ENDSEC;DATA;";
        let feet =
            format!("{header}#1=IFCSIUNIT(*,.LENGTHUNIT.,$,.FOOT.);ENDSEC;END-ISO-10303-21;");
        assert!(parse_ifc_text(&feet).unwrap_err().contains("lengteeenheid"));
        let duplicate = format!("{header}#1=IFCSIUNIT(*,.LENGTHUNIT.,$,.METRE.);#1=IFCSIUNIT(*,.LENGTHUNIT.,$,.METRE.);ENDSEC;END-ISO-10303-21;");
        assert!(parse_ifc_text(&duplicate)
            .unwrap_err()
            .contains("dubbel STEP-ID"));
    }

    #[test]
    fn uses_project_length_unit_and_rejects_duplicate_global_ids() {
        let text = "ISO-10303-21;HEADER;FILE_SCHEMA(( 'IFC4' ));ENDSEC;DATA;\
            #1=IFCSIUNIT(*,.LENGTHUNIT.,.MILLI.,.METRE.);\
            #2=IFCSIUNIT(*,.LENGTHUNIT.,$,.METRE.);\
            #3=IFCUNITASSIGNMENT((#1));\
            #4=IFCPROJECT('0123456789012345678901',$,'P',$,$,$,$,$,#3);\
            #5=IFCWINDOW('1123456789012345678901',$,'A',$,$,$,$,'A',1200.,900.,$,$,$);\
            ENDSEC;END-ISO-10303-21;";
        let result = parse_ifc_text(text).unwrap();
        assert_eq!(result.windows[0].width_mm, Some(900.0));
        let duplicate = text.replace("ENDSEC;END-ISO-10303-21;",
            "#6=IFCDOOR('1123456789012345678901',$,'B',$,$,$,$,'B',$,$,$,$,$);ENDSEC;END-ISO-10303-21;");
        assert!(parse_ifc_text(&duplicate)
            .unwrap_err()
            .contains("dubbel GlobalId"));
    }
}
