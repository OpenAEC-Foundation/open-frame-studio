use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::hardware::HardwareSet;
use crate::profile::ProfileRef;

/// Complete kozijn (window frame) definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Kozijn {
    pub id: Uuid,
    pub name: String,
    pub mark: String,
    pub frame: Frame,
    pub grid: Grid,
    pub cells: Vec<Cell>,
    #[serde(default)]
    pub placement: Placement,
    /// Series/batch reference (e.g., "Begane grond", "Verdieping 1")
    #[serde(default)]
    pub series_id: Option<String>,
}

impl Kozijn {
    pub fn new(name: &str, mark: &str, width: f64, height: f64) -> Self {
        let id = Uuid::new_v4();
        Self {
            id,
            name: name.to_string(),
            mark: mark.to_string(),
            frame: Frame {
                outer_width: width,
                outer_height: height,
                material: Material::Wood(WoodType::Meranti),
                profile: ProfileRef::default_frame(),
                sill_profile: Some(ProfileRef::default_sill()),
                color_inside: "RAL9010".into(),
                color_outside: "RAL9010".into(),
                frame_width: 67.0,
                frame_depth: 114.0,
                sill: None,
                top_profile: None,
                bottom_profile: None,
                left_profile: None,
                right_profile: None,
                shape: FrameShape::default(),
            },
            grid: Grid {
                columns: vec![GridDivision {
                    size: width - 2.0 * 67.0,
                    divider_profile: None,
                }],
                rows: vec![GridDivision {
                    size: height - 2.0 * 67.0,
                    divider_profile: None,
                }],
            },
            cells: vec![Cell::default()],
            placement: Placement::default(),
            series_id: None,
        }
    }

    /// Create a new kozijn using a sjabloon (template) for automatic profile assignment
    pub fn new_with_sjabloon(name: &str, mark: &str, width: f64, height: f64, sj: &crate::template::KozijnSjabloon) -> Self {
        let fw = sj.frame_width;
        let id = Uuid::new_v4();
        Self {
            id,
            name: name.to_string(),
            mark: mark.to_string(),
            frame: Frame {
                outer_width: width,
                outer_height: height,
                material: sj.material.clone(),
                profile: sj.stijl_profile.clone(),
                sill_profile: Some(sj.onderdorpel_profile.clone()),
                color_inside: sj.color_inside.clone(),
                color_outside: sj.color_outside.clone(),
                frame_width: fw,
                frame_depth: sj.frame_depth,
                sill: None,
                top_profile: Some(sj.bovendorpel_profile.clone()),
                bottom_profile: Some(sj.onderdorpel_profile.clone()),
                left_profile: Some(sj.stijl_profile.clone()),
                right_profile: Some(sj.stijl_profile.clone()),
                shape: FrameShape::default(),
            },
            grid: Grid {
                columns: vec![GridDivision {
                    size: width - 2.0 * fw,
                    divider_profile: None,
                }],
                rows: vec![GridDivision {
                    size: height - 2.0 * fw,
                    divider_profile: None,
                }],
            },
            cells: vec![Cell::default_with_glazing(&sj.default_glazing)],
            placement: Placement::default(),
            series_id: None,
        }
    }

    /// Duplicate this kozijn with a new mark and UUID
    pub fn duplicate(&self, new_mark: &str) -> Self {
        let mut clone = self.clone();
        clone.id = Uuid::new_v4();
        clone.mark = new_mark.to_string();
        clone.name = format!("{} (kopie)", self.name);
        clone
    }

    /// Recalculate cells after grid changes
    pub fn rebuild_cells(&mut self) {
        let num_cells = self.grid.columns.len() * self.grid.rows.len();
        self.cells.resize_with(num_cells, Cell::default);
    }

    /// Get cell at grid position
    pub fn cell_at(&self, col: usize, row: usize) -> Option<&Cell> {
        let idx = row * self.grid.columns.len() + col;
        self.cells.get(idx)
    }

    /// Get mutable cell at grid position
    pub fn cell_at_mut(&mut self, col: usize, row: usize) -> Option<&mut Cell> {
        let idx = row * self.grid.columns.len() + col;
        self.cells.get_mut(idx)
    }

    /// Inner width (excluding frame members)
    pub fn inner_width(&self) -> f64 {
        self.frame.outer_width - 2.0 * self.frame.frame_width
    }

    /// Inner height (excluding frame members)
    pub fn inner_height(&self) -> f64 {
        self.frame.outer_height - 2.0 * self.frame.frame_width
    }

    /// Add a vertical divider (creates a new column)
    pub fn add_column(&mut self, position: f64) {
        let total_existing: f64 = self.grid.columns.iter().map(|c| c.size).sum();

        if position > 0.0 && position < total_existing {
            let mut accumulated = 0.0;
            for i in 0..self.grid.columns.len() {
                let col_end = accumulated + self.grid.columns[i].size;
                if position > accumulated && position < col_end {
                    let left = position - accumulated;
                    let right = col_end - position;
                    self.grid.columns[i].size = left;
                    self.grid.columns.insert(
                        i + 1,
                        GridDivision {
                            size: right,
                            divider_profile: Some(ProfileRef::default_divider()),
                        },
                    );
                    break;
                }
                accumulated = col_end;
            }
        }
        self.rebuild_cells();
    }

    /// Add a horizontal divider (creates a new row)
    pub fn add_row(&mut self, position: f64) {
        let total_existing: f64 = self.grid.rows.iter().map(|r| r.size).sum();

        if position > 0.0 && position < total_existing {
            let mut accumulated = 0.0;
            for i in 0..self.grid.rows.len() {
                let row_end = accumulated + self.grid.rows[i].size;
                if position > accumulated && position < row_end {
                    let top = position - accumulated;
                    let bottom = row_end - position;
                    self.grid.rows[i].size = top;
                    self.grid.rows.insert(
                        i + 1,
                        GridDivision {
                            size: bottom,
                            divider_profile: Some(ProfileRef::default_divider()),
                        },
                    );
                    break;
                }
                accumulated = row_end;
            }
        }
        self.rebuild_cells();
    }
}

/// Frame definition (the outer border of the kozijn)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Frame {
    /// Overall outer width in mm
    pub outer_width: f64,
    /// Overall outer height in mm
    pub outer_height: f64,
    pub material: Material,
    pub profile: ProfileRef,
    pub sill_profile: Option<ProfileRef>,
    pub color_inside: String,
    pub color_outside: String,
    /// Frame member width (face width) in mm
    pub frame_width: f64,
    /// Frame member depth in mm
    pub frame_depth: f64,
    /// Structured sill configuration (v1.3+)
    #[serde(default)]
    pub sill: Option<crate::sill::Sill>,
    /// Per-member profile overrides (None = use default frame profile)
    #[serde(default)]
    pub top_profile: Option<ProfileRef>,
    #[serde(default)]
    pub bottom_profile: Option<ProfileRef>,
    #[serde(default)]
    pub left_profile: Option<ProfileRef>,
    #[serde(default)]
    pub right_profile: Option<ProfileRef>,
    /// Frame shape (rectangular, arched, round)
    #[serde(default)]
    pub shape: FrameShape,
}

/// Frame shape definition for arched/round kozijnen
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FrameShape {
    pub shape_type: ShapeType,
    /// Arch radius in mm (for arched shape)
    #[serde(default)]
    pub arch_radius: Option<f64>,
    /// Arch rise height above the rectangular top in mm
    #[serde(default)]
    pub arch_height: Option<f64>,
    /// Trapezoid: top width (can differ from outer_width)
    #[serde(default)]
    pub top_width: Option<f64>,
    /// Trapezoid: left stile angle in degrees (90 = vertical)
    #[serde(default)]
    pub left_angle: Option<f64>,
    /// Trapezoid: right stile angle in degrees (90 = vertical)
    #[serde(default)]
    pub right_angle: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ShapeType {
    #[default]
    Rectangular,
    Arched,      // getoogde bovendorpel (segmentboog)
    Round,       // volledig rond (cirkel)
    Elliptical,
    Trapezoid,   // schuine stijl(en) of dorpel
}

/// Grid subdivision — columns (vertical) and rows (horizontal)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Grid {
    pub columns: Vec<GridDivision>,
    pub rows: Vec<GridDivision>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridDivision {
    /// Size in mm
    pub size: f64,
    /// Divider profile (None for first column/row since there's no divider before it)
    pub divider_profile: Option<ProfileRef>,
}

/// A single cell (vak) in the kozijn grid
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cell {
    pub panel_type: PanelType,
    pub opening_direction: Option<OpeningDirection>,
    pub glazing: Glazing,
    /// Legacy hardware list — kept for backward compatibility with format_version "1.0"
    #[serde(default)]
    pub hardware: Vec<Hardware>,
    /// Structured hardware set (format_version "1.1"+)
    #[serde(default)]
    pub hardware_set: Option<HardwareSet>,
    /// Sash/door profile for operable cells (raamhout/deurhout)
    #[serde(default)]
    pub sash_profile: Option<ProfileRef>,
    /// Sash wood width in mm (54/67mm typical)
    #[serde(default)]
    pub sash_width: Option<f64>,
    /// Sash wood depth in mm (67/78/90mm typical)
    #[serde(default)]
    pub sash_depth: Option<f64>,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            panel_type: PanelType::FixedGlass,
            opening_direction: None,
            glazing: Glazing::default(),
            hardware: vec![],
            hardware_set: None,
            sash_profile: None,
            sash_width: None,
            sash_depth: None,
        }
    }
}

impl Cell {
    /// Create a cell with glazing preset from a sjabloon
    pub fn default_with_glazing(preset: &crate::template::GlazingPreset) -> Self {
        Self {
            glazing: Glazing {
                glass_type: preset.glass_type.clone(),
                thickness_mm: preset.thickness_mm,
                ug_value: preset.ug_value,
                panes: vec![],
                spacer_type: preset.spacer_type.clone(),
            },
            ..Self::default()
        }
    }

    /// Assign sash profile for operable cells (raamhout/deurhout)
    pub fn assign_sash_from_sjabloon(&mut self, sj: &crate::template::KozijnSjabloon) {
        match self.panel_type {
            PanelType::TurnTilt | PanelType::Turn | PanelType::Tilt | PanelType::Sliding => {
                self.sash_profile = Some(sj.raamhout_profile.clone());
                self.sash_width = Some(sj.sash_width);
                self.sash_depth = Some(sj.sash_depth);
            }
            PanelType::Door => {
                self.sash_profile = Some(sj.deurhout_profile.clone());
                self.sash_width = Some(sj.frame_width); // deurhout = kozijnhout breedte
                self.sash_depth = Some(sj.frame_depth);
            }
            _ => {
                self.sash_profile = None;
                self.sash_width = None;
                self.sash_depth = None;
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PanelType {
    FixedGlass,
    TurnTilt,    // draaikiepraam
    Turn,        // draairaam
    Tilt,        // kiepraam
    Sliding,     // schuifraam
    Door,        // deur
    Panel,       // dicht paneel
    Ventilation, // ventilatierooster
}

impl PanelType {
    pub fn label_nl(&self) -> &'static str {
        match self {
            Self::FixedGlass => "Vast glas",
            Self::TurnTilt => "Draaikiepraam",
            Self::Turn => "Draairaam",
            Self::Tilt => "Kiepraam",
            Self::Sliding => "Schuifraam",
            Self::Door => "Deur",
            Self::Panel => "Paneel",
            Self::Ventilation => "Ventilatie",
        }
    }

    pub fn is_operable(&self) -> bool {
        matches!(
            self,
            Self::TurnTilt | Self::Turn | Self::Tilt | Self::Sliding | Self::Door
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpeningDirection {
    Left,
    Right,
    Inward,
    Outward,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Glazing {
    pub glass_type: String,
    pub thickness_mm: f64,
    pub ug_value: f64,
    /// Individual pane layers (glass + spacer + glass + ...)
    #[serde(default)]
    pub panes: Vec<Pane>,
    /// Spacer type between panes
    #[serde(default = "default_spacer_type")]
    pub spacer_type: String,
}

fn default_spacer_type() -> String {
    "warm-edge".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pane {
    /// Pane thickness in mm (4, 5, 6, 8, 10)
    pub thickness_mm: f64,
    /// Pane type: "float", "gehard", "gelamineerd", "low-e"
    pub pane_type: String,
}

impl Default for Glazing {
    fn default() -> Self {
        Self {
            glass_type: "HR++".into(),
            thickness_mm: 24.0,
            ug_value: 1.0,
            panes: vec![
                Pane { thickness_mm: 4.0, pane_type: "float".into() },
                Pane { thickness_mm: 4.0, pane_type: "low-e".into() },
            ],
            spacer_type: "warm-edge".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hardware {
    pub hardware_type: HardwareType,
    pub position: String,
    pub brand: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HardwareType {
    Hinge,
    Handle,
    Lock,
    Ventilation,
    Closer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Material {
    Wood(WoodType),
    Aluminum,
    Pvc,
    WoodAluminum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WoodType {
    Meranti,
    Accoya,
    Vuren,
    Eiken,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Placement {
    pub wall_id: Option<String>,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub orientation: f64,
}

/// A complete project file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub format_version: String,
    pub project_info: ProjectInfo,
    pub kozijnen: Vec<Kozijn>,
    #[serde(default)]
    pub vliesgevels: Vec<crate::vliesgevel::Vliesgevel>,
    #[serde(default)]
    pub custom_profiles: Vec<crate::profile::ProfileDefinition>,
}

impl Project {
    pub fn new(name: &str, number: &str) -> Self {
        Self {
            format_version: "1.3".into(),
            project_info: ProjectInfo {
                name: name.to_string(),
                number: number.to_string(),
                client: String::new(),
                address: String::new(),
                series: vec![],
            },
            kozijnen: vec![],
            vliesgevels: vec![],
            custom_profiles: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
    pub name: String,
    pub number: String,
    pub client: String,
    pub address: String,
    /// Project series / building phases
    #[serde(default)]
    pub series: Vec<Series>,
}

/// A series / building phase for organizing kozijnen
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
}
