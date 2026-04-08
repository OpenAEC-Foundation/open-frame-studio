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
    /// Trapezoid outer polygon points [[x,y], ...] (for trapezoid frame shapes)
    #[serde(default)]
    pub trapezoid_outer: Vec<[f64; 2]>,
    /// Trapezoid inner polygon points [[x,y], ...] (for trapezoid frame shapes)
    #[serde(default)]
    pub trapezoid_inner: Vec<[f64; 2]>,
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
    // Divider width defaults to frame width (same profile family)
    let divider_width = fw;

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

    // Frame members — adjust for arched/special shapes
    let is_arched = kozijn.frame.shape.shape_type == ShapeType::Arched
        || kozijn.frame.shape.shape_type == ShapeType::Round;
    let top_rect_height = if is_arched { 0.0 } else { fw };

    let frame_rects = vec![
        // Top (hidden for arched — arc replaces it)
        Rect2D { x: 0.0, y: 0.0, width: ow, height: top_rect_height },
        // Bottom (sill)
        Rect2D { x: 0.0, y: oh - fw, width: ow, height: fw },
        // Left (full height for arched, starts at 0)
        Rect2D { x: 0.0, y: top_rect_height, width: fw, height: oh - top_rect_height - fw },
        // Right
        Rect2D { x: ow - fw, y: top_rect_height, width: fw, height: oh - top_rect_height - fw },
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

    // ── Dimension lines (GA Kozijn style) ──
    // Level 1 (nearest): profielmaat stijl + vakmaten + profielmaat stijl
    // Level 2 (furthest): buitenmaat totaal
    // Right side: same pattern for height
    let dim_offset = 20.0;
    let mut dimensions = Vec::new();
    let inner_w = ow - 2.0 * fw;
    let inner_h = oh - 2.0 * fw;
    let num_cols = kozijn.grid.columns.len();
    let num_rows = kozijn.grid.rows.len();

    // ── Bottom: Level 1 — profielmaat links + vakmaten + profielmaat rechts ──
    // Left stijl width
    dimensions.push(DimensionLine {
        x1: 0.0,
        y1: oh + dim_offset,
        x2: fw,
        y2: oh + dim_offset,
        label: format!("{:.0}", fw),
        side: DimensionSide::Bottom,
    });

    // Column widths (vakmaten)
    for (i, col) in kozijn.grid.columns.iter().enumerate() {
        let cx = col_positions[i];
        dimensions.push(DimensionLine {
            x1: cx,
            y1: oh + dim_offset,
            x2: cx + col.size,
            y2: oh + dim_offset,
            label: format!("{:.0}", col.size),
            side: DimensionSide::Bottom,
        });
    }

    // Right stijl width
    let last_col_end = col_positions.last()
        .map(|p| *p + kozijn.grid.columns.last().map(|c| c.size).unwrap_or(0.0))
        .unwrap_or(fw + inner_w);
    dimensions.push(DimensionLine {
        x1: last_col_end,
        y1: oh + dim_offset,
        x2: ow,
        y2: oh + dim_offset,
        label: format!("{:.0}", fw),
        side: DimensionSide::Bottom,
    });

    // ── Bottom: Level 2 — buitenmaat totaal ──
    dimensions.push(DimensionLine {
        x1: 0.0,
        y1: oh + dim_offset * 2.5,
        x2: ow,
        y2: oh + dim_offset * 2.5,
        label: format!("{:.0}", ow),
        side: DimensionSide::Bottom,
    });

    // ── Right: Level 1 — profielmaat boven + vakmaten + profielmaat onder ──
    // Top dorpel height
    dimensions.push(DimensionLine {
        x1: ow + dim_offset,
        y1: 0.0,
        x2: ow + dim_offset,
        y2: fw,
        label: format!("{:.0}", fw),
        side: DimensionSide::Right,
    });

    // Row heights (vakmaten)
    for (i, row) in kozijn.grid.rows.iter().enumerate() {
        let cy = row_positions[i];
        dimensions.push(DimensionLine {
            x1: ow + dim_offset,
            y1: cy,
            x2: ow + dim_offset,
            y2: cy + row.size,
            label: format!("{:.0}", row.size),
            side: DimensionSide::Right,
        });
    }

    // Bottom dorpel height
    let last_row_end = row_positions.last()
        .map(|p| *p + kozijn.grid.rows.last().map(|r| r.size).unwrap_or(0.0))
        .unwrap_or(fw + inner_h);
    dimensions.push(DimensionLine {
        x1: ow + dim_offset,
        y1: last_row_end,
        x2: ow + dim_offset,
        y2: oh,
        label: format!("{:.0}", fw),
        side: DimensionSide::Right,
    });

    // ── Right: Level 2 — buitenmaat totaal ──
    dimensions.push(DimensionLine {
        x1: ow + dim_offset * 2.5,
        y1: 0.0,
        x2: ow + dim_offset * 2.5,
        y2: oh,
        label: format!("{:.0}", oh),
        side: DimensionSide::Right,
    });

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
    } else if kozijn.frame.shape.shape_type == ShapeType::Trapezoid {
        // Trapezoid only — no arc, just angled stiles
    } else if kozijn.frame.shape.shape_type == ShapeType::ArchedTrapezoid {
        // Combined: arched top + angled stiles (CNCware-style)
        let arch_height = kozijn.frame.shape.arch_height.unwrap_or(ow / 4.0);
        let half_w = ow / 2.0;
        let radius = (half_w * half_w) / (2.0 * arch_height) + arch_height / 2.0;
        let center_y = oh - arch_height + radius;

        // Outer arc (arch above the rectangular part)
        let start_angle = ((half_w / radius).asin()).to_degrees();
        arcs.push(Arc2D {
            cx: half_w,
            cy: center_y,
            radius,
            start_angle: 180.0 - start_angle,
            end_angle: start_angle,
            stroke_width: fw,
        });

        // Inner arc
        let inner_radius = radius - fw;
        if inner_radius > 0.0 {
            arcs.push(Arc2D {
                cx: half_w,
                cy: center_y,
                radius: inner_radius,
                start_angle: 180.0 - start_angle,
                end_angle: start_angle,
                stroke_width: 1.0,
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

    // Trapezoid polygon computation
    let mut trapezoid_outer = Vec::new();
    let mut trapezoid_inner = Vec::new();
    if kozijn.frame.shape.shape_type == ShapeType::Trapezoid
        || kozijn.frame.shape.shape_type == ShapeType::ArchedTrapezoid
    {
        let top_w = kozijn.frame.shape.top_width.unwrap_or(ow * 0.6);
        let left_angle_deg = kozijn.frame.shape.left_angle.unwrap_or(90.0);
        let right_angle_deg = kozijn.frame.shape.right_angle.unwrap_or(90.0);

        // Offset from bottom edge to top edge based on angle
        // At 90°: offset = 0 (vertical). At <90°: offset > 0 (leaning inward)
        let left_offset = if left_angle_deg >= 89.9 { 0.0 } else {
            oh * (90.0 - left_angle_deg).to_radians().tan()
        };
        let right_offset = if right_angle_deg >= 89.9 { 0.0 } else {
            oh * (90.0 - right_angle_deg).to_radians().tan()
        };

        // Outer polygon (clockwise from bottom-left)
        trapezoid_outer = vec![
            [0.0, oh],                              // bottom-left
            [ow, oh],                               // bottom-right
            [ow - right_offset, 0.0],               // top-right
            [left_offset, 0.0],                     // top-left
        ];

        // Inner polygon (inside frame members)
        trapezoid_inner = vec![
            [fw, oh - fw],                          // bottom-left inner
            [ow - fw, oh - fw],                     // bottom-right inner
            [ow - right_offset - fw + (fw * right_offset / oh), fw],  // top-right inner
            [left_offset + fw - (fw * left_offset / oh), fw],         // top-left inner
        ];
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
        trapezoid_outer,
        trapezoid_inner,
    }
}
