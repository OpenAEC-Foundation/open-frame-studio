//! PDF export for kozijnstaat, workshop drawing, and production lists.
//!
//! Uses printpdf 0.9 Op-based API.

use printpdf::*;

use crate::kozijn::{Kozijn, Material, PanelType, Project, WoodType, OpeningDirection};
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
    ops.extend(text_ops("Open Frame Studio", 14.0, margin + 8.0, margin + tb_h - 14.0, font_bold(), WHITE_C));
    ops.extend(text_ops("OpenAEC Foundation", 8.0, margin + 8.0, margin + tb_h - 20.0, font(), AMBER));

    let kx = margin + 180.0;
    ops.extend(text_ops(&kozijn.mark, 16.0, kx, margin + tb_h - 20.0, font_bold(), AMBER));
    ops.extend(text_ops(&kozijn.name, 9.0, kx + 30.0, margin + tb_h - 18.0, font(), WHITE_C));
    ops.extend(text_ops(
        &format!("{} x {} mm", ow as i64, oh as i64),
        8.0, kx, margin + tb_h - 28.0, font(), (0.8, 0.8, 0.8),
    ));

    // Scale
    let dim_m: f32 = 60.0;
    let aw = daw - 2.0 * dim_m;
    let ah = dah - 2.0 * dim_m;
    let sc = (aw / ow).min(ah / oh) * 0.85;
    let ox = margin + dim_m + (aw - ow * sc) / 2.0;
    let oy = margin + tb_h + dim_m + (ah - oh * sc) / 2.0;

    let sx = |v: f32| -> f32 { ox + v * sc };
    let sy = |v: f32| -> f32 { oy + v * sc };

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

    let st = if sc < 1.0 { format!("1:{}", (1.0/sc) as i32) } else { format!("{:.1}:1", sc) };
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
            &format!("Productiestaten \u{2014} {} {}", prod.kozijn_mark, prod.kozijn_name),
            16.0, margin, y, font_bold(), DEEP_FORGE,
        ));
        y -= 10.0;

        if !prod.cut_list.is_empty() {
            ops.extend(text_ops("Kortlijst", 12.0, margin, y, font_bold(), AMBER));
            y -= 6.0;
            let h = ["Pos.","Part","Profile","Mat.","Net","Gross","L\u{00b0}","R\u{00b0}","Qty"];
            let w = [18.0,28.0,28.0,22.0,16.0,16.0,12.0,12.0,12.0_f32];
            let rows: Vec<Vec<String>> = prod.cut_list.iter().map(|i| vec![
                i.piece_id.clone(), i.member_type.label_nl().into(), i.profile_name.clone(),
                i.material.clone(), format!("{:.0}", i.net_length_mm), format!("{:.0}", i.gross_length_mm),
                format!("{:.0}\u{00b0}", i.miter_left_deg), format!("{:.0}\u{00b0}", i.miter_right_deg),
                format!("{}", i.quantity),
            ]).collect();
            y = pdf_table(&mut ops, margin, y, &h, &w, &rows);
            y -= 4.0;
        }

        if !prod.glass_list.is_empty() {
            ops.extend(text_ops("Glaslijst", 12.0, margin, y, font_bold(), AMBER));
            y -= 6.0;
            let h = ["Pos.","Type","W","H","D","Ug","m\u{00b2}","Qty"];
            let w = [18.0,30.0,18.0,18.0,18.0,14.0,16.0,12.0_f32];
            let rows: Vec<Vec<String>> = prod.glass_list.iter().map(|i| vec![
                i.piece_id.clone(), i.glass_type.clone(),
                format!("{:.0}", i.width_mm), format!("{:.0}", i.height_mm),
                format!("{:.0}", i.thickness_mm), format!("{:.1}", i.ug_value),
                format!("{:.2}", i.area_m2), format!("{}", i.quantity),
            ]).collect();
            y = pdf_table(&mut ops, margin, y, &h, &w, &rows);
            y -= 4.0;
        }

        if !prod.bom.is_empty() {
            ops.extend(text_ops("Stuklijst (BOM)", 12.0, margin, y, font_bold(), AMBER));
            y -= 6.0;
            let h = ["Category","Description","Unit","Qty"];
            let w = [25.0,60.0,16.0,16.0_f32];
            let rows: Vec<Vec<String>> = prod.bom.iter().map(|i| vec![
                i.category.clone(), i.description.clone(), i.unit.clone(), format!("{:.2}", i.quantity),
            ]).collect();
            pdf_table(&mut ops, margin, y, &h, &w, &rows);
        }

        doc.pages.push(PdfPage::new(Mm(A4_W), Mm(A4_H), ops));
    }

    if doc.pages.is_empty() {
        doc.pages.push(PdfPage::new(Mm(A4_W), Mm(A4_H), vec![]));
    }

    save_doc(&doc, output_path)
}

fn pdf_table(ops: &mut Vec<Op>, x0: f32, y0: f32, headers: &[&str], widths: &[f32], rows: &[Vec<String>]) -> f32 {
    let rh: f32 = 5.0;
    let tw: f32 = widths.iter().sum();

    ops.extend(fill_rect_ops(x0, y0 - rh, tw, rh, DEEP_FORGE));
    let mut x = x0;
    for (i, h) in headers.iter().enumerate() {
        ops.extend(text_ops(h, 7.0, x + 1.0, y0 - rh + 1.5, font_bold(), WHITE_C));
        x += widths[i];
    }

    let mut y = y0 - rh;
    for (ri, row) in rows.iter().enumerate() {
        if y < 15.0 { break; }
        if ri % 2 == 0 {
            ops.extend(fill_rect_ops(x0, y - rh, tw, rh, ALT_ROW));
        }
        ops.extend(line_ops(x0, y - rh, x0 + tw, y - rh, GRID_CLR, 0.3));
        let mut x = x0;
        for (i, v) in row.iter().enumerate() {
            if i < widths.len() {
                let s = if v.len() > 30 { &v[..27] } else { v.as_str() };
                ops.extend(text_ops(s, 7.0, x + 1.0, y - rh + 1.5, font(), DEEP_FORGE));
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
