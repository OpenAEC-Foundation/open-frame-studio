use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::glaslat::Glaslat;
use crate::hardware::HardwareSet;
use crate::panel_filling::PanelFilling;
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
    /// Free-form extensions beyond the grid (e.g., extended dorpel, extra stijl)
    #[serde(default)]
    pub extensions: Vec<FrameExtension>,
    /// Free subdivision tree (additive). When present, supersedes the rectangular
    /// grid for layout/geometry; old projects (grid only) keep working.
    #[serde(default)]
    pub layout: Option<crate::layout::VakNode>,
}

/// A free-form member extension beyond the rectangular grid
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameExtension {
    /// Extension type
    pub extension_type: ExtensionType,
    /// Profile used for this extension
    pub profile: ProfileRef,
    /// Start point (absolute coordinates in mm)
    pub start_x: f64,
    pub start_y: f64,
    /// End point (absolute coordinates in mm)
    pub end_x: f64,
    pub end_y: f64,
    /// Width of the member (face width in mm)
    #[serde(default = "default_ext_width")]
    pub member_width: f64,
    /// Which grid member(s) this connects to
    #[serde(default)]
    pub connects_to: Vec<String>,
}

fn default_ext_width() -> f64 { 67.0 }

/// Extension member type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionType {
    /// Extended dorpel (wider than frame)
    Dorpel,
    /// Extra stijl (outside the grid)
    Stijl,
    /// Balk (structural beam)
    Balk,
    /// Neut (corner block at sill)
    Neut,
    /// Spouwlat (cavity batten)
    Spouwlat,
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
                profile_snapshot: None,
                sill: None,
                top_profile: None,
                bottom_profile: None,
                left_profile: None,
                right_profile: None,
                top_material: None,
                bottom_material: None,
                left_material: None,
                right_material: None,
                shape: FrameShape::default(),
                corner_joints: vec![
                    crate::joint::Joint::default(),
                    crate::joint::Joint::default(),
                    crate::joint::Joint::default(),
                    crate::joint::Joint::default(),
                ],
                joints_configured: false,
                edges: vec![],
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
            extensions: vec![],
            layout: None,
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
                profile_snapshot: None,
                sill: None,
                top_profile: Some(sj.bovendorpel_profile.clone()),
                bottom_profile: Some(sj.onderdorpel_profile.clone()),
                left_profile: Some(sj.stijl_profile.clone()),
                right_profile: Some(sj.stijl_profile.clone()),
                top_material: None,
                bottom_material: None,
                left_material: None,
                right_material: None,
                shape: FrameShape::default(),
                corner_joints: vec![
                    crate::joint::Joint::default(),
                    crate::joint::Joint::default(),
                    crate::joint::Joint::default(),
                    crate::joint::Joint::default(),
                ],
                joints_configured: false,
                edges: vec![],
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
            extensions: vec![],
            layout: None,
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

    /// Inner width (excluding frame members)
    pub fn inner_width(&self) -> f64 {
        self.frame.outer_width - 2.0 * self.frame.frame_width
    }

    /// Inner height (excluding frame members)
    pub fn inner_height(&self) -> f64 {
        self.frame.outer_height - 2.0 * self.frame.frame_width
    }

    /// Add a vertical divider (creates a new column). The new tussenstijl
    /// occupies `frame_width` mm, which is subtracted proportionally from the
    /// two new column sizes so that Σ(kolommen) + Σ(tussenstijlen) stays
    /// exactly the binnenmaat — the level-1 dimension chain keeps adding up
    /// to the buitenwerkse maat and the last vak no longer runs underneath
    /// the stijl.
    pub fn add_column(&mut self, position: f64) {
        let divider_w = self.frame.frame_width;
        let total_existing: f64 = self.grid.columns.iter().map(|c| c.size).sum();

        if position > 0.0 && position < total_existing {
            let mut accumulated = 0.0;
            for i in 0..self.grid.columns.len() {
                let col_end = accumulated + self.grid.columns[i].size;
                if position > accumulated && position < col_end {
                    let mut left = position - accumulated;
                    let mut right = col_end - position;
                    // Reserve room for the divider itself: shrink both halves
                    // proportionally. Skipped when the vak is not wider than
                    // the divider (degenerate split, sizes must stay > 0).
                    let span = left + right;
                    if span > divider_w {
                        let scale = (span - divider_w) / span;
                        left *= scale;
                        right *= scale;
                    }
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

    /// Add a horizontal divider (creates a new row). The new tussendorpel
    /// occupies `frame_width` mm, subtracted proportionally from the two new
    /// row sizes — see `add_column` for the invariant.
    pub fn add_row(&mut self, position: f64) {
        let divider_w = self.frame.frame_width;
        let total_existing: f64 = self.grid.rows.iter().map(|r| r.size).sum();

        if position > 0.0 && position < total_existing {
            let mut accumulated = 0.0;
            for i in 0..self.grid.rows.len() {
                let row_end = accumulated + self.grid.rows[i].size;
                if position > accumulated && position < row_end {
                    let mut top = position - accumulated;
                    let mut bottom = row_end - position;
                    // Reserve room for the divider itself (see add_column).
                    let span = top + bottom;
                    if span > divider_w {
                        let scale = (span - divider_w) / span;
                        top *= scale;
                        bottom *= scale;
                    }
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
    /// Resolved dimensions of the selected frame profile (sponning, glaslat,
    /// aanzichtbreedte), captured when the profile is chosen. `None` on
    /// pre-snapshot projects — drawing and checks fall back to the classic
    /// defaults. Skipped when absent so old payloads stay byte-identical.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_snapshot: Option<crate::profile::ProfileSnapshot>,
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
    /// Per-member material overrides (None = use default frame material)
    #[serde(default)]
    pub top_material: Option<Material>,
    #[serde(default)]
    pub bottom_material: Option<Material>,
    #[serde(default)]
    pub left_material: Option<Material>,
    #[serde(default)]
    pub right_material: Option<Material>,
    /// Corner joint configurations [top-left, top-right, bottom-left, bottom-right]
    #[serde(default)]
    pub corner_joints: Vec<crate::joint::Joint>,
    /// `true` once the user has explicitly saved corner joints (set by the
    /// update_corner_joints commands). `Kozijn::new` auto-populates
    /// `corner_joints` with four defaults, so the values alone cannot tell a
    /// deliberate "pen/slis, stijl doorlopend" apart from "never touched";
    /// this flag can. `false` on old .ofs files, which keeps the historic
    /// sentinel fallback (`crate::joint::effective_corner_joints`).
    #[serde(default)]
    pub joints_configured: bool,
    /// Edge configurations [left, right, top, bottom] — wall connection details
    #[serde(default)]
    pub edges: Vec<crate::edge::EdgeConfig>,
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
    /// Elliptical: horizontal radius in mm (default ow/2)
    #[serde(default)]
    pub ellipse_rx: Option<f64>,
    /// Elliptical: vertical radius in mm (default oh/3)
    #[serde(default)]
    pub ellipse_ry: Option<f64>,
    /// Polygon: custom polygon points [[x,y], ...] in mm (for Polygon shape)
    #[serde(default)]
    pub polygon_points: Option<Vec<[f64; 2]>>,
    /// Triangle: apex horizontal offset from center in mm (for Triangle shape)
    #[serde(default)]
    pub apex_offset: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ShapeType {
    #[default]
    Rectangular,
    Arched,         // getoogde bovendorpel (segmentboog)
    Round,          // volledig rond (cirkel)
    Elliptical,
    Trapezoid,      // schuine stijl(en) of dorpel
    ArchedTrapezoid, // boog + schuine stijlen gecombineerd (CNCware-stijl)
    Triangle,       // driehoekig kozijn
    Polygon,        // vrije veelhoek
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
    /// Infill panel spec for Panel/Ventilation cells (vakvulling). None = use defaults.
    #[serde(default)]
    pub panel_filling: Option<PanelFilling>,
    /// Glazing bead spec for glazed cells (glaslat). None = no explicit bead.
    #[serde(default)]
    pub glaslat: Option<Glaslat>,
    /// Designated as an escape window (vluchtraam) — triggers Bouwbesluit checks.
    #[serde(default)]
    pub is_escape: bool,
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
            panel_filling: None,
            glaslat: None,
            is_escape: false,
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
            PanelType::TurnTilt | PanelType::Turn | PanelType::Tilt | PanelType::Sliding
            | PanelType::TopHung | PanelType::BottomHung | PanelType::LiftSlide | PanelType::Pivot => {
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
    TopHung,     // klapraam / klep
    BottomHung,  // tuimelraam
    LiftSlide,   // hefschuifpui
    Pivot,       // melkmeisje / pivot raam
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
            Self::TopHung => "Klapraam",
            Self::BottomHung => "Tuimelraam",
            Self::LiftSlide => "Hefschuifpui",
            Self::Pivot => "Pivotraam",
        }
    }

    pub fn is_operable(&self) -> bool {
        matches!(
            self,
            Self::TurnTilt | Self::Turn | Self::Tilt | Self::Sliding | Self::Door
            | Self::TopHung | Self::BottomHung | Self::LiftSlide | Self::Pivot
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
    #[serde(default)]
    pub custom_sjablonen: Vec<crate::template::KozijnSjabloon>,
    /// Stable IFC GUIDs: maps element path (e.g. "kozijn:{id}") to a persistent UUID
    #[serde(default)]
    pub ifc_guid_map: std::collections::HashMap<String, uuid::Uuid>,
    /// Pricing / quotation configuration
    #[serde(default)]
    pub pricing_config: Option<crate::pricing::PricingConfig>,
    /// BCF topics for issue tracking
    #[serde(default)]
    pub bcf_topics: Vec<crate::bcf::BcfTopic>,
    /// Quotation versions
    #[serde(default)]
    pub quotations: Vec<crate::quotation::Quotation>,
    #[serde(default)]
    pub combinations: Vec<crate::combination::CombinationKozijn>,
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
            custom_sjablonen: vec![],
            ifc_guid_map: std::collections::HashMap::new(),
            pricing_config: None,
            bcf_topics: vec![],
            quotations: vec![],
            combinations: vec![],
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_column_reserves_divider_space() {
        let mut k = Kozijn::new("Test", "K01", 1200.0, 1500.0);
        let inner_w = k.inner_width(); // 1066
        let fw = k.frame.frame_width; // 67 (= divider width)
        k.add_column(400.0);

        assert_eq!(k.grid.columns.len(), 2);
        assert_eq!(k.cells.len(), 2);
        let sum: f64 = k.grid.columns.iter().map(|c| c.size).sum();
        // Σ kolommen + tussenstijl = binnenmaat: the dimension chain closes.
        assert!((sum + fw - inner_w).abs() < 1e-6, "sum {} + fw {} != inner {}", sum, fw, inner_w);
        // The divider space is taken proportionally: the 400:666 ratio holds.
        let ratio = k.grid.columns[0].size / k.grid.columns[1].size;
        assert!((ratio - 400.0 / 666.0).abs() < 1e-6);
        // Both columns stay positive.
        assert!(k.grid.columns.iter().all(|c| c.size > 0.0));
    }

    #[test]
    fn add_row_reserves_divider_space() {
        let mut k = Kozijn::new("Test", "K01", 1200.0, 1500.0);
        let inner_h = k.inner_height(); // 1366
        let fw = k.frame.frame_width;
        k.add_row(500.0);

        assert_eq!(k.grid.rows.len(), 2);
        let sum: f64 = k.grid.rows.iter().map(|r| r.size).sum();
        assert!((sum + fw - inner_h).abs() < 1e-6, "sum {} + fw {} != inner {}", sum, fw, inner_h);
        let ratio = k.grid.rows[0].size / k.grid.rows[1].size;
        assert!((ratio - 500.0 / 866.0).abs() < 1e-6);
    }

    #[test]
    fn add_column_on_degenerate_vak_keeps_positive_sizes() {
        // A vak narrower than the divider cannot reserve the space; the split
        // still happens but sizes stay positive (no negative column widths).
        let mut k = Kozijn::new("Test", "K02", 190.0, 400.0);
        // inner width = 190 − 134 = 56 < divider width 67
        assert!(k.inner_width() < k.frame.frame_width);
        k.add_column(28.0);
        assert_eq!(k.grid.columns.len(), 2);
        assert!(k.grid.columns.iter().all(|c| c.size > 0.0));
    }
}
