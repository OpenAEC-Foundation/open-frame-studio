//! DXF workshop drawing generation for kozijnen.
//!
//! Writes DXF R2013 text format directly — no external crate needed.

use std::fmt::Write as FmtWrite;
use std::io::Write;

use crate::kozijn::{Kozijn, PanelType, OpeningDirection};

/// Generate a DXF workshop drawing from a kozijn definition.
pub fn generate_dxf(kozijn: &Kozijn, output_path: &str) -> Result<(), String> {
    let mut dxf = String::with_capacity(64 * 1024);

    let frame = &kozijn.frame;
    let grid = &kozijn.grid;
    let cells = &kozijn.cells;

    let ow = frame.outer_width;
    let oh = frame.outer_height;
    let fw = frame.frame_width;

    // === HEADER ===
    w_section(&mut dxf, "HEADER");
    w_var(&mut dxf, "$ACADVER", 1, "AC1027"); // R2013
    w_var(&mut dxf, "$INSUNITS", 70, "4"); // mm
    w_endsec(&mut dxf);

    // === TABLES ===
    w_section(&mut dxf, "TABLES");

    // Layer table
    w_group(&mut dxf, 0, "TABLE");
    w_group(&mut dxf, 2, "LAYER");
    w_group(&mut dxf, 70, "5");

    w_layer(&mut dxf, "FRAME", 7);      // white
    w_layer(&mut dxf, "GLASS", 5);      // blue
    w_layer(&mut dxf, "DIMENSIONS", 3); // green
    w_layer(&mut dxf, "TEXT", 7);       // white
    w_layer(&mut dxf, "SYMBOLS", 1);    // red

    w_group(&mut dxf, 0, "ENDTAB");
    w_endsec(&mut dxf);

    // === ENTITIES ===
    w_section(&mut dxf, "ENTITIES");

    // Outer frame
    draw_rect(&mut dxf, 0.0, 0.0, ow, oh, "FRAME");
    // Inner opening
    draw_rect(&mut dxf, fw, fw, ow - fw, oh - fw, "FRAME");

    // Vertical dividers
    let columns = &grid.columns;
    let mut x = fw;
    for (i, col) in columns.iter().enumerate() {
        x += col.size;
        if i < columns.len() - 1 {
            draw_rect(&mut dxf, x, fw, x + fw, oh - fw, "FRAME");
            x += fw;
        }
    }

    // Horizontal dividers
    let rows = &grid.rows;
    let mut y = fw;
    for (i, row) in rows.iter().enumerate() {
        y += row.size;
        if i < rows.len() - 1 {
            draw_rect(&mut dxf, fw, y, ow - fw, y + fw, "FRAME");
            y += fw;
        }
    }

    // Calculate cell positions
    let mut col_positions = Vec::with_capacity(columns.len());
    let mut cx = fw;
    for (i, col) in columns.iter().enumerate() {
        col_positions.push(cx);
        cx += col.size;
        if i < columns.len() - 1 {
            cx += fw;
        }
    }

    let mut row_positions = Vec::with_capacity(rows.len());
    let mut ry = fw;
    for (i, row) in rows.iter().enumerate() {
        row_positions.push(ry);
        ry += row.size;
        if i < rows.len() - 1 {
            ry += fw;
        }
    }

    // Draw cells
    let num_cols = columns.len();
    for (row_idx, row) in rows.iter().enumerate() {
        for (col_idx, col) in columns.iter().enumerate() {
            let cell_idx = row_idx * num_cols + col_idx;
            if cell_idx >= cells.len() {
                continue;
            }
            let cell = &cells[cell_idx];
            let x1 = col_positions[col_idx];
            let y1 = row_positions[row_idx];
            let x2 = x1 + col.size;
            let y2 = y1 + row.size;

            // Glass diagonals
            let pt = cell.panel_type;
            if matches!(
                pt,
                PanelType::FixedGlass
                    | PanelType::TurnTilt
                    | PanelType::Turn
                    | PanelType::Tilt
                    | PanelType::Sliding
            ) {
                draw_line(&mut dxf, x1, y1, x2, y2, "GLASS");
                draw_line(&mut dxf, x1, y2, x2, y1, "GLASS");
            }

            // Opening symbols
            draw_opening_symbol(&mut dxf, cell, x1, y1, x2, y2);

            // Cell label
            let label = panel_label(pt);
            let mid_x = (x1 + x2) / 2.0;
            let mid_y = (y1 + y2) / 2.0;
            let text_h = ((x2 - x1).min(y2 - y1)) * 0.12;
            draw_text(&mut dxf, label, mid_x, mid_y, text_h, "TEXT", true);
        }
    }

    // Overall dimensions
    let dim_offset = 80.0;

    // Width dimension (bottom)
    draw_dim_linear(
        &mut dxf,
        0.0,
        0.0,
        ow,
        0.0,
        ow / 2.0,
        -dim_offset,
        false,
    );

    // Height dimension (right)
    draw_dim_linear(
        &mut dxf,
        ow,
        0.0,
        ow,
        oh,
        ow + dim_offset,
        oh / 2.0,
        true,
    );

    // Title block
    let title = format!("{} - {}", kozijn.mark, kozijn.name);
    draw_text(&mut dxf, &title, 0.0, -dim_offset * 3.0, 40.0, "TEXT", false);
    let dims = format!("{} x {} mm", ow as i64, oh as i64);
    draw_text(
        &mut dxf,
        &dims,
        0.0,
        -dim_offset * 3.0 - 50.0,
        25.0,
        "TEXT",
        false,
    );

    w_endsec(&mut dxf);

    // === EOF ===
    w_group(&mut dxf, 0, "EOF");

    // Write file
    let mut file = std::fs::File::create(output_path)
        .map_err(|e| format!("Kan DXF bestand niet aanmaken: {}", e))?;
    file.write_all(dxf.as_bytes())
        .map_err(|e| format!("Kan DXF niet schrijven: {}", e))?;

    Ok(())
}

// ── DXF writing helpers ───────────────────────────────────────────

fn w_group(dxf: &mut String, code: i32, value: &str) {
    let _ = writeln!(dxf, "{:>3}", code);
    let _ = writeln!(dxf, "{}", value);
}

fn w_group_f(dxf: &mut String, code: i32, value: f64) {
    let _ = writeln!(dxf, "{:>3}", code);
    let _ = writeln!(dxf, "{:.6}", value);
}

fn w_section(dxf: &mut String, name: &str) {
    w_group(dxf, 0, "SECTION");
    w_group(dxf, 2, name);
}

fn w_endsec(dxf: &mut String) {
    w_group(dxf, 0, "ENDSEC");
}

fn w_var(dxf: &mut String, name: &str, code: i32, value: &str) {
    w_group(dxf, 9, name);
    w_group(dxf, code, value);
}

fn w_layer(dxf: &mut String, name: &str, color: i32) {
    w_group(dxf, 0, "LAYER");
    w_group(dxf, 2, name);
    w_group(dxf, 70, "0");
    w_group(dxf, 62, &color.to_string());
    w_group(dxf, 6, "CONTINUOUS");
}

fn draw_line(dxf: &mut String, x1: f64, y1: f64, x2: f64, y2: f64, layer: &str) {
    w_group(dxf, 0, "LINE");
    w_group(dxf, 8, layer);
    w_group_f(dxf, 10, x1);
    w_group_f(dxf, 20, y1);
    w_group_f(dxf, 30, 0.0);
    w_group_f(dxf, 11, x2);
    w_group_f(dxf, 21, y2);
    w_group_f(dxf, 31, 0.0);
}

fn draw_rect(dxf: &mut String, x1: f64, y1: f64, x2: f64, y2: f64, layer: &str) {
    draw_line(dxf, x1, y1, x2, y1, layer);
    draw_line(dxf, x2, y1, x2, y2, layer);
    draw_line(dxf, x2, y2, x1, y2, layer);
    draw_line(dxf, x1, y2, x1, y1, layer);
}

fn draw_text(
    dxf: &mut String,
    text: &str,
    x: f64,
    y: f64,
    height: f64,
    layer: &str,
    center: bool,
) {
    w_group(dxf, 0, "TEXT");
    w_group(dxf, 8, layer);
    w_group_f(dxf, 10, x);
    w_group_f(dxf, 20, y);
    w_group_f(dxf, 30, 0.0);
    w_group_f(dxf, 40, height);
    w_group(dxf, 1, text);
    if center {
        // Middle-center alignment
        w_group(dxf, 72, "1"); // horizontal: center
        w_group(dxf, 73, "2"); // vertical: middle
        w_group_f(dxf, 11, x);
        w_group_f(dxf, 21, y);
        w_group_f(dxf, 31, 0.0);
    }
}

fn draw_dim_linear(
    dxf: &mut String,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    dx: f64,
    dy: f64,
    vertical: bool,
) {
    w_group(dxf, 0, "DIMENSION");
    w_group(dxf, 8, "DIMENSIONS");
    // Dimension line location
    w_group_f(dxf, 10, dx);
    w_group_f(dxf, 20, dy);
    w_group_f(dxf, 30, 0.0);
    // First extension line origin
    w_group_f(dxf, 13, x1);
    w_group_f(dxf, 23, y1);
    w_group_f(dxf, 33, 0.0);
    // Second extension line origin
    w_group_f(dxf, 14, x2);
    w_group_f(dxf, 24, y2);
    w_group_f(dxf, 34, 0.0);
    // Type: aligned (0) or rotated
    if vertical {
        w_group(dxf, 70, "0"); // linear dimension
        w_group_f(dxf, 50, 90.0); // rotation angle
    } else {
        w_group(dxf, 70, "0");
        w_group_f(dxf, 50, 0.0);
    }
}

fn draw_opening_symbol(
    dxf: &mut String,
    cell: &crate::kozijn::Cell,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
) {
    let pt = cell.panel_type;
    let dir = cell.opening_direction;

    match pt {
        PanelType::TurnTilt | PanelType::Turn => {
            let mid_y = (y1 + y2) / 2.0;
            let is_left = dir.is_none()
                || matches!(dir, Some(OpeningDirection::Left));
            if is_left {
                draw_line(dxf, x1, y1, x2, mid_y, "SYMBOLS");
                draw_line(dxf, x1, y2, x2, mid_y, "SYMBOLS");
            } else {
                draw_line(dxf, x2, y1, x1, mid_y, "SYMBOLS");
                draw_line(dxf, x2, y2, x1, mid_y, "SYMBOLS");
            }
        }
        PanelType::Tilt => {
            let mid_x = (x1 + x2) / 2.0;
            draw_line(dxf, x1, y2, mid_x, y1, "SYMBOLS");
            draw_line(dxf, x2, y2, mid_x, y1, "SYMBOLS");
        }
        PanelType::Door => {
            let is_left = dir.is_none()
                || matches!(
                    dir,
                    Some(OpeningDirection::Left) | Some(OpeningDirection::Inward)
                );
            if is_left {
                draw_line(dxf, x1, y1, x2, y2, "SYMBOLS");
            } else {
                draw_line(dxf, x2, y1, x1, y2, "SYMBOLS");
            }
        }
        _ => {}
    }
}

fn panel_label(pt: PanelType) -> &'static str {
    match pt {
        PanelType::FixedGlass => "VG",
        PanelType::TurnTilt => "DK",
        PanelType::Turn => "D",
        PanelType::Tilt => "K",
        PanelType::Sliding => "S",
        PanelType::Door => "DR",
        PanelType::Panel => "P",
        PanelType::Ventilation => "V",
    }
}
