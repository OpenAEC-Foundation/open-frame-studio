use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::hardware::HardwareSet;
use crate::kozijn::{Glazing, Material, OpeningDirection, PanelType, Placement};
use crate::profile::ProfileRef;

/// A curtain wall (vliesgevel) — non-load-bearing facade hung from the building structure.
/// Unlike a Kozijn, it has no outer frame border; mullions and transoms define the grid directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vliesgevel {
    pub id: Uuid,
    pub name: String,
    pub mark: String,
    /// Overall facade width in mm
    pub overall_width: f64,
    /// Overall facade height in mm
    pub overall_height: f64,
    pub material: Material,
    /// Default profile for mullions (vertical members)
    pub mullion_profile: ProfileRef,
    /// Default profile for transoms (horizontal members)
    pub transom_profile: ProfileRef,
    /// Profile face width for mullions in mm
    pub mullion_width: f64,
    /// Profile face width for transoms in mm
    pub transom_width: f64,
    /// Vertical member lines (mullions), sorted by x_position
    pub mullions: Vec<MullionLine>,
    /// Horizontal member lines (transoms), sorted by y_position
    pub transoms: Vec<TransomLine>,
    /// Panels filling the grid cells (row-major: row * num_cols + col)
    pub panels: Vec<CurtainPanel>,
    #[serde(default)]
    pub placement: Placement,
}

/// A vertical mullion line in the curtain wall grid.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MullionLine {
    /// X position in mm from the left edge
    pub x_position: f64,
    /// Override profile (None = use default mullion_profile)
    pub profile: Option<ProfileRef>,
    /// Anchor points connecting to building structure
    #[serde(default)]
    pub anchors: Vec<AnchorPoint>,
}

/// A horizontal transom line in the curtain wall grid.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransomLine {
    /// Y position in mm from the bottom edge
    pub y_position: f64,
    /// Override profile (None = use default transom_profile)
    pub profile: Option<ProfileRef>,
    /// Anchor points
    #[serde(default)]
    pub anchors: Vec<AnchorPoint>,
}

/// An anchor/fixing point connecting mullion/transom to building structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnchorPoint {
    /// Position along the member in mm
    pub position_along: f64,
    pub anchor_type: AnchorType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnchorType {
    Spigot,
    Bolted,
    Welded,
    Bracket,
}

/// A single panel in the curtain wall grid.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurtainPanel {
    /// Column index (0 = leftmost zone)
    pub col: usize,
    /// Row index (0 = bottom zone)
    pub row: usize,
    pub panel_type: CurtainPanelType,
    pub glazing: Option<Glazing>,
    /// For operable panels (rare in curtain walls)
    #[serde(default)]
    pub opening_direction: Option<OpeningDirection>,
    #[serde(default)]
    pub hardware_set: Option<HardwareSet>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CurtainPanelType {
    Glass,
    OpaquePanel,
    SpandrelPanel,
    Ventilation,
    Door,
    OpenableWindow,
}

impl CurtainPanelType {
    pub fn label_nl(&self) -> &'static str {
        match self {
            Self::Glass => "Glas",
            Self::OpaquePanel => "Dicht paneel",
            Self::SpandrelPanel => "Borstwering",
            Self::Ventilation => "Ventilatie",
            Self::Door => "Deur",
            Self::OpenableWindow => "Draairaam",
        }
    }

    pub fn is_glazed(&self) -> bool {
        matches!(self, Self::Glass | Self::OpenableWindow)
    }

    pub fn is_operable(&self) -> bool {
        matches!(self, Self::Door | Self::OpenableWindow)
    }
}

impl Default for CurtainPanel {
    fn default() -> Self {
        Self {
            col: 0,
            row: 0,
            panel_type: CurtainPanelType::Glass,
            glazing: Some(Glazing::default()),
            opening_direction: None,
            hardware_set: None,
        }
    }
}

impl Vliesgevel {
    /// Number of vertical zones (columns) = mullions.len() + 1
    pub fn num_cols(&self) -> usize {
        self.mullions.len() + 1
    }

    /// Number of horizontal zones (rows) = transoms.len() + 1
    pub fn num_rows(&self) -> usize {
        self.transoms.len() + 1
    }

    /// Get the X boundaries of each column zone.
    /// Returns (start_x, end_x) pairs for each column.
    pub fn column_zones(&self) -> Vec<(f64, f64)> {
        let mut zones = Vec::new();
        let mut x = 0.0;
        for m in &self.mullions {
            zones.push((x, m.x_position - self.mullion_width / 2.0));
            x = m.x_position + self.mullion_width / 2.0;
        }
        zones.push((x, self.overall_width));
        zones
    }

    /// Get the Y boundaries of each row zone.
    /// Returns (start_y, end_y) pairs for each row.
    pub fn row_zones(&self) -> Vec<(f64, f64)> {
        let mut zones = Vec::new();
        let mut y = 0.0;
        for t in &self.transoms {
            zones.push((y, t.y_position - self.transom_width / 2.0));
            y = t.y_position + self.transom_width / 2.0;
        }
        zones.push((y, self.overall_height));
        zones
    }

    /// Get panel at grid position
    pub fn panel_at(&self, col: usize, row: usize) -> Option<&CurtainPanel> {
        self.panels.iter().find(|p| p.col == col && p.row == row)
    }

    /// Get mutable panel at grid position
    pub fn panel_at_mut(&mut self, col: usize, row: usize) -> Option<&mut CurtainPanel> {
        self.panels.iter_mut().find(|p| p.col == col && p.row == row)
    }

    /// Rebuild panels after grid changes.
    /// Preserves existing panels where possible, adds defaults for new cells.
    pub fn rebuild_panels(&mut self) {
        let nc = self.num_cols();
        let nr = self.num_rows();
        let mut new_panels = Vec::with_capacity(nc * nr);

        for row in 0..nr {
            for col in 0..nc {
                if let Some(existing) = self.panels.iter().find(|p| p.col == col && p.row == row) {
                    new_panels.push(existing.clone());
                } else {
                    new_panels.push(CurtainPanel {
                        col,
                        row,
                        ..CurtainPanel::default()
                    });
                }
            }
        }
        self.panels = new_panels;
    }

    /// Add a mullion at the given x position, maintaining sorted order.
    pub fn add_mullion(&mut self, x_position: f64) {
        if x_position > self.mullion_width && x_position < self.overall_width - self.mullion_width {
            let line = MullionLine {
                x_position,
                profile: None,
                anchors: vec![],
            };
            let idx = self.mullions.iter()
                .position(|m| m.x_position > x_position)
                .unwrap_or(self.mullions.len());
            self.mullions.insert(idx, line);
            self.rebuild_panels();
        }
    }

    /// Add a transom at the given y position, maintaining sorted order.
    pub fn add_transom(&mut self, y_position: f64) {
        if y_position > self.transom_width && y_position < self.overall_height - self.transom_width {
            let line = TransomLine {
                y_position,
                profile: None,
                anchors: vec![],
            };
            let idx = self.transoms.iter()
                .position(|t| t.y_position > y_position)
                .unwrap_or(self.transoms.len());
            self.transoms.insert(idx, line);
            self.rebuild_panels();
        }
    }

    /// Remove a mullion by index.
    pub fn remove_mullion(&mut self, idx: usize) {
        if idx < self.mullions.len() {
            self.mullions.remove(idx);
            self.rebuild_panels();
        }
    }

    /// Remove a transom by index.
    pub fn remove_transom(&mut self, idx: usize) {
        if idx < self.transoms.len() {
            self.transoms.remove(idx);
            self.rebuild_panels();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kozijn::{Material, WoodType};

    fn make_test_vliesgevel() -> Vliesgevel {
        Vliesgevel {
            id: Uuid::new_v4(),
            name: "Test vliesgevel".into(),
            mark: "VG01".into(),
            overall_width: 6000.0,
            overall_height: 3600.0,
            material: Material::Aluminum,
            mullion_profile: ProfileRef::default_frame(),
            transom_profile: ProfileRef::default_frame(),
            mullion_width: 50.0,
            transom_width: 50.0,
            mullions: vec![
                MullionLine { x_position: 1500.0, profile: None, anchors: vec![] },
                MullionLine { x_position: 3000.0, profile: None, anchors: vec![] },
                MullionLine { x_position: 4500.0, profile: None, anchors: vec![] },
            ],
            transoms: vec![
                TransomLine { y_position: 1200.0, profile: None, anchors: vec![] },
                TransomLine { y_position: 2400.0, profile: None, anchors: vec![] },
            ],
            panels: vec![],
            placement: Placement::default(),
        }
    }

    #[test]
    fn test_grid_zones() {
        let vg = make_test_vliesgevel();
        assert_eq!(vg.num_cols(), 4); // 3 mullions -> 4 zones
        assert_eq!(vg.num_rows(), 3); // 2 transoms -> 3 zones

        let cols = vg.column_zones();
        assert_eq!(cols.len(), 4);
        assert!((cols[0].0 - 0.0).abs() < 0.01);
        assert!((cols[0].1 - 1475.0).abs() < 0.01); // 1500 - 50/2

        let rows = vg.row_zones();
        assert_eq!(rows.len(), 3);
    }

    #[test]
    fn test_rebuild_panels() {
        let mut vg = make_test_vliesgevel();
        vg.rebuild_panels();
        assert_eq!(vg.panels.len(), 12); // 4 cols x 3 rows
    }

    #[test]
    fn test_add_remove_mullion() {
        let mut vg = make_test_vliesgevel();
        vg.rebuild_panels();
        assert_eq!(vg.num_cols(), 4);

        vg.add_mullion(2250.0);
        assert_eq!(vg.num_cols(), 5);
        assert_eq!(vg.panels.len(), 15); // 5 x 3

        vg.remove_mullion(0);
        assert_eq!(vg.num_cols(), 4);
        assert_eq!(vg.panels.len(), 12); // 4 x 3
    }
}
