//! CSV production list generation.
//!
//! Generates semicolon-delimited CSV files (UTF-8 BOM) for:
//! kortlijst, glaslijst, beslaglijst, rubberlijst, stuklijst.

use std::io::Write;

use crate::production::ProductionData;

/// Generate CSV production list files in the given output directory.
pub fn generate_production_csv(
    production_data: &[ProductionData],
    output_dir: &str,
) -> Result<(), String> {
    std::fs::create_dir_all(output_dir)
        .map_err(|e| format!("Kan directory niet aanmaken: {}", e))?;

    write_kortlijst(production_data, output_dir)?;
    write_glaslijst(production_data, output_dir)?;
    write_beslaglijst(production_data, output_dir)?;
    write_rubberlijst(production_data, output_dir)?;
    write_stuklijst(production_data, output_dir)?;

    // Panel list only if any panels exist
    let has_panels = production_data.iter().any(|p| !p.panel_list.is_empty());
    if has_panels {
        write_paneellijst(production_data, output_dir)?;
    }

    Ok(())
}

fn create_csv(dir: &str, name: &str) -> Result<std::io::BufWriter<std::fs::File>, String> {
    let path = std::path::Path::new(dir).join(name);
    let file = std::fs::File::create(&path)
        .map_err(|e| format!("Kan {} niet aanmaken: {}", name, e))?;
    let mut writer = std::io::BufWriter::new(file);
    // UTF-8 BOM for Excel compatibility
    writer.write_all(b"\xEF\xBB\xBF").map_err(|e| e.to_string())?;
    Ok(writer)
}

fn write_row(w: &mut impl Write, fields: &[&str]) -> Result<(), String> {
    let line = fields.join(";");
    writeln!(w, "{}", line).map_err(|e| e.to_string())
}

fn write_kortlijst(data: &[ProductionData], dir: &str) -> Result<(), String> {
    let mut w = create_csv(dir, "kortlijst.csv")?;
    write_row(
        &mut w,
        &[
            "Kozijn",
            "Pos",
            "Onderdeel",
            "Profiel",
            "Materiaal",
            "Netto_mm",
            "Bruto_mm",
            "Hoek_L",
            "Hoek_R",
            "Aantal",
        ],
    )?;
    for prod in data {
        for item in &prod.cut_list {
            write_row(
                &mut w,
                &[
                    &prod.kozijn_mark,
                    &item.piece_id,
                    item.member_type.label_nl(),
                    &item.profile_name,
                    &item.material,
                    &format!("{}", item.net_length_mm.round() as i64),
                    &format!("{}", item.gross_length_mm.round() as i64),
                    &format!("{}", item.miter_left_deg.round() as i64),
                    &format!("{}", item.miter_right_deg.round() as i64),
                    &format!("{}", item.quantity),
                ],
            )?;
        }
    }
    Ok(())
}

fn write_glaslijst(data: &[ProductionData], dir: &str) -> Result<(), String> {
    let mut w = create_csv(dir, "glaslijst.csv")?;
    write_row(
        &mut w,
        &[
            "Kozijn",
            "Pos",
            "Glastype",
            "Breedte_mm",
            "Hoogte_mm",
            "Dikte_mm",
            "Ug",
            "Opp_m2",
            "Aantal",
        ],
    )?;
    for prod in data {
        for item in &prod.glass_list {
            write_row(
                &mut w,
                &[
                    &prod.kozijn_mark,
                    &item.piece_id,
                    &item.glass_type,
                    &format!("{}", item.width_mm.round() as i64),
                    &format!("{}", item.height_mm.round() as i64),
                    &format!("{}", item.thickness_mm.round() as i64),
                    &format!("{:.1}", item.ug_value),
                    &format!("{:.2}", item.area_m2),
                    &format!("{}", item.quantity),
                ],
            )?;
        }
    }
    Ok(())
}

fn write_beslaglijst(data: &[ProductionData], dir: &str) -> Result<(), String> {
    let mut w = create_csv(dir, "beslaglijst.csv")?;
    write_row(&mut w, &["Kozijn", "Cel", "Component", "Omschrijving", "Aantal"])?;
    for prod in data {
        for item in &prod.hardware_list {
            write_row(
                &mut w,
                &[
                    &prod.kozijn_mark,
                    &format!("{}", item.cell_index + 1),
                    &item.component,
                    &item.description,
                    &format!("{}", item.quantity),
                ],
            )?;
        }
    }
    Ok(())
}

fn write_rubberlijst(data: &[ProductionData], dir: &str) -> Result<(), String> {
    let mut w = create_csv(dir, "rubberlijst.csv")?;
    write_row(&mut w, &["Kozijn", "Type", "Lengte_mm", "Aantal"])?;
    for prod in data {
        for item in &prod.gasket_list {
            write_row(
                &mut w,
                &[
                    &prod.kozijn_mark,
                    item.gasket_type.label_nl(),
                    &format!("{}", item.length_mm.round() as i64),
                    &format!("{}", item.quantity),
                ],
            )?;
        }
    }
    Ok(())
}

fn write_stuklijst(data: &[ProductionData], dir: &str) -> Result<(), String> {
    let mut w = create_csv(dir, "stuklijst.csv")?;
    write_row(
        &mut w,
        &["Kozijn", "Categorie", "Omschrijving", "Eenheid", "Hoeveelheid"],
    )?;
    for prod in data {
        for item in &prod.bom {
            write_row(
                &mut w,
                &[
                    &prod.kozijn_mark,
                    &item.category,
                    &item.description,
                    &item.unit,
                    &format!("{:.2}", item.quantity),
                ],
            )?;
        }
    }
    Ok(())
}

fn write_paneellijst(data: &[ProductionData], dir: &str) -> Result<(), String> {
    let mut w = create_csv(dir, "paneellijst.csv")?;
    write_row(
        &mut w,
        &["Kozijn", "Pos", "Breedte_mm", "Hoogte_mm", "Type", "Aantal"],
    )?;
    for prod in data {
        for item in &prod.panel_list {
            write_row(
                &mut w,
                &[
                    &prod.kozijn_mark,
                    &item.piece_id,
                    &format!("{}", item.width_mm.round() as i64),
                    &format!("{}", item.height_mm.round() as i64),
                    &item.panel_type,
                    &format!("{}", item.quantity),
                ],
            )?;
        }
    }
    Ok(())
}
