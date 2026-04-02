//! Parse profile catalogs from JSON, CSV, or Excel files.

use std::collections::HashMap;
use std::io::{BufRead, BufReader};

use crate::import::dxf_profile::Sponning;

/// Result of parsing a catalog profile.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogProfile {
    pub id: String,
    pub name: String,
    pub material: String,
    pub material_subtype: Option<String>,
    pub width: f64,
    pub depth: f64,
    pub sightline: f64,
    pub glazing_rebate: f64,
    pub cross_section: Vec<[f64; 2]>,
    pub uf_value: f64,
    pub applicable_as: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sponning: Option<Sponning>,
}

// ── Supplier column mappings ───────────────────────────────────

struct ColumnMapping {
    name: &'static str,
    width: &'static str,
    depth: &'static str,
    sightline: &'static str,
    glazing_rebate: &'static str,
    uf_value: &'static str,
    material: &'static str,
    sponning_width: &'static str,
    sponning_depth: &'static str,
}

const GENERIC: ColumnMapping = ColumnMapping {
    name: "name", width: "width", depth: "depth",
    sightline: "sightline", glazing_rebate: "glazingRebate",
    uf_value: "ufValue", material: "material",
    sponning_width: "sponningWidth", sponning_depth: "sponningDepth",
};

const REYNAERS: ColumnMapping = ColumnMapping {
    name: "Description", width: "Width (mm)", depth: "Depth (mm)",
    sightline: "Sightline (mm)", glazing_rebate: "Glazing Rebate (mm)",
    uf_value: "Uf (W/m²K)", material: "Material",
    sponning_width: "Rebate Width (mm)", sponning_depth: "Rebate Depth (mm)",
};

const SCHUCO: ColumnMapping = ColumnMapping {
    name: "Bezeichnung", width: "Ansichtsbreite", depth: "Bautiefe",
    sightline: "Ansichtsbreite Fl.", glazing_rebate: "Falzmaß",
    uf_value: "Uf", material: "Werkstoff",
    sponning_width: "Falzbreite", sponning_depth: "Falztiefe",
};

const GEALAN: ColumnMapping = ColumnMapping {
    name: "Profilname", width: "Breite", depth: "Tiefe",
    sightline: "Ansichtsbreite", glazing_rebate: "Glasfalz",
    uf_value: "Uf-Wert", material: "Material",
    sponning_width: "Falzbreite", sponning_depth: "Falztiefe",
};

fn get_mapping(supplier: &str) -> &'static ColumnMapping {
    match supplier {
        "reynaers" => &REYNAERS,
        "schuco" => &SCHUCO,
        "gealan" => &GEALAN,
        _ => &GENERIC,
    }
}

fn detect_supplier(headers: &[String]) -> &'static str {
    let lower: Vec<String> = headers.iter().map(|h| h.to_lowercase()).collect();
    if lower.iter().any(|h| h.contains("bezeichnung")) {
        "schuco"
    } else if lower.iter().any(|h| h.contains("profilname")) {
        "gealan"
    } else if lower.iter().any(|h| h.contains("description")) {
        "reynaers"
    } else {
        "generic"
    }
}

/// Parse a catalog file (JSON, CSV, or XLSX).
pub fn parse_catalog(filepath: &str, supplier: Option<&str>) -> Result<Vec<CatalogProfile>, String> {
    let ext = std::path::Path::new(filepath)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "json" => parse_json_catalog(filepath, supplier),
        "csv" => parse_csv_catalog(filepath, supplier),
        #[cfg(feature = "import")]
        "xlsx" | "xls" => parse_excel_catalog(filepath, supplier),
        #[cfg(not(feature = "import"))]
        "xlsx" | "xls" => Err("Excel import niet beschikbaar (calamine feature uitgeschakeld)".into()),
        other => Err(format!(
            "Onbekend bestandsformaat: .{}. Ondersteund: .json, .xlsx, .xls, .csv",
            other
        )),
    }
}

// ── JSON ───────────────────────────────────────────────────────

fn parse_json_catalog(filepath: &str, supplier: Option<&str>) -> Result<Vec<CatalogProfile>, String> {
    let content = std::fs::read_to_string(filepath)
        .map_err(|e| format!("Kan JSON niet lezen: {}", e))?;
    let data: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("JSON parse fout: {}", e))?;

    let profiles_arr = match &data {
        serde_json::Value::Array(arr) => arr.clone(),
        serde_json::Value::Object(obj) => {
            if let Some(arr) = obj.get("profiles").or_else(|| obj.get("items")) {
                arr.as_array().cloned().unwrap_or_else(|| vec![data.clone()])
            } else {
                vec![data.clone()]
            }
        }
        _ => return Err("Onverwacht JSON formaat".into()),
    };

    let mapping = get_mapping(supplier.unwrap_or("generic"));
    let mut result = Vec::new();
    for (i, item) in profiles_arr.iter().enumerate() {
        if let serde_json::Value::Object(obj) = item {
            let row: HashMap<String, String> = obj
                .iter()
                .map(|(k, v)| {
                    let val = match v {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Number(n) => n.to_string(),
                        other => other.to_string(),
                    };
                    (k.clone(), val)
                })
                .collect();
            match map_profile(&row, mapping, i) {
                Ok(p) => result.push(p),
                Err(_) => continue,
            }
        }
    }
    Ok(result)
}

// ── CSV ────────────────────────────────────────────────────────

fn parse_csv_catalog(filepath: &str, supplier: Option<&str>) -> Result<Vec<CatalogProfile>, String> {
    let content = std::fs::read_to_string(filepath)
        .map_err(|e| format!("Kan CSV niet lezen: {}", e))?;

    // Try semicolon first, then comma
    let (headers, rows) = parse_csv_content(&content, ';')
        .or_else(|_| parse_csv_content(&content, ','))?;

    if rows.is_empty() {
        return Err("CSV bestand is leeg".into());
    }

    let supplier = supplier.unwrap_or_else(|| detect_supplier(&headers));
    let mapping = get_mapping(supplier);

    let mut result = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        let row_map: HashMap<String, String> = headers
            .iter()
            .zip(row.iter())
            .map(|(h, v)| (h.clone(), v.clone()))
            .collect();
        match map_profile(&row_map, mapping, i) {
            Ok(p) => result.push(p),
            Err(_) => continue,
        }
    }
    Ok(result)
}

fn parse_csv_content(content: &str, delimiter: char) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    // Strip BOM
    let content = content.strip_prefix('\u{feff}').unwrap_or(content);
    let mut lines = content.lines();

    let header_line = lines.next().ok_or("Leeg bestand")?;
    let headers: Vec<String> = header_line
        .split(delimiter)
        .map(|s| s.trim().trim_matches('"').to_string())
        .collect();

    if headers.len() <= 1 {
        return Err("Slechts 1 kolom gevonden".into());
    }

    let rows: Vec<Vec<String>> = lines
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            l.split(delimiter)
                .map(|s| s.trim().trim_matches('"').to_string())
                .collect()
        })
        .collect();

    Ok((headers, rows))
}

// ── Excel ──────────────────────────────────────────────────────

#[cfg(feature = "import")]
fn parse_excel_catalog(filepath: &str, supplier: Option<&str>) -> Result<Vec<CatalogProfile>, String> {
    use calamine::{Reader, open_workbook_auto, Data};

    let mut workbook = open_workbook_auto(filepath)
        .map_err(|e| format!("Kan Excel bestand niet openen: {}", e))?;

    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or("Geen werkbladen gevonden")?;

    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| format!("Kan werkblad niet lezen: {}", e))?;

    let mut rows_iter = range.rows();
    let header_row = rows_iter.next().ok_or("Excel bestand bevat geen data")?;

    let headers: Vec<String> = header_row
        .iter()
        .map(|c| match c {
            Data::String(s) => s.trim().to_string(),
            other => format!("{}", other),
        })
        .collect();

    let supplier = supplier.unwrap_or_else(|| detect_supplier(&headers));
    let mapping = get_mapping(supplier);

    let mut result = Vec::new();
    for (i, row) in rows_iter.enumerate() {
        let row_map: HashMap<String, String> = headers
            .iter()
            .zip(row.iter())
            .map(|(h, c)| {
                let val = match c {
                    Data::String(s) => s.clone(),
                    Data::Float(f) => format!("{}", f),
                    Data::Int(n) => format!("{}", n),
                    Data::Bool(b) => format!("{}", b),
                    _ => String::new(),
                };
                (h.clone(), val)
            })
            .collect();
        match map_profile(&row_map, mapping, i) {
            Ok(p) => result.push(p),
            Err(_) => continue,
        }
    }

    Ok(result)
}

// ── Profile mapping ────────────────────────────────────────────

fn get_val(row: &HashMap<String, String>, key: &str) -> Option<String> {
    // Direct match
    if let Some(v) = row.get(key) {
        if !v.is_empty() {
            return Some(v.clone());
        }
    }
    // Case-insensitive match
    let lower = key.to_lowercase();
    for (k, v) in row {
        if k.to_lowercase() == lower && !v.is_empty() {
            return Some(v.clone());
        }
    }
    None
}

fn get_float(row: &HashMap<String, String>, key: &str, default: f64) -> f64 {
    get_val(row, key)
        .and_then(|v| v.replace(',', ".").parse::<f64>().ok())
        .unwrap_or(default)
}

fn map_profile(
    row: &HashMap<String, String>,
    mapping: &ColumnMapping,
    index: usize,
) -> Result<CatalogProfile, String> {
    let name = get_val(row, mapping.name)
        .unwrap_or_else(|| format!("Profiel {}", index + 1));
    let width = get_float(row, mapping.width, 0.0);
    let depth = get_float(row, mapping.depth, 0.0);

    if width <= 0.0 || depth <= 0.0 {
        return Err(format!("Ongeldige afmetingen: {}x{}", width, depth));
    }

    let id = format!(
        "imported-{}-{}",
        name.to_lowercase()
            .replace(' ', "-")
            .replace('/', "-")
            .chars()
            .take(40)
            .collect::<String>(),
        index
    );

    let sightline = get_float(row, mapping.sightline, (width * 0.8 * 10.0).round() / 10.0);
    let glazing_rebate = get_float(row, mapping.glazing_rebate, (width * 0.36 * 10.0).round() / 10.0);
    let uf_value = get_float(row, mapping.uf_value, 2.0);
    let material = get_val(row, mapping.material)
        .unwrap_or_else(|| "unknown".into())
        .to_lowercase();

    let sp_width = get_float(row, mapping.sponning_width, 0.0);
    let sp_depth = get_float(row, mapping.sponning_depth, 0.0);
    let sponning = if sp_width > 0.0 && sp_depth > 0.0 {
        Some(Sponning {
            width: sp_width,
            depth: sp_depth,
            position: "buiten".into(),
        })
    } else {
        None
    };

    Ok(CatalogProfile {
        id,
        name,
        material,
        material_subtype: None,
        width,
        depth,
        sightline,
        glazing_rebate,
        cross_section: vec![],
        uf_value,
        applicable_as: vec!["frame".into(), "sash".into(), "divider".into()],
        sponning,
    })
}
