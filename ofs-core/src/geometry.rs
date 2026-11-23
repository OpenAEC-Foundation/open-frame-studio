use crate::curved_contour::{Contour, CurvedFrame, FrameCurve};
use crate::kozijn::{Kozijn, ShapeType};
use crate::layout::{
    compute_layout_geometry, compute_layout_geometry_unclipped, LayoutGeometry, LayoutRect,
    Vakvulling,
};
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub perimeter_stop: Option<crate::perimeter_stop::StopGeometry>,
    /// Effective choices, including material defaults for untouched projects.
    #[serde(default)]
    pub corner_settings: [crate::joint::Joint; 4],
    /// Wooden tenon joints at physical head-to-jamb and half-ring seams.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arch_joints: Vec<crate::curved_timber::ArchJoint>,
    /// Analytic frame boundaries and clipped divider faces, in elevation mm.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub curved_frame: Option<CurvedFrame>,
    /// Finished outside frame members, including their shared joint faces.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub curved_timber: Vec<crate::curved_timber::CurvedTimberMember>,
    /// In the same order as v_dividers followed by h_dividers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub curved_dividers: Vec<Contour>,
    /// Finished pieces with the through member retained at each crossing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub curved_divider_timber: Vec<crate::curved_timber::CurvedDividerMember>,
    /// Face width used to partition the optional free subdivision tree.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_divider_width: Option<f64>,
    /// Overall bounding box (outer frame)
    pub outer_rect: Rect2D,
    /// Inner opening (inside the frame)
    pub inner_rect: Rect2D,
    /// Frame members (top, bottom/sill, left, right)
    pub frame_rects: Vec<Rect2D>,
    /// Frame drawn as filled polygons instead of the plain `frame_rects`
    /// rectangles. Populated ONLY for mitered (verstek) / mixed corners and for
    /// stepped (melkmeisje / `Buiten`-leaf) outlines; empty otherwise, so plain
    /// butt-jointed kozijnen and old payloads stay byte-identical (the field is
    /// skipped when empty). Each entry is a closed ring `[[x, y], ...]`. When
    /// present, consumers should draw these for the frame body and ignore
    /// `frame_rects` (which stays populated for rectangle-only consumers such as
    /// the 3D viewer). Order matches `frame_rects` (top, bottom, left, right)
    /// for the per-member miter case; the stepped case is one ring per real vak.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frame_polygons: Vec<Vec<[f64; 2]>>,
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
    /// Getekende sponninghoogte (glasinval) in mm from the frame profile
    /// snapshot — offset of the front-view rebate hidden line inside each
    /// frame member. This is the glass edge cover: for PVC the Glaseinstand
    /// (VEKA 82 MD: 20), not the Falzhöhe (28). When the snapshot lacks
    /// sponning values it resolves to the material norm (hout 17 / PVC 20 /
    /// alu 25 — see `profile::norm_glasinval`). Omitted when the kozijn has
    /// no snapshot at all (old payloads stay byte-equal); the frontend then
    /// falls back to the classic 17 mm.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sponning_hoogte: Option<f64>,
    /// Glaslat face width (oplegbreedte, KVT min 13 binnen / 15 buiten) in mm
    /// from the frame profile snapshot. NB: this is NOT the bead line offset —
    /// per KVT 12.3.2 the front-view bead line sits `glaslat_hoogte −
    /// sponning_hoogte` inside the day line (see `glaslat_hoogte`). Kept for
    /// existing consumers; omitted when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub glaslat_breedte: Option<f64>,
    /// Glaslathoogte in mm (face plane) — drives the front-view bead line:
    /// offset `glaslat_hoogte − sponning_hoogte` inside the day line (0 for
    /// a 17x17 bead on a 17 mm sponning, 11 for a 28 mm handelslat, KVT
    /// 12.3.2). Resolves to the material fallback when the snapshot lacks it
    /// (hout max(17, sponninghoogte) / PVC & alu kliklat 25); omitted when
    /// the kozijn has no snapshot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub glaslat_hoogte: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellRect {
    pub rect: Rect2D,
    /// Generic PVC/aluminium fittings on the actual moving leaf outlines.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hardware_leaves: Vec<crate::hardware::HardwareLeaf>,
    /// Panes specified by the infill which have no manufacturable curved opening.
    #[serde(skip)]
    pub(crate) missing_glass_panes: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opening_contour: Option<Contour>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opening_bounds: Option<Rect2D>,
    /// Glazing and sash boundaries; paired doors have one fit per leaf.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fit: Option<crate::glazing_fit::GlazingFit>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub second_fit: Option<crate::glazing_fit::GlazingFit>,
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
    let divider_width = kozijn.layout_divider_width();
    let column_divider_widths: Vec<f64> = kozijn
        .grid
        .columns
        .iter()
        .skip(1)
        .map(|d| {
            d.divider_width
                .filter(|w| w.is_finite() && *w > 0.0)
                .unwrap_or(fw)
        })
        .collect();
    let row_divider_widths: Vec<f64> = kozijn
        .grid
        .rows
        .iter()
        .skip(1)
        .map(|d| {
            d.divider_width
                .filter(|w| w.is_finite() && *w > 0.0)
                .unwrap_or(fw)
        })
        .collect();

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
    // dimension chains. The canvas uses the same divider width and inner
    // rectangle before the openings are intersected with the frame day.
    let frame_curve = FrameCurve::new(&kozijn.frame);
    let layout_geom = kozijn.layout.as_ref().map(|node| {
        let rect = LayoutRect {
            x: fw,
            y: fw,
            width: ow - 2.0 * fw,
            height: oh - 2.0 * fw,
        };
        if frame_curve.is_some() {
            // Stock bordering a wall on just one side still runs down to its
            // actual joint. Divider machining removes double-sided wall spans.
            compute_layout_geometry_unclipped(node, rect, divider_width)
        } else {
            compute_layout_geometry(node, rect, divider_width)
        }
    });

    // Frame members — adjust for arched/special shapes
    let is_arched = kozijn.frame.shape.shape_type == ShapeType::Arched
        || kozijn.frame.shape.shape_type == ShapeType::ArchedTrapezoid;
    let is_round = kozijn.frame.shape.shape_type == ShapeType::Round;
    let is_elliptical = kozijn.frame.shape.shape_type == ShapeType::Elliptical;
    let top_rect_height = if is_arched || is_round || is_elliptical {
        0.0
    } else {
        fw
    };

    // For arched frames, stiles start at the arch spring line (not y=0)
    let stile_top_y = if is_arched {
        match &frame_curve {
            Some(FrameCurve::Arch { rise, .. }) => *rise,
            _ => fw,
        }
    } else {
        top_rect_height
    };

    // Frame members. Order stays [top, bottom, left, right] for every branch
    // (the 3D viewer and canvas index members by position). Plain rectangular
    // frames become joint-aware here — the through-member and any verstek come
    // from the SAME shared helper the saw list uses (`crate::joint`), so the
    // picture and the cut list agree (issue 7) — and step around `Buiten`
    // leaves (issue 10). Every other shape keeps its exact historic rects.
    let mut frame_polygons: Vec<Vec<[f64; 2]>> = Vec::new();
    let frame_rects = if is_round || is_elliptical {
        vec![
            Rect2D {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
            Rect2D {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
            Rect2D {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
            Rect2D {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
        ]
    } else if kozijn.frame.shape.shape_type == ShapeType::Rectangular {
        let corners = crate::joint::frame_corner_model(
            &kozijn.frame.material,
            &kozijn.frame.corner_joints,
            kozijn.frame.joints_configured,
        );
        let has_buiten = layout_geom
            .as_ref()
            .map_or(false, |lg| lg.leaves.iter().any(|l| l.vulling.is_buiten()));
        // Rects: stiles run full height only when every corner is an
        // explicitly configured stijl-through butt; the wood/wood-alu DEFAULT
        // is dorpel-through (NL-timmerpraktijk, KVT katern 15: dorpels
        // doorlopend, stijlen ertussen gepend — see `joint::frame_corner_model`),
        // which uses the historic dorpel-through rects (also byte-identical
        // for alu/PVC verstek). For verstek/mixed/melkmeisje the accurate
        // frame rides in `frame_polygons`.
        let all_stijl_through = corners
            .iter()
            .all(|c| !c.verstek && c.through == crate::joint::ThroughMember::Stijl);
        let rects = if all_stijl_through {
            vec![
                Rect2D {
                    x: fw,
                    y: 0.0,
                    width: ow - 2.0 * fw,
                    height: fw,
                }, // top rail between stiles
                Rect2D {
                    x: fw,
                    y: oh - fw,
                    width: ow - 2.0 * fw,
                    height: fw,
                }, // bottom rail between stiles
                Rect2D {
                    x: 0.0,
                    y: 0.0,
                    width: fw,
                    height: oh,
                }, // left stile full height
                Rect2D {
                    x: ow - fw,
                    y: 0.0,
                    width: fw,
                    height: oh,
                }, // right stile full height
            ]
        } else {
            vec![
                Rect2D {
                    x: 0.0,
                    y: 0.0,
                    width: ow,
                    height: fw,
                },
                Rect2D {
                    x: 0.0,
                    y: oh - fw,
                    width: ow,
                    height: fw,
                },
                Rect2D {
                    x: 0.0,
                    y: fw,
                    width: fw,
                    height: oh - 2.0 * fw,
                },
                Rect2D {
                    x: ow - fw,
                    y: fw,
                    width: fw,
                    height: oh - 2.0 * fw,
                },
            ]
        };
        if has_buiten {
            if let Some(lg) = &layout_geom {
                frame_polygons = stepped_frame_polygons(lg, ow, oh, fw);
            }
        } else {
            let needs_polygons = corners.iter().any(|c| c.verstek)
                || !corners.iter().all(|c| c.through == corners[0].through);
            if needs_polygons {
                frame_polygons = rectangular_frame_member_polygons(ow, oh, fw, &corners);
            }
        }
        rects
    } else {
        vec![
            // Top (hidden for arched — arc replaces it)
            Rect2D {
                x: 0.0,
                y: 0.0,
                width: ow,
                height: top_rect_height,
            },
            // Bottom (sill)
            Rect2D {
                x: 0.0,
                y: oh - fw,
                width: ow,
                height: fw,
            },
            // Left stile (starts at arch spring for arched frames)
            Rect2D {
                x: 0.0,
                y: stile_top_y,
                width: fw,
                height: oh - stile_top_y - fw,
            },
            // Right stile
            Rect2D {
                x: ow - fw,
                y: stile_top_y,
                width: fw,
                height: oh - stile_top_y - fw,
            },
        ]
    };

    // Calculate column positions (x coordinates of cell starts)
    let mut col_positions = Vec::new();
    let mut x = fw;
    for (i, col) in kozijn.grid.columns.iter().enumerate() {
        col_positions.push(x);
        x += col.size;
        if i < kozijn.grid.columns.len() - 1 {
            x += column_divider_widths[i]; // space for the selected article
        }
    }

    // Calculate row positions (y coordinates of cell starts)
    // For arched frames, the first row starts at the arch spring line and the
    // row heights are scaled proportionally so the cells fit exactly between
    // spring line and sill. The stored grid is NOT mutated — this geometry is
    // a derived view.
    let num_rows = kozijn.grid.rows.len();
    let rows_from_spring = is_arched && !kozijn.grid.rows_from_day;
    let row_area_top = if rows_from_spring { stile_top_y } else { fw };
    let rows_total: f64 = kozijn.grid.rows.iter().map(|r| r.size).sum();
    let row_scale = if rows_from_spring && rows_total > 0.0 {
        let divider_total: f64 = row_divider_widths.iter().sum();
        ((oh - fw - row_area_top - divider_total) / rows_total).max(0.0)
    } else {
        1.0
    };
    let row_display_sizes: Vec<f64> = kozijn
        .grid
        .rows
        .iter()
        .map(|r| r.size * row_scale)
        .collect();

    let mut row_positions = Vec::new();
    let mut row_divider_sizes = Vec::new();
    let mut y = row_area_top;
    for (i, size) in row_display_sizes.iter().enumerate() {
        row_positions.push(y);
        y += size;
        if i < num_rows - 1 {
            let divider_width = row_divider_widths[i];
            let height = if rows_from_spring {
                divider_width.min((oh - fw - y).max(0.0))
            } else {
                divider_width
            };
            row_divider_sizes.push(height);
            y += height;
        }
    }

    // Dividers + cell rects — from the layout tree when present (cell_index =
    // stable depth-first leaf index, `Buiten` leaves yield no cell), otherwise
    // from the rectangular grid (legacy path, unchanged).
    let (v_dividers, h_dividers, mut cell_rects) = if let Some(lg) = &layout_geom {
        layout_dividers_and_cells(lg)
    } else {
        // Vertical dividers
        let mut v_dividers = Vec::new();
        let mut vx = fw;
        for i in 0..kozijn.grid.columns.len() {
            vx += kozijn.grid.columns[i].size;
            if i < kozijn.grid.columns.len() - 1 {
                let divider_width = column_divider_widths[i];
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
        for (i, &height) in row_divider_sizes.iter().enumerate() {
            h_dividers.push(Rect2D {
                x: fw,
                y: row_positions[i] + row_display_sizes[i],
                width: ow - 2.0 * fw,
                height,
            });
        }

        // Cell rects
        let num_cols = kozijn.grid.columns.len();
        let mut cell_rects = Vec::new();
        for row_idx in 0..num_rows {
            for (col_idx, col) in kozijn.grid.columns.iter().enumerate() {
                let cx = col_positions[col_idx];
                let cy = row_positions[row_idx];
                cell_rects.push(CellRect {
                    hardware_leaves: Vec::new(),
                    missing_glass_panes: 0,
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
                    fit: None,
                    second_fit: None,
                    opening_contour: None,
                    opening_bounds: None,
                });
            }
        }
        (v_dividers, h_dividers, cell_rects)
    };

    // Keep grid dimensions below the spring line stable when editing a head.
    // Its top row and the mullions supporting it extend into the curved cap.
    let curved_window = |r: &Rect2D, reaches_head: bool| {
        if rows_from_spring && layout_geom.is_none() && reaches_head {
            Rect2D {
                y: fw,
                height: r.y + r.height - fw,
                ..r.clone()
            }
        } else {
            r.clone()
        }
    };
    let curved_frame = frame_curve.as_ref().map(|shape| CurvedFrame {
        outer: shape.outline(0.0),
        day: shape.outline(fw),
    });
    let mut curved_dividers: Vec<_> = frame_curve.as_ref().map_or_else(Vec::new, |shape| {
        v_dividers
            .iter()
            .map(|r| shape.window(fw, &curved_window(r, true)))
            .chain(h_dividers.iter().map(|r| shape.window(fw, r)))
            .collect()
    });
    let mut curved_divider_timber = crate::curved_timber::divider_members(
        &curved_dividers,
        &v_dividers,
        &h_dividers,
        layout_geom.as_ref(),
        kozijn.frame.material.is_profile_system(),
    );
    if let Some(curve) = &curved_frame {
        crate::curved_timber::resolve_system_heads(kozijn, &curve.day, &v_dividers, &h_dividers, &mut curved_divider_timber);
    }
    if layout_geom
        .as_ref()
        .is_some_and(|l| l.leaves.iter().any(|leaf| leaf.vulling.is_buiten()))
    {
        // Even legacy views must not paint an unused splitter through a wall.
        let mut has_stock = vec![false; curved_dividers.len()];
        for part in &curved_divider_timber {
            has_stock[part.source_index + if part.vertical { 0 } else { v_dividers.len() }] = true;
        }
        for (contour, exists) in curved_dividers.iter_mut().zip(has_stock) {
            if !exists {
                contour.segments.clear();
            }
        }
    }
    let cells_aligned = layout_geom
        .as_ref()
        .map_or(true, |g| g.leaves.len() == kozijn.cells.len());
    for cr in &mut cell_rects {
        let cell = kozijn.cells.get(cr.cell_index).filter(|_| cells_aligned);
        let window = curved_window(&cr.rect, cr.row == 0);
        let mount;
        let article_overlap_limit = cell
            .and_then(|c| c.sash_profile.as_ref())
            .and_then(|p| crate::glazing_fit::inward_window_frame(&p.id))
            .filter(|&reference| reference == kozijn.frame.profile.id)
            .map(|_| {
                sash_overlap_limit(&window, fw, &v_dividers, &h_dividers,
                    frame_curve.as_ref().map(|_| curved_divider_timber.as_slice()))
            });
        (cr.fit, cr.second_fit, mount) = crate::glazing_fit::glazing_fits(
            &window, &kozijn.frame, cell, cr.vulling.as_ref(), article_overlap_limit,
        );
        if let Some(shape) = &frame_curve {
            let contour = shape.window(fw, &window);
            cr.opening_bounds = Some(contour.bounds());
            let empty = contour.segments.is_empty();
            cr.opening_contour = Some(contour);
            for (i, fit) in [&mut cr.fit, &mut cr.second_fit].into_iter().enumerate() {
                if fit
                    .as_mut()
                    .is_some_and(|f| empty || !shape.shape_fit(&window, fw, f, mount.as_ref(), i))
                {
                    cr.missing_glass_panes += 1;
                    *fit = None;
                }
            }
            // A paired door in an off-centre segment can retain just one pane.
            if cr.fit.is_none() {
                cr.fit = cr.second_fit.take();
            }
        }
        cr.hardware_leaves = crate::hardware::hardware_leaves(&kozijn.frame, cell, cr);
    }

    // ── Dimension lines (NEN 3576 / GA Kozijn style) ──
    // Level 1 (closest): houtdiktes + vakmaten (stijl, kolommen, stijl)
    // Level 2: dagmaat (inner opening)
    // Level 3 (outermost): buitenwerkse maat (overall)
    let dim_gap = 35.0;
    let dim_start = 20.0;
    let mut dimensions = Vec::new();
    let inner_w = ow - 2.0 * fw;
    let inner_h = oh - 2.0 * fw;
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
                    x1: pair[0],
                    y1: bot_y1,
                    x2: pair[1],
                    y2: bot_y1,
                    label: format!("{:.0}", pair[1] - pair[0]),
                    side: DimensionSide::Bottom,
                });
            }
        } else {
            // Left stijl
            dimensions.push(DimensionLine {
                x1: 0.0,
                y1: bot_y1,
                x2: fw,
                y2: bot_y1,
                label: format!("{:.0}", fw),
                side: DimensionSide::Bottom,
            });
            // Column widths (vakmaten)
            for (i, col) in kozijn.grid.columns.iter().enumerate() {
                let cx = col_positions[i];
                dimensions.push(DimensionLine {
                    x1: cx,
                    y1: bot_y1,
                    x2: cx + col.size,
                    y2: bot_y1,
                    label: format!("{:.0}", col.size),
                    side: DimensionSide::Bottom,
                });
                // Divider width (tussenstijl)
                if i < num_cols - 1 {
                    let dx = cx + col.size;
                    let divider_width = column_divider_widths[i];
                    dimensions.push(DimensionLine {
                        x1: dx,
                        y1: bot_y1,
                        x2: dx + divider_width,
                        y2: bot_y1,
                        label: format!("{:.0}", divider_width),
                        side: DimensionSide::Bottom,
                    });
                }
            }
            // Right stijl
            dimensions.push(DimensionLine {
                x1: ow - fw,
                y1: bot_y1,
                x2: ow,
                y2: bot_y1,
                label: format!("{:.0}", fw),
                side: DimensionSide::Bottom,
            });
        }
    }

    // ── Bottom Level 2: dagmaat ──
    let bot_y2 = oh + dim_start + dim_gap;
    dimensions.push(DimensionLine {
        x1: fw,
        y1: bot_y2,
        x2: ow - fw,
        y2: bot_y2,
        label: format!("{:.0}", inner_w),
        side: DimensionSide::Bottom,
    });

    // ── Bottom Level 3: buitenwerkse maat ──
    let bot_y3 = oh + dim_start + dim_gap * 2.0;
    dimensions.push(DimensionLine {
        x1: 0.0,
        y1: bot_y3,
        x2: ow,
        y2: bot_y3,
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
                    x1: right_x1,
                    y1: pair[0],
                    x2: right_x1,
                    y2: pair[1],
                    label: format!("{:.0}", pair[1] - pair[0]),
                    side: DimensionSide::Right,
                });
            }
        } else {
            // For a day-referenced arch the row chain starts at the crown.
            let top_h = if is_arched && !rows_from_spring {
                fw
            } else {
                top_rect_height
            };
            if top_h > 0.0 {
                dimensions.push(DimensionLine {
                    x1: right_x1,
                    y1: 0.0,
                    x2: right_x1,
                    y2: top_h,
                    label: format!("{:.0}", top_h),
                    side: DimensionSide::Right,
                });
            }
            // Boogpijl (arch rise) for arched frames
            if rows_from_spring {
                dimensions.push(DimensionLine {
                    x1: right_x1,
                    y1: 0.0,
                    x2: right_x1,
                    y2: stile_top_y,
                    label: format!("{:.0}", stile_top_y),
                    side: DimensionSide::Right,
                });
            }
            // Row heights (vakmaten) — displayed sizes (scaled for arched frames)
            for i in 0..num_rows {
                let cy = row_positions[i];
                let size = row_display_sizes[i];
                dimensions.push(DimensionLine {
                    x1: right_x1,
                    y1: cy,
                    x2: right_x1,
                    y2: cy + size,
                    label: format!("{:.0}", size),
                    side: DimensionSide::Right,
                });
                // Divider height (tussendorpel)
                if i < num_rows - 1 {
                    let dy = cy + size;
                    let height = row_divider_sizes[i];
                    dimensions.push(DimensionLine {
                        x1: right_x1,
                        y1: dy,
                        x2: right_x1,
                        y2: dy + height,
                        label: format!("{:.0}", height),
                        side: DimensionSide::Right,
                    });
                }
            }
            // Onderdorpel
            dimensions.push(DimensionLine {
                x1: right_x1,
                y1: oh - fw,
                x2: right_x1,
                y2: oh,
                label: format!("{:.0}", fw),
                side: DimensionSide::Right,
            });
        }
    }

    // ── Right Level 2: dagmaat ──
    let right_x2 = ow + dim_start + dim_gap;
    dimensions.push(DimensionLine {
        x1: right_x2,
        y1: fw,
        x2: right_x2,
        y2: oh - fw,
        label: format!("{:.0}", inner_h),
        side: DimensionSide::Right,
    });

    // ── Right Level 3: buitenwerkse maat ──
    let right_x3 = ow + dim_start + dim_gap * 2.0;
    dimensions.push(DimensionLine {
        x1: right_x3,
        y1: 0.0,
        x2: right_x3,
        y2: oh,
        label: format!("{:.0}", oh),
        side: DimensionSide::Right,
    });

    // Arched frame geometry
    let mut arcs = Vec::new();
    let mut arch_band: Vec<[f64; 2]> = Vec::new();
    if is_arched {
        // Keep existing SVG/export consumers on the same springs and radius.
        if let Some(frame) = &curved_frame {
            for (outline, stroke) in [(&frame.outer, fw), (&frame.day, 1.0)] {
                for segment in &outline.segments {
                    if let crate::curved_contour::Segment::Arc {
                        center,
                        radius,
                        start_angle,
                        sweep_angle,
                        ..
                    } = segment
                    {
                        let start = (-start_angle.to_degrees()).rem_euclid(360.0);
                        arcs.push(Arc2D {
                            cx: center[0],
                            cy: center[1],
                            radius: *radius,
                            start_angle: start,
                            end_angle: start - sweep_angle.to_degrees(),
                            stroke_width: stroke,
                        });
                    }
                }
            }
            for s in &frame.outer.segments {
                if matches!(s, crate::curved_contour::Segment::Arc { .. }) {
                    arch_band.extend((0..=32).map(|i| s.point(i as f64 / 32.0)));
                }
            }
            for s in frame.day.segments.iter().rev() {
                if matches!(s, crate::curved_contour::Segment::Arc { .. }) {
                    arch_band.extend((0..=32).rev().map(|i| s.point(i as f64 / 32.0)));
                }
            }
        }
    } else if kozijn.frame.shape.shape_type == ShapeType::Round {
        let radius = ow.min(oh) / 2.0;
        let cx = ow / 2.0;
        let cy = oh / 2.0;
        // Outer circle (two semicircles, thin stroke — SVG fill handles the frame area)
        arcs.push(Arc2D {
            cx,
            cy,
            radius,
            start_angle: 180.0,
            end_angle: 0.0,
            stroke_width: 2.0,
        });
        arcs.push(Arc2D {
            cx,
            cy,
            radius,
            start_angle: 0.0,
            end_angle: -180.0,
            stroke_width: 2.0,
        });
        // Inner circle (glass opening edge)
        let inner_r = radius - fw;
        if inner_r > 0.0 {
            arcs.push(Arc2D {
                cx,
                cy,
                radius: inner_r,
                start_angle: 180.0,
                end_angle: 0.0,
                stroke_width: 1.5,
            });
            arcs.push(Arc2D {
                cx,
                cy,
                radius: inner_r,
                start_angle: 0.0,
                end_angle: -180.0,
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
            cx,
            cy,
            radius: rx,
            start_angle: 180.0,
            end_angle: 0.0,
            stroke_width: 2.0,
        });
        // Outer ellipse (bottom half)
        arcs.push(Arc2D {
            cx,
            cy,
            radius: rx,
            start_angle: 0.0,
            end_angle: -180.0,
            stroke_width: 2.0,
        });
        // Inner ellipse (top half)
        let inner_rx = rx - fw;
        let inner_ry = ry - fw;
        if inner_rx > 0.0 && inner_ry > 0.0 {
            arcs.push(Arc2D {
                cx,
                cy,
                radius: inner_rx,
                start_angle: 180.0,
                end_angle: 0.0,
                stroke_width: 1.5,
            });
            arcs.push(Arc2D {
                cx,
                cy,
                radius: inner_rx,
                start_angle: 0.0,
                end_angle: -180.0,
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
            [0.0, oh],        // bottom-left
            [ow, oh],         // bottom-right
            [apex_x, apex_y], // apex
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
                trapezoid_inner = points
                    .iter()
                    .map(|p| {
                        let dx = cx - p[0];
                        let dy = cy - p[1];
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist > 0.0 {
                            [p[0] + dx / dist * fw, p[1] + dy / dist * fw]
                        } else {
                            *p
                        }
                    })
                    .collect();
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
        let left_offset = if left_angle_deg >= 89.9 {
            0.0
        } else {
            oh * (90.0 - left_angle_deg).to_radians().tan()
        };
        let right_offset = if right_angle_deg >= 89.9 {
            0.0
        } else {
            oh * (90.0 - right_angle_deg).to_radians().tan()
        };

        // Outer polygon (clockwise from bottom-left)
        trapezoid_outer = vec![
            [0.0, oh],                // bottom-left
            [ow, oh],                 // bottom-right
            [ow - right_offset, 0.0], // top-right
            [left_offset, 0.0],       // top-left
        ];

        // Inner polygon (inside frame members)
        trapezoid_inner = vec![
            [fw, oh - fw],                                           // bottom-left inner
            [ow - fw, oh - fw],                                      // bottom-right inner
            [ow - right_offset - fw + (fw * right_offset / oh), fw], // top-right inner
            [left_offset + fw - (fw * left_offset / oh), fw],        // top-left inner
        ];
    }

    // Profile snapshot (resolved at selection time): sponning/glaslat values
    // ride along in the payload so the frontend no longer hard-codes 17 mm.
    // Fields the snapshot does not carry resolve to the norm per kozijn
    // material (hout 17 mm KVT 12.2 / PVC 20 mm VEKA Glaseinstand / alu 25 mm
    // bibliotheekwaarde — see profile::norm_glasinval). Kozijnen without any
    // snapshot emit no values: old payloads stay byte-identical and the
    // frontend keeps its classic 17 mm fallback.
    let snap = kozijn.frame.profile_snapshot.as_ref();
    let material = &kozijn.frame.material;

    let curved_timber = curved_frame.as_ref().map_or_else(Vec::new, |c| {
        crate::curved_timber::frame_members(&kozijn.frame, c, layout_geom.as_ref())
    });
    let corner_settings = crate::joint::resolved_frame_joints(
        material,
        &kozijn.frame.corner_joints,
        kozijn.frame.joints_configured,
    );
    let arch_joints = crate::curved_timber::fit_timber_tenons(
        kozijn,
        &curved_timber,
        &mut curved_divider_timber,
        &cell_rects,
        &corner_settings,
    );
    KozijnGeometry2D {
        layout_divider_width: kozijn.layout.as_ref().map(|_| divider_width),
        perimeter_stop: crate::perimeter_stop::outline(
            &kozijn.frame,
            frame_curve.as_ref(),
            &trapezoid_outer,
            layout_geom
                .as_ref()
                .is_some_and(|layout| layout.leaves.iter().any(|l| l.vulling.is_buiten())),
        ),
        corner_settings,
        arch_joints,
        curved_timber,
        curved_frame,
        curved_divider_timber,
        curved_dividers,
        outer_rect,
        inner_rect,
        frame_rects,
        frame_polygons,
        h_dividers,
        v_dividers,
        cell_rects,
        dimensions,
        arcs,
        arch_band,
        trapezoid_outer,
        trapezoid_inner,
        sponning_hoogte: snap.map(|s| s.resolved_glasinval(material)),
        glaslat_breedte: snap.and_then(|s| s.glaslat_breedte),
        glaslat_hoogte: snap.map(|s| s.resolved_glaslat_hoogte(material)),
    }
}

/// A frame can receive the full overlap. Adjacent sashes can each use up to
/// half a T-section. A divider outside the curved bay cannot limit its sash.
fn sash_overlap_limit(
    window: &Rect2D,
    frame_width: f64,
    posts: &[Rect2D],
    rails: &[Rect2D],
    curved_parts: Option<&[crate::curved_timber::CurvedDividerMember]>,
) -> f64 {
    let touches = |a: f64, b: f64| (a - b).abs() < 1e-6;
    let post_side = |d: &Rect2D| touches(d.x + d.width, window.x) || touches(d.x, window.x + window.width);
    let rail_side = |d: &Rect2D| touches(d.y + d.height, window.y) || touches(d.y, window.y + window.height);
    if let Some(parts) = curved_parts {
        let envelope = window.inset(-frame_width);
        return parts.iter().filter_map(|p| {
            let (touches_side, width) = if p.vertical {
                let rect = &posts[p.source_index];
                (post_side(rect), rect.width)
            } else {
                let rect = &rails[p.source_index];
                (rail_side(rect), rect.height)
            };
            (touches_side && !p.outline.clone().clipped(&envelope).segments.is_empty()).then_some(width / 2.)
        }).fold(frame_width, f64::min);
    }
    let vertical = posts.iter().filter(|d| {
        post_side(d) && d.y < window.y + window.height && d.y + d.height > window.y
    }).map(|d| d.width / 2.);
    let horizontal = rails.iter().filter(|d| {
        rail_side(d) && d.x < window.x + window.width && d.x + d.width > window.x
    }).map(|d| d.height / 2.);
    vertical.chain(horizontal).fold(frame_width, f64::min)
}

/// Map the free-subdivision layout geometry onto renderable dividers + cells.
/// `cell_index` is the stable depth-first leaf index in the tree (with gaps
/// where `Buiten` leaves sit — those yield no cell); `col`/`row` are grid
/// concepts and stay 0 on this path: consumers key layout cells by
/// `cell_index` and read the vulling straight from the cell.
fn layout_dividers_and_cells(lg: &LayoutGeometry) -> (Vec<Rect2D>, Vec<Rect2D>, Vec<CellRect>) {
    let to_rect = |r: &LayoutRect| Rect2D {
        x: r.x,
        y: r.y,
        width: r.width,
        height: r.height,
    };
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
            hardware_leaves: Vec::new(),
            rect: to_rect(&l.rect),
            col: 0,
            row: 0,
            cell_index: l.leaf_index,
            vulling: Some(l.vulling.clone()),
            missing_glass_panes: 0,
            fit: None,
            second_fit: None,
            opening_contour: None,
            opening_bounds: None,
        })
        .collect();
    (v_dividers, h_dividers, cell_rects)
}

/// Frame drawn as four member polygons for a plain rectangular frame, honoring
/// per-corner verstek (miter) and through-member from the shared corner model.
/// Members tile the frame border exactly, so verstek corners meet on their 45°
/// diagonal and butt corners let the through member own the corner. Order is
/// [top, bottom, left, right], matching `frame_rects`.
fn rectangular_frame_member_polygons(
    ow: f64,
    oh: f64,
    fw: f64,
    corners: &[crate::joint::CornerModel; 4],
) -> Vec<Vec<[f64; 2]>> {
    use crate::joint::{CornerModel, ThroughMember};
    // At a corner: outer corner (ox,oy), inner corner (ix,iy). Each end returns
    // (outer_point, inner_point). Verstek → the 45° diagonal; otherwise the
    // through member runs to the outer edge and the other stops at the inset.
    let rail_end = |c: &CornerModel, ox: f64, oy: f64, ix: f64, iy: f64| -> ([f64; 2], [f64; 2]) {
        if c.verstek {
            ([ox, oy], [ix, iy])
        } else if c.through == ThroughMember::Dorpel {
            ([ox, oy], [ox, iy]) // rail runs through to the outer x
        } else {
            ([ix, oy], [ix, iy]) // stile through → rail stops at the inset x
        }
    };
    let stile_end = |c: &CornerModel, ox: f64, oy: f64, ix: f64, iy: f64| -> ([f64; 2], [f64; 2]) {
        if c.verstek {
            ([ox, oy], [ix, iy])
        } else if c.through == ThroughMember::Stijl {
            ([ox, oy], [ix, oy]) // stile runs through to the outer y
        } else {
            ([ox, iy], [ix, iy]) // dorpel through → stile stops at the inset y
        }
    };

    // Top rail: TL=corners[0], TR=corners[1].
    let (tl_o, tl_i) = rail_end(&corners[0], 0.0, 0.0, fw, fw);
    let (tr_o, tr_i) = rail_end(&corners[1], ow, 0.0, ow - fw, fw);
    let top = vec![tl_o, tr_o, tr_i, tl_i];

    // Bottom rail: BL=corners[2], BR=corners[3].
    let (bl_o, bl_i) = rail_end(&corners[2], 0.0, oh, fw, oh - fw);
    let (br_o, br_i) = rail_end(&corners[3], ow, oh, ow - fw, oh - fw);
    let bottom = vec![bl_o, br_o, br_i, bl_i];

    // Left stile: TL=corners[0], BL=corners[2].
    let (lt_o, lt_i) = stile_end(&corners[0], 0.0, 0.0, fw, fw);
    let (lb_o, lb_i) = stile_end(&corners[2], 0.0, oh, fw, oh - fw);
    let left = vec![lt_o, lb_o, lb_i, lt_i];

    // Right stile: TR=corners[1], BR=corners[3].
    let (rt_o, rt_i) = stile_end(&corners[1], ow, 0.0, ow - fw, fw);
    let (rb_o, rb_i) = stile_end(&corners[3], ow, oh, ow - fw, oh - fw);
    let right = vec![rt_o, rb_o, rb_i, rt_i];

    vec![top, bottom, left, right]
}

/// Stepped frame mass for a melkmeisje (layout with `Buiten` leaves): one ring
/// per real (non-`Buiten`) vak, expanded outward by the frame width and clamped
/// to the outer bounds. Their union is the stepped outline — the outline steps
/// up around the raised side light and there is no frame (sill/stile) over the
/// `Buiten` wall zone. Mirrors LayoutProtoView's per-vak frame-mass approach.
fn stepped_frame_polygons(lg: &LayoutGeometry, ow: f64, oh: f64, fw: f64) -> Vec<Vec<[f64; 2]>> {
    let mut polys = Vec::new();
    for l in &lg.leaves {
        if l.vulling.is_buiten() {
            continue;
        }
        let r = l.rect;
        let x0 = (r.x - fw).max(0.0);
        let y0 = (r.y - fw).max(0.0);
        let x1 = (r.x + r.width + fw).min(ow);
        let y1 = (r.y + r.height + fw).min(oh);
        polys.push(vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]]);
    }
    polys
}

/// Boundaries of the level-1 dimension chain along one axis, derived from the
/// layout tree: the fixed frame edges (0, fw, total-fw, total) plus the edges
/// of every non-`Buiten` leaf that touches the chain's inner edge — the bottom
/// of the opening for the horizontal chain, the right side for the vertical
/// one. Consecutive boundaries form the chain segments (stijl/dorpel, vak,
/// divider or unanchored rest). Mirrors `layoutChain()` in the frontend
/// (ui/src/components/editor/KozijnCanvas.svelte) — keep the two in sync,
/// including the 1.0 mm touch tolerance and 0.5 mm dedup.
fn layout_chain_bounds(
    lg: &LayoutGeometry,
    horizontal: bool,
    ow: f64,
    oh: f64,
    fw: f64,
) -> Vec<f64> {
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

/// Explicitly reset a grid to one cell. Changing its outer shape preserves the
/// grid; older callers can still request a one-cell layout separately.
pub fn normalize_grid_single_cell(kozijn: &mut Kozijn) {
    let single = |size| {
        vec![crate::kozijn::GridDivision {
            size,
            divider_profile: None,
            divider_width: None,
            divider_depth: None,
        }]
    };
    kozijn.grid.columns = single(kozijn.frame.outer_width - 2.0 * kozijn.frame.frame_width);
    kozijn.grid.rows = single(kozijn.frame.outer_height - 2.0 * kozijn.frame.frame_width);
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
        // The stored grid itself keeps the add_row result (geometry is a
        // view): 600 scaled for the reserved tussendorpel, not overwritten
        // by the arch scaling.
        let inner_h = oh - 2.0 * fw; // 1366
        assert!((k.grid.rows[0].size - 600.0 * (inner_h - fw) / inner_h).abs() < 1e-6);
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
    fn round_grid_keeps_its_four_clipped_panes() {
        let mut k = Kozijn::new("Test", "K01", 900.0, 900.0);
        k.add_column(300.0);
        k.add_row(300.0);
        assert_eq!(k.cells.len(), 4);
        k.frame.shape = FrameShape {
            shape_type: ShapeType::Round,
            ..FrameShape::default()
        };
        let g = compute_2d_geometry(&k);
        assert_eq!(g.cell_rects.len(), 4);
        assert_eq!(g.curved_dividers.len(), 2);
        for cr in &g.cell_rects {
            let opening = cr.opening_contour.as_ref().unwrap();
            assert!(opening.area() > 0. && opening.area() < cr.rect.width * cr.rect.height);
            assert!(cr.fit.as_ref().unwrap().contours.is_some());
        }
    }

    // ── Corner-joint aware frame (issue 7) ──

    #[test]
    fn wood_default_draws_dorpels_full_width() {
        // Wood dorpel-through default (NL-timmerpraktijk, KVT katern 15:
        // dorpels doorlopend, stijlen ertussen gepend): rails run the full
        // outer width, stiles fit between them (agreeing with the saw list),
        // and no polygons.
        let k = Kozijn::new("Test", "K01", 1200.0, 1500.0);
        let g = compute_2d_geometry(&k);
        let (ow, oh, fw) = (1200.0, 1500.0, 67.0);
        // frame_rects order: [top, bottom, left, right]
        let top = &g.frame_rects[0];
        let bottom = &g.frame_rects[1];
        assert!((top.x - 0.0).abs() < 1e-6 && (top.width - ow).abs() < 1e-6);
        assert!((bottom.x - 0.0).abs() < 1e-6 && (bottom.width - ow).abs() < 1e-6);
        assert!((bottom.y - (oh - fw)).abs() < 1e-6);
        // Stiles sit between the rails.
        let left = &g.frame_rects[2];
        let right = &g.frame_rects[3];
        assert!((left.y - fw).abs() < 1e-6 && (left.height - (oh - 2.0 * fw)).abs() < 1e-6);
        assert!(
            (right.x - (ow - fw)).abs() < 1e-6 && (right.height - (oh - 2.0 * fw)).abs() < 1e-6
        );
        // No miter/step polygons for a plain butt frame — payload byte-compat.
        assert!(g.frame_polygons.is_empty());
        let json = serde_json::to_string(&g).unwrap();
        assert!(
            !json.contains("framePolygons"),
            "plain frame grew a key: {}",
            json
        );
    }

    #[test]
    fn configured_stijl_through_still_draws_stiles_full_height() {
        // An explicitly configured stijl-through pen/slis (non-default pen
        // length so it counts as configured) keeps the stijl-doorlopend
        // drawing — only the untouched DEFAULT flipped to dorpel-through.
        let mut k = Kozijn::new("Test", "K01", 1200.0, 1500.0);
        let pen = crate::joint::Joint {
            joint_type: crate::joint::JointType::PenSlis,
            through_member: crate::joint::ThroughMember::Stijl,
            angle: 90.0,
            pen_length: 30.0,
        };
        k.frame.corner_joints = vec![pen.clone(), pen.clone(), pen.clone(), pen];
        let g = compute_2d_geometry(&k);
        let (ow, oh, fw) = (1200.0, 1500.0, 67.0);
        let left = &g.frame_rects[2];
        assert!((left.height - oh).abs() < 1e-6 && (left.x - 0.0).abs() < 1e-6);
        let top = &g.frame_rects[0];
        assert!((top.x - fw).abs() < 1e-6 && (top.width - (ow - 2.0 * fw)).abs() < 1e-6);
        assert!(g.frame_polygons.is_empty());
    }

    #[test]
    fn aluminum_default_keeps_rects_and_adds_miter_polygons() {
        // Alu/PVC default = verstek: rects stay the historic dorpel-through set
        // (byte-identical), and four mitered member polygons are added.
        let mut k = Kozijn::new("Test", "K01", 1200.0, 1500.0);
        k.frame.material = crate::kozijn::Material::Aluminum;
        let g = compute_2d_geometry(&k);
        let (ow, fw) = (1200.0, 67.0);
        // Historic top rail spans the full width.
        let top = &g.frame_rects[0];
        assert!((top.x - 0.0).abs() < 1e-6 && (top.width - ow).abs() < 1e-6);
        assert_eq!(g.frame_polygons.len(), 4);
        assert!(g.frame_polygons.iter().all(|p| p.len() == 4));
        // Top rail miter trapezoid: outer edge full width, inner edge inset fw.
        let top_poly = &g.frame_polygons[0];
        assert!((top_poly[0][0] - 0.0).abs() < 1e-6 && (top_poly[0][1] - 0.0).abs() < 1e-6);
        assert!((top_poly[1][0] - ow).abs() < 1e-6 && (top_poly[1][1] - 0.0).abs() < 1e-6);
        assert!((top_poly[2][0] - (ow - fw)).abs() < 1e-6 && (top_poly[2][1] - fw).abs() < 1e-6);
        assert!((top_poly[3][0] - fw).abs() < 1e-6 && (top_poly[3][1] - fw).abs() < 1e-6);
        let json = serde_json::to_string(&g).unwrap();
        assert!(json.contains("framePolygons"));
    }

    #[test]
    fn configured_verstek_on_wood_emits_polygons() {
        let mut k = Kozijn::new("Test", "K01", 1200.0, 1500.0);
        let verstek = crate::joint::Joint {
            joint_type: crate::joint::JointType::Verstek,
            through_member: crate::joint::ThroughMember::Stijl,
            angle: 45.0,
            pen_length: 0.0,
        };
        k.frame.corner_joints = vec![verstek.clone(), verstek.clone(), verstek.clone(), verstek];
        let g = compute_2d_geometry(&k);
        assert_eq!(g.frame_polygons.len(), 4);
        // Left stile miter trapezoid: [(0,0),(0,oh),(fw,oh-fw),(fw,fw)]
        let (oh, fw) = (1500.0, 67.0);
        let left = &g.frame_polygons[2];
        assert!((left[0][0] - 0.0).abs() < 1e-6 && (left[0][1] - 0.0).abs() < 1e-6);
        assert!((left[1][0] - 0.0).abs() < 1e-6 && (left[1][1] - oh).abs() < 1e-6);
        assert!((left[2][0] - fw).abs() < 1e-6 && (left[2][1] - (oh - fw)).abs() < 1e-6);
        assert!((left[3][0] - fw).abs() < 1e-6 && (left[3][1] - fw).abs() < 1e-6);
    }

    #[test]
    fn special_shapes_keep_historic_frame_rects() {
        // Arched frame: frame_rects unchanged (top hidden, stiles from spring
        // line), no polygons.
        let k = arched_kozijn(1200.0, 1500.0, 300.0);
        let g = compute_2d_geometry(&k);
        assert!(g.frame_polygons.is_empty());
        // Top rect hidden (arc replaces it).
        assert!((g.frame_rects[0].height - 0.0).abs() < 1e-6);
    }

    // ── Free-subdivision layout as the geometry source ──

    /// Two-vaks kozijn: fw + vak + tussenstijl + vak + fw over the width.
    fn layout_kozijn_2vaks() -> Kozijn {
        let mut k = Kozijn::new("Test", "K01", 1200.0, 1500.0);
        k.layout = Some(split_row(vec![
            VakChild {
                size: 1.0,
                node: glas(),
            },
            VakChild {
                size: 1.0,
                node: raam("draaikiep"),
            },
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
        assert!(matches!(c0.vulling, Some(Vakvulling::Glas { .. })));
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
        assert_eq!(
            chain.len(),
            5,
            "stijl+vak+tussenstijl+vak+stijl: {:?}",
            chain
        );
        assert!((chain[0].x1 - 0.0).abs() < 1e-6);
        assert!((chain.last().unwrap().x2 - ow).abs() < 1e-6);
        for w in chain.windows(2) {
            assert!(
                (w[0].x2 - w[1].x1).abs() < 1e-6,
                "chain gap: {:?} → {:?}",
                w[0],
                w[1]
            );
        }
        let sum: f64 = chain.iter().map(|d| d.x2 - d.x1).sum();
        assert!((sum - ow).abs() < 1e-6, "chain sums to buitenmaat");
        // Interior segments (without the two stijlen) sum to the dagmaat
        let interior: f64 = chain[1..chain.len() - 1].iter().map(|d| d.x2 - d.x1).sum();
        assert!(
            (interior - (ow - 2.0 * fw)).abs() < 1e-6,
            "interior = binnenmaat"
        );

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
        let deur_w = (inner_w - fw) * 900.0 / 1400.0;
        let glas_h = (inner_h - fw) * 1200.0 / 2100.0;

        // Buiten leaf yields no cell rect; leaf indices keep their gaps
        assert_eq!(g.cell_rects.len(), 2);
        assert_eq!(g.cell_rects[0].cell_index, 0); // deur (full height)
        assert_eq!(g.cell_rects[1].cell_index, 1); // zijlicht glas (buiten = 2)
        assert!(matches!(
            g.cell_rects[0].vulling,
            Some(Vakvulling::Deur { .. })
        ));
        // The tussenstijl is clipped to the side light (borders buiten below);
        // the side-light onderdorpel borders buiten directly, so it drops out.
        assert_eq!(g.v_dividers.len(), 1);
        assert!(
            g.v_dividers[0].height < inner_h - 1e-6,
            "tussenstijl must be clipped"
        );
        assert_eq!(g.h_dividers.len(), 0);
        // Stepped frame outline: one ring per real vak (deur + side light).
        assert_eq!(g.frame_polygons.len(), 2);
        assert!(g.frame_polygons.iter().all(|p| p.len() == 4));

        // Bottom chain: zijlicht doesn't reach the bottom of the opening, so
        // its area stays one unanchored segment — chain still closes to ow
        let bot_y1 = oh + 20.0;
        let chain: Vec<_> = g
            .dimensions
            .iter()
            .filter(|d| matches!(d.side, DimensionSide::Bottom) && (d.y1 - bot_y1).abs() < 1e-6)
            .collect();
        assert_eq!(chain.len(), 4, "stijl+deur+rest+stijl: {:?}", chain);
        assert!((chain[1].x2 - chain[1].x1 - deur_w).abs() < 1e-6);
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

    // ── Profile snapshot → model serde + geometry payload ──

    #[test]
    fn profile_snapshot_flows_into_geometry_payload() {
        let mut k = Kozijn::new("Test", "K01", 1200.0, 1500.0);
        k.frame.profile_snapshot = Some(crate::profile::ProfileSnapshot {
            sponning_diepte: Some(24.0),
            sponning_hoogte: Some(20.0),
            glaslat_breedte: Some(15.0),
            aanzichtbreedte: Some(54.0),
            glasinval: None,
            glaslat_hoogte: None,
            ..Default::default()
        });
        let g = compute_2d_geometry(&k);
        assert_eq!(g.sponning_hoogte, Some(20.0));
        assert_eq!(g.glaslat_breedte, Some(15.0));
        // Wood fallback bead height follows KVT 12.3.2: >= sponninghoogte.
        assert_eq!(g.glaslat_hoogte, Some(20.0));
        // camelCase on the wire, like the rest of the payload
        let json = serde_json::to_string(&g).unwrap();
        assert!(
            json.contains("\"sponningHoogte\":20.0"),
            "payload: {}",
            json
        );
        assert!(
            json.contains("\"glaslatBreedte\":15.0"),
            "payload: {}",
            json
        );
        assert!(json.contains("\"glaslatHoogte\":20.0"), "payload: {}", json);
    }

    #[test]
    fn snapshot_missing_values_resolve_to_material_norms() {
        // A snapshot without sponning data (e.g. imported profile) resolves
        // to the norm for the kozijn material: hout 17 (KVT 12.2) / PVC 20
        // (VEKA Glaseinstand) / alu 25 (bibliotheekwaarde CS77/AWS), bead
        // fallback hout 17 (KVT 12.3.2) / PVC & alu 25 (kliklat).
        let mut k = Kozijn::new("Test", "K01", 1200.0, 1500.0);
        k.frame.profile_snapshot = Some(crate::profile::ProfileSnapshot::default());

        let g = compute_2d_geometry(&k); // default material: wood (meranti)
        assert_eq!(g.sponning_hoogte, Some(17.0));
        assert_eq!(g.glaslat_hoogte, Some(17.0));

        k.frame.material = crate::kozijn::Material::Pvc;
        let g = compute_2d_geometry(&k);
        assert_eq!(g.sponning_hoogte, Some(20.0));
        assert_eq!(g.glaslat_hoogte, Some(25.0));

        k.frame.material = crate::kozijn::Material::Aluminum;
        let g = compute_2d_geometry(&k);
        assert_eq!(g.sponning_hoogte, Some(25.0));
        assert_eq!(g.glaslat_hoogte, Some(25.0));

        // PVC met Falzhöhe én Glaseinstand (VEKA 82 MD: 28/20): de tekening
        // toont de glasinval, niet de Falzhöhe.
        k.frame.material = crate::kozijn::Material::Pvc;
        k.frame.profile_snapshot = Some(crate::profile::ProfileSnapshot {
            sponning_hoogte: Some(28.0),
            glasinval: Some(20.0),
            ..Default::default()
        });
        let g = compute_2d_geometry(&k);
        assert_eq!(g.sponning_hoogte, Some(20.0));
    }

    #[test]
    fn profile_snapshot_serde_roundtrip_and_partial_json() {
        let mut k = Kozijn::new("Test", "K01", 1000.0, 1000.0);
        k.frame.profile_snapshot = Some(crate::profile::ProfileSnapshot {
            sponning_diepte: Some(24.0),
            sponning_hoogte: Some(17.0),
            glaslat_breedte: None,
            aanzichtbreedte: Some(54.0),
            glasinval: None,
            glaslat_hoogte: Some(17.0),
            ..Default::default()
        });
        let json = serde_json::to_string(&k).unwrap();
        assert!(json.contains("\"profileSnapshot\":{"), "model: {}", json);
        let back: Kozijn = serde_json::from_str(&json).unwrap();
        let s = back
            .frame
            .profile_snapshot
            .expect("snapshot survives roundtrip");
        assert_eq!(s.sponning_diepte, Some(24.0));
        assert_eq!(s.sponning_hoogte, Some(17.0));
        assert_eq!(s.glaslat_breedte, None);
        assert_eq!(s.aanzichtbreedte, Some(54.0));
        assert_eq!(s.glasinval, None);
        assert_eq!(s.glaslat_hoogte, Some(17.0));

        // Frontend-shaped payloads: nulls, integers and missing keys all parse
        // (old snapshots without the 2026-07 fields included)
        let js = r#"{"sponningHoogte":20,"glaslatBreedte":null}"#;
        let s2: crate::profile::ProfileSnapshot = serde_json::from_str(js).unwrap();
        assert_eq!(s2.sponning_hoogte, Some(20.0));
        assert_eq!(s2.sponning_diepte, None);
        assert_eq!(s2.glaslat_breedte, None);
        assert_eq!(s2.aanzichtbreedte, None);
        assert_eq!(s2.glasinval, None);
        assert_eq!(s2.glaslat_hoogte, None);

        // New frontend fields parse too
        let js3 = r#"{"sponningHoogte":28,"glasinval":20,"glaslatHoogte":25}"#;
        let s3: crate::profile::ProfileSnapshot = serde_json::from_str(js3).unwrap();
        assert_eq!(s3.sponning_hoogte, Some(28.0));
        assert_eq!(s3.glasinval, Some(20.0));
        assert_eq!(s3.glaslat_hoogte, Some(25.0));
    }

    #[test]
    fn no_snapshot_keeps_model_and_payload_byte_compatible() {
        // Regression: kozijnen without a snapshot behave exactly as before —
        // no new keys in the model JSON or the geometry payload, and old JSON
        // (without the key) still deserializes to None.
        let k = Kozijn::new("Test", "K01", 1200.0, 1500.0);
        assert!(k.frame.profile_snapshot.is_none());

        let g = compute_2d_geometry(&k);
        assert_eq!(g.sponning_hoogte, None);
        assert_eq!(g.glaslat_breedte, None);
        assert_eq!(g.glaslat_hoogte, None);
        let gj = serde_json::to_string(&g).unwrap();
        assert!(!gj.contains("sponningHoogte") && !gj.contains("glaslatBreedte"));
        assert!(!gj.contains("glaslatHoogte"));

        let kj = serde_json::to_string(&k).unwrap();
        assert!(
            !kj.contains("profileSnapshot"),
            "model must not grow a key: {}",
            kj
        );
        let back: Kozijn = serde_json::from_str(&kj).unwrap();
        assert!(back.frame.profile_snapshot.is_none());
    }

    #[test]
    fn grid_path_without_layout_is_unchanged() {
        // Regression: a kozijn without layout keeps the grid path — geometry
        // values, dim chain and serialized payload. add_column reserves the
        // tussenstijl width proportionally (the historical overlap quirk is
        // fixed), so the level-1 chain closes exactly on the buitenmaat.
        let mut k = Kozijn::new("Test", "K01", 1200.0, 1500.0);
        k.add_column(400.0);
        assert!(k.layout.is_none());
        let g = compute_2d_geometry(&k);

        // 400/666 split of the 1066 binnenmaat, minus the 67 tussenstijl.
        let left = 400.0 * (1066.0 - 67.0) / 1066.0; // ≈ 374.86
        let right = 666.0 * (1066.0 - 67.0) / 1066.0; // ≈ 624.14
        assert_eq!(g.cell_rects.len(), 2);
        let c0 = &g.cell_rects[0];
        let c1 = &g.cell_rects[1];
        assert_eq!((c0.col, c0.row, c0.cell_index), (0, 0, 0));
        assert_eq!((c1.col, c1.row, c1.cell_index), (1, 0, 1));
        assert!((c0.rect.x - 67.0).abs() < 1e-6 && (c0.rect.width - left).abs() < 1e-6);
        assert!((c1.rect.x - (134.0 + left)).abs() < 1e-6 && (c1.rect.width - right).abs() < 1e-6);
        // Last vak ends exactly at the right stijl (no overlap anymore).
        assert!((c1.rect.x + c1.rect.width - (1200.0 - 67.0)).abs() < 1e-6);
        assert_eq!(g.v_dividers.len(), 1);
        assert!((g.v_dividers[0].x - (67.0 + left)).abs() < 1e-6);

        // Level-1 chain: stijl + vak + tussenstijl + vak + stijl sums to the
        // buitenwerkse maat (was 1267 ≠ 1200 with the old overlap quirk).
        let bot_y1 = 1500.0 + 20.0;
        let chain: Vec<_> = g
            .dimensions
            .iter()
            .filter(|d| matches!(d.side, DimensionSide::Bottom) && (d.y1 - bot_y1).abs() < 1e-6)
            .collect();
        let sum: f64 = chain.iter().map(|d| d.x2 - d.x1).sum();
        assert!(
            (sum - 1200.0).abs() < 1e-6,
            "level-1 chain sums to {} != 1200",
            sum
        );
        let labels: Vec<&str> = chain.iter().map(|d| d.label.as_str()).collect();
        assert_eq!(labels, vec!["67", "375", "67", "624", "67"]);

        // No vulling on the grid path, and the payload stays byte-identical:
        // the optional field is skipped entirely when absent
        assert!(g.cell_rects.iter().all(|c| c.vulling.is_none()));
        let json = serde_json::to_string(&g).unwrap();
        assert!(
            !json.contains("vulling"),
            "grid payload must not grow a vulling key"
        );
    }
}
