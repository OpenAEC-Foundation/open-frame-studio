//! Workshop dimensions in millimetres. Annotation offsets and type sizes stay fixed on paper.
use super::*;
use crate::geometry::{DimensionSide, KozijnGeometry2D};
use crate::kozijn::ShapeType;

pub(super) fn mm(value: f64) -> String {
    let s = format!("{value:.2}");
    s.trim_end_matches('0')
        .trim_end_matches('.')
        .replace('.', ",")
}

fn centred(ops: &mut Vec<Op>, value: &str, x: f32, y: f32, size: f32, bold: bool) {
    let half_width = value.chars().count() as f32 * size * 0.095;
    ops.extend(text_ops(
        value,
        size,
        x - half_width,
        y,
        if bold { font_bold() } else { font() },
        DEEP_FORGE,
    ));
}

/// Projected witness marks. Separate parallel dimension chains share these lines.
fn witness(ops: &mut Vec<Op>, x: f32, y: f32, end_x: f32, end_y: f32) {
    ops.extend(line_ops(x, y, end_x, end_y, GRID_CLR, 0.28));
}

fn tick(ops: &mut Vec<Op>, x: f32, y: f32) {
    ops.extend(line_ops(
        x - 0.8,
        y - 1.15,
        x + 0.8,
        y + 1.15,
        DEEP_FORGE,
        0.55,
    ));
}

pub(super) fn draw(
    ops: &mut Vec<Op>,
    k: &Kozijn,
    g: &KozijnGeometry2D,
    origin: [f32; 2],
    scale: f32,
) -> Vec<String> {
    let px = |x: f64| origin[0] + x as f32 * scale;
    let py = |y: f64| origin[1] - y as f32 * scale;
    let outline = g.curved_frame.as_ref().map(|f| f.outer.bounds());
    let outer = outline.as_ref().unwrap_or(&g.outer_rect);
    let (left, right) = (px(outer.x), px(outer.x + outer.width));
    let (top, bottom) = (py(outer.y), py(outer.y + outer.height));
    let rect = k.frame.shape.shape_type == ShapeType::Rectangular;
    let physical_openings = rect || outline.is_some();
    let mut extra = Vec::new();

    ops.extend(text_ops(
        "Maatvoering",
        12.,
        15.,
        A3_H - 15.,
        font_bold(),
        DEEP_FORGE,
    ));
    let note = if rect {
        "Vrije kozijnopening per vak (b x h). Alle maten in mm."
    } else if outline.is_some() {
        "Maximale vakbreedte en -hoogte binnen de dagcontour (mm)."
    } else {
        "Buitenwerkse maten in mm."
    };
    ops.extend(text_ops(note, 8., 15., A3_H - 20., font(), DEEP_FORGE));
    if let Some(curve) = &g.curved_frame {
        if k.frame.shape.shape_type == ShapeType::Round {
            ops.extend(text_ops(
                &format!("Buitendiameter {} mm", mm(outer.width)),
                9.,
                245.,
                A3_H - 16.,
                font_bold(),
                DEEP_FORGE,
            ));
        } else if let Some((radius, spring)) = curve.outer.segments.iter().find_map(|s| match s {
            crate::curved_contour::Segment::Arc { radius, .. } => Some((*radius, s.from()[1])),
            _ => None,
        }) {
            let summary = format!(
                "Buitenboog  R {} mm   |   pijl {} mm",
                mm(radius),
                mm(spring - outer.y)
            );
            ops.extend(text_ops(
                &summary,
                9.,
                245.,
                A3_H - 16.,
                font_bold(),
                DEEP_FORGE,
            ));
        }
    }

    for cell in &g.cell_rects {
        let Some(kind) = workshop_infill(cell, k) else {
            continue;
        };
        let r = cell.opening_bounds.as_ref().unwrap_or(&cell.rect);
        if r.width <= 0. || r.height <= 0. {
            continue;
        }
        let id = format!("V{}  {}", cell.cell_index + 1, kind);
        let size = format!("{} x {}", mm(r.width), mm(r.height));
        let x = px(r.x + r.width / 2.);
        let y = py(r.y + r.height / 2.);
        let fits = r.width as f32 * scale >= 4. + size.len().max(id.len()) as f32 * 1.65
            && r.height as f32 * scale >= 11.;
        if physical_openings && fits {
            centred(ops, &id, x, y + 0.75, 8., true);
            centred(ops, &size, x, y - 3.6, 8., false);
        } else {
            if r.width as f32 * scale >= 3. + id.len() as f32 * 1.65
                && r.height as f32 * scale >= 5.
            {
                centred(ops, &id, x, y - 1., 8., true);
            }
            if physical_openings {
                extra.push(format!(
                    "{id}   {size} mm   |   links {} mm, boven {} mm vanaf de buitencontour",
                    mm(r.x - outer.x),
                    mm(r.y - outer.y)
                ));
            }
        }
    }

    // The outer chain spans the actual finished envelope, including an offset circle.
    let yh = bottom - 16.;
    let xv = right + 18.;
    ops.extend(line_ops(left, yh, right, yh, DEEP_FORGE, 0.45));
    ops.extend(line_ops(xv, bottom, xv, top, DEEP_FORGE, 0.45));
    for x in [left, right] {
        witness(ops, x, bottom - 1., x, yh - 2.);
        tick(ops, x, yh);
    }
    for y in [bottom, top] {
        witness(ops, right + 1., y, xv + 2., y);
        tick(ops, xv, y);
    }
    centred(
        ops,
        &mm(outer.width),
        (left + right) / 2.,
        yh + 1.1,
        8.,
        true,
    );
    ops.extend(text_ops(
        &mm(outer.height),
        8.,
        xv + 2.,
        (bottom + top) / 2. - 1.,
        font_bold(),
        DEEP_FORGE,
    ));

    if rect {
        // The first chain for each direction is the same edge-specific clear/stock
        // chain as the editor. The two outer chains in that model use different offsets.
        for horizontal in [true, false] {
            let mut chain = g.dimensions.iter().filter(|d| {
                matches!(
                    (horizontal, d.side),
                    (true, DimensionSide::Bottom) | (false, DimensionSide::Right)
                )
            });
            let Some(first) = chain.next() else { continue };
            let level = if horizontal { first.y1 } else { first.x1 };
            let chain: Vec<_> = std::iter::once(first)
                .chain(chain)
                .take_while(|d| ((if horizontal { d.y1 } else { d.x1 }) - level).abs() < 1e-6)
                .map(|d| {
                    if horizontal {
                        (d.x1, d.x2)
                    } else {
                        (d.y1, d.y2)
                    }
                })
                .collect();
            // In unusually dense frames use the clear sizes written inside the cells or on the supplement.
            if !chain.iter().all(|(a, b)| {
                (b - a) as f32 * scale
                    >= if horizontal {
                        1.5 + mm(b - a).len() as f32 * 1.4
                    } else {
                        4.
                    }
            }) {
                continue;
            }
            for (a, b) in chain {
                let label = mm(b - a);
                if horizontal {
                    let y = bottom - 7.;
                    witness(ops, px(a), bottom - 1., px(a), y - 1.5);
                    ops.extend(line_ops(px(a), y, px(b), y, DEEP_FORGE, 0.32));
                    tick(ops, px(a), y);
                    tick(ops, px(b), y);
                    centred(ops, &label, px((a + b) / 2.), y + 1.1, 7., false);
                } else {
                    let x = right + 6.;
                    witness(ops, right + 1., py(a), x + 1.5, py(a));
                    ops.extend(line_ops(x, py(a), x, py(b), DEEP_FORGE, 0.32));
                    tick(ops, x, py(a));
                    tick(ops, x, py(b));
                    ops.extend(text_ops(
                        &label,
                        7.,
                        x + 1.5,
                        py((a + b) / 2.) - 0.7,
                        font(),
                        DEEP_FORGE,
                    ));
                }
            }
        }
    }
    if physical_openings {
        for horizontal in [true, false] {
            let members = if horizontal {
                &g.v_dividers
            } else {
                &g.h_dividers
            };
            let mut centres: Vec<f64> = members
                .iter()
                .enumerate()
                .filter(|(index, r)| {
                    r.width > 0.
                        && r.height > 0.
                        && (g.curved_frame.is_none()
                            || g.curved_divider_timber
                                .iter()
                                .any(|p| p.vertical == horizontal && p.source_index == *index))
                })
                .map(|(_, r)| {
                    if horizontal {
                        r.x + r.width / 2. - outer.x
                    } else {
                        outer.y + outer.height - r.y - r.height / 2.
                    }
                })
                .collect();
            centres.sort_by(f64::total_cmp);
            centres.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
            if centres.is_empty() {
                continue;
            }
            let legend = if horizontal {
                "Hart X vanaf links"
            } else {
                "Hart Y vanaf onder"
            };
            // The coordinates are ordinates from a common physical datum. Never merge
            // close but distinct members just because their labels would overlap on paper.
            if std::iter::once(&0.)
                .chain(centres.iter())
                .zip(centres.iter())
                .any(|(a, b)| {
                    (b - a) as f32 * scale
                        < if horizontal {
                            3. + mm(*b).len() as f32 * 1.7
                        } else {
                            5.
                        }
                })
            {
                extra.extend(centres.iter().map(|v| format!("{legend}   {} mm", mm(*v))));
                continue;
            }
            let last = *centres.last().unwrap();
            if horizontal {
                let y = top + 8.;
                ops.extend(line_ops(left, y, px(outer.x + last), y, DEEP_FORGE, 0.38));
                ops.extend(text_ops(legend, 7., left, y + 5.4, font(), DEEP_FORGE));
                for v in std::iter::once(0.).chain(centres) {
                    let x = px(outer.x + v);
                    witness(ops, x, top + 1., x, y + 1.5);
                    tick(ops, x, y);
                    centred(ops, &mm(v), x, y + 1.3, 7., false);
                }
            } else {
                let x = left - 11.;
                ops.extend(line_ops(
                    x,
                    bottom,
                    x,
                    bottom + last as f32 * scale,
                    DEEP_FORGE,
                    0.38,
                ));
                ops.extend(text_ops(
                    legend,
                    7.,
                    x - 16.,
                    bottom - 5.5,
                    font(),
                    DEEP_FORGE,
                ));
                for v in std::iter::once(0.).chain(centres) {
                    let y = bottom + v as f32 * scale;
                    witness(ops, x - 1.5, y, left - 1., y);
                    tick(ops, x, y);
                    ops.extend(text_ops(
                        &mm(v),
                        7.,
                        x - 2. - mm(v).len() as f32 * 1.4,
                        y + 0.5,
                        font(),
                        DEEP_FORGE,
                    ));
                }
            }
        }
    }
    if !extra.is_empty() {
        ops.extend(text_ops(
            "Aanvullende vak- en hartmaten op het vervolgblad.",
            8.,
            15.,
            A3_H - 25.,
            font(),
            DEEP_FORGE,
        ));
    }
    extra
}

pub(super) fn append_overflow(doc: &mut PdfDocument, mark: &str, labels: &[String]) {
    for chunk in labels.chunks(37) {
        let mut ops = text_ops(
            &format!("Maatstaat {mark}"),
            14.,
            15.,
            A3_H - 20.,
            font_bold(),
            DEEP_FORGE,
        );
        for (i, label) in chunk.iter().enumerate() {
            ops.extend(text_ops(
                label,
                9.,
                15.,
                A3_H - 32. - i as f32 * 6.5,
                font(),
                DEEP_FORGE,
            ));
        }
        doc.pages.push(PdfPage::new(Mm(A3_W), Mm(A3_H), ops));
    }
}
