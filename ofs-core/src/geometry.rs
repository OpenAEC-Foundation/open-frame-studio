use crate::kozijn::{GridDivision, Kozijn, ShapeType};
use crate::layout::{compute_layout_geometry, LayoutGeometry, LayoutRect, Vakvulling};
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
    /// Arched top band as a closed polygon [[x,y], ...]: outer arc (left→right)
    /// followed by inner arc (right→left), ready for a filled SVG path
    /// (for arched frame shapes; empty otherwise)
    #[serde(default)]
    pub arch_band: Vec<[f64; 2]>,
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
    /// Vakvulling carried along on the free-subdivision layout path, so
    /// consumers no longer need to index `kozijn.cells[cell_index]` (which is
    /// grid-shaped and does not match a layout tree). `None` on the grid path;
    /// omitted from the payload when absent (old payloads stay byte-equal).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vulling: Option<Vakvulling>,
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

    // Free-subdivision layout (kozijn.layout) — when present it supersedes the
    // rectangular grid as the source of dividers, cells and the level-1
    // dimension chains. The tree is laid out in the plain rectangular
    // binnenrect with divider width = frame width, exactly mirroring the
    // frontend canvas (layoutToRects in ui/src/lib/layout.js), so backend
    // geometry and the canvas overlay coincide. Documented follow-ups, out of
    // scope here: a stepped frame outline around `Buiten` leaves (the outer
    // frame stays the full rectangle for now) and arch-aware (spring-line)
    // placement of the tree for arched/round shapes.
    let layout_geom = kozijn.layout.as_ref().map(|node| {
        compute_layout_geometry(
            node,
            LayoutRect { x: fw, y: fw, width: ow - 2.0 * fw, height: oh - 2.0 * fw },
            divider_width,
        )
    });

    // Frame members — adjust for arched/special shapes
    let is_arched = kozijn.frame.shape.shape_type == ShapeType::Arched
        || kozijn.frame.shape.shape_type == ShapeType::ArchedTrapezoid;
    let is_round = kozijn.frame.shape.shape_type == ShapeType::Round;
    let is_elliptical = kozijn.frame.shape.shape_type == ShapeType::Elliptical;
    let top_rect_height = if is_arched || is_round || is_elliptical { 0.0 } else { fw };

    // For arched frames, stiles start at the arch spring line (not y=0)
    let stile_top_y = if is_arched {
        let arch_h = kozijn.frame.shape.arch_height.unwrap_or(ow / 4.0);
        arch_h // stiles begin where the arch springs from
    } else {
        top_rect_height
    };

    // For round/elliptical frames, all rectangular members are hidden (ellipse/circle replaces everything)
    let frame_rects = if is_round || is_elliptical {
        vec![
            Rect2D { x: 0.0, y: 0.0, width: 0.0, height: 0.0 },
            Rect2D { x: 0.0, y: 0.0, width: 0.0, height: 0.0 },
            Rect2D { x: 0.0, y: 0.0, width: 0.0, height: 0.0 },
            Rect2D { x: 0.0, y: 0.0, width: 0.0, height: 0.0 },
        ]
    } else {
        vec![
            // Top (hidden for arched — arc replaces it)
            Rect2D { x: 0.0, y: 0.0, width: ow, height: top_rect_height },
            // Bottom (sill)
            Rect2D { x: 0.0, y: oh - fw, width: ow, height: fw },
            // Left stile (starts at arch spring for arched frames)
            Rect2D { x: 0.0, y: stile_top_y, width: fw, height: oh - stile_top_y - fw },
            // Right stile
            Rect2D { x: ow - fw, y: stile_top_y, width: fw, height: oh - stile_top_y - fw },
        ]
    };

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
    // For arched frames, the first row starts at the arch spring line and the
    // row heights are scaled proportionally so the cells fit exactly between
    // spring line and sill. The stored grid is NOT mutated — this geometry is
    // a derived view.
    let num_rows = kozijn.grid.rows.len();
    let row_area_top = if is_arched { stile_top_y } else { fw };
    let rows_total: f64 = kozijn.grid.rows.iter().map(|r| r.size).sum();
    let row_scale = if is_arched && rows_total > 0.0 {
        let divider_total = divider_width * (num_rows.saturating_sub(1) as f64);
        ((oh - fw - row_area_top - divider_total) / rows_total).max(0.0)
    } else {
        1.0
    };
    let row_display_sizes: Vec<f64> =
        kozijn.grid.rows.iter().map(|r| r.size * row_scale).collect();

    let mut row_positions = Vec::new();
    let mut y = row_area_top;
    for (i, size) in row_display_sizes.iter().enumerate() {
        row_positions.push(y);
        y += size;
        if i < num_rows - 1 {
            y += divider_width;
        }
    }

    // Dividers + cell rects — from the layout tree when present (cell_index =
    // stable depth-first leaf index, `Buiten` leaves yield no cell), otherwise
    // from the rectangular grid (legacy path, unchanged).
    let (v_dividers, h_dividers, cell_rects) = if let Some(lg) = &layout_geom {
        layout_dividers_and_cells(lg)
    } else {
        // Vertical dividers
        let mut v_dividers = Vec::new();
        let mut vx = fw;
        for i in 0..kozijn.grid.columns.len() {
            vx += kozijn.grid.columns[i].size;
            if i < kozijn.grid.columns.len() - 1 {
                v_dividers.push(Rect2D {
                    x: vx,
                    y: row_area_top,
                    width: divider_width,
                    height: oh - row_area_top - fw,
                });
                vx += divider_width;
            }
        }

        // Horizontal dividers (follow the displayed — possibly scaled — row heights)
        let mut h_dividers = Vec::new();
        let mut hy = row_area_top;
        for (i, size) in row_display_sizes.iter().enumerate() {
            hy += size;
            if i < num_rows - 1 {
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
        for row_idx in 0..num_rows {
            for (col_idx, col) in kozijn.grid.columns.iter().enumerate() {
                let cx = col_positions[col_idx];
                let cy = row_positions[row_idx];
                cell_rects.push(CellRect {
                    rect: Rect2D {
                        x: cx,
                        y: cy,
                        width: col.size,
                        height: row_display_sizes[row_idx],
                    },
                    col: col_idx,
                    row: row_idx,
                    cell_index: row_idx * num_cols + col_idx,
                    vulling: None,
                });
            }
        }
        (v_dividers, h_dividers, cell_rects)
    };

    // ── Dimension lines (NEN 3576 / GA Kozijn style) ──
    // Level 1 (closest): houtdiktes + vakmaten (stijl, kolommen, stijl)
    // Level 2: dagmaat (inner opening)
    // Level 3 (outermost): buitenwerkse maat (overall)
    let dim_gap = 35.0;
    let dim_start = 20.0;
    let mut dimensions = Vec::new();
    let inner_w = ow - 2.0 * fw;
    let inner_h = oh - 2.0 * fw;
    let divider_width = fw;
    let num_cols = kozijn.grid.columns.len();
    let num_rows = kozijn.grid.rows.len();

    // ── Bottom Level 1: houtdiktes + vakmaten (complete dimension chain) ──
    // Suppressed for round/elliptical frames (no rectangular members: only
    // dagmaat + buitenwerkse maat apply)
    let bot_y1 = oh + dim_start;
    if !(is_round || is_elliptical) {
        if let Some(lg) = &layout_geom {
            // Layout chain: stijl + segments derived from the leaves that
            // touch the bottom of the opening (+ divider/rest gaps) + stijl.
            // Contiguous by construction: the segments sum to the buitenmaat.
            for pair in layout_chain_bounds(lg, true, ow, oh, fw).windows(2) {
                dimensions.push(DimensionLine {
                    x1: pair[0], y1: bot_y1, x2: pair[1], y2: bot_y1,
                    label: format!("{:.0}", pair[1] - pair[0]),
                    side: DimensionSide::Bottom,
                });
            }
        } else {
            // Left stijl
            dimensions.push(DimensionLine {
                x1: 0.0, y1: bot_y1, x2: fw, y2: bot_y1,
                label: format!("{:.0}", fw),
                side: DimensionSide::Bottom,
            });
            // Column widths (vakmaten)
            for (i, col) in kozijn.grid.columns.iter().enumerate() {
                let cx = col_positions[i];
                dimensions.push(DimensionLine {
                    x1: cx, y1: bot_y1, x2: cx + col.size, y2: bot_y1,
                    label: format!("{:.0}", col.size),
                    side: DimensionSide::Bottom,
                });
                // Divider width (tussenstijl)
                if i < num_cols - 1 {
                    let dx = cx + col.size;
                    dimensions.push(DimensionLine {
                        x1: dx, y1: bot_y1, x2: dx + divider_width, y2: bot_y1,
                        label: format!("{:.0}", divider_width),
                        side: DimensionSide::Bottom,
                    });
                }
            }
            // Right stijl
            dimensions.push(DimensionLine {
                x1: ow - fw, y1: bot_y1, x2: ow, y2: bot_y1,
                label: format!("{:.0}", fw),
                side: DimensionSide::Bottom,
            });
        }
    }

    // ── Bottom Level 2: dagmaat ──
    let bot_y2 = oh + dim_start + dim_gap;
    dimensions.push(DimensionLine {
        x1: fw, y1: bot_y2, x2: ow - fw, y2: bot_y2,
        label: format!("{:.0}", inner_w),
        side: DimensionSide::Bottom,
    });

    // ── Bottom Level 3: buitenwerkse maat ──
    let bot_y3 = oh + dim_start + dim_gap * 2.0;
    dimensions.push(DimensionLine {
        x1: 0.0, y1: bot_y3, x2: ow, y2: bot_y3,
        label: format!("{:.0}", ow),
        side: DimensionSide::Bottom,
    });

    // ── Right Level 1: houtdiktes + vakmaten ──
    // Suppressed for round/elliptical frames (only dagmaat + buitenwerkse maat)
    let right_x1 = ow + dim_start;
    if !(is_round || is_elliptical) {
        if let Some(lg) = &layout_geom {
            // Layout chain: dorpel + segments derived from the leaves that
            // touch the right side of the opening (+ divider/rest gaps) +
            // dorpel. Same level (offset) as the grid chain.
            for pair in layout_chain_bounds(lg, false, ow, oh, fw).windows(2) {
                dimensions.push(DimensionLine {
                    x1: right_x1, y1: pair[0], x2: right_x1, y2: pair[1],
                    label: format!("{:.0}", pair[1] - pair[0]),
                    side: DimensionSide::Right,
                });
            }
        } else {
            // Bovendorpel (0 for arched — the arc replaces it)
            let top_h = top_rect_height;
            if top_h > 0.0 {
                dimensions.push(DimensionLine {
                    x1: right_x1, y1: 0.0, x2: right_x1, y2: top_h,
                    label: format!("{:.0}", top_h),
                    side: DimensionSide::Right,
                });
            }
            // Boogpijl (arch rise) for arched frames
            if is_arched {
                dimensions.push(DimensionLine {
                    x1: right_x1, y1: 0.0, x2: right_x1, y2: stile_top_y,
                    label: format!("{:.0}", stile_top_y),
                    side: DimensionSide::Right,
                });
            }
            // Row heights (vakmaten) — displayed sizes (scaled for arched frames)
            for i in 0..num_rows {
                let cy = row_positions[i];
                let size = row_display_sizes[i];
                dimensions.push(DimensionLine {
                    x1: right_x1, y1: cy, x2: right_x1, y2: cy + size,
                    label: format!("{:.0}", size),
                    side: DimensionSide::Right,
                });
                // Divider height (tussendorpel)
                if i < num_rows - 1 {
                    let dy = cy + size;
                    dimensions.push(DimensionLine {
                        x1: right_x1, y1: dy, x2: right_x1, y2: dy + divider_width,
                        label: format!("{:.0}", divider_width),
                        side: DimensionSide::Right,
                    });
                }
            }
            // Onderdorpel
            dimensions.push(DimensionLine {
                x1: right_x1, y1: oh - fw, x2: right_x1, y2: oh,
                label: format!("{:.0}", fw),
                side: DimensionSide::Right,
            });
        }
    }

    // ── Right Level 2: dagmaat ──
    let right_x2 = ow + dim_start + dim_gap;
    dimensions.push(DimensionLine {
        x1: right_x2, y1: fw, x2: right_x2, y2: oh - fw,
        label: format!("{:.0}", inner_h),
        side: DimensionSide::Right,
    });

    // ── Right Level 3: buitenwerkse maat ──
    let right_x3 = ow + dim_start + dim_gap * 2.0;
    dimensions.push(DimensionLine {
        x1: right_x3, y1: 0.0, x2: right_x3, y2: oh,
        label: format!("{:.0}", oh),
        side: DimensionSide::Right,
    });

    // Arched frame geometry
    let mut arcs = Vec::new();
    let mut arch_band: Vec<[f64; 2]> = Vec::new();
    if kozijn.frame.shape.shape_type == ShapeType::Arched {
        let arch_height = kozijn.frame.shape.arch_height.unwrap_or(ow / 4.0);
        arch_band = arch_band_polygon(ow, fw, arch_height);
        // Segmental arch: peak at y=0 (top of frame)
        // For a segmental arch of width W and rise H:
        // radius = (W/2)^2 / (2*H) + H/2
        // Center is at y = radius (below the peak in SVG Y-down coords)
        let half_w = ow / 2.0;
        let radius = (half_w * half_w) / (2.0 * arch_height) + arch_height / 2.0;
        let center_y = radius; // peak at y = center_y - radius = 0

        // Outer arc: springs at (0, arch_height) and (ow, arch_height)
        // cos(angle) = half_w / radius at the spring points
        let spring_angle = (half_w / radius).acos().to_degrees();
        arcs.push(Arc2D {
            cx: half_w,
            cy: center_y,
            radius,
            start_angle: 180.0 - spring_angle,
            end_angle: spring_angle,
            stroke_width: fw,
        });

        // Inner arc (smaller radius, spans inner opening from x=fw to x=ow-fw)
        let inner_radius = radius - fw;
        let inner_half_w = half_w - fw;
        if inner_radius > 0.0 && inner_half_w > 0.0 {
            let inner_ratio = (inner_half_w / inner_radius).min(1.0);
            let inner_spring_angle = inner_ratio.acos().to_degrees();
            arcs.push(Arc2D {
                cx: half_w,
                cy: center_y,
                radius: inner_radius,
                start_angle: 180.0 - inner_spring_angle,
                end_angle: inner_spring_angle,
                stroke_width: 1.0, // thin line for inner edge
            });
        }
    } else if kozijn.frame.shape.shape_type == ShapeType::Trapezoid {
        // Trapezoid only — no arc, just angled stiles
    } else if kozijn.frame.shape.shape_type == ShapeType::ArchedTrapezoid {
        // Combined: arched top + angled stiles (CNCware-style)
        let arch_height = kozijn.frame.shape.arch_height.unwrap_or(ow / 4.0);
        arch_band = arch_band_polygon(ow, fw, arch_height);
        let half_w = ow / 2.0;
        let radius = (half_w * half_w) / (2.0 * arch_height) + arch_height / 2.0;
        let center_y = radius; // peak at y=0

        // Outer arc
        let spring_angle = (half_w / radius).acos().to_degrees();
        arcs.push(Arc2D {
            cx: half_w,
            cy: center_y,
            radius,
            start_angle: 180.0 - spring_angle,
            end_angle: spring_angle,
            stroke_width: fw,
        });

        // Inner arc
        let inner_radius = radius - fw;
        let inner_half_w = half_w - fw;
        if inner_radius > 0.0 && inner_half_w > 0.0 {
            let inner_ratio = (inner_half_w / inner_radius).min(1.0);
            let inner_spring_angle = inner_ratio.acos().to_degrees();
            arcs.push(Arc2D {
                cx: half_w,
                cy: center_y,
                radius: inner_radius,
                start_angle: 180.0 - inner_spring_angle,
                end_angle: inner_spring_angle,
                stroke_width: 1.0,
            });
        }
    } else if kozijn.frame.shape.shape_type == ShapeType::Round {
        let radius = ow.min(oh) / 2.0;
        let cx = ow / 2.0;
        let cy = oh / 2.0;
        // Outer circle (two semicircles, thin stroke — SVG fill handles the frame area)
        arcs.push(Arc2D {
            cx, cy, radius,
            start_angle: 180.0, end_angle: 0.0,
            stroke_width: 2.0,
        });
        arcs.push(Arc2D {
            cx, cy, radius,
            start_angle: 0.0, end_angle: -180.0,
            stroke_width: 2.0,
        });
        // Inner circle (glass opening edge)
        let inner_r = radius - fw;
        if inner_r > 0.0 {
            arcs.push(Arc2D {
                cx, cy, radius: inner_r,
                start_angle: 180.0, end_angle: 0.0,
                stroke_width: 1.5,
            });
            arcs.push(Arc2D {
                cx, cy, radius: inner_r,
                start_angle: 0.0, end_angle: -180.0,
                stroke_width: 1.5,
            });
        }
    } else if kozijn.frame.shape.shape_type == ShapeType::Elliptical {
        // Elliptical frame: uses ellipse_rx and ellipse_ry from the shape config
        let rx = kozijn.frame.shape.ellipse_rx.unwrap_or(ow / 2.0);
        let ry = kozijn.frame.shape.ellipse_ry.unwrap_or(oh / 3.0);
        let cx = ow / 2.0;
        let cy = oh / 2.0;
        // Outer ellipse as two half-ellipses (top and bottom arcs)
        // We encode rx in the radius field and ry in the stroke_width with a special convention:
        // For ellipses, stroke_width encodes ry as a negative value marker isn't ideal,
        // so instead we'll use pairs: first arc stores rx, second stores ry
        // Actually, arcs use circular arcs. For SVG ellipses, we need a different approach.
        // We'll still emit arcs with a convention: if radius < 0, it's an ellipse marker.
        // Better: just use the existing arc system with the SVG elliptical arc command.
        // SVG arc: A rx ry x-axis-rotation large-arc-flag sweep-flag x y
        // We store rx in radius, ry in stroke_width (overloaded for ellipses)
        // The frontend will detect elliptical shape and render <ellipse> elements directly.
        // For the geometry data, we emit placeholder arcs that the frontend can interpret.
        // Outer ellipse (top half)
        arcs.push(Arc2D {
            cx, cy, radius: rx,
            start_angle: 180.0, end_angle: 0.0,
            stroke_width: 2.0,
        });
        // Outer ellipse (bottom half)
        arcs.push(Arc2D {
            cx, cy, radius: rx,
            start_angle: 0.0, end_angle: -180.0,
            stroke_width: 2.0,
        });
        // Inner ellipse (top half)
        let inner_rx = rx - fw;
        let inner_ry = ry - fw;
        if inner_rx > 0.0 && inner_ry > 0.0 {
            arcs.push(Arc2D {
                cx, cy, radius: inner_rx,
                start_angle: 180.0, end_angle: 0.0,
                stroke_width: 1.5,
            });
            arcs.push(Arc2D {
                cx, cy, radius: inner_rx,
                start_angle: 0.0, end_angle: -180.0,
                stroke_width: 1.5,
            });
        }
    }

    // Trapezoid / Triangle / Polygon polygon computation
    let mut trapezoid_outer = Vec::new();
    let mut trapezoid_inner = Vec::new();
    if kozijn.frame.shape.shape_type == ShapeType::Triangle {
        // Triangle: 3 vertices — bottom-left, bottom-right, apex
        let apex_offset = kozijn.frame.shape.apex_offset.unwrap_or(0.0);
        let apex_x = ow / 2.0 + apex_offset;
        let apex_y = 0.0;

        trapezoid_outer = vec![
            [0.0, oh],         // bottom-left
            [ow, oh],          // bottom-right
            [apex_x, apex_y],  // apex
        ];

        // Inner polygon (offset inward by frame width)
        // Left slope angle
        let left_dx = apex_x - 0.0;
        let left_dy = oh;
        let left_len = (left_dx * left_dx + left_dy * left_dy).sqrt();
        let _left_nx = left_dy / left_len; // normal x
        let _left_ny = -left_dx / left_len; // normal y (inward)

        // Right slope angle
        let right_dx = ow - apex_x;
        let right_dy = oh;
        let right_len = (right_dx * right_dx + right_dy * right_dy).sqrt();
        let _right_nx = -right_dy / right_len;
        let _right_ny = -right_dx / right_len;

        // Simplified inner triangle offset by fw
        let inner_bottom_y = oh - fw;
        let inner_left_x = fw;
        let inner_right_x = ow - fw;
        // Apex shifts down by fw / sin(half-angle) approximately
        let inner_apex_y = fw * left_len / left_dy;
        let inner_apex_x = apex_x;

        trapezoid_inner = vec![
            [inner_left_x, inner_bottom_y],
            [inner_right_x, inner_bottom_y],
            [inner_apex_x, inner_apex_y],
        ];
    } else if kozijn.frame.shape.shape_type == ShapeType::Polygon {
        // Custom polygon: use provided points, or fall back to rectangular
        if let Some(ref points) = kozijn.frame.shape.polygon_points {
            if points.len() >= 3 {
                trapezoid_outer = points.clone();

                // Compute centroid for inward offset
                let n = points.len() as f64;
                let cx: f64 = points.iter().map(|p| p[0]).sum::<f64>() / n;
                let cy: f64 = points.iter().map(|p| p[1]).sum::<f64>() / n;

                // Offset each point toward centroid by fw
                trapezoid_inner = points.iter().map(|p| {
                    let dx = cx - p[0];
                    let dy = cy - p[1];
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist > 0.0 {
                        [p[0] + dx / dist * fw, p[1] + dy / dist * fw]
                    } else {
                        *p
                    }
                }).collect();
            }
        }
    } else if kozijn.frame.shape.shape_type == ShapeType::Trapezoid
        || kozijn.frame.shape.shape_type == ShapeType::ArchedTrapezoid
    {
        let _top_w = kozijn.frame.shape.top_width.unwrap_or(ow * 0.6);
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
        arch_band,
        trapezoid_outer,
        trapezoid_inner,
    }
}

/// Map the free-subdivision layout geometry onto renderable dividers + cells.
/// `cell_index` is the stable depth-first leaf index in the tree (with gaps
/// where `Buiten` leaves sit — those yield no cell); `col`/`row` are grid
/// concepts and stay 0 on this path: consumers key layout cells by
/// `cell_index` and read the vulling straight from the cell.
fn layout_dividers_and_cells(lg: &LayoutGeometry) -> (Vec<Rect2D>, Vec<Rect2D>, Vec<CellRect>) {
    let to_rect = |r: &LayoutRect| Rect2D { x: r.x, y: r.y, width: r.width, height: r.height };
    let v_dividers = lg
        .dividers
        .iter()
        .filter(|d| d.direction == "v")
        .map(|d| to_rect(&d.rect))
        .collect();
    let h_dividers = lg
        .dividers
        .iter()
        .filter(|d| d.direction == "h")
        .map(|d| to_rect(&d.rect))
        .collect();
    let cell_rects = lg
        .leaves
        .iter()
        .filter(|l| !l.vulling.is_buiten())
        .map(|l| CellRect {
            rect: to_rect(&l.rect),
            col: 0,
            row: 0,
            cell_index: l.leaf_index,
            vulling: Some(l.vulling.clone()),
        })
        .collect();
    (v_dividers, h_dividers, cell_rects)
}

/// Boundaries of the level-1 dimension chain along one axis, derived from the
/// layout tree: the fixed frame edges (0, fw, total-fw, total) plus the edges
/// of every non-`Buiten` leaf that touches the chain's inner edge — the bottom
/// of the opening for the horizontal chain, the right side for the vertical
/// one. Consecutive boundaries form the chain segments (stijl/dorpel, vak,
/// divider or unanchored rest). Mirrors `layoutChain()` in the frontend
/// (ui/src/components/editor/KozijnCanvas.svelte) — keep the two in sync,
/// including the 1.0 mm touch tolerance and 0.5 mm dedup.
fn layout_chain_bounds(lg: &LayoutGeometry, horizontal: bool, ow: f64, oh: f64, fw: f64) -> Vec<f64> {
    let total = if horizontal { ow } else { oh };
    let inner_edge = if horizontal { oh - fw } else { ow - fw };
    let mut bounds = vec![0.0, fw, total - fw, total];
    for l in &lg.leaves {
        if l.vulling.is_buiten() {
            continue;
        }
        let touches = if horizontal {
            (l.rect.y + l.rect.height - inner_edge).abs() < 1.0
        } else {
            (l.rect.x + l.rect.width - inner_edge).abs() < 1.0
        };
        if !touches {
            continue;
        }
        let (a, len) = if horizontal {
            (l.rect.x, l.rect.width)
        } else {
            (l.rect.y, l.rect.height)
        };
        bounds.push(a);
        bounds.push(a + len);
    }
    bounds.sort_by(|p, q| p.total_cmp(q));
    let mut uniq: Vec<f64> = Vec::new();
    for v in bounds {
        if uniq.last().map_or(true, |u| v - *u >= 0.5) {
            uniq.push(v);
        }
    }
    uniq
}

/// Closed polygon [[x,y], ...] for the arched top band: outer arc sampled from
/// the left spring point over the peak to the right spring point, then the
/// inner arc back from right to left. The straight closing segments between
/// the arc ends (sluitstukken) fall within the stile footprints, so the
/// polygon can be filled directly as the toogband. Falls back to just the
/// outer arc (closed by its chord) when the frame width leaves no inner arc.
fn arch_band_polygon(ow: f64, fw: f64, arch_height: f64) -> Vec<[f64; 2]> {
    let half_w = ow / 2.0;
    // Segmental arch: radius = (W/2)^2 / (2*H) + H/2, peak at y = 0
    let radius = (half_w * half_w) / (2.0 * arch_height) + arch_height / 2.0;
    let center_y = radius;
    let segments = 32;

    let mut points: Vec<[f64; 2]> = Vec::new();
    // Outer arc (left spring → peak → right spring)
    let spring_angle = (half_w / radius).min(1.0).acos();
    let start = std::f64::consts::PI - spring_angle;
    let end = spring_angle;
    for i in 0..=segments {
        let a = start + (end - start) * (i as f64 / segments as f64);
        points.push([half_w + radius * a.cos(), center_y - radius * a.sin()]);
    }
    // Inner arc (right spring → peak → left spring), closing the band
    let inner_radius = radius - fw;
    let inner_half_w = half_w - fw;
    if inner_radius > 0.0 && inner_half_w > 0.0 {
        let inner_spring = (inner_half_w / inner_radius).min(1.0).acos();
        let start = inner_spring;
        let end = std::f64::consts::PI - inner_spring;
        for i in 0..=segments {
            let a = start + (end - start) * (i as f64 / segments as f64);
            points.push([
                half_w + inner_radius * a.cos(),
                center_y - inner_radius * a.sin(),
            ]);
        }
    }
    points
}

/// Normalize the grid to a single 1×1 cell (used when switching to the Round
/// shape, where dividers don't apply). Rebuilds cells with the same logic as
/// `Kozijn::add_column`/`add_row`.
pub fn normalize_grid_single_cell(kozijn: &mut Kozijn) {
    let fw = kozijn.frame.frame_width;
    kozijn.grid.columns = vec![GridDivision {
        size: kozijn.frame.outer_width - 2.0 * fw,
        divider_profile: None,
    }];
    kozijn.grid.rows = vec![GridDivision {
        size: kozijn.frame.outer_height - 2.0 * fw,
        divider_profile: None,
    }];
    kozijn.rebuild_cells();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kozijn::FrameShape;
    use crate::layout::{glas, melkmeisje1, raam, split_row, VakChild};

    fn arched_kozijn(w: f64, h: f64, arch_height: f64) -> Kozijn {
        let mut k = Kozijn::new("Test", "K01", w, h);
        k.frame.shape = FrameShape {
            shape_type: ShapeType::Arched,
            arch_height: Some(arch_height),
            ..FrameShape::default()
        };
        k
    }

    #[test]
    fn arched_rows_scaled_to_fit_frame() {
        let mut k = arched_kozijn(1200.0, 1500.0, 300.0);
        k.add_row(600.0); // two rows
        let fw = k.frame.frame_width;
        let oh = k.frame.outer_height;
        let g = compute_2d_geometry(&k);

        // No cell may extend below the top of the sill
        let bottom = g
            .cell_rects
            .iter()
            .map(|c| c.rect.y + c.rect.height)
            .fold(f64::MIN, f64::max);
        assert!(
            bottom <= oh - fw + 1e-6,
            "cells overflow the sill: {} > {}",
            bottom,
            oh - fw
        );

        // Rows + dividers span exactly from the arch spring line to the sill
        let rows_sum: f64 = g
            .cell_rects
            .iter()
            .filter(|c| c.col == 0)
            .map(|c| c.rect.height)
            .sum();
        let dividers_sum: f64 = g.h_dividers.iter().map(|d| d.height).sum();
        let expected = oh - fw - 300.0;
        assert!((rows_sum + dividers_sum - expected).abs() < 1e-6);
        // First row starts at the arch spring line
        assert!((g.cell_rects[0].rect.y - 300.0).abs() < 1e-6);
        // The stored grid itself is untouched (geometry is a view)
        assert!((k.grid.rows[0].size - 600.0).abs() < 1e-6);
    }

    #[test]
    fn arched_emits_arch_rise_dimension_and_band() {
        let k = arched_kozijn(1200.0, 1500.0, 250.0);
        let g = compute_2d_geometry(&k);
        let has_rise = g.dimensions.iter().any(|d| {
            matches!(d.side, DimensionSide::Right) && d.y1 == 0.0 && (d.y2 - 250.0).abs() < 1e-6
        });
        assert!(has_rise, "missing boogpijl dimension in right chain");
        // Band polygon present and closed-fillable (outer + inner arc samples)
        assert!(g.arch_band.len() >= 4, "arch band polygon missing");
        // Outer arc springs at (0, arch_height) and (ow, arch_height)
        let first = g.arch_band.first().unwrap();
        assert!((first[0] - 0.0).abs() < 1e-6 && (first[1] - 250.0).abs() < 1e-6);
    }

    #[test]
    fn elliptical_has_no_phantom_frame_width_dimension() {
        let mut k = Kozijn::new("Test", "K01", 1200.0, 800.0);
        k.frame.shape = FrameShape {
            shape_type: ShapeType::Elliptical,
            ..FrameShape::default()
        };
        let g = compute_2d_geometry(&k);
        let right: Vec<_> = g
            .dimensions
            .iter()
            .filter(|d| matches!(d.side, DimensionSide::Right))
            .collect();
        // Only dagmaat + buitenwerkse maat remain in the right chain
        assert_eq!(right.len(), 2, "right chain: {:?}", right);
        let fw_label = format!("{:.0}", k.frame.frame_width);
        assert!(right.iter().all(|d| d.label != fw_label));
        // Bottom chain equally reduced to dagmaat + buitenwerkse maat
        let bottom_count = g
            .dimensions
            .iter()
            .filter(|d| matches!(d.side, DimensionSide::Bottom))
            .count();
        assert_eq!(bottom_count, 2);
    }

    #[test]
    fn round_normalized_grid_is_single_cell() {
        let mut k = Kozijn::new("Test", "K01", 900.0, 900.0);
        k.add_column(300.0);
        k.add_row(300.0);
        assert_eq!(k.cells.len(), 4);
        k.frame.shape = FrameShape {
            shape_type: ShapeType::Round,
            ..FrameShape::default()
        };
        normalize_grid_single_cell(&mut k);
        assert_eq!(k.grid.columns.len(), 1);
        assert_eq!(k.grid.rows.len(), 1);
        assert_eq!(k.cells.len(), 1);
        let g = compute_2d_geometry(&k);
        assert_eq!(g.cell_rects.len(), 1);
        assert!(g.v_dividers.is_empty() && g.h_dividers.is_empty());
    }

    // ── Free-subdivision layout as the geometry source ──

    /// Two-vaks kozijn: fw + vak + tussenstijl + vak + fw over the width.
    fn layout_kozijn_2vaks() -> Kozijn {
        let mut k = Kozijn::new("Test", "K01", 1200.0, 1500.0);
        k.layout = Some(split_row(vec![
            VakChild { size: 1.0, node: glas() },
            VakChild { size: 1.0, node: raam("draaikiep") },
        ]));
        k // grid stays the default 1×1 — the tree must win
    }

    #[test]
    fn layout_cell_rects_follow_tree_not_grid() {
        let k = layout_kozijn_2vaks();
        let g = compute_2d_geometry(&k);
        let fw = k.frame.frame_width; // 67
        let vak_w = (1200.0 - 2.0 * fw - fw) / 2.0; // 499.5

        // The 1×1 grid would give a single cell; the tree gives two + a stijl
        assert_eq!(g.cell_rects.len(), 2);
        assert_eq!(g.v_dividers.len(), 1);
        assert!(g.h_dividers.is_empty());
        let c0 = &g.cell_rects[0];
        let c1 = &g.cell_rects[1];
        assert!((c0.rect.x - fw).abs() < 1e-6);
        assert!((c0.rect.width - vak_w).abs() < 1e-6);
        assert!((c1.rect.x - (fw + vak_w + fw)).abs() < 1e-6);
        assert!((c1.rect.x + c1.rect.width - (1200.0 - fw)).abs() < 1e-6);
        assert!((g.v_dividers[0].x - (fw + vak_w)).abs() < 1e-6);
        // cell_index = stable depth-first leaf index; vulling rides along
        assert_eq!((c0.cell_index, c1.cell_index), (0, 1));
        assert!(matches!(c0.vulling, Some(Vakvulling::Glas)));
        assert!(matches!(
            &c1.vulling,
            Some(Vakvulling::Raam { open_type, .. }) if open_type == "draaikiep"
        ));
    }

    #[test]
    fn layout_dimension_chains_close() {
        let k = layout_kozijn_2vaks();
        let g = compute_2d_geometry(&k);
        let fw = k.frame.frame_width;
        let (ow, oh) = (1200.0, 1500.0);

        // Bottom level-1 chain (at oh + 20): contiguous 0 → ow
        let bot_y1 = oh + 20.0;
        let chain: Vec<_> = g
            .dimensions
            .iter()
            .filter(|d| matches!(d.side, DimensionSide::Bottom) && (d.y1 - bot_y1).abs() < 1e-6)
            .collect();
        assert_eq!(chain.len(), 5, "stijl+vak+tussenstijl+vak+stijl: {:?}", chain);
        assert!((chain[0].x1 - 0.0).abs() < 1e-6);
        assert!((chain.last().unwrap().x2 - ow).abs() < 1e-6);
        for w in chain.windows(2) {
            assert!((w[0].x2 - w[1].x1).abs() < 1e-6, "chain gap: {:?} → {:?}", w[0], w[1]);
        }
        let sum: f64 = chain.iter().map(|d| d.x2 - d.x1).sum();
        assert!((sum - ow).abs() < 1e-6, "chain sums to buitenmaat");
        // Interior segments (without the two stijlen) sum to the dagmaat
        let interior: f64 = chain[1..chain.len() - 1].iter().map(|d| d.x2 - d.x1).sum();
        assert!((interior - (ow - 2.0 * fw)).abs() < 1e-6, "interior = binnenmaat");

        // Right level-1 chain (at ow + 20): both leaves are full height, only
        // the one bordering the right stijl anchors → dorpel + dagmaat + dorpel
        let right_x1 = ow + 20.0;
        let rchain: Vec<_> = g
            .dimensions
            .iter()
            .filter(|d| matches!(d.side, DimensionSide::Right) && (d.x1 - right_x1).abs() < 1e-6)
            .collect();
        assert_eq!(rchain.len(), 3, "dorpel+vak+dorpel: {:?}", rchain);
        assert!((rchain[0].y1 - 0.0).abs() < 1e-6);
        assert!((rchain.last().unwrap().y2 - oh).abs() < 1e-6);
        let rsum: f64 = rchain.iter().map(|d| d.y2 - d.y1).sum();
        assert!((rsum - oh).abs() < 1e-6);
        // Dagmaat (level 2) and buitenwerkse maat (level 3) still present
        assert!(g.dimensions.iter().any(|d| {
            matches!(d.side, DimensionSide::Bottom) && (d.y1 - (oh + 55.0)).abs() < 1e-6
        }));
        assert!(g.dimensions.iter().any(|d| {
            matches!(d.side, DimensionSide::Bottom) && (d.y1 - (oh + 90.0)).abs() < 1e-6
        }));
    }

    #[test]
    fn layout_melkmeisje_buiten_gets_no_cell_and_chain_skips_side_light() {
        let mut k = Kozijn::new("Test", "K01", 1400.0, 1900.0);
        k.layout = Some(melkmeisje1());
        let g = compute_2d_geometry(&k);
        let fw = k.frame.frame_width;
        let (ow, oh) = (1400.0, 1900.0);
        let inner_w = ow - 2.0 * fw;
        let inner_h = oh - 2.0 * fw;
        let raam_w = (inner_w - fw) * 900.0 / 1400.0;
        let glas_h = (inner_h - fw) * 900.0 / 1900.0;

        // Buiten leaf yields no cell rect; leaf indices keep their gaps
        assert_eq!(g.cell_rects.len(), 2);
        assert_eq!(g.cell_rects[0].cell_index, 0); // raam (full height)
        assert_eq!(g.cell_rects[1].cell_index, 1); // zijlicht glas (buiten = 2)
        assert!(matches!(g.cell_rects[0].vulling, Some(Vakvulling::Raam { .. })));
        // One tussenstijl + the zijlicht onderdorpel as h-divider
        assert_eq!(g.v_dividers.len(), 1);
        assert_eq!(g.h_dividers.len(), 1);

        // Bottom chain: zijlicht doesn't reach the bottom of the opening, so
        // its area stays one unanchored segment — chain still closes to ow
        let bot_y1 = oh + 20.0;
        let chain: Vec<_> = g
            .dimensions
            .iter()
            .filter(|d| matches!(d.side, DimensionSide::Bottom) && (d.y1 - bot_y1).abs() < 1e-6)
            .collect();
        assert_eq!(chain.len(), 4, "stijl+raam+rest+stijl: {:?}", chain);
        assert!((chain[1].x2 - chain[1].x1 - raam_w).abs() < 1e-6);
        let sum: f64 = chain.iter().map(|d| d.x2 - d.x1).sum();
        assert!((sum - ow).abs() < 1e-6);

        // Right chain: the zijlicht glass anchors → its height appears
        let right_x1 = ow + 20.0;
        let rchain: Vec<_> = g
            .dimensions
            .iter()
            .filter(|d| matches!(d.side, DimensionSide::Right) && (d.x1 - right_x1).abs() < 1e-6)
            .collect();
        assert_eq!(rchain.len(), 4, "dorpel+zijlicht+rest+dorpel: {:?}", rchain);
        assert!((rchain[1].y2 - rchain[1].y1 - glas_h).abs() < 1e-6);
        let rsum: f64 = rchain.iter().map(|d| d.y2 - d.y1).sum();
        assert!((rsum - oh).abs() < 1e-6);
    }

    #[test]
    fn grid_path_without_layout_is_unchanged() {
        // Regression: a kozijn without layout must keep the exact pre-layout
        // grid behavior — geometry values, dim chain and serialized payload.
        let mut k = Kozijn::new("Test", "K01", 1200.0, 1500.0);
        k.add_column(400.0); // columns [400, 666] (grid semantics untouched)
        assert!(k.layout.is_none());
        let g = compute_2d_geometry(&k);

        assert_eq!(g.cell_rects.len(), 2);
        let c0 = &g.cell_rects[0];
        let c1 = &g.cell_rects[1];
        assert_eq!((c0.col, c0.row, c0.cell_index), (0, 0, 0));
        assert_eq!((c1.col, c1.row, c1.cell_index), (1, 0, 1));
        assert!((c0.rect.x - 67.0).abs() < 1e-6 && (c0.rect.width - 400.0).abs() < 1e-6);
        assert!((c1.rect.x - 534.0).abs() < 1e-6 && (c1.rect.width - 666.0).abs() < 1e-6);
        assert_eq!(g.v_dividers.len(), 1);
        assert!((g.v_dividers[0].x - 467.0).abs() < 1e-6);

        // Grid-walk dim chain exactly as before (incl. the historical overlap
        // quirk where added columns keep their pre-divider sizes)
        let bottom: Vec<_> = g
            .dimensions
            .iter()
            .filter(|d| matches!(d.side, DimensionSide::Bottom))
            .collect();
        let labels: Vec<&str> = bottom.iter().map(|d| d.label.as_str()).collect();
        assert_eq!(labels, vec!["67", "400", "67", "666", "67", "1066", "1200"]);

        // No vulling on the grid path, and the payload stays byte-identical:
        // the optional field is skipped entirely when absent
        assert!(g.cell_rects.iter().all(|c| c.vulling.is_none()));
        let json = serde_json::to_string(&g).unwrap();
        assert!(!json.contains("vulling"), "grid payload must not grow a vulling key");
    }
}
