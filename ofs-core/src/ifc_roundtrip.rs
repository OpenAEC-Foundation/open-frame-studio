use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::import::ifc_import::{IfcImportResult, ImportedPart};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IfcDiff {
    pub added: Vec<DiffItem>,
    pub removed: Vec<DiffItem>,
    pub modified: Vec<DiffModification>,
    pub unchanged: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffItem {
    pub guid: String,
    pub name: String,
    pub entity_type: String,
    pub width_mm: Option<f64>,
    pub height_mm: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffModification {
    pub guid: String,
    pub name: String,
    pub changes: Vec<PropertyChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyChange {
    pub property: String,
    pub old_value: String,
    pub new_value: String,
}

struct Snapshot<'a> {
    entity_type: &'static str,
    name: &'a str,
    width_mm: Option<f64>,
    height_mm: Option<f64>,
    properties: &'a std::collections::HashMap<String, String>,
    parts: &'a [ImportedPart],
}

fn snapshots(result: &IfcImportResult) -> BTreeMap<&str, Snapshot<'_>> {
    let mut map = BTreeMap::new();
    for window in &result.windows {
        map.insert(
            window.guid.as_str(),
            Snapshot {
                entity_type: "IfcWindow",
                name: &window.name,
                width_mm: window.width_mm,
                height_mm: window.height_mm,
                properties: &window.properties,
                parts: &[],
            },
        );
    }
    for door in &result.doors {
        map.insert(
            door.guid.as_str(),
            Snapshot {
                entity_type: "IfcDoor",
                name: &door.name,
                width_mm: door.width_mm,
                height_mm: door.height_mm,
                properties: &door.properties,
                parts: &[],
            },
        );
    }
    for assembly in &result.assemblies {
        map.insert(
            assembly.guid.as_str(),
            Snapshot {
                entity_type: "IfcElementAssembly",
                name: &assembly.name,
                width_mm: assembly.width_mm,
                height_mm: assembly.height_mm,
                properties: &assembly.properties,
                parts: &assembly.parts,
            },
        );
    }
    map
}

pub fn compare_ifc_imports(old: &IfcImportResult, new: &IfcImportResult) -> IfcDiff {
    let old_objects = snapshots(old);
    let new_objects = snapshots(new);
    let mut diff = IfcDiff {
        added: Vec::new(),
        removed: Vec::new(),
        modified: Vec::new(),
        unchanged: 0,
    };
    for (guid, item) in &new_objects {
        let Some(previous) = old_objects.get(guid) else {
            diff.added.push(as_diff_item(guid, item));
            continue;
        };
        let mut changes = Vec::new();
        compare_text(
            &mut changes,
            "Entiteitstype",
            previous.entity_type,
            item.entity_type,
        );
        compare_text(&mut changes, "Naam", previous.name, item.name);
        compare_measure(&mut changes, "Breedte", previous.width_mm, item.width_mm);
        compare_measure(&mut changes, "Hoogte", previous.height_mm, item.height_mm);
        compare_properties(&mut changes, "", previous.properties, item.properties);
        compare_parts(&mut changes, previous.parts, item.parts);
        if changes.is_empty() {
            diff.unchanged += 1;
        } else {
            diff.modified.push(DiffModification {
                guid: (*guid).to_string(),
                name: item.name.to_string(),
                changes,
            });
        }
    }
    for (guid, item) in &old_objects {
        if !new_objects.contains_key(guid) {
            diff.removed.push(as_diff_item(guid, item));
        }
    }
    diff
}

fn as_diff_item(guid: &str, item: &Snapshot<'_>) -> DiffItem {
    DiffItem {
        guid: guid.to_string(),
        name: item.name.to_string(),
        entity_type: item.entity_type.to_string(),
        width_mm: item.width_mm,
        height_mm: item.height_mm,
    }
}

fn compare_text(changes: &mut Vec<PropertyChange>, key: &str, old: &str, new: &str) {
    if old != new {
        changes.push(PropertyChange {
            property: key.to_string(),
            old_value: old.to_string(),
            new_value: new.to_string(),
        });
    }
}

fn compare_measure(
    changes: &mut Vec<PropertyChange>,
    key: &str,
    old: Option<f64>,
    new: Option<f64>,
) {
    if old == new || old.zip(new).is_some_and(|(a, b)| (a - b).abs() <= 0.01) {
        return;
    }
    changes.push(PropertyChange {
        property: key.to_string(),
        old_value: old
            .map(|v| format!("{v:.2} mm"))
            .unwrap_or_else(|| "onbekend".into()),
        new_value: new
            .map(|v| format!("{v:.2} mm"))
            .unwrap_or_else(|| "onbekend".into()),
    });
}

fn compare_properties(
    changes: &mut Vec<PropertyChange>,
    prefix: &str,
    old: &std::collections::HashMap<String, String>,
    new: &std::collections::HashMap<String, String>,
) {
    let keys: BTreeSet<&str> = old.keys().chain(new.keys()).map(String::as_str).collect();
    for key in keys {
        let before = old.get(key).map(String::as_str).unwrap_or("(ontbreekt)");
        let after = new.get(key).map(String::as_str).unwrap_or("(ontbreekt)");
        compare_text(changes, &format!("{prefix}{key}"), before, after);
    }
}

fn compare_parts(changes: &mut Vec<PropertyChange>, old: &[ImportedPart], new: &[ImportedPart]) {
    let before: BTreeMap<&str, &ImportedPart> = old.iter().map(|p| (p.guid.as_str(), p)).collect();
    let after: BTreeMap<&str, &ImportedPart> = new.iter().map(|p| (p.guid.as_str(), p)).collect();
    for (guid, part) in &after {
        if let Some(previous) = before.get(guid) {
            let prefix = format!("Onderdeel {}: ", part.tag);
            compare_text(
                changes,
                &format!("{prefix}type"),
                &previous.entity_type,
                &part.entity_type,
            );
            compare_text(
                changes,
                &format!("{prefix}naam"),
                &previous.name,
                &part.name,
            );
            compare_properties(changes, &prefix, &previous.properties, &part.properties);
        } else {
            changes.push(PropertyChange {
                property: "Onderdeel toegevoegd".into(),
                old_value: String::new(),
                new_value: format!("{} ({})", part.tag, guid),
            });
        }
    }
    for (guid, part) in &before {
        if !after.contains_key(guid) {
            changes.push(PropertyChange {
                property: "Onderdeel verwijderd".into(),
                old_value: format!("{} ({})", part.tag, guid),
                new_value: String::new(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::ifc::generate_ifc_parts_project;
    use crate::import::ifc_import::parse_ifc_file;
    use crate::kozijn::{Kozijn, Project};

    #[test]
    fn roundtrip_detects_changed_assembly_and_part_data_by_guid() {
        let mut project = Project::new("Diff", "D-1");
        project
            .kozijnen
            .push(Kozijn::new("Origineel", "K-1", 900.0, 1200.0));
        let old_path = std::env::temp_dir().join(format!("ofs_diff_old_{}.ifc", project.id));
        let new_path = std::env::temp_dir().join(format!("ofs_diff_new_{}.ifc", project.id));
        generate_ifc_parts_project(&project, &old_path.to_string_lossy()).unwrap();
        project.kozijnen[0].name = "Gewijzigd".into();
        project.kozijnen[0].mark = "K-2".into();
        project.kozijnen[0].frame.outer_width = 1000.0;
        project.kozijnen[0].grid.columns[0].size += 100.0;
        generate_ifc_parts_project(&project, &new_path.to_string_lossy()).unwrap();
        let old = parse_ifc_file(&old_path.to_string_lossy()).unwrap();
        let new = parse_ifc_file(&new_path.to_string_lossy()).unwrap();
        let _ = std::fs::remove_file(&old_path);
        let _ = std::fs::remove_file(&new_path);
        let diff = compare_ifc_imports(&old, &new);
        assert!(diff.added.is_empty());
        assert!(diff.removed.is_empty());
        assert_eq!(diff.modified.len(), 1);
        let changes = &diff.modified[0].changes;
        assert!(changes.iter().any(|c| c.property == "Breedte"));
        assert!(changes.iter().any(|c| c.property == "Naam"));
        assert!(changes
            .iter()
            .any(|c| c.property.contains("OFS_Onderdeel.StukID")));
    }
}
