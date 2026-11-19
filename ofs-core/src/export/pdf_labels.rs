//! PDF label generation for production pieces (stickers for workshop).

use printpdf::*;

use crate::kozijn::Project;
use crate::production::compute_production_data;

/// Configuration for label sheet layout
#[derive(Debug, Clone)]
pub struct LabelConfig {
    /// Label width in mm
    pub label_width_mm: f32,
    /// Label height in mm
    pub label_height_mm: f32,
    /// Number of columns on the sheet
    pub columns: usize,
    /// Number of rows on the sheet
    pub rows: usize,
    /// Margin around each label in mm
    pub margin_mm: f32,
}

impl Default for LabelConfig {
    fn default() -> Self {
        Self {
            label_width_mm: 70.0,
            label_height_mm: 37.0,
            columns: 3,
            rows: 7,
            margin_mm: 5.0,
        }
    }
}

fn col(c: (f32, f32, f32)) -> Color {
    Color::Rgb(Rgb { r: c.0, g: c.1, b: c.2, icc_profile: None })
}

fn font() -> PdfFontHandle {
    PdfFontHandle::Builtin(BuiltinFont::Helvetica)
}

fn font_bold() -> PdfFontHandle {
    PdfFontHandle::Builtin(BuiltinFont::HelveticaBold)
}

const BLACK: (f32, f32, f32) = (0.0, 0.0, 0.0);
const GRAY: (f32, f32, f32) = (0.4, 0.4, 0.4);

/// A single label's data
struct LabelData {
    project_name: String,
    kozijn_mark: String,
    piece_id: String,
    profile_desc: String,
    net_length_mm: f64,
    gross_length_mm: Option<f64>,
    precision: usize,
    date: String,
}

/// Generate a PDF with labels for all production pieces in the project.
pub fn generate_labels_pdf(project: &Project, config: &LabelConfig) -> Result<Vec<u8>, String> {
    // Collect all label data
    let mut labels: Vec<LabelData> = Vec::new();
    let date = {
        #[cfg(feature = "export")]
        {
            chrono::Local::now().format("%Y-%m-%d").to_string()
        }
        #[cfg(not(feature = "export"))]
        {
            String::from("--")
        }
    };

    for kozijn in &project.kozijnen {
        let prod = compute_production_data(kozijn);
        for (i, item) in prod.cut_list.iter().enumerate() {
            let has_arch_feet = !prod.arch_joints.is_empty();
            labels.push(LabelData {
                project_name: project.project_info.name.clone(),
                kozijn_mark: prod.kozijn_mark.clone(),
                piece_id: if has_arch_feet { item.piece_id.clone() } else { format!("{}-{}", prod.kozijn_mark, i + 1) },
                profile_desc: format!("{} {}", item.member_label_nl(), item.profile_name),
                net_length_mm: item.net_length_mm,
                gross_length_mm: prod.blank_length_mm(item),
                precision: usize::from(has_arch_feet),
                date: date.clone(),
            });
        }
    }

    if labels.is_empty() {
        return Err("Geen productiestukken gevonden".to_string());
    }

    // Page dimensions: A4 landscape for typical label sheets
    let page_w: f32 = 210.0;
    let page_h: f32 = 297.0;

    let labels_per_page = config.columns * config.rows;
    let mut doc = PdfDocument::new("Productielabels");

    // Compute starting offsets to center the grid on the page
    let grid_w = config.columns as f32 * config.label_width_mm;
    let grid_h = config.rows as f32 * config.label_height_mm;
    let start_x = (page_w - grid_w) / 2.0;
    let start_y = page_h - (page_h - grid_h) / 2.0;

    let total_pages = (labels.len() + labels_per_page - 1) / labels_per_page;

    for page_idx in 0..total_pages {
        let mut ops: Vec<Op> = Vec::new();

        for slot in 0..labels_per_page {
            let label_idx = page_idx * labels_per_page + slot;
            if label_idx >= labels.len() {
                break;
            }
            let label = &labels[label_idx];

            let col_idx = slot % config.columns;
            let row_idx = slot / config.columns;

            let lx = start_x + col_idx as f32 * config.label_width_mm + config.margin_mm;
            let ly = start_y - row_idx as f32 * config.label_height_mm - config.margin_mm;

            let _usable_w = config.label_width_mm - 2.0 * config.margin_mm;
            let line_h: f32 = 4.0;

            // Project name (small, gray)
            let mut cy = ly - 3.0;
            ops.extend(text_ops(&label.project_name, 5.0, lx, cy, font(), GRAY));

            // Kozijn mark (large, bold)
            cy -= line_h + 2.0;
            ops.extend(text_ops(&label.kozijn_mark, 10.0, lx, cy, font_bold(), BLACK));

            // Piece ID
            let mark_w = label.kozijn_mark.len() as f32 * 5.5 + 4.0;
            ops.extend(text_ops(&label.piece_id, 7.0, lx + mark_w, cy + 0.5, font(), GRAY));

            // Profile description
            cy -= line_h + 3.0;
            let desc = truncate_str(&label.profile_desc, 40);
            ops.extend(text_ops(&desc, 6.0, lx, cy, font(), BLACK));

            // Net / Gross lengths
            cy -= line_h + 1.5;
            let precision = label.precision;
            let gross = label.gross_length_mm.map(|v| format!("{v:.precision$} mm")).unwrap_or_else(|| "onbepaald".into());
            let lengths = format!("Netto: {:.*} mm  |  Bruto: {gross}", precision, label.net_length_mm);
            ops.extend(text_ops(&lengths, 5.5, lx, cy, font(), BLACK));

            // Date (bottom right)
            cy -= line_h + 1.0;
            ops.extend(text_ops(&label.date, 5.0, lx, cy, font(), GRAY));
        }

        let page = PdfPage::new(Mm(page_w), Mm(page_h), ops);
        doc.with_pages(vec![page]);
    }

    let mut warnings = Vec::new();
    let bytes = doc.save(&PdfSaveOptions::default(), &mut warnings);
    Ok(bytes)
}

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

fn truncate_str(s: &str, max_chars: usize) -> String {
    if s.len() <= max_chars {
        s.to_string()
    } else {
        format!("{}...", &s[..max_chars - 3])
    }
}
