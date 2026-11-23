//! PDF export for kozijnstaat, workshop drawing, and production lists.
//!
//! Uses printpdf 0.9 Op-based API.

use printpdf::*;

use crate::kozijn::{Kozijn, Material, PanelType, Project, WoodType};
use crate::production::ProductionData;

const DEEP_FORGE: (f32, f32, f32) = (0.212, 0.212, 0.243);
const AMBER: (f32, f32, f32) = (0.851, 0.467, 0.024);
const LIGHT_GRAY: (f32, f32, f32) = (0.906, 0.898, 0.906);
const ALT_ROW: (f32, f32, f32) = (0.961, 0.961, 0.957);
const GRID_CLR: (f32, f32, f32) = (0.8, 0.8, 0.8);
const WHITE_C: (f32, f32, f32) = (1.0, 1.0, 1.0);

const A4_W: f32 = 297.0;
const A4_H: f32 = 210.0;
const A3_W: f32 = 420.0;
const A3_H: f32 = 297.0;

fn col(c: (f32, f32, f32)) -> Color {
    Color::Rgb(Rgb { r: c.0, g: c.1, b: c.2, icc_profile: None })
}

fn font() -> PdfFontHandle { PdfFontHandle::Builtin(BuiltinFont::Helvetica) }
fn font_bold() -> PdfFontHandle { PdfFontHandle::Builtin(BuiltinFont::HelveticaBold) }

fn text_ops(s: &str, size: f32, x: f32, y: f32, f: PdfFontHandle, c: (f32, f32, f32)) -> Vec<Op> {
    vec![
        Op::SetFillColor { col: col(c) },
        Op::StartTextSection,
        Op::SetFont { font: f, size: Pt(size) },
        Op::SetTextCursor { pos: Point { x: Mm(x).into(), y: Mm(y).into() } },
        Op::ShowText { items: vec![TextItem::Text(s.to_string())] },
        Op::EndTextSection,
    ]
}

fn fill_rect_ops(x: f32, y: f32, w: f32, h: f32, c: (f32, f32, f32)) -> Vec<Op> {
    vec![
        Op::SetFillColor { col: col(c) },
        Op::DrawPolygon { polygon: Polygon {
            rings: vec![PolygonRing {
                points: vec![
                    LinePoint { p: Point { x: Mm(x).into(), y: Mm(y).into() }, bezier: false },
                    LinePoint { p: Point { x: Mm(x+w).into(), y: Mm(y).into() }, bezier: false },
                    LinePoint { p: Point { x: Mm(x+w).into(), y: Mm(y+h).into() }, bezier: false },
                    LinePoint { p: Point { x: Mm(x).into(), y: Mm(y+h).into() }, bezier: false },
                ],
            }],
            mode: PaintMode::Fill,
            winding_order: WindingOrder::NonZero,
        }},
    ]
}

fn line_ops(x1: f32, y1: f32, x2: f32, y2: f32, c: (f32, f32, f32), t: f32) -> Vec<Op> {
    vec![
        Op::SetOutlineColor { col: col(c) },
        Op::SetOutlineThickness { pt: Pt(t) },
        Op::DrawLine { line: Line {
            points: vec![
                LinePoint { p: Point { x: Mm(x1).into(), y: Mm(y1).into() }, bezier: false },
                LinePoint { p: Point { x: Mm(x2).into(), y: Mm(y2).into() }, bezier: false },
            ],
            is_closed: false,
        }},
    ]
}

fn save_doc(doc: &PdfDocument, path: &str) -> Result<(), String> {
    let mut warnings = Vec::new();
    let bytes = doc.save(&PdfSaveOptions::default(), &mut warnings);
    std::fs::write(path, bytes).map_err(|e| format!("Kan PDF niet opslaan: {}", e))
}

// ══════════════════════════════════════════════════════════════
// Kozijnstaat PDF
// ══════════════════════════════════════════════════════════════

pub fn generate_kozijnstaat_pdf(project: &Project, output_path: &str) -> Result<(), String> {
    let mut doc = PdfDocument::new("Kozijnstaat");
    let margin: f32 = 15.0;
    let mut ops = Vec::new();
    let mut y = A4_H - margin;

    ops.extend(text_ops(
        &format!("Kozijnstaat \u{2014} {}", project.project_info.name),
        18.0, margin, y, font_bold(), DEEP_FORGE,
    ));
    y -= 8.0;
    ops.extend(text_ops(
        &format!("Nr: {} | {}", project.project_info.number, project.project_info.client),
        10.0, margin, y, font(), (0.5, 0.5, 0.5),
    ));
    y -= 12.0;

    let col_w: [f32; 12] = [12.0,18.0,14.0,14.0,18.0,10.0,10.0,10.0,35.0,20.0,16.0,16.0];
    let rh: f32 = 6.0;
    let tw: f32 = col_w.iter().sum();

    ops.extend(fill_rect_ops(margin, y - rh, tw, rh, DEEP_FORGE));
    let headers = ["Mark","Name","W","H","Material","Col","Row","Cells","Types","Glass","In","Out"];
    let mut x = margin;
    for (i, h) in headers.iter().enumerate() {
        ops.extend(text_ops(h, 7.0, x + 1.0, y - rh + 1.5, font_bold(), WHITE_C));
        x += col_w[i];
    }
    ops.extend(line_ops(margin, y - rh, margin + tw, y - rh, AMBER, 1.5));
    y -= rh;

    for (ri, kozijn) in project.kozijnen.iter().enumerate() {
        if y < margin + rh { break; }
        if ri % 2 == 1 {
            ops.extend(fill_rect_ops(margin, y - rh, tw, rh, ALT_ROW));
        }
        let f = &kozijn.frame;
        let vals = [
            kozijn.mark.clone(), kozijn.name.clone(),
            format!("{}", f.outer_width as i64), format!("{}", f.outer_height as i64),
            mat_label(&f.material),
            format!("{}", kozijn.grid.columns.len()), format!("{}", kozijn.grid.rows.len()),
            format!("{}", kozijn.cells.len()),
            String::new(), String::new(),
            f.color_inside.clone(), f.color_outside.clone(),
        ];
        let mut x = margin;
        for (i, v) in vals.iter().enumerate() {
            let s = if v.len() > 25 { &v[..22] } else { v.as_str() };
            ops.extend(text_ops(s, 7.0, x + 1.0, y - rh + 1.5, font(), DEEP_FORGE));
            x += col_w[i];
        }
        ops.extend(line_ops(margin, y - rh, margin + tw, y - rh, GRID_CLR, 0.3));
        y -= rh;
    }

    y -= 6.0;
    ops.extend(text_ops(
        "Generated by Open Frame Studio \u{2014} OpenAEC Foundation",
        8.0, margin, y, font(), (0.5, 0.5, 0.5),
    ));

    doc.pages.push(PdfPage::new(Mm(A4_W), Mm(A4_H), ops));
    save_doc(&doc, output_path)
}

// ══════════════════════════════════════════════════════════════
// Workshop drawing PDF
// ══════════════════════════════════════════════════════════════

/// Elevation coordinates are y-down. Sampling a circular edge at this scale
/// keeps the sagitta under 0.02 mm on paper and includes each exact end point.
fn workshop_contour(ops: &mut Vec<Op>, contour: &crate::curved_contour::Contour,
    origin: [f32; 2], scale: f32, fill: (f32, f32, f32)) {
    use crate::curved_contour::Segment;
    let mut points = Vec::new();
    for edge in &contour.segments {
        let steps = match edge {
            Segment::Line { .. } => 1,
            Segment::Arc { radius, sweep_angle, .. } => {
                let paper_r = radius * f64::from(scale);
                (sweep_angle.abs() * (paper_r / (8.0 * 0.02)).sqrt()).ceil().max(2.) as usize
            }
        };
        for n in 0..steps {
            let [x, y] = edge.point(n as f64 / steps as f64);
            points.push(LinePoint { p: Point {
                x: Mm(origin[0] + x as f32 * scale).into(),
                y: Mm(origin[1] - y as f32 * scale).into(),
            }, bezier: false });
        }
    }
    if points.len() >= 3 {
        ops.push(Op::SetFillColor { col: col(fill) });
        ops.push(Op::DrawPolygon { polygon: Polygon { rings: vec![PolygonRing { points }],
            mode: PaintMode::FillStroke, winding_order: WindingOrder::NonZero } });
    }
}

pub fn generate_workshop_pdf(
    kozijn: &Kozijn,
    project: &Project,
    output_path: &str,
) -> Result<(), String> {
    let mut doc = PdfDocument::new("Werkplaatstekening");
    let mut ops = Vec::new();
    let margin: f32 = 15.0;
    let tb_h: f32 = 40.0;
    let daw = A3_W - 2.0 * margin;
    let dah = A3_H - 2.0 * margin - tb_h;

    let frame = &kozijn.frame;
    let ow = frame.outer_width as f32;
    let oh = frame.outer_height as f32;
    let fw = frame.frame_width as f32;

    // Title block
    ops.extend(fill_rect_ops(margin, margin, daw, tb_h, DEEP_FORGE));
    ops.extend(line_ops(margin, margin + tb_h, margin + daw, margin + tb_h, AMBER, 3.0));
    let pi = &project.project_info;
    ops.extend(text_ops("Open Frame Studio", 14.0, margin + 8.0, margin + tb_h - 14.0, font_bold(), WHITE_C));
    ops.extend(text_ops("OpenAEC Foundation", 8.0, margin + 8.0, margin + tb_h - 20.0, font(), AMBER));

    // Project info
    let px = margin + 90.0;
    ops.extend(text_ops(&pi.name, 10.0, px, margin + tb_h - 14.0, font_bold(), WHITE_C));
    ops.extend(text_ops(
        &format!("Nr: {} | {}", pi.number, pi.client),
        8.0, px, margin + tb_h - 22.0, font(), (0.8, 0.8, 0.8),
    ));

    // Kozijn info
    let kx = margin + 220.0;
    ops.extend(text_ops(&kozijn.mark, 16.0, kx, margin + tb_h - 20.0, font_bold(), AMBER));
    ops.extend(text_ops(&kozijn.name, 9.0, kx + 30.0, margin + tb_h - 18.0, font(), WHITE_C));
    ops.extend(text_ops(
        &format!("{} x {} mm | {}", ow as i64, oh as i64, mat_label(&frame.material)),
        8.0, kx, margin + tb_h - 28.0, font(), (0.8, 0.8, 0.8),
    ));

    // Scale
    let geometry = crate::geometry::compute_2d_geometry(kozijn);
    let dim_m: f32 = if geometry.curved_frame.is_some() { 25.0 } else { 60.0 };
    let aw = daw - 2.0 * dim_m;
    let ah = dah - 2.0 * dim_m;
    let fitted_scale = (aw / ow).min(ah / oh) * 0.85;
    let sc = if geometry.curved_frame.is_some() {
        1.0 / (1.0 / fitted_scale).ceil().max(1.0)
    } else {
        fitted_scale
    };
    let ox = margin + dim_m + (aw - ow * sc) / 2.0;
    let oy = margin + tb_h + dim_m + (ah - oh * sc) / 2.0;

    let sx = |v: f32| -> f32 { ox + v * sc };
    let sy = |v: f32| -> f32 { oy + v * sc };

    if let Some(curve) = &geometry.curved_frame {
        let origin = [ox, sy(oh)];
        if geometry.curved_divider_timber.iter().any(|p| p.start.machining_status.is_some()) {
            ops.extend(text_ops("Koppen in aanzicht op de daglijn. Afkortmaat en bewerkingsstatus per kop staan in de zaaglijst.",
                8.0, margin + 35.0, margin + tb_h + 5.0, font(), DEEP_FORGE));
        }
        ops.push(Op::SetOutlineColor { col: col(DEEP_FORGE) });
        ops.push(Op::SetOutlineThickness { pt: Pt(0.45) });
        if geometry.curved_timber.is_empty() {
            workshop_contour(&mut ops, &curve.outer, origin, sc, LIGHT_GRAY);
            workshop_contour(&mut ops, &curve.day, origin, sc, WHITE_C);
        } else {
            for member in &geometry.curved_timber {
                workshop_contour(&mut ops, &member.outline, origin, sc, LIGHT_GRAY);
                for notch in &member.masonry_notches {
                    workshop_contour(&mut ops, &member.outline.clone().clipped(notch), origin, sc, WHITE_C);
                }
            }
        }
        for part in &geometry.curved_divider_timber {
            workshop_contour(&mut ops, &part.outline, origin, sc, LIGHT_GRAY);
        }
        for cell in &geometry.cell_rects {
            let Some(bounds) = &cell.opening_bounds else { continue };
            if bounds.width <= 0. || bounds.height <= 0. { continue; }
            for fit in [&cell.fit, &cell.second_fit].into_iter().flatten() {
                let Some(edges) = &fit.contours else { continue };
                if let Some(sash) = &edges.sash_outer {
                    workshop_contour(&mut ops, sash, origin, sc, LIGHT_GRAY);
                }
                let inside = fit.bead_position == crate::glaslat::GlaslatPosition::Binnen;
                workshop_contour(&mut ops, if inside { &edges.rebate } else { &edges.day }, origin, sc, LIGHT_GRAY);
                if inside && fit.bead_height < edges.rebate_height {
                    workshop_contour(&mut ops, &edges.bead_inner, origin, sc, LIGHT_GRAY);
                }
                let visible_glass = if fit.bead_height >= edges.rebate_height { &edges.bead_inner } else { &edges.day };
                workshop_contour(&mut ops, visible_glass, origin, sc, WHITE_C);
            }
            let label = match &cell.vulling {
                Some(crate::layout::Vakvulling::Buiten) => continue,
                Some(crate::layout::Vakvulling::Glas { .. }) => "VG",
                Some(crate::layout::Vakvulling::Raam { .. }) => "Raam",
                Some(crate::layout::Vakvulling::Deur { .. }) => "Deur",
                Some(crate::layout::Vakvulling::Paneel { .. }) => "PN",
                Some(crate::layout::Vakvulling::Rooster) => "VR",
                None => kozijn.cells.get(cell.cell_index).map_or("", |c| panel_label(c.panel_type)),
            };
            let x = (bounds.x + bounds.width / 2.) as f32;
            let y = (bounds.y + bounds.height / 2.) as f32;
            ops.extend(text_ops(label, 8., sx(x) - 2., sy(oh - y) - 1.5, font_bold(), DEEP_FORGE));
        }
    } else {
        // Outer frame
        ops.extend(line_ops(sx(0.0), sy(0.0), sx(ow), sy(0.0), DEEP_FORGE, 2.0));
        ops.extend(line_ops(sx(ow), sy(0.0), sx(ow), sy(oh), DEEP_FORGE, 2.0));
        ops.extend(line_ops(sx(ow), sy(oh), sx(0.0), sy(oh), DEEP_FORGE, 2.0));
        ops.extend(line_ops(sx(0.0), sy(oh), sx(0.0), sy(0.0), DEEP_FORGE, 2.0));

        // Frame members
        for (fx, fy, fw2, fh) in [
            (0.0, oh - fw, ow, fw), (0.0, 0.0, ow, fw),
            (0.0, fw, fw, oh - 2.0 * fw), (ow - fw, fw, fw, oh - 2.0 * fw),
        ] {
            ops.extend(fill_rect_ops(sx(fx), sy(fy), fw2 * sc, fh * sc, LIGHT_GRAY));
        }

        // Cell labels
        let columns = &kozijn.grid.columns;
        let rows = &kozijn.grid.rows;
        let nc = columns.len();
        let mut col_pos = Vec::new();
        let mut cx = fw;
        for (i, c) in columns.iter().enumerate() {
            col_pos.push(cx);
            cx += c.size as f32;
            if i < columns.len() - 1 { cx += fw; }
        }
        let mut row_pos = Vec::new();
        let mut ry = fw;
        for (i, r) in rows.iter().enumerate() {
            row_pos.push(ry);
            ry += r.size as f32;
            if i < rows.len() - 1 { ry += fw; }
        }

        for (ri, row) in rows.iter().enumerate() {
            for (ci, col) in columns.iter().enumerate() {
                let idx = ri * nc + ci;
                if idx >= kozijn.cells.len() { continue; }
                let cx = col_pos[ci];
                let cy = row_pos[ri];
                let cw = col.size as f32;
                let ch = row.size as f32;
                ops.extend(text_ops(
                    panel_label(kozijn.cells[idx].panel_type),
                    8.0, sx(cx + cw/2.0) - 2.0, sy(cy + ch/2.0) - 1.5, font_bold(), DEEP_FORGE,
                ));
            }
        }

    }
    // Dimensions
    let dim_off: f32 = 15.0;
    let tick: f32 = 3.0;
    let yd = sy(0.0) - dim_off;
    ops.extend(line_ops(sx(0.0), yd, sx(ow), yd, DEEP_FORGE, 0.5));
    ops.extend(line_ops(sx(0.0), yd-tick, sx(0.0), yd+tick, DEEP_FORGE, 0.5));
    ops.extend(line_ops(sx(ow), yd-tick, sx(ow), yd+tick, DEEP_FORGE, 0.5));
    ops.extend(text_ops(&format!("{}", ow as i32), 7.0, sx(ow/2.0)-5.0, yd-5.0, font(), DEEP_FORGE));

    let xd = sx(ow) + dim_off;
    ops.extend(line_ops(xd, sy(0.0), xd, sy(oh), DEEP_FORGE, 0.5));
    ops.extend(line_ops(xd-tick, sy(0.0), xd+tick, sy(0.0), DEEP_FORGE, 0.5));
    ops.extend(line_ops(xd-tick, sy(oh), xd+tick, sy(oh), DEEP_FORGE, 0.5));
    ops.extend(text_ops(&format!("{}", oh as i32), 7.0, xd+3.0, sy(oh/2.0), font(), DEEP_FORGE));

    let st = if sc < 1.0 { format!("1:{:.1}", 1.0/sc) } else { format!("{:.1}:1", sc) };
    ops.extend(text_ops(&st, 8.0, margin+5.0, margin+tb_h+5.0, font(), (0.5,0.5,0.5)));

    doc.pages.push(PdfPage::new(Mm(A3_W), Mm(A3_H), ops));
    save_doc(&doc, output_path)
}

// ══════════════════════════════════════════════════════════════
// Production lists PDF
// ══════════════════════════════════════════════════════════════

pub fn generate_production_pdf(
    production_data: &[ProductionData],
    output_path: &str,
) -> Result<(), String> {
    let mut doc = PdfDocument::new("Productiestaten");
    let margin: f32 = 12.0;

    for prod in production_data {
        let mut ops = Vec::new();
        let mut y = A4_H - margin;

        ops.extend(text_ops(
            &format!("Productiestaten - {} {}", prod.kozijn_mark, prod.kozijn_name),
            16.0, margin, y, font_bold(), DEEP_FORGE,
        ));
        y -= 10.0;

        if !prod.cut_list.is_empty() {
            let arcs: Vec<_> = prod.cut_list.iter().filter_map(|i| i.arc.as_ref().map(|a| (i, a))).collect();
            ops.extend(text_ops("Kortlijst", 12.0, margin, y, font_bold(), AMBER));
            y -= 6.0;
            if !arcs.is_empty() {
                ops.extend(text_ops("Netto booglengte langs de buitenrand. Bruto ontwikkelde lengte inclusief toeslagen (mm).",
                    8.0, margin, y, font(), DEEP_FORGE));
                y -= 5.0;
                ops.extend(text_ops("Zaaghoeken t.o.v. de lengterichting, bij bogen t.o.v. de raaklijn aan de buitenrand. 90 graden = haaks.",
                    8.0, margin, y, font(), DEEP_FORGE));
                y -= 4.0;
                let h = ["Pos.", "R buiten (mm)", "Boog buiten (mm)", "R hart (mm)", "Boog hart (mm)", "Booghoek buiten (graden)"];
                let w = [25.0, 44.0, 52.0, 44.0, 52.0, 51.0];
                let rows: Vec<_> = arcs.iter().map(|(i, a)| vec![
                    i.piece_id.clone(), format!("{:.1}", a.outer_radius_mm), format!("{:.1}", a.outer_length_mm),
                    format!("{:.1}", a.centerline_radius_mm), format!("{:.1}", a.centerline_length_mm),
                    format!("{:.2}", a.sweep_deg),
                ]).collect();
                y = pdf_table(&mut doc, &mut ops, &prod.kozijn_mark, margin, y, &h, &w, &rows) - 3.0;
            }
            let h = ["Pos.","Onderdeel","Profiel","Materiaal","Netto (mm)","Bruto (mm)","Begin (graden)","Eind (graden)","Aantal"];
            let w = [25.0,36.0,64.0,29.0,21.0,21.0,28.0,28.0,16.0_f32];
            let with_curved_cuts = prod.cut_list.iter().any(|i| i.has_curved_cut());
            if prod.cut_list.iter().any(|i| i.divider_start.is_some()) {
                y = ensure_table_space(&mut doc, &mut ops, &prod.kozijn_mark, y, 17.0);
                ops.extend(text_ops("Meet de nettolengte van lang punt tot lang punt. Bij een ronde of samengestelde kop geldt de hoek tussen de koorde en de lengterichting.",
                    8.0, margin, y, font(), DEEP_FORGE));
                y -= 5.0;
            }
            let mm = if with_curved_cuts { 1 } else { 0 };
            let deg = if with_curved_cuts { 2 } else { 0 };
            let rows: Vec<Vec<String>> = prod.cut_list.iter().map(|i| vec![
                i.piece_id.clone(), i.member_label_nl().into(), i.profile_name.clone(),
                i.material.clone(), format!("{:.*}", mm, i.net_length_mm),
                prod.blank_length_mm(i).map(|v| format!("{:.*}", mm, v)).unwrap_or_else(|| "Onbepaald".into()),
                crate::production::CutListItem::end_text(i.miter_left_deg, i.start_cope_radius_mm, deg),
                crate::production::CutListItem::end_text(i.miter_right_deg, i.end_cope_radius_mm, deg),
                format!("{}", i.quantity),
            ]).collect();
            y = pdf_table(&mut doc, &mut ops, &prod.kozijn_mark, margin, y, &h, &w, &rows);
            y -= 4.0;
            let system_heads: Vec<_> = prod.cut_list.iter().filter(|i| i.has_system_heads()).map(|i| {
                let status = |end: &Option<crate::curved_timber::DividerEnd>| {
                    end.as_ref().and_then(|e| e.machining_status).map_or("", |s| s.label_nl()).to_owned()
                };
                vec![i.piece_id.clone(), status(&i.divider_start), status(&i.divider_end),
                    i.system_saw_cut_mm().map_or_else(|| "Onbepaald".to_owned(), |v| format!("{v:.1}"))]
            }).collect();
            if !system_heads.is_empty() {
                y = ensure_table_space(&mut doc, &mut ops, &prod.kozijn_mark, y, 27.0);
                ops.extend(text_ops("Bewerking T-profielen", 12.0, margin, y, font_bold(), AMBER));
                y -= 5.0;
                ops.extend(text_ops("De kopradius en koordehoek beschrijven de aansluiting in aanzicht. GLM is de PVC-zaagmaat voor de standaardverbinder, exclusief zaagsnede.",
                    8.0, margin, y, font(), DEEP_FORGE));
                y -= 4.5;
                let h = ["Pos.", "Fabrikantbewerking begin", "Fabrikantbewerking eind", "PVC GLM (mm)"];
                let w = [28.0, 98.0, 98.0, 40.0];
                y = pdf_table(&mut doc, &mut ops, &prod.kozijn_mark, margin, y, &h, &w, &wrap_details(&system_heads)) - 4.0;
            }
            for kind in super::arch_joints::KINDS {
                let joints: Vec<_> = prod.arch_joints.iter().filter(|j| j.joint.kind == kind).collect();
                if joints.is_empty() { continue; }
                let ring = kind == crate::curved_timber::ArchJointKind::RingSeam;
                let side = if ring { "Naad" } else { "Voet" };
                y = ensure_table_space(&mut doc, &mut ops, &prod.kozijn_mark, y, if ring { 67.0 } else { 62.0 });
                let title = format!("{} - pen en gat", super::arch_joints::table_name(kind));
                ops.extend(text_ops(&title, 12.0, margin, y, font_bold(), AMBER));
                y -= 5.0;
                if ring {
                    ops.extend(text_ops("De bovenboog loopt van links naar rechts, de onderboog van rechts naar links. Begin en eind volgen de eigen boog.",
                        8.0, margin, y, font(), DEEP_FORGE));
                    y -= 4.5;
                }
                ops.extend(text_ops("Schouderhoek in graden vanaf de lengteas of buitenraaklijn. Penlengte en gatdiepte in mm langs de insteekas vanaf het hart van de gatmond.",
                    8.0, margin, y, font(), DEEP_FORGE));
                y -= 4.5;
                ops.extend(text_ops("Overmaat telt eenmaal mee in de bruto lengte voorbij het langpunt, bij de boog in de ontwikkelde buitenlengte. Z-banden in mm vanaf het binnenvlak.",
                    8.0, margin, y, font(), DEEP_FORGE));
                y -= 3.5;
                let h = [side, "Penstuk / kop", "Gatstuk / kop", "Hoek pen", "Hoek gat", "Aantal", "Penlengte", "Gatdiepte", "Bodemvrij", "Overmaat", "Breedte", "Z-banden"];
                let w = [14., 40., 40., 20., 20., 12., 18., 18., 18., 17., 16., 39.];
                let mut sizes = Vec::new();
                let mut locations = Vec::new();
                let xy = |[x, y]: [f64; 2]| format!("{x:.1}, {y:.1}");
                for row in &joints {
                    let j = &row.joint;
                    let mut dims = vec![row.side_nl().into(), format!("{} / {}", row.piece_id, row.end.label_nl()),
                        format!("{} / {}", row.receiver_piece_id, row.receiver_end.label_nl()),
                        format!("{:.2}", j.source_angle_deg), format!("{:.2}", j.receiver_angle_deg)];
                    let mut loc = vec![row.side_nl().into(), xy(j.shoulder_face[0]), xy(j.shoulder_face[1])];
                    if let Some(t) = &j.tenon {
                        let [a, b] = t.tip;
                        dims.push(t.z_from_inside_mm.len().to_string());
                        dims.extend([t.tenon_length_mm, t.mortise_depth_mm, t.bottom_clearance_mm,
                            t.blank_extension_mm, (a[0] - b[0]).hypot(a[1] - b[1])].map(|n| format!("{n:.1}")));
                        dims.push(crate::production::z_band_text(&t.z_from_inside_mm));
                        let [x, y] = t.shoulder_center;
                        let [ox, oy] = j.shoulder_face[0];
                        let [a, b] = t.bottom;
                        loc.extend([xy(t.shoulder_center), format!("{:.1}", (x - ox).hypot(y - oy)),
                            format!("{:.2}", t.direction[1].atan2(t.direction[0]).to_degrees().rem_euclid(360.)),
                            xy([(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5])]);
                    }
                    sizes.push(dims);
                    locations.push(loc);
                }
                y = pdf_table(&mut doc, &mut ops, &prod.kozijn_mark, margin, y, &h, &w, &sizes) - 5.0;
                ops.extend(text_ops("Afstellen gatstuk. X rechts en Y omlaag vanaf kozijn linksboven. Afstand tot het hart langs de schouder vanaf buitenpunt. Insteekhoek rechtsom vanaf X+.",
                    8.0, margin, y, font(), DEEP_FORGE));
                y -= 3.5;
                let h = [side, "Buitenpunt X, Y", "Dagpunt X, Y", "Hart gatmond X, Y", "Afstand tot hart (mm)", "Insteekhoek (graden)", "Hart gatbodem X, Y"];
                let w = [14., 37., 37., 41., 47., 44., 42.];
                y = pdf_table(&mut doc, &mut ops, &prod.kozijn_mark, margin, y, &h, &w, &locations) - 5.0;
                for row in joints {
                    if let Some(issue) = row.joint.issue {
                        y = ensure_table_space(&mut doc, &mut ops, &prod.kozijn_mark, y, 9.0);
                        let note = format!("{}   {} / {}   {}   (opgegeven pen {:.1} mm)",
                            row.side_nl(), row.piece_id, row.receiver_piece_id, issue, row.joint.requested_length_mm);
                        ops.extend(text_ops(&note, 8.0, margin, y, font_bold(), DEEP_FORGE));
                        y -= 5.0;
                    }
                }
            }
            let mut heads = Vec::new();
            for item in &prod.cut_list {
                for (name, end) in [("Begin", &item.divider_start), ("Eind", &item.divider_end)] {
                    let Some(end) = end else { continue };
                    if end.radius_mm.is_none() && end.segments.len() == 1 && (end.angle_deg - 90.0).abs() < 1e-7 { continue; }
                    for edge in &end.segments {
                        use crate::curved_contour::Segment;
                        let xy = |[x, y]: [f64; 2]| format!("{x:.1}, {y:.1}");
                        let (kind, center, radius, sweep) = match edge {
                            Segment::Line { .. } => ("Recht", String::new(), String::new(), String::new()),
                            Segment::Arc { center, radius, sweep_angle, .. } =>
                                ("Ronding", xy(*center), format!("{radius:.1}"), format!("{:.2}", sweep_angle.to_degrees())),
                        };
                        heads.push(vec![item.piece_id.clone(), name.into(), kind.into(), format!("{:.1}", end.edge_offsets_mm[0]),
                            format!("{:.1}", end.edge_offsets_mm[1]), xy(edge.from()), xy(edge.to()), center, radius, sweep]);
                    }
                }
            }
            if !heads.is_empty() {
                y = ensure_table_space(&mut doc, &mut ops, &prod.kozijn_mark, y, 32.0);
                ops.extend(text_ops("Kopprofielen van tussenstijlen en tussendorpels", 12.0, margin, y, font_bold(), AMBER));
                y -= 5.0;
                ops.extend(text_ops("Insprong in mm vanaf het langste punt van de kop. Bij een stijl is rand 1 links en rand 2 rechts. Bij een dorpel is rand 1 boven en rand 2 onder.",
                    8.0, margin, y, font(), DEEP_FORGE));
                y -= 4.5;
                ops.extend(text_ops("Begin is boven bij een stijl en links bij een dorpel. X en Y in mm vanaf kozijn linksboven, naar rechts en omlaag. De booghoek is positief met de klok mee.",
                    8.0, margin, y, font(), DEEP_FORGE));
                y -= 3.5;
                let h = ["Pos.", "Kop", "Vorm", "Insprong rand 1", "Insprong rand 2", "Van X, Y", "Naar X, Y", "Middelpunt X, Y", "Radius (mm)", "Boog (graden)"];
                let w = [26.0, 15.0, 19.0, 26.0, 26.0, 34.0, 34.0, 40.0, 24.0, 24.0];
                y = pdf_table(&mut doc, &mut ops, &prod.kozijn_mark, margin, y, &h, &w, &heads) - 6.0;
            }
            if !prod.divider_joints.is_empty() {
                y = ensure_table_space(&mut doc, &mut ops, &prod.kozijn_mark, y, 34.0);
                ops.extend(text_ops("Pen-en-gatverbindingen tussenhout", 12.0, margin, y, font_bold(), AMBER));
                y -= 5.0;
                ops.extend(text_ops("Penlengte en gatdiepte langs de houtas vanaf het schouderhart. Overmaat is de extra blanklengte voorbij het langste punt van de kop.",
                    8.0, margin, y, font(), DEEP_FORGE));
                y -= 4.5;
                ops.extend(text_ops("Alle maten in mm. Hart X, Y vanaf linksboven in het kozijn. De Z-banden geven per pen en gat de beide wangen vanaf het binnenvlak van het kozijn.",
                    8.0, margin, y, font(), DEEP_FORGE));
                y -= 3.5;
                let h = ["Penstuk", "Kop", "Gatstuk", "Aantal", "Penlengte", "Gatdiepte", "Bodemvrij", "Overmaat", "Penbreedte", "Hart X, Y", "Z-banden"];
                let w = [28.0, 13.0, 28.0, 13.0, 19.0, 19.0, 20.0, 19.0, 20.0, 32.0, 39.0];
                let rows: Vec<_> = prod.divider_joints.iter().filter_map(|j| {
                    let t = j.tenon.as_ref()?;
                    Some(vec![j.piece_id.clone(), j.end.label_nl().into(), j.receiver_piece_id.clone().unwrap_or_default(),
                        t.z_from_inside_mm.len().to_string(), format!("{:.1}", t.tenon_length_mm),
                        format!("{:.1}", t.mortise_depth_mm), format!("{:.1}", t.bottom_clearance_mm),
                        format!("{:.1}", t.blank_extension_mm), format!("{:.1}", t.cross_range_mm[1] - t.cross_range_mm[0]),
                        format!("{:.1}, {:.1}", t.shoulder_center[0], t.shoulder_center[1]), j.z_bands()])
                }).collect();
                if !rows.is_empty() {
                    y = pdf_table(&mut doc, &mut ops, &prod.kozijn_mark, margin, y, &h, &w, &rows) - 7.0;
                }
                for j in prod.divider_joints.iter().filter(|j| j.issue.is_some()) {
                    y = ensure_table_space(&mut doc, &mut ops, &prod.kozijn_mark, y, 9.0);
                    let note = format!("{}   {}   {}", j.piece_id, j.end.label_nl(), j.issue_text());
                    ops.extend(text_ops(&note, 8.0, margin, y, font_bold(), DEEP_FORGE));
                    y -= 5.0;
                }
            }
            for cut in &prod.cut_list {
                for rect in &cut.masonry_notches {
                    if y < 28.0 {
                        doc.pages.push(PdfPage::new(Mm(A4_W), Mm(A4_H), std::mem::take(&mut ops)));
                        y = A4_H - margin;
                    }
                    let note = format!("{}   Uitsparing vanaf linksboven kozijn (mm)   {}",
                        cut.piece_id, crate::production::CutListItem::notch_text(rect));
                    ops.extend(text_ops(&note, 8.0, margin, y, font(), DEEP_FORGE));
                    y -= 6.0;
                }
            }
        }

        if !prod.glass_list.is_empty() {
            let first_rows = prod.glass_list.len().min(2) as f32;
            y = ensure_table_space(&mut doc, &mut ops, &prod.kozijn_mark, y, 11.0 + 5.0 * first_rows);
            ops.extend(text_ops("Glaslijst", 12.0, margin, y, font_bold(), AMBER));
            y -= 6.0;
            let mut h = vec!["Pos.","Type","W","H","D","Ug","m2","Qty"];
            let mut w = vec![18.0,30.0,18.0,18.0,18.0,14.0,16.0,12.0_f32];
            if prod.glass_list.iter().any(|g| g.cut_contour.is_some()) {
                h.push("Snijvorm (mm)");
                w.push(36.0);
            }
            let rows: Vec<Vec<String>> = prod.glass_list.iter().map(|i| vec![
                i.piece_id.clone(),
                if i.production_ready {
                    i.glass_type.clone()
                } else {
                    format!("{} [AFGEKEURD]", i.glass_type)
                },
                i.dimension_text(i.width_mm), i.dimension_text(i.height_mm),
                format!("{:.0}", i.thickness_mm), format!("{:.1}", i.ug_value),
                format!("{:.2}", i.area_m2), format!("{}", i.quantity),
                i.shape_label_nl().replace('Ø', "diam. "),
            ]).collect();
            y = pdf_table(&mut doc, &mut ops, &prod.kozijn_mark, margin, y, &h, &w, &rows);
            y -= 4.0;
        }

        let seals: Vec<_> = prod.gasket_list.iter().filter_map(|item| {
            let r = item.reference.as_ref()?;
            let quantity = if item.quantity == 0 { "opgave".into() } else { item.quantity.to_string() };
            Some(vec![r.piece_id.clone(), r.profile_name.clone(), item.article_label_nl(),
                r.line.label_nl().into(), format!("{:.1}", item.length_mm), quantity])
        }).collect();
        if !seals.is_empty() {
            y = ensure_table_space(&mut doc, &mut ops, &prod.kozijn_mark, y, 25.0);
            ops.extend(text_ops("Rubberlijst", 12.0, margin, y, font_bold(), AMBER));
            y -= 5.0;
            ops.extend(text_ops("Netto modelomtrek in mm. Bepaal groefpositie en snijtoeslag volgens het profielsysteem.",
                8.0, margin, y, font(), DEEP_FORGE));
            y -= 4.0;
            y = pdf_table(&mut doc, &mut ops, &prod.kozijn_mark, margin, y,
                &["Positie", "Profiel", "Dichting", "Meetlijn", "Lengte (mm)", "Aantal"],
                &[29.0, 57.0, 52.0, 47.0, 24.0, 16.0], &wrap_details(&seals)) - 5.0;
        }
        let hardware: Vec<_> = prod.hardware_list.iter().filter(|item| item.quantity > 0).filter_map(|item| {
            let piece = item.piece_id.as_ref()?;
            Some(vec![piece.clone(), item.component.clone(), item.description.clone(), item.quantity.to_string()])
        }).collect();
        if !hardware.is_empty() {
            y = ensure_table_space(&mut doc, &mut ops, &prod.kozijn_mark, y, 22.0);
            ops.extend(text_ops("Beslag per vleugel", 12.0, margin, y, font_bold(), AMBER));
            y -= 6.0;
            y = pdf_table(&mut doc, &mut ops, &prod.kozijn_mark, margin, y,
                &["Positie", "Component", "Montage / omschrijving", "Aantal"],
                &[32.0, 44.0, 72.0, 16.0], &wrap_details(&hardware)) - 5.0;
        }
        for item in prod.hardware_list.iter().filter(|i| i.piece_id.is_some() && i.quantity == 0) {
            y = ensure_table_space(&mut doc, &mut ops, &prod.kozijn_mark, y, 11.0);
            let line = format!("{}   {}   {}", item.piece_id.as_deref().unwrap_or(""), item.component, item.description);
            ops.extend(text_ops(&line, 8.0, margin, y, font_bold(), DEEP_FORGE));
            y -= 6.0;
        }
        if !prod.bom.is_empty() {
            y = ensure_table_space(&mut doc, &mut ops, &prod.kozijn_mark, y, 16.0);
            ops.extend(text_ops("Stuklijst (BOM)", 12.0, margin, y, font_bold(), AMBER));
            y -= 6.0;
            let h = ["Category","Description","Unit","Qty"];
            let w = [25.0,60.0,16.0,16.0_f32];
            let rows: Vec<Vec<String>> = prod.bom.iter().map(|i| vec![
                i.category.clone(), i.description.clone(), i.unit.clone(), format!("{:.2}", i.quantity),
            ]).collect();
            let rows = if seals.is_empty() { rows } else { wrap_details(&rows) };
            pdf_table(&mut doc, &mut ops, &prod.kozijn_mark, margin, y, &h, &w, &rows);
        }

        doc.pages.push(PdfPage::new(Mm(A4_W), Mm(A4_H), ops));
        // Circular segments of opposite handedness can share W, H and radius.
        // Give the workshop the ordered cut edges on a separate sheet.
        let edges: Vec<_> = prod.glass_list.iter()
            .filter(|i| i.cut_contour.as_ref().is_some_and(|c| c.segments.iter()
                .any(|s| matches!(s, crate::curved_contour::Segment::Line { .. }))))
            .flat_map(|i| i.cut_steps().into_iter().map(move |step| (&i.piece_id, step)))
            .collect();
        for page in edges.chunks(28) {
            let mut ops = text_ops(&format!("Snijcontouren glas - {}", prod.kozijn_mark),
                14.0, margin, A4_H - 20.0, font_bold(), DEEP_FORGE);
            ops.extend(text_ops("mm vanaf linksboven, x naar rechts en y omlaag. M = middelpunt, R = straal.",
                9.0, margin, A4_H - 28.0, font(), DEEP_FORGE));
            for (row, (piece, step)) in page.iter().enumerate() {
                let y = A4_H - 40.0 - row as f32 * 5.5;
                ops.extend(text_ops(piece, 8.0, margin, y, font_bold(), DEEP_FORGE));
                ops.extend(text_ops(&step.replace('°', " graden"), 8.0, margin + 30.0, y, font(), DEEP_FORGE));
            }
            doc.pages.push(PdfPage::new(Mm(A4_W), Mm(A4_H), ops));
        }
    }

    if doc.pages.is_empty() {
        doc.pages.push(PdfPage::new(Mm(A4_W), Mm(A4_H), vec![]));
    }

    save_doc(&doc, output_path)
}

fn ensure_table_space(doc: &mut PdfDocument, ops: &mut Vec<Op>, mark: &str, y: f32, needed: f32) -> f32 {
    if y - needed >= 12.0 { return y; }
    doc.pages.push(PdfPage::new(Mm(A4_W), Mm(A4_H), std::mem::take(ops)));
    ops.extend(text_ops(&format!("Productiestaten - {mark} (vervolg)"), 14.0, 12.0, A4_H - 12.0, font_bold(), DEEP_FORGE));
    A4_H - 22.0
}

// The workshop table renderer permits 30 characters per cell. Keep the full
// system/stock designation by flowing long values onto continuation lines.
fn wrap_details(rows: &[Vec<String>]) -> Vec<Vec<String>> {
    let mut out = Vec::new();
    for row in rows {
        let columns: Vec<Vec<String>> = row.iter().map(|value| {
            let mut lines = Vec::new();
            let mut rest = value.as_str();
            while let Some((end, _)) = rest.char_indices().nth(30) {
                let split = rest[..end].rfind(' ').filter(|i| *i > 0).unwrap_or(end);
                lines.push(rest[..split].to_owned());
                rest = rest[split..].trim_start();
            }
            lines.push(rest.to_owned());
            lines
        }).collect();
        for i in 0..columns.iter().map(Vec::len).max().unwrap_or(0) {
            out.push(columns.iter().map(|column| column.get(i).cloned().unwrap_or_default()).collect());
        }
    }
    out
}

fn pdf_table(doc: &mut PdfDocument, ops: &mut Vec<Op>, mark: &str, x0: f32, y0: f32, headers: &[&str], widths: &[f32], rows: &[Vec<String>]) -> f32 {
    let rh: f32 = 5.0;
    let tw: f32 = widths.iter().sum();

    let header = |ops: &mut Vec<Op>, y: f32| {
        ops.extend(fill_rect_ops(x0, y - rh, tw, rh, DEEP_FORGE));
        let mut x = x0;
        for (i, h) in headers.iter().enumerate() {
            ops.extend(text_ops(h, 7.0, x + 1.0, y - rh + 1.5, font_bold(), WHITE_C));
            x += widths[i];
        }
        y - rh
    };

    let top = ensure_table_space(doc, ops, mark, y0, 2.0 * rh);
    let mut y = header(ops, top);
    for (ri, row) in rows.iter().enumerate() {
        let next = ensure_table_space(doc, ops, mark, y, rh);
        if next != y { y = header(ops, next); }
        if ri % 2 == 0 {
            ops.extend(fill_rect_ops(x0, y - rh, tw, rh, ALT_ROW));
        }
        ops.extend(line_ops(x0, y - rh, x0 + tw, y - rh, GRID_CLR, 0.3));
        let mut x = x0;
        for (i, v) in row.iter().enumerate() {
            if i < widths.len() {
                let end = v.char_indices().nth(30).map_or(v.len(), |(at, _)| at);
                ops.extend(text_ops(&v[..end], 7.0, x + 1.0, y - rh + 1.5, font(), DEEP_FORGE));
                x += widths[i];
            }
        }
        y -= rh;
    }
    y
}

fn panel_label(pt: PanelType) -> &'static str {
    match pt {
        PanelType::FixedGlass => "VG", PanelType::TurnTilt => "DK",
        PanelType::Turn => "D", PanelType::Tilt => "K",
        PanelType::Sliding => "S", PanelType::Door => "DR",
        PanelType::Panel => "P", PanelType::Ventilation => "V",
        PanelType::TopHung => "KL", PanelType::BottomHung => "TM",
        PanelType::LiftSlide => "HS", PanelType::Pivot => "PV",
    }
}

fn mat_label(m: &Material) -> String {
    match m {
        Material::Wood(w) => format!("Hout ({})", match w {
            WoodType::Meranti => "meranti", WoodType::Accoya => "accoya",
            WoodType::Vuren => "vuren", WoodType::Eiken => "eiken",
        }),
        Material::Aluminum => "Aluminium".into(),
        Material::Pvc => "Kunststof".into(),
        Material::WoodAluminum => "Hout-Aluminium".into(),
    }
}
