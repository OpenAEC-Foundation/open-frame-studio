use serde::{Deserialize, Serialize};

use crate::vliesgevel::Vliesgevel;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VliesgevelGeometry2D {
    pub overall_width: f64,
    pub overall_height: f64,
    pub mullion_rects: Vec<Rect>,
    pub transom_rects: Vec<Rect>,
    pub panel_rects: Vec<PanelRect>,
    pub dimensions: Vec<DimensionLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub col: usize,
    pub row: usize,
    pub panel_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DimensionLine {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub label: String,
    pub offset: f64,
}

pub fn compute_vliesgevel_2d(vg: &Vliesgevel) -> VliesgevelGeometry2D {
    let mw = vg.mullion_width;
    let tw = vg.transom_width;

    // Mullion rectangles (full height)
    let mullion_rects: Vec<Rect> = vg.mullions.iter().map(|m| Rect {
        x: m.x_position - mw / 2.0,
        y: 0.0,
        width: mw,
        height: vg.overall_height,
    }).collect();

    // Transom rectangles (span between mullions / edges)
    let transom_rects: Vec<Rect> = vg.transoms.iter().map(|t| Rect {
        x: 0.0,
        y: t.y_position - tw / 2.0,
        width: vg.overall_width,
        height: tw,
    }).collect();

    // Panel rectangles
    let col_zones = vg.column_zones();
    let row_zones = vg.row_zones();
    let mut panel_rects = Vec::new();

    for panel in &vg.panels {
        if panel.col < col_zones.len() && panel.row < row_zones.len() {
            let (x1, x2) = col_zones[panel.col];
            let (y1, y2) = row_zones[panel.row];
            panel_rects.push(PanelRect {
                x: x1,
                y: y1,
                width: (x2 - x1).max(0.0),
                height: (y2 - y1).max(0.0),
                col: panel.col,
                row: panel.row,
                panel_type: format!("{:?}", panel.panel_type),
            });
        }
    }

    // Dimension lines
    let mut dimensions = Vec::new();
    let dim_offset = 40.0;

    // Overall width
    dimensions.push(DimensionLine {
        x1: 0.0, y1: -dim_offset,
        x2: vg.overall_width, y2: -dim_offset,
        label: format!("{:.0}", vg.overall_width),
        offset: dim_offset,
    });

    // Overall height
    dimensions.push(DimensionLine {
        x1: -dim_offset, y1: 0.0,
        x2: -dim_offset, y2: vg.overall_height,
        label: format!("{:.0}", vg.overall_height),
        offset: dim_offset,
    });

    // Mullion spacings (top dimension line)
    let mut prev_x = 0.0;
    for m in &vg.mullions {
        let spacing = m.x_position - prev_x;
        dimensions.push(DimensionLine {
            x1: prev_x, y1: -dim_offset * 2.0,
            x2: m.x_position, y2: -dim_offset * 2.0,
            label: format!("{:.0}", spacing),
            offset: dim_offset * 2.0,
        });
        prev_x = m.x_position;
    }
    let last_spacing = vg.overall_width - prev_x;
    dimensions.push(DimensionLine {
        x1: prev_x, y1: -dim_offset * 2.0,
        x2: vg.overall_width, y2: -dim_offset * 2.0,
        label: format!("{:.0}", last_spacing),
        offset: dim_offset * 2.0,
    });

    VliesgevelGeometry2D {
        overall_width: vg.overall_width,
        overall_height: vg.overall_height,
        mullion_rects,
        transom_rects,
        panel_rects,
        dimensions,
    }
}
