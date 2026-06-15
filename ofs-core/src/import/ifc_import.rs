use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IfcImportResult {
    pub windows: Vec<ImportedWindow>,
    pub doors: Vec<ImportedDoor>,
    pub openings: Vec<ImportedOpening>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedWindow {
    pub guid: String,
    pub name: String,
    pub width_mm: f64,
    pub height_mm: f64,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedDoor {
    pub guid: String,
    pub name: String,
    pub width_mm: f64,
    pub height_mm: f64,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedOpening {
    pub guid: String,
    pub width_mm: f64,
    pub height_mm: f64,
    pub wall_guid: Option<String>,
}

/// Parse a simplified IFC STEP file to extract windows, doors, and openings
pub fn parse_ifc_file(filepath: &str) -> Result<IfcImportResult, String> {
    let content = std::fs::read_to_string(filepath)
        .map_err(|e| format!("Kan IFC niet lezen: {}", e))?;

    let mut windows = Vec::new();
    let mut doors = Vec::new();
    let mut openings = Vec::new();

    // Simple regex-free STEP parser -- extract entity lines
    for line in content.lines() {
        let line = line.trim();
        if !line.starts_with('#') { continue; }

        if line.contains("IFCWINDOW(") || line.contains("IFCWINDOWSTANDARDCASE(") {
            if let Some(win) = parse_ifc_window(line) {
                windows.push(win);
            }
        } else if line.contains("IFCDOOR(") || line.contains("IFCDOORSTANDARDCASE(") {
            if let Some(door) = parse_ifc_door(line) {
                doors.push(door);
            }
        } else if line.contains("IFCOPENINGELEMENT(") {
            if let Some(opening) = parse_ifc_opening(line) {
                openings.push(opening);
            }
        }
    }

    Ok(IfcImportResult { windows, doors, openings })
}

fn parse_ifc_window(line: &str) -> Option<ImportedWindow> {
    // Extract GUID (first string parameter after entity type)
    let guid = extract_quoted_string(line, 0)?;
    let name = extract_quoted_string(line, 2).unwrap_or_else(|| "Unnamed Window".into());

    // Try to extract dimensions from the entity parameters
    let (w, h) = extract_dimensions(line).unwrap_or((900.0, 1400.0));

    Some(ImportedWindow {
        guid,
        name,
        width_mm: w * 1000.0, // IFC uses meters
        height_mm: h * 1000.0,
        properties: HashMap::new(),
    })
}

fn parse_ifc_door(line: &str) -> Option<ImportedDoor> {
    let guid = extract_quoted_string(line, 0)?;
    let name = extract_quoted_string(line, 2).unwrap_or_else(|| "Unnamed Door".into());
    let (w, h) = extract_dimensions(line).unwrap_or((1000.0, 2400.0));

    Some(ImportedDoor {
        guid,
        name,
        width_mm: w * 1000.0,
        height_mm: h * 1000.0,
        properties: HashMap::new(),
    })
}

fn parse_ifc_opening(line: &str) -> Option<ImportedOpening> {
    let guid = extract_quoted_string(line, 0)?;
    Some(ImportedOpening {
        guid,
        width_mm: 0.0,
        height_mm: 0.0,
        wall_guid: None,
    })
}

fn extract_quoted_string(line: &str, index: usize) -> Option<String> {
    let mut count = 0;
    let mut in_quote = false;
    let mut current = String::new();

    for ch in line.chars() {
        if ch == '\'' && !in_quote {
            in_quote = true;
            current.clear();
        } else if ch == '\'' && in_quote {
            if count == index {
                return Some(current.clone());
            }
            count += 1;
            in_quote = false;
        } else if in_quote {
            current.push(ch);
        }
    }
    None
}

fn extract_dimensions(line: &str) -> Option<(f64, f64)> {
    // Look for two consecutive float numbers near the end of the entity
    let parts: Vec<&str> = line.split(',').collect();
    let mut floats = Vec::new();
    for part in parts.iter().rev().take(5) {
        let trimmed = part.trim().trim_end_matches(')').trim_end_matches(';');
        if let Ok(f) = trimmed.parse::<f64>() {
            if f > 0.0 && f < 100.0 { // reasonable dimensions in meters
                floats.push(f);
            }
        }
    }
    if floats.len() >= 2 {
        // Reverse scan: floats[0] is the LAST attribute (OverallWidth),
        // floats[1] the one before it (OverallHeight)
        Some((floats[0], floats[1])) // (width, height)
    } else {
        None
    }
}
