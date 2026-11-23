//! Excel (.xlsx) export for kozijnstaat and production lists.

use rust_xlsxwriter::{Format, FormatAlign, FormatBorder, Color, Workbook};

use crate::kozijn::{Material, Project, WoodType};
use crate::production::ProductionData;

const DEEP_FORGE: u32 = 0x36363E;
const ALT_BG: u32 = 0xF9FAFB;
const BORDER_COLOR: u32 = 0xE5E7EB;

fn header_format() -> Format {
    Format::new()
        .set_bold()
        .set_font_name("Calibri")
        .set_font_size(9.0)
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(DEEP_FORGE))
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin)
        .set_border_color(Color::RGB(BORDER_COLOR))
}

fn data_format(alt: bool) -> Format {
    let mut f = Format::new()
        .set_font_name("Calibri")
        .set_font_size(9.0)
        .set_border(FormatBorder::Thin)
        .set_border_color(Color::RGB(BORDER_COLOR));
    if alt {
        f = f.set_background_color(Color::RGB(ALT_BG));
    }
    f
}

fn data_format_center(alt: bool) -> Format {
    data_format(alt).set_align(FormatAlign::Center)
}

// ── Kozijnstaat (window schedule) ──────────────────────────────

pub fn generate_kozijnstaat_xlsx(project: &Project, output_path: &str) -> Result<(), String> {
    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();
    ws.set_name("Kozijnstaat").map_err(|e| e.to_string())?;

    let hfmt = header_format();

    let headers = [
        "Merk",
        "Naam",
        "Breedte (mm)",
        "Hoogte (mm)",
        "Materiaal",
        "Kolommen",
        "Rijen",
        "Cellen",
        "Paneel types",
        "Beglazing",
        "Kleur binnen",
        "Kleur buiten",
    ];

    for (col, header) in headers.iter().enumerate() {
        ws.write_string_with_format(0, col as u16, *header, &hfmt)
            .map_err(|e| e.to_string())?;
    }

    for (row_idx, kozijn) in project.kozijnen.iter().enumerate() {
        let row = (row_idx + 1) as u32;
        let alt = row % 2 == 0;
        let dfmt = data_format(alt);
        let dfmt_c = data_format_center(alt);

        let frame = &kozijn.frame;
        let cells = &kozijn.cells;
        let grid = &kozijn.grid;

        // Panel type summary
        let mut type_counts: std::collections::BTreeMap<&str, usize> =
            std::collections::BTreeMap::new();
        for cell in cells {
            let label = cell.panel_type.label_nl();
            *type_counts.entry(label).or_insert(0) += 1;
        }
        let type_summary: Vec<String> = type_counts
            .iter()
            .map(|(k, v)| format!("{}x {}", v, k))
            .collect();
        let type_str = type_summary.join(", ");

        let mat_label = material_label_xlsx(&frame.material);

        let glaz_label = cells
            .first()
            .map(|c| {
                format!(
                    "{} {}mm",
                    c.glazing.glass_type,
                    c.glazing.thickness_mm as i64
                )
            })
            .unwrap_or_else(|| "-".into());

        ws.write_string_with_format(row, 0, &kozijn.mark, &dfmt)
            .map_err(|e| e.to_string())?;
        ws.write_string_with_format(row, 1, &kozijn.name, &dfmt)
            .map_err(|e| e.to_string())?;
        ws.write_number_with_format(row, 2, frame.outer_width as f64, &dfmt_c)
            .map_err(|e| e.to_string())?;
        ws.write_number_with_format(row, 3, frame.outer_height as f64, &dfmt_c)
            .map_err(|e| e.to_string())?;
        ws.write_string_with_format(row, 4, &mat_label, &dfmt)
            .map_err(|e| e.to_string())?;
        ws.write_number_with_format(row, 5, grid.columns.len() as f64, &dfmt_c)
            .map_err(|e| e.to_string())?;
        ws.write_number_with_format(row, 6, grid.rows.len() as f64, &dfmt_c)
            .map_err(|e| e.to_string())?;
        ws.write_number_with_format(row, 7, cells.len() as f64, &dfmt_c)
            .map_err(|e| e.to_string())?;
        ws.write_string_with_format(row, 8, &type_str, &dfmt)
            .map_err(|e| e.to_string())?;
        ws.write_string_with_format(row, 9, &glaz_label, &dfmt)
            .map_err(|e| e.to_string())?;
        ws.write_string_with_format(row, 10, &frame.color_inside, &dfmt)
            .map_err(|e| e.to_string())?;
        ws.write_string_with_format(row, 11, &frame.color_outside, &dfmt)
            .map_err(|e| e.to_string())?;
    }

    // Set column widths
    let widths = [10.0, 15.0, 12.0, 12.0, 15.0, 10.0, 10.0, 10.0, 25.0, 15.0, 12.0, 12.0];
    for (col, w) in widths.iter().enumerate() {
        ws.set_column_width(col as u16, *w)
            .map_err(|e| e.to_string())?;
    }

    wb.save(output_path)
        .map_err(|e| format!("Kan XLSX niet opslaan: {}", e))?;

    Ok(())
}

fn material_label_xlsx(mat: &Material) -> String {
    match mat {
        Material::Wood(wt) => {
            let name = match wt {
                WoodType::Meranti => "meranti",
                WoodType::Accoya => "accoya",
                WoodType::Vuren => "vuren",
                WoodType::Eiken => "eiken",
            };
            format!("wood ({})", name)
        }
        Material::Aluminum => "aluminum".into(),
        Material::Pvc => "pvc".into(),
        Material::WoodAluminum => "wood_aluminum".into(),
    }
}

// ── Production lists XLSX ──────────────────────────────────────

pub fn generate_production_xlsx(
    production_data: &[ProductionData],
    output_path: &str,
) -> Result<(), String> {
    let mut wb = Workbook::new();
    let hfmt = header_format();

    // Kortlijst
    let with_joints = production_data.iter().any(|p| !p.divider_joints.is_empty());
    let arch_kinds: Vec<_> = super::arch_joints::KINDS.into_iter()
        .filter(|&kind| super::arch_joints::present(production_data, kind)).collect();
    {
        let ws = wb.add_worksheet();
        ws.set_name("Kortlijst").map_err(|e| e.to_string())?;
        let with_arcs = production_data.iter().flat_map(|p| &p.cut_list).any(|i| i.has_curved_cut());
        let with_heads = production_data.iter().flat_map(|p| &p.cut_list).any(|i| i.divider_start.is_some());
        let with_system_heads = production_data.iter().flat_map(|p| &p.cut_list).any(|i| i.has_system_heads());
        let with_notches = production_data.iter().flat_map(|p| &p.cut_list).any(|i| !i.masonry_notches.is_empty());
        let mut headers = vec![
            "Kozijn", "Pos.", "Onderdeel", "Profiel", "Materiaal",
            "Netto (mm)", "Bruto (mm)", "Hoek L", "Hoek R", "Aantal",
        ];
        if with_arcs {
            headers[7] = "Beginhoek (90° haaks)";
            headers[8] = "Eindhoek (90° haaks)";
            headers.extend(["Buitenstraal (mm)", "Buitenbooglengte (mm)", "Hartstraal (mm)", "Hartbooglengte (mm)", "Buitenbooghoek (°)"]);
            headers.extend(if with_system_heads {
                ["Kopradius begin in aanzicht (mm)", "Kopradius eind in aanzicht (mm)"]
            } else {
                ["Pasfrezen begin R (mm)", "Pasfrezen eind R (mm)"]
            });
        }
        let head_col = headers.len() as u16;
        if with_heads {
            headers[7] = "Zaag-/koordehoek begin (°)";
            headers[8] = "Zaag-/koordehoek eind (°)";
            headers.extend(["Begin insprong links/boven (mm)", "Begin insprong rechts/onder (mm)",
                "Begin kopprofiel XY (mm vanaf kozijn linksboven)", "Eind insprong links/boven (mm)",
                "Eind insprong rechts/onder (mm)", "Eind kopprofiel XY (mm vanaf kozijn linksboven)"]);
        }
        let notch_col = headers.len() as u16;
        if with_notches { headers.push("Uitsparing (mm vanaf kozijn linksboven)"); }
        let joint_col = headers.len() as u16;
        if with_joints {
            headers.extend(["Pen begin (mm)", "Gatdiepte begin (mm)", "Gatstuk begin",
                "Pen eind (mm)", "Gatdiepte eind (mm)", "Gatstuk eind", "Ontvangt gaten van (aantal x diepte)"]);
        }
        let arch_col = headers.len() as u16;
        for &kind in &arch_kinds { headers.extend(super::arch_joints::cut_headers(kind)); }
        let machining_col = headers.len() as u16;
        if with_system_heads {
            headers.extend(["Fabrikantbewerking begin", "Fabrikantbewerking eind", "PVC zaagmaat GLM excl. zaagsnede (mm)"]);
        }
        write_headers(ws, &headers, &hfmt)?;

        let mut row = 1u32;
        for prod in production_data {
            let joints = prod.joint_index();
            for item in &prod.cut_list {
                let alt = row % 2 == 0;
                let df = data_format(alt);
                let dfc = data_format_center(alt);

                ws.write_string_with_format(row, 0, &prod.kozijn_mark, &df).map_err(|e| e.to_string())?;
                ws.write_string_with_format(row, 1, &item.piece_id, &df).map_err(|e| e.to_string())?;
                ws.write_string_with_format(row, 2, item.member_label_nl(), &df).map_err(|e| e.to_string())?;
                ws.write_string_with_format(row, 3, &item.profile_name, &df).map_err(|e| e.to_string())?;
                ws.write_string_with_format(row, 4, &item.material, &df).map_err(|e| e.to_string())?;
                if with_arcs {
                    let mm = dfc.clone().set_num_format("0.0");
                    let degrees = dfc.clone().set_num_format("0.00\"°\"");
                    ws.write_number_with_format(row, 5, item.net_length_mm, &mm).map_err(|e| e.to_string())?;
                    if let Some(value) = prod.blank_length_mm(item) {
                        ws.write_number_with_format(row, 6, value, &mm).map_err(|e| e.to_string())?;
                    }
                    for (col, value, cope) in [(7, item.miter_left_deg, item.start_cope_radius_mm),
                        (8, item.miter_right_deg, item.end_cope_radius_mm)] {
                        ws.write_number_with_format(row, col, value, &degrees).map_err(|e| e.to_string())?;
                        if let Some(r) = cope {
                            ws.write_number_with_format(row, col + 8, r, &mm).map_err(|e| e.to_string())?;
                        }
                    }
                    let mut lines = 1;
                    for (col, end) in [(head_col, &item.divider_start), (head_col + 3, &item.divider_end)] {
                        if let Some(end) = end {
                            for (i, value) in end.edge_offsets_mm.iter().enumerate() {
                                ws.write_number_with_format(row, col + i as u16, *value, &mm).map_err(|e| e.to_string())?;
                            }
                            let desc = end.segments.iter().map(crate::production::cut_step).collect::<Vec<_>>().join("\n");
                            ws.write_string_with_format(row, col + 2, desc, &df.clone().set_text_wrap()).map_err(|e| e.to_string())?;
                            lines = lines.max(end.segments.len());
                        }
                    }
                    if lines > 1 { ws.set_row_height(row, 16.0 * lines as f64).map_err(|e| e.to_string())?; }
                    if let Some(arc) = &item.arc {
                        for (col, value) in [(10, arc.outer_radius_mm), (11, arc.outer_length_mm),
                            (12, arc.centerline_radius_mm), (13, arc.centerline_length_mm)] {
                            ws.write_number_with_format(row, col, value, &mm).map_err(|e| e.to_string())?;
                        }
                        ws.write_number_with_format(row, 14, arc.sweep_deg, &degrees).map_err(|e| e.to_string())?;
                    }
                } else {
                    ws.write_number_with_format(row, 5, item.net_length_mm.round(), &dfc).map_err(|e| e.to_string())?;
                    ws.write_number_with_format(row, 6, item.gross_length_mm.round(), &dfc).map_err(|e| e.to_string())?;
                    ws.write_string_with_format(row, 7, &format!("{:.0}\u{00b0}", item.miter_left_deg), &dfc).map_err(|e| e.to_string())?;
                    ws.write_string_with_format(row, 8, &format!("{:.0}\u{00b0}", item.miter_right_deg), &dfc).map_err(|e| e.to_string())?;
                }
                ws.write_number_with_format(row, 9, item.quantity as f64, &dfc).map_err(|e| e.to_string())?;
                if !item.masonry_notches.is_empty() {
                    let desc = item.masonry_notches.iter().map(crate::production::CutListItem::notch_text)
                        .collect::<Vec<_>>().join("\n");
                    ws.write_string_with_format(row, notch_col, desc,
                        &df.clone().set_text_wrap()).map_err(|e| e.to_string())?;
                }
                if let Some(joinery) = joints.get(item.piece_id.as_str()) {
                    let mm = dfc.clone().set_num_format("0.0");
                    for (i, j) in joinery.ends.iter().enumerate() {
                        let Some(j) = j else { continue };
                        let col = joint_col + 3 * i as u16;
                        if let Some(t) = &j.tenon {
                            ws.write_number_with_format(row, col, t.tenon_length_mm, &mm).map_err(|e| e.to_string())?;
                            ws.write_number_with_format(row, col + 1, t.mortise_depth_mm, &mm).map_err(|e| e.to_string())?;
                        }
                        ws.write_string_with_format(row, col + 2, j.receiver_piece_id.clone().unwrap_or_else(|| j.issue_text()), &df)
                            .map_err(|e| e.to_string())?;
                    }
                    ws.write_string_with_format(row, joint_col + 6, joinery.received_text(), &df).map_err(|e| e.to_string())?;
                }
                for (kind_index, &kind) in arch_kinds.iter().enumerate() {
                    use crate::production::DividerJointEnd::{Start, End};
                    for (offset, end) in [Start, End].into_iter().enumerate() {
                        if let Some(s) = prod.arch_joint_text(&item.piece_id, end, kind) {
                            ws.write_string_with_format(row, arch_col + (kind_index * 2 + offset) as u16, s, &df).map_err(|e| e.to_string())?;
                        }
                    }
                }
                if with_system_heads {
                    for (side, end) in [&item.divider_start, &item.divider_end].into_iter().enumerate() {
                        if let Some(status) = end.as_ref().and_then(|e| e.machining_status) {
                            ws.write_string_with_format(row, machining_col + side as u16, status.label_nl(), &df)
                                .map_err(|e| e.to_string())?;
                        }
                    }
                    if let Some(glm) = item.system_saw_cut_mm() {
                        ws.write_number_with_format(row, machining_col + 2, glm, &dfc.clone().set_num_format("0.0"))
                            .map_err(|e| e.to_string())?;
                    }
                }
                row += 1;
            }
        }
        autofit_widths(ws, &headers, row)?;
        if with_system_heads {
            for col in machining_col..machining_col + 3 {
                ws.set_column_width(col, 54.0).map_err(|e| e.to_string())?;
            }
        }
        for col in arch_col..(arch_col + 2 * arch_kinds.len() as u16) {
            ws.set_column_width(col, 54.0).map_err(|e| e.to_string())?;
        }
        if with_heads {
            for col in [head_col, head_col + 3] {
                ws.set_column_width(col, 34.0).map_err(|e| e.to_string())?;
                ws.set_column_width(col + 1, 34.0).map_err(|e| e.to_string())?;
                ws.set_column_width(col + 2, 76.0).map_err(|e| e.to_string())?;
            }
        }
    }

    for &kind in &arch_kinds {
        use super::arch_joints::{fields, headers, table_name, Field};
        let ws = wb.add_worksheet();
        ws.set_name(table_name(kind)).map_err(|e| e.to_string())?;
        let headers = headers(kind);
        write_headers(ws, &headers, &hfmt)?;
        let mut row = 1;
        for prod in production_data {
            for joint in prod.arch_joints.iter().filter(|j| j.joint.kind == kind) {
                let df = data_format(row % 2 == 0);
                for (col, field) in fields(&prod.kozijn_mark, joint).into_iter().enumerate() {
                    let col = col as u16;
                    match field {
                        Field::Text(s) => ws.write_string_with_format(row, col, s, &df),
                        Field::Number(n, precision) => {
                            let fmt = match precision { 2 => "0.00", 1 => "0.0", _ => "0" };
                            ws.write_number_with_format(row, col, n, &df.clone().set_num_format(fmt))
                        }
                        Field::Empty => continue,
                    }.map_err(|e| e.to_string())?;
                }
                row += 1;
            }
        }
        for (col, header) in headers.iter().enumerate() {
            let width = (header.chars().count() + 3).max(14);
            ws.set_column_width(col as u16, if col >= 26 { 55.0 } else { width as f64 }).map_err(|e| e.to_string())?;
        }
    }

    if with_joints {
        let ws = wb.add_worksheet();
        ws.set_name("Pen en gat").map_err(|e| e.to_string())?;
        let headers = ["Kozijn", "Penstuk", "Kop", "Ontvangend gatstuk", "Aantal pennen / gaten",
            "Penlengte (mm)", "Gatdiepte (mm)", "Bodemspeling (mm)", "Overmaat voorbij langpunt (mm)",
            "Penbreedte (mm)", "Schouderhart X (mm)", "Schouderhart Y (mm)", "Z-banden vanaf binnen (mm)", "Passing"];
        write_headers(ws, &headers, &hfmt)?;
        let mut row = 1;
        for prod in production_data {
            for j in &prod.divider_joints {
                let df = data_format(row % 2 == 0);
                for (col, value) in [&*prod.kozijn_mark, &*j.piece_id, j.end.label_nl(),
                    j.receiver_piece_id.as_deref().unwrap_or("")].into_iter().enumerate() {
                    ws.write_string_with_format(row, col as u16, value, &df).map_err(|e| e.to_string())?;
                }
                if let Some(t) = &j.tenon {
                    let mm = df.clone().set_num_format("0.0");
                    ws.write_number_with_format(row, 4, t.z_from_inside_mm.len() as f64, &df).map_err(|e| e.to_string())?;
                    for (i, n) in [t.tenon_length_mm, t.mortise_depth_mm, t.bottom_clearance_mm, t.blank_extension_mm,
                        t.cross_range_mm[1] - t.cross_range_mm[0], t.shoulder_center[0], t.shoulder_center[1]].into_iter().enumerate() {
                        ws.write_number_with_format(row, 5 + i as u16, n, &mm).map_err(|e| e.to_string())?;
                    }
                    ws.write_string_with_format(row, 12, j.z_bands(), &df).map_err(|e| e.to_string())?;
                }
                ws.write_string_with_format(row, 13, j.issue_text(), &df).map_err(|e| e.to_string())?;
                row += 1;
            }
        }
        autofit_widths(ws, &headers, row)?;
        ws.set_column_width(13, 34.0).map_err(|e| e.to_string())?;
    }

    // Glaslijst
    {
        let ws = wb.add_worksheet();
        ws.set_name("Glaslijst").map_err(|e| e.to_string())?;
        let mut headers = vec![
            "Kozijn", "Pos.", "Glastype", "Breedte", "Hoogte", "Dikte", "Ug", "Opp. (m\u{00b2})", "Aantal",
        ];
        let with_contours = production_data.iter().flat_map(|p| &p.glass_list).any(|i| i.cut_contour.is_some());
        if with_contours { headers.extend(["Snijvorm (mm)", "Contour in mm vanaf linksboven, x rechts / y omlaag"]); }
        headers.extend(["Productiegereed", "Blokkeerreden"]);
        write_headers(ws, &headers, &hfmt)?;
        let ready_col = headers.len() as u16 - 2;
        let issue_col = headers.len() as u16 - 1;

        let mut row = 1u32;
        for prod in production_data {
            for item in &prod.glass_list {
                let alt = row % 2 == 0;
                let df = data_format(alt);
                let dfc = data_format_center(alt);

                ws.write_string_with_format(row, 0, &prod.kozijn_mark, &df).map_err(|e| e.to_string())?;
                ws.write_string_with_format(row, 1, &item.piece_id, &df).map_err(|e| e.to_string())?;
                ws.write_string_with_format(row, 2, &item.glass_type, &df).map_err(|e| e.to_string())?;
                let dim = crate::production::GlassListItem::order_dimension_mm;
                ws.write_number_with_format(row, 3, dim(item.width_mm), &dfc).map_err(|e| e.to_string())?;
                ws.write_number_with_format(row, 4, dim(item.height_mm), &dfc).map_err(|e| e.to_string())?;
                ws.write_number_with_format(row, 5, item.thickness_mm.round(), &dfc).map_err(|e| e.to_string())?;
                ws.write_number_with_format(row, 6, (item.ug_value * 10.0).round() / 10.0, &dfc).map_err(|e| e.to_string())?;
                ws.write_number_with_format(row, 7, (item.area_m2 * 100.0).round() / 100.0, &dfc).map_err(|e| e.to_string())?;
                ws.write_number_with_format(row, 8, item.quantity as f64, &dfc).map_err(|e| e.to_string())?;
                if with_contours {
                    ws.write_string_with_format(row, 9, item.shape_label_nl(), &df).map_err(|e| e.to_string())?;
                    let steps = item.cut_steps();
                    if !steps.is_empty() {
                        ws.set_row_height(row, 15.0 * steps.len() as f64).map_err(|e| e.to_string())?;
                        ws.write_string_with_format(row, 10, steps.join("\n"), &df.clone().set_text_wrap())
                            .map_err(|e| e.to_string())?;
                    }
                }
                ws.write_string_with_format(
                    row,
                    ready_col,
                    if item.production_ready { "Ja" } else { "Nee" },
                    &df,
                )
                .map_err(|e| e.to_string())?;
                ws.write_string_with_format(
                    row,
                    issue_col,
                    item.production_issue.as_deref().unwrap_or(""),
                    &df,
                )
                .map_err(|e| e.to_string())?;
                row += 1;
            }
        }
        autofit_widths(ws, &headers, row)?;
        if with_contours { ws.set_column_width(10, 72.0).map_err(|e| e.to_string())?; }
    }

    // Beslaglijst
    {
        let ws = wb.add_worksheet();
        ws.set_name("Beslaglijst").map_err(|e| e.to_string())?;
        let mut headers = vec!["Kozijn", "Cel", "Component", "Omschrijving", "Aantal"];
        if production_data.iter().flat_map(|p| &p.hardware_list).any(|h| h.piece_id.is_some()) {
            headers.push("Positie");
        }
        write_headers(ws, &headers, &hfmt)?;

        let mut row = 1u32;
        for prod in production_data {
            for item in &prod.hardware_list {
                let alt = row % 2 == 0;
                let df = data_format(alt);
                let dfc = data_format_center(alt);

                ws.write_string_with_format(row, 0, &prod.kozijn_mark, &df).map_err(|e| e.to_string())?;
                ws.write_number_with_format(row, 1, (item.cell_index + 1) as f64, &dfc).map_err(|e| e.to_string())?;
                ws.write_string_with_format(row, 2, &item.component, &df).map_err(|e| e.to_string())?;
                ws.write_string_with_format(row, 3, &item.description, &df).map_err(|e| e.to_string())?;
                if item.quantity > 0 {
                    ws.write_number_with_format(row, 4, item.quantity as f64, &dfc).map_err(|e| e.to_string())?;
                }
                if let Some(piece) = &item.piece_id {
                    ws.write_string_with_format(row, 5, piece, &df).map_err(|e| e.to_string())?;
                }
                row += 1;
            }
        }
        autofit_widths(ws, &headers, row)?;
    }

    // Rubberlijst
    {
        let ws = wb.add_worksheet();
        ws.set_name("Rubberlijst").map_err(|e| e.to_string())?;
        let mut headers = vec!["Kozijn", "Type", "Lengte (mm)", "Aantal"];
        let refs = production_data.iter().flat_map(|p| &p.gasket_list).any(|g| g.reference.is_some());
        if refs { headers.extend(["Positie", "Profiel", "Netto meetlijn"]); }
        write_headers(ws, &headers, &hfmt)?;

        let mut row = 1u32;
        for prod in production_data {
            for item in &prod.gasket_list {
                let alt = row % 2 == 0;
                let df = data_format(alt);
                let dfc = data_format_center(alt);

                ws.write_string_with_format(row, 0, &prod.kozijn_mark, &df).map_err(|e| e.to_string())?;
                ws.write_string_with_format(row, 1, item.article_label_nl(), &df).map_err(|e| e.to_string())?;
                if let Some(r) = &item.reference {
                    ws.write_number_with_format(row, 2, item.length_mm, &dfc.clone().set_num_format("0.0")).map_err(|e| e.to_string())?;
                    ws.write_string_with_format(row, 4, &r.piece_id, &df).map_err(|e| e.to_string())?;
                    ws.write_string_with_format(row, 5, &r.profile_name, &df).map_err(|e| e.to_string())?;
                    ws.write_string_with_format(row, 6, r.line.label_nl(), &df).map_err(|e| e.to_string())?;
                } else {
                    ws.write_number_with_format(row, 2, item.length_mm.round(), &dfc).map_err(|e| e.to_string())?;
                }
                if item.quantity > 0 {
                    ws.write_number_with_format(row, 3, item.quantity as f64, &dfc).map_err(|e| e.to_string())?;
                }
                row += 1;
            }
        }
        autofit_widths(ws, &headers, row)?;
        if refs {
            for (column, width) in [(1, 39.0), (5, 40.0), (6, 30.0)] {
                ws.set_column_width(column, width).map_err(|e| e.to_string())?;
            }
        }
    }

    // Paneellijst (only if data exists)
    let has_panels = production_data.iter().any(|p| !p.panel_list.is_empty());
    if has_panels {
        let ws = wb.add_worksheet();
        ws.set_name("Paneellijst").map_err(|e| e.to_string())?;
        let headers = ["Kozijn", "Pos.", "Breedte", "Hoogte", "Type", "Aantal"];
        write_headers(ws, &headers, &hfmt)?;

        let mut row = 1u32;
        for prod in production_data {
            for item in &prod.panel_list {
                let alt = row % 2 == 0;
                let df = data_format(alt);
                let dfc = data_format_center(alt);

                ws.write_string_with_format(row, 0, &prod.kozijn_mark, &df).map_err(|e| e.to_string())?;
                ws.write_string_with_format(row, 1, &item.piece_id, &df).map_err(|e| e.to_string())?;
                ws.write_number_with_format(row, 2, item.width_mm.round(), &dfc).map_err(|e| e.to_string())?;
                ws.write_number_with_format(row, 3, item.height_mm.round(), &dfc).map_err(|e| e.to_string())?;
                ws.write_string_with_format(row, 4, &item.panel_type, &df).map_err(|e| e.to_string())?;
                ws.write_number_with_format(row, 5, item.quantity as f64, &dfc).map_err(|e| e.to_string())?;
                row += 1;
            }
        }
        autofit_widths(ws, &headers, row)?;
    }

    // Stuklijst
    {
        let ws = wb.add_worksheet();
        ws.set_name("Stuklijst").map_err(|e| e.to_string())?;
        let headers = ["Kozijn", "Categorie", "Omschrijving", "Eenheid", "Hoeveelheid"];
        write_headers(ws, &headers, &hfmt)?;

        let mut row = 1u32;
        for prod in production_data {
            for item in &prod.bom {
                let alt = row % 2 == 0;
                let df = data_format(alt);
                let dfc = data_format_center(alt);

                ws.write_string_with_format(row, 0, &prod.kozijn_mark, &df).map_err(|e| e.to_string())?;
                ws.write_string_with_format(row, 1, &item.category, &df).map_err(|e| e.to_string())?;
                ws.write_string_with_format(row, 2, &item.description, &df).map_err(|e| e.to_string())?;
                ws.write_string_with_format(row, 3, &item.unit, &df).map_err(|e| e.to_string())?;
                ws.write_number_with_format(row, 4, (item.quantity * 100.0).round() / 100.0, &dfc).map_err(|e| e.to_string())?;
                row += 1;
            }
        }
        autofit_widths(ws, &headers, row)?;
    }

    wb.save(output_path)
        .map_err(|e| format!("Kan XLSX niet opslaan: {}", e))?;

    Ok(())
}

fn write_headers(
    ws: &mut rust_xlsxwriter::Worksheet,
    headers: &[&str],
    fmt: &Format,
) -> Result<(), String> {
    for (col, header) in headers.iter().enumerate() {
        ws.write_string_with_format(0, col as u16, *header, fmt)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn autofit_widths(
    ws: &mut rust_xlsxwriter::Worksheet,
    headers: &[&str],
    _row_count: u32,
) -> Result<(), String> {
    for (col, header) in headers.iter().enumerate() {
        let w = (header.len() as f64 + 3.0).min(30.0).max(8.0);
        ws.set_column_width(col as u16, w)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
