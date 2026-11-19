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
    // When re-exporting to a previous folder, replace outdated joint rows too.
    for kind in super::arch_joints::KINDS {
        let name = format!("{}.csv", super::arch_joints::table_name(kind).to_ascii_lowercase());
        if !super::arch_joints::present(production_data, kind) && !std::path::Path::new(output_dir).join(&name).exists() {
            continue;
        }
        let mut w = create_csv(output_dir, &name)?;
        write_row(&mut w, &super::arch_joints::headers(kind))?;
        for prod in production_data {
            for joint in prod.arch_joints.iter().filter(|j| j.joint.kind == kind) {
                let fields: Vec<_> = super::arch_joints::fields(&prod.kozijn_mark, joint)
                    .iter().map(|f| f.csv()).collect();
                write_row(&mut w, &fields.iter().map(String::as_str).collect::<Vec<_>>())?;
            }
        }
        w.flush().map_err(|e| e.to_string())?;
    }
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
    let line = fields.iter().map(|field| {
        if field.contains([';', '"', '\r', '\n']) {
            format!("\"{}\"", field.replace('"', "\"\""))
        } else {
            (*field).to_owned()
        }
    }).collect::<Vec<_>>().join(";");
    writeln!(w, "{}", line).map_err(|e| e.to_string())
}

fn write_kortlijst(data: &[ProductionData], dir: &str) -> Result<(), String> {
    let mut w = create_csv(dir, "kortlijst.csv")?;
    let with_arcs = data.iter().flat_map(|p| &p.cut_list).any(|i| i.has_curved_cut());
    let with_heads = data.iter().flat_map(|p| &p.cut_list).any(|i| i.divider_start.is_some());
    let with_notches = data.iter().flat_map(|p| &p.cut_list).any(|i| !i.masonry_notches.is_empty());
    let with_joints = data.iter().any(|p| !p.divider_joints.is_empty());
    let arch_kinds: Vec<_> = super::arch_joints::KINDS.into_iter().filter(|&kind| super::arch_joints::present(data, kind)).collect();
    let mut headers = vec!["Kozijn", "Pos", "Onderdeel", "Profiel", "Materiaal", "Netto_mm", "Bruto_mm",
        "Hoek_L", "Hoek_R", "Aantal"];
    if with_arcs {
        headers[7] = "Zaaghoek_begin_graden_90_haaks";
        headers[8] = "Zaaghoek_eind_graden_90_haaks";
        headers.extend(["Buitenstraal_mm", "Buitenbooglengte_mm", "Hartstraal_mm", "Hartbooglengte_mm", "Buitenbooghoek_graden"]);
        headers.extend(["Pasfrezen_begin_R_mm", "Pasfrezen_eind_R_mm"]);
    }
    if with_heads {
        headers[7] = "Zaag_of_koordehoek_begin_graden_90_haaks";
        headers[8] = "Zaag_of_koordehoek_eind_graden_90_haaks";
        headers.extend(["Begin_insprong_links_of_boven_mm", "Begin_insprong_rechts_of_onder_mm",
            "Begin_kopprofiel_XY_mm_vanaf_kozijn_linksboven", "Eind_insprong_links_of_boven_mm",
            "Eind_insprong_rechts_of_onder_mm", "Eind_kopprofiel_XY_mm_vanaf_kozijn_linksboven"]);
    }
    if with_notches { headers.push("Metselwerkuitsparing_mm_vanaf_kozijn_linksboven"); }
    if with_joints {
        headers.extend(["Begin_penlengte_mm", "Begin_gatdiepte_mm", "Begin_gatstuk", "Begin_aantal_pennen",
            "Begin_overmaat_langpunt_mm", "Begin_bodemvrij_mm", "Begin_penbreedte_mm",
            "Begin_schouderhart_X_mm", "Begin_schouderhart_Y_mm", "Begin_Z_banden_mm_vanaf_binnen", "Begin_passing",
            "Eind_penlengte_mm", "Eind_gatdiepte_mm", "Eind_gatstuk", "Eind_aantal_pennen",
            "Eind_overmaat_langpunt_mm", "Eind_bodemvrij_mm", "Eind_penbreedte_mm",
            "Eind_schouderhart_X_mm", "Eind_schouderhart_Y_mm", "Eind_Z_banden_mm_vanaf_binnen", "Eind_passing",
            "Ontvangt_gaten_van_kop_aantal_diepte_mm"]);
    }
    for &kind in &arch_kinds {
        use crate::curved_timber::ArchJointKind;
        headers.extend(match kind {
            ArchJointKind::Springing => ["Boogvoet_begin", "Boogvoet_eind"],
            ArchJointKind::RingSeam => ["Ringnaad_begin", "Ringnaad_eind"],
        });
    }
    write_row(&mut w, &headers)?;
    for prod in data {
        let joints = prod.joint_index();
        for item in &prod.cut_list {
            let mm = |v: f64| if with_arcs { format!("{v:.1}") } else { v.round().to_string() };
            let angle = |v: f64| if with_arcs { format!("{v:.2}") } else { v.round().to_string() };
            let mut fields = vec![
                prod.kozijn_mark.clone(), item.piece_id.clone(), item.member_label_nl().into(),
                item.profile_name.clone(), item.material.clone(),
                mm(item.net_length_mm), prod.blank_length_mm(item).map(mm).unwrap_or_default(),
                angle(item.miter_left_deg), angle(item.miter_right_deg),
                item.quantity.to_string(),
            ];
            if with_arcs {
                if let Some(arc) = &item.arc {
                    fields.extend([format!("{:.1}", arc.outer_radius_mm), format!("{:.1}", arc.outer_length_mm),
                        format!("{:.1}", arc.centerline_radius_mm), format!("{:.1}", arc.centerline_length_mm),
                        format!("{:.2}", arc.sweep_deg)]);
                } else {
                    fields.resize(15, String::new());
                }
                for radius in [item.start_cope_radius_mm, item.end_cope_radius_mm] {
                    fields.push(radius.map_or_else(String::new, |r| format!("{r:.1}")));
                }
            }
            if with_heads {
                for end in [&item.divider_start, &item.divider_end] {
                    if let Some(end) = end {
                        fields.extend(end.edge_offsets_mm.iter().map(|d| format!("{d:.1}")));
                        fields.push(end.segments.iter().map(crate::production::cut_step).collect::<Vec<_>>().join(" | "));
                    } else {
                        fields.resize(fields.len() + 3, String::new());
                    }
                }
            }
            if with_notches {
                fields.push(item.masonry_notches.iter().map(crate::production::CutListItem::notch_text)
                    .collect::<Vec<_>>().join(" | "));
            }
            if with_joints {
                let cut = joints.get(item.piece_id.as_str());
                for side in 0..2 {
                    let joint = cut.and_then(|c| c.ends[side]);
                    if let Some((j, t)) = joint.and_then(|j| j.tenon.as_ref().map(|t| (j, t))) {
                        fields.extend([mm(t.tenon_length_mm), mm(t.mortise_depth_mm),
                            j.receiver_piece_id.clone().unwrap_or_default(), t.z_from_inside_mm.len().to_string(),
                            mm(t.blank_extension_mm), mm(t.bottom_clearance_mm), mm(t.cross_range_mm[1] - t.cross_range_mm[0]),
                            mm(t.shoulder_center[0]), mm(t.shoulder_center[1]), j.z_bands()]);
                    } else {
                        fields.resize(fields.len() + 10, String::new());
                    }
                    fields.push(joint.map_or_else(String::new, |j| j.issue_text()));
                }
                fields.push(cut.map_or_else(String::new, |c| c.received_text()));
            }
            for &kind in &arch_kinds {
                use crate::production::DividerJointEnd::{Start, End};
                for end in [Start, End] {
                    fields.push(prod.arch_joint_text(&item.piece_id, end, kind).unwrap_or_default());
                }
            }
            write_row(&mut w, &fields.iter().map(String::as_str).collect::<Vec<_>>())?;
        }
    }
    Ok(())
}

fn write_glaslijst(data: &[ProductionData], dir: &str) -> Result<(), String> {
    let mut w = create_csv(dir, "glaslijst.csv")?;
    let with_contours = data.iter().flat_map(|p| &p.glass_list).any(|i| i.cut_contour.is_some());
    let mut headers = vec!["Kozijn", "Pos", "Glastype", "Breedte_mm", "Hoogte_mm", "Dikte_mm", "Ug", "Opp_m2", "Aantal"];
    if with_contours { headers.extend(["Snijvorm_mm", "Contour_mm_vanaf_linksboven_x_rechts_y_omlaag"]); }
    write_row(&mut w, &headers)?;
    for prod in data {
        for item in &prod.glass_list {
            let mut fields = vec![
                prod.kozijn_mark.clone(), item.piece_id.clone(), item.glass_type.clone(),
                item.dimension_text(item.width_mm), item.dimension_text(item.height_mm),
                format!("{:.0}", item.thickness_mm), format!("{:.1}", item.ug_value),
                format!("{:.2}", item.area_m2), item.quantity.to_string(),
            ];
            if with_contours { fields.extend([item.shape_label_nl(), item.cut_steps().join(" | ")]); }
            write_row(&mut w, &fields.iter().map(String::as_str).collect::<Vec<_>>())?;
        }
    }
    Ok(())
}

fn write_beslaglijst(data: &[ProductionData], dir: &str) -> Result<(), String> {
    let mut w = create_csv(dir, "beslaglijst.csv")?;
    let positions = data.iter().flat_map(|p| &p.hardware_list).any(|h| h.piece_id.is_some());
    let mut headers = vec!["Kozijn", "Cel", "Component", "Omschrijving", "Aantal"];
    if positions { headers.push("Positie"); }
    write_row(&mut w, &headers)?;
    for prod in data {
        for item in &prod.hardware_list {
            let cell = (item.cell_index + 1).to_string();
            let quantity = if item.quantity > 0 { item.quantity.to_string() } else { String::new() };
            let mut fields = vec![prod.kozijn_mark.as_str(), &cell, &item.component, &item.description, &quantity];
            if positions { fields.push(item.piece_id.as_deref().unwrap_or("")); }
            write_row(&mut w, &fields)?;
        }
    }
    Ok(())
}

fn write_rubberlijst(data: &[ProductionData], dir: &str) -> Result<(), String> {
    let mut w = create_csv(dir, "rubberlijst.csv")?;
    let refs = data.iter().flat_map(|p| &p.gasket_list).any(|g| g.reference.is_some());
    let mut headers = vec!["Kozijn", "Type", "Lengte_mm", "Aantal"];
    if refs { headers.extend(["Positie", "Profiel", "Netto meetlijn"]); }
    write_row(&mut w, &headers)?;
    for prod in data {
        for item in &prod.gasket_list {
            let length = if item.reference.is_some() { format!("{:.1}", item.length_mm) } else { format!("{:.0}", item.length_mm) };
            let quantity = if item.quantity > 0 { item.quantity.to_string() } else { String::new() };
            let mut fields = vec![prod.kozijn_mark.as_str(), item.gasket_type.label_nl(), &length, &quantity];
            if refs {
                fields.extend(item.reference.as_ref().map_or([""; 3], |r| [&r.piece_id, &r.profile_name, r.line.label_nl()]));
            }
            write_row(&mut w, &fields)?;
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
