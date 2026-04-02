//! Parse a DXF file containing a profile cross-section.
//!
//! Reads DXF text format directly — extracts LINE, LWPOLYLINE, and POLYLINE
//! entities to build a cross-section contour.

use std::io::{BufRead, BufReader};

/// Result of parsing a DXF profile cross-section.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedProfile {
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Sponning {
    pub width: f64,
    pub depth: f64,
    pub position: String,
}

/// Parse a DXF file and extract a profile cross-section.
pub fn parse_dxf_profile(filepath: &str) -> Result<ImportedProfile, String> {
    let file = std::fs::File::open(filepath)
        .map_err(|e| format!("Kan DXF bestand niet openen: {}", e))?;
    let reader = BufReader::new(file);

    let points = extract_points(reader)?;

    if points.is_empty() {
        return Err("Geen geometrie gevonden in DXF bestand".into());
    }

    // Build convex hull contour
    let contour = convex_hull(&points);

    // Bounding box
    let min_x = contour.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
    let max_x = contour.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
    let min_y = contour.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let max_y = contour.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
    let width = round1(max_x - min_x);
    let depth = round1(max_y - min_y);

    // Normalize to origin
    let cross_section: Vec<[f64; 2]> = contour
        .iter()
        .map(|p| [round2(p.0 - min_x), round2(p.1 - min_y)])
        .collect();

    // Detect sponning
    let sponning = detect_sponning(&contour, width, depth);

    // Estimates
    let sightline = round1(width * 0.8);
    let glazing_rebate = round1(width * 0.36);

    // Name from filepath
    let path = std::path::Path::new(filepath);
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Profiel")
        .to_string();
    let id = format!("imported-{}", name.to_lowercase().replace(' ', "-"));

    Ok(ImportedProfile {
        id,
        name,
        material: "unknown".into(),
        material_subtype: None,
        width,
        depth,
        sightline,
        glazing_rebate,
        cross_section,
        uf_value: 2.0,
        applicable_as: vec!["frame".into(), "sash".into(), "divider".into()],
        sponning,
    })
}

// ── DXF text parser ────────────────────────────────────────────

fn extract_points(reader: BufReader<std::fs::File>) -> Result<Vec<(f64, f64)>, String> {
    let mut points = Vec::new();
    let lines: Vec<String> = reader
        .lines()
        .map(|l| l.unwrap_or_default().trim().to_string())
        .collect();

    let mut i = 0;
    while i < lines.len() {
        // Look for entity type markers (group code 0)
        if lines[i] == "0" && i + 1 < lines.len() {
            let entity_type = &lines[i + 1];
            match entity_type.as_str() {
                "LINE" => {
                    // Extract start (10/20) and end (11/21) points
                    let (start, end, next) = parse_line_entity(&lines, i + 2);
                    if let Some(s) = start {
                        points.push(s);
                    }
                    if let Some(e) = end {
                        points.push(e);
                    }
                    i = next;
                    continue;
                }
                "LWPOLYLINE" => {
                    let (pts, next) = parse_lwpolyline_entity(&lines, i + 2);
                    points.extend(pts);
                    i = next;
                    continue;
                }
                "POLYLINE" => {
                    // POLYLINE entities have VERTEX sub-entities
                    i += 2;
                    continue;
                }
                "VERTEX" => {
                    let (pt, next) = parse_vertex_entity(&lines, i + 2);
                    if let Some(p) = pt {
                        points.push(p);
                    }
                    i = next;
                    continue;
                }
                _ => {}
            }
        }
        i += 1;
    }

    Ok(points)
}

fn parse_line_entity(lines: &[String], start: usize) -> (Option<(f64, f64)>, Option<(f64, f64)>, usize) {
    let mut x1 = None;
    let mut y1 = None;
    let mut x2 = None;
    let mut y2 = None;
    let mut i = start;

    while i + 1 < lines.len() {
        if lines[i] == "0" {
            break; // Next entity
        }
        match lines[i].as_str() {
            "10" => x1 = lines[i + 1].parse().ok(),
            "20" => y1 = lines[i + 1].parse().ok(),
            "11" => x2 = lines[i + 1].parse().ok(),
            "21" => y2 = lines[i + 1].parse().ok(),
            _ => {}
        }
        i += 2;
    }

    let p1 = match (x1, y1) {
        (Some(x), Some(y)) => Some((x, y)),
        _ => None,
    };
    let p2 = match (x2, y2) {
        (Some(x), Some(y)) => Some((x, y)),
        _ => None,
    };
    (p1, p2, i)
}

fn parse_lwpolyline_entity(lines: &[String], start: usize) -> (Vec<(f64, f64)>, usize) {
    let mut points = Vec::new();
    let mut current_x: Option<f64> = None;
    let mut i = start;

    while i + 1 < lines.len() {
        if lines[i] == "0" {
            break;
        }
        match lines[i].as_str() {
            "10" => {
                // Flush previous point if we have a new X
                if let (Some(x), Some(y)) = (current_x, None::<f64>) {
                    // Wait for Y
                }
                current_x = lines[i + 1].parse().ok();
            }
            "20" => {
                if let (Some(x), Some(y)) = (current_x, lines[i + 1].parse::<f64>().ok()) {
                    points.push((x, y));
                    current_x = None;
                }
            }
            _ => {}
        }
        i += 2;
    }
    (points, i)
}

fn parse_vertex_entity(lines: &[String], start: usize) -> (Option<(f64, f64)>, usize) {
    let mut x = None;
    let mut y = None;
    let mut i = start;

    while i + 1 < lines.len() {
        if lines[i] == "0" {
            break;
        }
        match lines[i].as_str() {
            "10" => x = lines[i + 1].parse().ok(),
            "20" => y = lines[i + 1].parse().ok(),
            _ => {}
        }
        i += 2;
    }

    let pt = match (x, y) {
        (Some(x), Some(y)) => Some((x, y)),
        _ => None,
    };
    (pt, i)
}

// ── Convex hull (Graham scan) ──────────────────────────────────

fn convex_hull(points: &[(f64, f64)]) -> Vec<(f64, f64)> {
    if points.len() < 3 {
        return points.to_vec();
    }

    // Deduplicate
    let mut pts: Vec<(f64, f64)> = Vec::new();
    for p in points {
        let dup = pts.iter().any(|q| (q.0 - p.0).abs() < 1e-9 && (q.1 - p.1).abs() < 1e-9);
        if !dup {
            pts.push(*p);
        }
    }

    if pts.len() < 3 {
        return pts;
    }

    // Find lowest-leftmost point
    let start_idx = pts
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| a.1.partial_cmp(&b.1).unwrap().then(a.0.partial_cmp(&b.0).unwrap()))
        .unwrap()
        .0;
    let start = pts.remove(start_idx);

    // Sort by polar angle
    pts.sort_by(|a, b| {
        let angle_a = (a.1 - start.1).atan2(a.0 - start.0);
        let angle_b = (b.1 - start.1).atan2(b.0 - start.0);
        angle_a.partial_cmp(&angle_b).unwrap()
    });

    let mut hull = vec![start];
    for p in pts {
        while hull.len() > 1 && cross(hull[hull.len() - 2], hull[hull.len() - 1], p) <= 0.0 {
            hull.pop();
        }
        hull.push(p);
    }

    hull
}

fn cross(o: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0)
}

// ── Sponning detection ─────────────────────────────────────────

fn detect_sponning(contour: &[(f64, f64)], width: f64, depth: f64) -> Option<Sponning> {
    if contour.len() < 6 {
        return None;
    }

    let n = contour.len();
    for i in 0..n {
        let p0 = contour[i];
        let p1 = contour[(i + 1) % n];
        let p2 = contour[(i + 2) % n];

        let dx1 = (p1.0 - p0.0).abs();
        let dy1 = (p1.1 - p0.1).abs();
        let dx2 = (p2.0 - p1.0).abs();
        let dy2 = (p2.1 - p1.1).abs();

        // Horizontal then vertical step
        if dx1 > 2.0 && dy1 < 1.0 && dx2 < 1.0 && dy2 > 2.0 {
            let sp_w = round1(dx1);
            let sp_d = round1(dy2);
            if (5.0..=30.0).contains(&sp_w) && (5.0..=40.0).contains(&sp_d) {
                let mid_x = (p0.0 + p1.0) / 2.0;
                let position = if mid_x > width / 2.0 { "buiten" } else { "binnen" };
                return Some(Sponning {
                    width: sp_w,
                    depth: sp_d,
                    position: position.into(),
                });
            }
        }

        // Vertical then horizontal step
        if dy1 > 2.0 && dx1 < 1.0 && dy2 < 1.0 && dx2 > 2.0 {
            let sp_w = round1(dx2);
            let sp_d = round1(dy1);
            if (5.0..=30.0).contains(&sp_w) && (5.0..=40.0).contains(&sp_d) {
                let mid_y = (p0.1 + p1.1) / 2.0;
                let position = if mid_y > depth / 2.0 { "buiten" } else { "binnen" };
                return Some(Sponning {
                    width: sp_w,
                    depth: sp_d,
                    position: position.into(),
                });
            }
        }
    }

    None
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}
