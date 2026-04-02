//! Excel (.xlsx) export for kozijnstaat and production lists.

use rust_xlsxwriter::{Format, FormatAlign, FormatBorder, Color, Workbook};

use crate::kozijn::{Material, Project, WoodType};
use crate::production::ProductionData;

const DEEP_FORGE: u32 = 0x36363E;
const AMBER: u32 = 0xD97706;
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
    {
        let ws = wb.add_worksheet();
        ws.set_name("Kortlijst").map_err(|e| e.to_string())?;
        let headers = [
            "Kozijn", "Pos.", "Onderdeel", "Profiel", "Materiaal",
            "Netto (mm)", "Bruto (mm)", "Hoek L", "Hoek R", "Aantal",
        ];
        write_headers(ws, &headers, &hfmt)?;

        let mut row = 1u32;
        for prod in production_data {
            for item in &prod.cut_list {
                let alt = row % 2 == 0;
                let df = data_format(alt);
                let dfc = data_format_center(alt);

                ws.write_string_with_format(row, 0, &prod.kozijn_mark, &df).map_err(|e| e.to_string())?;
                ws.write_string_with_format(row, 1, &item.piece_id, &df).map_err(|e| e.to_string())?;
                ws.write_string_with_format(row, 2, item.member_type.label_nl(), &df).map_err(|e| e.to_string())?;
                ws.write_string_with_format(row, 3, &item.profile_name, &df).map_err(|e| e.to_string())?;
                ws.write_string_with_format(row, 4, &item.material, &df).map_err(|e| e.to_string())?;
                ws.write_number_with_format(row, 5, item.net_length_mm.round(), &dfc).map_err(|e| e.to_string())?;
                ws.write_number_with_format(row, 6, item.gross_length_mm.round(), &dfc).map_err(|e| e.to_string())?;
                ws.write_string_with_format(row, 7, &format!("{:.0}\u{00b0}", item.miter_left_deg), &dfc).map_err(|e| e.to_string())?;
                ws.write_string_with_format(row, 8, &format!("{:.0}\u{00b0}", item.miter_right_deg), &dfc).map_err(|e| e.to_string())?;
                ws.write_number_with_format(row, 9, item.quantity as f64, &dfc).map_err(|e| e.to_string())?;
                row += 1;
            }
        }
        autofit_widths(ws, &headers, row)?;
    }

    // Glaslijst
    {
        let ws = wb.add_worksheet();
        ws.set_name("Glaslijst").map_err(|e| e.to_string())?;
        let headers = [
            "Kozijn", "Pos.", "Glastype", "Breedte", "Hoogte", "Dikte", "Ug", "Opp. (m\u{00b2})", "Aantal",
        ];
        write_headers(ws, &headers, &hfmt)?;

        let mut row = 1u32;
        for prod in production_data {
            for item in &prod.glass_list {
                let alt = row % 2 == 0;
                let df = data_format(alt);
                let dfc = data_format_center(alt);

                ws.write_string_with_format(row, 0, &prod.kozijn_mark, &df).map_err(|e| e.to_string())?;
                ws.write_string_with_format(row, 1, &item.piece_id, &df).map_err(|e| e.to_string())?;
                ws.write_string_with_format(row, 2, &item.glass_type, &df).map_err(|e| e.to_string())?;
                ws.write_number_with_format(row, 3, item.width_mm.round(), &dfc).map_err(|e| e.to_string())?;
                ws.write_number_with_format(row, 4, item.height_mm.round(), &dfc).map_err(|e| e.to_string())?;
                ws.write_number_with_format(row, 5, item.thickness_mm.round(), &dfc).map_err(|e| e.to_string())?;
                ws.write_number_with_format(row, 6, (item.ug_value * 10.0).round() / 10.0, &dfc).map_err(|e| e.to_string())?;
                ws.write_number_with_format(row, 7, (item.area_m2 * 100.0).round() / 100.0, &dfc).map_err(|e| e.to_string())?;
                ws.write_number_with_format(row, 8, item.quantity as f64, &dfc).map_err(|e| e.to_string())?;
                row += 1;
            }
        }
        autofit_widths(ws, &headers, row)?;
    }

    // Beslaglijst
    {
        let ws = wb.add_worksheet();
        ws.set_name("Beslaglijst").map_err(|e| e.to_string())?;
        let headers = ["Kozijn", "Cel", "Component", "Omschrijving", "Aantal"];
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
                ws.write_number_with_format(row, 4, item.quantity as f64, &dfc).map_err(|e| e.to_string())?;
                row += 1;
            }
        }
        autofit_widths(ws, &headers, row)?;
    }

    // Rubberlijst
    {
        let ws = wb.add_worksheet();
        ws.set_name("Rubberlijst").map_err(|e| e.to_string())?;
        let headers = ["Kozijn", "Type", "Lengte (mm)", "Aantal"];
        write_headers(ws, &headers, &hfmt)?;

        let mut row = 1u32;
        for prod in production_data {
            for item in &prod.gasket_list {
                let alt = row % 2 == 0;
                let df = data_format(alt);
                let dfc = data_format_center(alt);

                ws.write_string_with_format(row, 0, &prod.kozijn_mark, &df).map_err(|e| e.to_string())?;
                ws.write_string_with_format(row, 1, item.gasket_type.label_nl(), &df).map_err(|e| e.to_string())?;
                ws.write_number_with_format(row, 2, item.length_mm.round(), &dfc).map_err(|e| e.to_string())?;
                ws.write_number_with_format(row, 3, item.quantity as f64, &dfc).map_err(|e| e.to_string())?;
                row += 1;
            }
        }
        autofit_widths(ws, &headers, row)?;
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
