use crate::kozijn::{Kozijn, ShapeType};
use serde::{Deserialize, Serialize};

/// 2D rectangle for SVG rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rect2D {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// 2D arc for SVG rendering (arched/round kozijnen)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Arc2D {
    /// Center X
    pub cx: f64,
    /// Center Y
    pub cy: f64,
    /// Radius
    pub radius: f64,
    /// Start angle in degrees (0 = right, 90 = top)
    pub start_angle: f64,
    /// End angle in degrees
    pub end_angle: f64,
    /// Stroke width (frame width)
    pub stroke_width: f64,
}

/// Complete 2D geometry for rendering a kozijn as SVG
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KozijnGeometry2D {
    /// Overall bounding box (outer frame)
    pub outer_rect: Rect2D,
    /// Inner opening (inside the frame)
    pub inner_rect: Rect2D,
    /// Frame members (top, bottom/sill, left, right)
    pub frame_rects: Vec<Rect2D>,
    /// Horizontal dividers
    pub h_dividers: Vec<Rect2D>,
    /// Vertical dividers
    pub v_dividers: Vec<Rect2D>,
    /// Cell rectangles (the glazing/panel areas)
    pub cell_rects: Vec<CellRect>,
    /// Dimension lines
    pub dimensions: Vec<DimensionLine>,
    /// Arcs (for arched/round frame shapes)
    #[serde(default)]
    pub arcs: Vec<Arc2D>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellRect {
    pub rect: Rect2D,
    pub col: usize,
    pub row: usize,
    pub cell_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DimensionLine {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub label: String,
    pub side: DimensionSide,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DimensionSide {
    Top,
    Bottom,
    Left,
    Right,
}

/// Compute 2D geometry from a kozijn model
pub fn compute_2d_geometry(kozijn: &Kozijn) -> KozijnGeometry2D {
    let fw = kozijn.frame.frame_width;
    let ow = kozijn.frame.outer_width;
    let oh = kozijn.frame.outer_height;
    let divider_width = fw; // dividers use same width as frame

    // Outer rect
    let outer_rect = Rect2D {
        x: 0.0,
        y: 0.0,
        width: ow,
        height: oh,
    };

    // Inner rect
    let inner_rect = Rect2D {
        x: fw,
        y: fw,
        width: ow - 2.0 * fw,
        height: oh - 2.0 * fw,
    };

    // Frame members
    let frame_rects = vec![
        // Top
        Rect2D { x: 0.0, y: 0.0, width: ow, height: fw },
        // Bottom (sill)
        Rect2D { x: 0.0, y: oh - fw, width: ow, height: fw },
        // Left
        Rect2D { x: 0.0, y: fw, width: fw, height: oh - 2.0 * fw },
        // Right
        Rect2D { x: ow - fw, y: fw, width: fw, height: oh - 2.0 * fw },
    ];

    // Calculate column positions (x coordinates of cell starts)
    let mut col_positions = Vec::new();
    let mut x = fw;
    for (i, col) in kozijn.grid.columns.iter().enumerate() {
        col_positions.push(x);
        x += col.size;
        if i < kozijn.grid.columns.len() - 1 {
            x += divider_width; // space for divider
        }
    }

    // Calculate row positions (y coordinates of cell starts)
    let mut row_positions = Vec::new();
    let mut y = fw;
    for (i, row) in kozijn.grid.rows.iter().enumerate() {
        row_positions.push(y);
        y += row.size;
        if i < kozijn.grid.rows.len() - 1 {
            y += divider_width;
        }
    }

    // Vertical dividers
    let mut v_dividers = Vec::new();
    let mut vx = fw;
    for i in 0..kozijn.grid.columns.len() {
        vx += kozijn.grid.columns[i].size;
        if i < kozijn.grid.columns.len() - 1 {
            v_dividers.push(Rect2D {
                x: vx,
                y: fw,
                width: divider_width,
                height: oh - 2.0 * fw,
            });
            vx += divider_width;
        }
    }

    // Horizontal dividers
    let mut h_dividers = Vec::new();
    let mut hy = fw;
    for i in 0..kozijn.grid.rows.len() {
        hy += kozijn.grid.rows[i].size;
        if i < kozijn.grid.rows.len() - 1 {
            h_dividers.push(Rect2D {
                x: fw,
                y: hy,
                width: ow - 2.0 * fw,
                height: divider_width,
            });
            hy += divider_width;
        }
    }

    // Cell rects
    let num_cols = kozijn.grid.columns.len();
    let mut cell_rects = Vec::new();
    for (row_idx, row) in kozijn.grid.rows.iter().enumerate() {
        for (col_idx, col) in kozijn.grid.columns.iter().enumerate() {
            let cx = col_positions[col_idx];
            let cy = row_positions[row_idx];
            cell_rects.push(CellRect {
                rect: Rect2D {
                    x: cx,
                    y: cy,
                    width: col.size,
                    height: row.size,
                },
                col: col_idx,
                row: row_idx,
                cell_index: row_idx * num_cols + col_idx,
            });
        }
    }

    // Dimension lines
    let dim_offset = 30.0;
    let mut dimensions = Vec::new();

    // Overall width (bottom)
    dimensions.push(DimensionLine {
        x1: 0.0,
        y1: oh + dim_offset,
        x2: ow,
        y2: oh + dim_offset,
        label: format!("{:.0}", ow),
        side: DimensionSide::Bottom,
    });

    // Overall height (right)
    dimensions.push(DimensionLine {
        x1: ow + dim_offset,
        y1: 0.0,
        x2: ow + dim_offset,
        y2: oh,
        label: format!("{:.0}", oh),
        side: DimensionSide::Right,
    });

    // Column widths
    for (i, col) in kozijn.grid.columns.iter().enumerate() {
        let cx = col_positions[i];
        dimensions.push(DimensionLine {
            x1: cx,
            y1: oh + dim_offset * 2.0,
            x2: cx + col.size,
            y2: oh + dim_offset * 2.0,
            label: format!("{:.0}", col.size),
            side: DimensionSide::Bottom,
        });
    }

    // Row heights
    for (i, row) in kozijn.grid.rows.iter().enumerate() {
        let cy = row_positions[i];
        dimensions.push(DimensionLine {
            x1: ow + dim_offset * 2.0,
            y1: cy,
            x2: ow + dim_offset * 2.0,
            y2: cy + row.size,
            label: format!("{:.0}", row.size),
            side: DimensionSide::Right,
        });
    }

    // Arched frame geometry
    let mut arcs = Vec::new();
    if kozijn.frame.shape.shape_type == ShapeType::Arched {
        let arch_height = kozijn.frame.shape.arch_height.unwrap_or(ow / 4.0);
        // Segmental arch: center is below the arch line
        // For a segmental arch of width W and rise H:
        // radius = (W/2)^2 / (2*H) + H/2
        let half_w = ow / 2.0;
        let radius = (half_w * half_w) / (2.0 * arch_height) + arch_height / 2.0;
        let center_y = oh - arch_height + radius; // center below the peak

        // Outer arc
        let start_angle = ((half_w / radius).asin()).to_degrees();
        arcs.push(Arc2D {
            cx: half_w,
            cy: center_y,
            radius,
            start_angle: 180.0 - start_angle,
            end_angle: start_angle,
            stroke_width: fw,
        });

        // Inner arc (smaller radius)
        let inner_radius = radius - fw;
        if inner_radius > 0.0 {
            arcs.push(Arc2D {
                cx: half_w,
                cy: center_y,
                radius: inner_radius,
                start_angle: 180.0 - start_angle,
                end_angle: start_angle,
                stroke_width: 1.0, // thin line for inner edge
            });
        }
    } else if kozijn.frame.shape.shape_type == ShapeType::Round {
        let radius = ow.min(oh) / 2.0;
        arcs.push(Arc2D {
            cx: ow / 2.0,
            cy: oh / 2.0,
            radius,
            start_angle: 0.0,
            end_angle: 360.0,
            stroke_width: fw,
        });
    }

    KozijnGeometry2D {
        outer_rect,
        inner_rect,
        frame_rects,
        h_dividers,
        v_dividers,
        cell_rects,
        dimensions,
        arcs,
    }
}
