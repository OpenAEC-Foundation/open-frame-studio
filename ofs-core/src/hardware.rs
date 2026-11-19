use serde::{Deserialize, Serialize};

use crate::kozijn::{Material, OpeningDirection, PanelType};

mod placement;
pub use placement::{FittingKind, HardwareComponent, HardwareFitIssue, HardwareLeaf, HardwareMount};
pub(crate) use placement::hardware_leaves;

// ── Security classification ────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SecurityClass {
    #[default]
    None,
    #[serde(rename = "rc1", alias = "RC1", alias = "r_c1")]
    RC1,
    #[serde(rename = "rc2", alias = "RC2", alias = "r_c2")]
    RC2,
    #[serde(rename = "rc3", alias = "RC3", alias = "r_c3")]
    RC3,
    #[serde(rename = "rc4", alias = "RC4", alias = "r_c4")]
    RC4,
    #[serde(rename = "rc5", alias = "RC5", alias = "r_c5")]
    RC5,
    #[serde(rename = "rc6", alias = "RC6", alias = "r_c6")]
    RC6,
}

impl SecurityClass {
    pub fn min_locking_points(&self) -> u8 {
        match self {
            Self::None => 1,
            Self::RC1 => 2,
            Self::RC2 => 3,
            Self::RC3 => 5,
            Self::RC4 => 7,
            Self::RC5 => 9,
            Self::RC6 => 11,
        }
    }

    pub fn requires_mushroom_cams(&self) -> bool {
        matches!(self, Self::RC2 | Self::RC3 | Self::RC4 | Self::RC5 | Self::RC6)
    }

    pub fn label_nl(&self) -> &'static str {
        match self {
            Self::None => "Geen",
            Self::RC1 => "RC1 (basis)",
            Self::RC2 => "RC2 (standaard nieuwbouw)",
            Self::RC3 => "RC3 (verhoogd)",
            Self::RC4 => "RC4 (hoog)",
            Self::RC5 => "RC5 (zeer hoog)",
            Self::RC6 => "RC6 (maximaal)",
        }
    }
}

// ── Hinge configuration ────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HingeType {
    Opleg,      // surface-mounted paumelle
    Inboor,     // concealed bore-in hinge
    Verdekt,    // fully concealed hinge
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HingeConfig {
    pub hinge_type: HingeType,
    pub count: u8,
    pub load_capacity_kg: f64,
    pub side: HingeSide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HingeSide {
    Left,
    Right,
    Top,
    Bottom,
}

// ── Handle configuration ───────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandleType {
    Kruk,         // lever handle
    Knop,         // knob
    TGreep,       // T-handle (sliding)
    InlaatGreep,  // flush pull (sliding)
    KrukKruk,     // double lever (doors)
    StangenGreep, // bar handle (doors)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandleConfig {
    pub handle_type: HandleType,
    pub side: HandleSide,
    pub height_mm: f64,
    pub lockable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandleSide {
    Left,
    Right,
    Center,
}

// ── Locking configuration ──────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LockType {
    Espagnolet,     // cremone bolt (windows)
    MultiPoint,     // multipoint lock (doors)
    CylinderLock,   // single cylinder
    SlidingLock,    // sliding door lock
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CamType {
    RolNok,            // roller cam
    PaddenstoelNok,    // mushroom cam (anti-burglary)
    HaakSluitNok,      // hook cam
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CylinderType {
    None,
    EuroProfile,
    Skg1,
    Skg2,
    Skg3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockingConfig {
    pub lock_type: LockType,
    pub locking_points: u8,
    pub cam_type: CamType,
    pub cylinder: CylinderType,
}

// ── Ventilation & closer ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VentilationConfig {
    pub vent_type: String,
    pub capacity_dm3s: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CloserType {
    Surface,    // opleg deurdranger
    Concealed,  // verdekt ingebouwde dranger
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloserConfig {
    pub closer_type: CloserType,
    pub force_class: u8, // EN 1154: class 1-6
}

// ── Complete hardware set ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareSet {
    pub hinges: Option<HingeConfig>,
    pub handle: Option<HandleConfig>,
    pub locking: Option<LockingConfig>,
    pub ventilation: Option<VentilationConfig>,
    pub closer: Option<CloserConfig>,
    pub security_class: SecurityClass,
    #[serde(default)]
    pub auto_selected: bool,
}

impl HardwareSet {
    /// A change of handing mirrors the assembly while preserving the user's
    /// configured hinge model, quantity, handle height and lock selection.
    fn change_handing(
        &mut self,
        panel: PanelType,
        previous: Option<OpeningDirection>,
        next: Option<OpeningDirection>,
    ) {
        if previous == next
            || !matches!(
                panel,
                PanelType::Turn
                    | PanelType::TurnTilt
                    | PanelType::Door
                    | PanelType::Sliding
                    | PanelType::LiftSlide
                    | PanelType::Pivot
            )
        {
            return;
        }
        let right = match next {
            Some(OpeningDirection::Right) => true,
            Some(OpeningDirection::Left) => false,
            _ => return,
        };
        let was_right = match self.hinges.as_ref().map(|h| h.side) {
            Some(HingeSide::Right) => true,
            Some(HingeSide::Left) => false,
            _ => previous == Some(OpeningDirection::Right),
        };
        if right == was_right {
            return;
        }
        if let Some(h) = self
            .hinges
            .as_mut()
            .filter(|h| matches!(h.side, HingeSide::Right | HingeSide::Left))
        {
            h.side = if right {
                HingeSide::Right
            } else {
                HingeSide::Left
            };
        }
        if let Some(h) = &mut self.handle {
            h.side = match h.side {
                HandleSide::Right => HandleSide::Left,
                HandleSide::Left => HandleSide::Right,
                HandleSide::Center => HandleSide::Center,
            };
        }
    }

    fn supports_motion(&self, panel: PanelType) -> bool {
        let sliding = matches!(panel, PanelType::Sliding | PanelType::LiftSlide);
        if !sliding
            && self
                .locking
                .as_ref()
                .is_some_and(|l| l.lock_type == LockType::SlidingLock)
        {
            return false;
        }
        let Some(h) = self.hinges.as_ref().filter(|h| h.count > 0) else {
            return true;
        };
        match panel {
            PanelType::Turn | PanelType::TurnTilt | PanelType::Door => {
                matches!(h.side, HingeSide::Left | HingeSide::Right)
            }
            PanelType::TopHung | PanelType::Pivot => h.side == HingeSide::Top,
            PanelType::Tilt | PanelType::BottomHung => h.side == HingeSide::Bottom,
            _ => false,
        }
    }
}

impl crate::kozijn::Kozijn {
    fn check_hardware_cell(&self, index: usize) -> Result<(), String> {
        fn leaves(node: &crate::layout::VakNode) -> usize {
            match node {
                crate::layout::VakNode::Leaf { .. } => 1,
                crate::layout::VakNode::Split { children, .. } => {
                    children.iter().map(|child| leaves(&child.node)).sum()
                }
            }
        }
        if self
            .layout
            .as_ref()
            .is_some_and(|l| leaves(l) != self.cells.len())
        {
            return Err("Dit indelingsvak heeft geen zelfstandig opgeslagen beslagset".into());
        }
        self.cells
            .get(index)
            .map(|_| ())
            .ok_or_else(|| "Cel niet gevonden".into())
    }

    /// Store explicit selections separately from the derived locations.
    pub fn set_manual_hardware(
        &mut self,
        index: usize,
        mut set: HardwareSet,
    ) -> Result<(), String> {
        self.check_hardware_cell(index)?;
        let c = &mut self.cells[index];
        set.auto_selected = false;
        c.hardware_set = Some(set);
        Ok(())
    }

    /// Re-select from the assembled leaf, including the glass build-up and
    /// profile. A manually changed set is replaced only on explicit selection.
    pub fn select_hardware(
        &mut self,
        index: usize,
        security: Option<SecurityClass>,
    ) -> Result<(), String> {
        self.check_hardware_cell(index)?;
        let c = &mut self.cells[index];
        let level = security
            .or_else(|| c.hardware_set.as_ref().map(|s| s.security_class))
            .unwrap_or_default();
        if self.layout.is_none() && !c.panel_type.is_operable() {
            if !self.frame.material.is_profile_system() {
                c.hardware_set = None;
            }
            return Ok(());
        }
        let previous = c.hardware_set.take();
        c.hardware_set = Some(HardwareSet {
            hinges: None,
            locking: None,
            handle: None,
            closer: None,
            ventilation: None,
            security_class: level,
            auto_selected: true,
        });
        let geo = crate::geometry::compute_2d_geometry(self);
        let resolved = geo
            .cell_rects
            .iter()
            .find(|c| c.cell_index == index)
            .and_then(|cr| {
                if self.frame.material.is_profile_system() {
                    return cr
                        .hardware_leaves
                        .iter()
                        .find_map(|leaf| leaf.set.as_ref().filter(|s| s.handle.is_some()))
                        .or_else(|| cr.hardware_leaves.first().and_then(|l| l.set.as_ref()))
                        .cloned();
                }
                let c = &self.cells[index];
                default_hardware_set(
                    c.panel_type,
                    c.opening_direction,
                    cr.rect.width,
                    cr.rect.height,
                    c.glazing.thickness_mm,
                    &self.frame.material,
                    level,
                )
            });
        self.cells[index].hardware_set = if self.frame.material.is_profile_system() {
            resolved.or(previous)
        } else {
            resolved
        };
        Ok(())
    }

    pub fn set_hardware_security(
        &mut self,
        index: usize,
        level: SecurityClass,
    ) -> Result<(), String> {
        self.check_hardware_cell(index)?;
        let c = &mut self.cells[index];
        if let Some(s) = c.hardware_set.as_mut().filter(|s| !s.auto_selected) {
            s.security_class = level;
            return Ok(());
        }
        if c.hardware_set.is_none() && !c.hardware.is_empty() {
            return Err("Kies eerst een beslagset om de veiligheidsklasse toe te kennen".into());
        }
        self.select_hardware(index, Some(level))
    }

    pub fn change_cell_opening(
        &mut self,
        index: usize,
        panel: PanelType,
        direction: Option<OpeningDirection>,
    ) -> Result<(), String> {
        self.check_hardware_cell(index)?;
        let c = &mut self.cells[index];
        let previous = c.opening_direction;
        c.change_panel_type(panel, &self.frame);
        c.opening_direction = direction;
        if self.frame.material.is_profile_system() {
            if let Some(set) = c.hardware_set.as_mut().filter(|s| !s.auto_selected) {
                if set.supports_motion(panel) {
                    set.change_handing(panel, previous, direction);
                }
                return Ok(());
            }
            if !c.hardware.is_empty() && c.hardware_set.is_none() {
                return Ok(());
            }
        }
        self.select_hardware(index, None)
    }
}

// ── Auto-selection engine ──────────────────────────────────────────

/// Estimate sash weight in kg based on dimensions, glass, and material.
pub fn estimate_sash_weight(
    width_mm: f64,
    height_mm: f64,
    glass_thickness_mm: f64,
    material: &Material,
) -> f64 {
    let area_m2 = (width_mm / 1000.0) * (height_mm / 1000.0);

    // Glass weight: ~2.5 kg/m2 per mm thickness
    let glass_weight = area_m2 * glass_thickness_mm * 2.5;

    // Profile weight per meter (approximate)
    let profile_weight_per_m = match material {
        Material::Wood(_) => 3.5,       // ~3.5 kg/m for typical 67x114mm
        Material::Aluminum => 2.0,      // ~2.0 kg/m for typical aluminum
        Material::Pvc => 2.5,           // ~2.5 kg/m including steel reinforcement
        Material::WoodAluminum => 4.5,  // ~4.5 kg/m combined
    };

    let perimeter_m = 2.0 * (width_mm + height_mm) / 1000.0;
    let frame_weight = perimeter_m * profile_weight_per_m;

    glass_weight + frame_weight
}

/// Calculate recommended number of locking points based on sash perimeter.
/// Rule of thumb: 1 point per ~400mm, minimum 2.
pub fn calculate_locking_points(perimeter_mm: f64, security_class: &SecurityClass) -> u8 {
    let by_perimeter = ((perimeter_mm / 400.0).ceil() as u8).max(2);
    let by_security = security_class.min_locking_points();
    by_perimeter.max(by_security)
}

/// Calculate recommended number of hinges based on sash height and weight.
fn calculate_hinge_count(height_mm: f64, weight_kg: f64) -> u8 {
    if weight_kg > 100.0 || height_mm > 2000.0 {
        4
    } else if weight_kg > 60.0 || height_mm > 1400.0 {
        3
    } else {
        2
    }
}

/// Determine hinge side from opening direction.
fn hinge_side_from_direction(direction: Option<OpeningDirection>, panel_type: PanelType) -> HingeSide {
    match panel_type {
        PanelType::Tilt => HingeSide::Bottom,
        _ => match direction {
            Some(OpeningDirection::Left) => HingeSide::Left,
            Some(OpeningDirection::Right) => HingeSide::Right,
            _ => HingeSide::Left, // default
        },
    }
}

/// Determine handle side (opposite of hinges).
fn handle_side_from_hinge(hinge_side: HingeSide) -> HandleSide {
    match hinge_side {
        HingeSide::Left => HandleSide::Right,
        HingeSide::Right => HandleSide::Left,
        HingeSide::Top | HingeSide::Bottom => HandleSide::Center,
    }
}

/// Generate a default hardware set for a given cell configuration.
/// Returns None for non-operable panel types (FixedGlass, Panel, Ventilation).
pub fn default_hardware_set(
    panel_type: PanelType,
    opening_direction: Option<OpeningDirection>,
    cell_width_mm: f64,
    cell_height_mm: f64,
    glass_thickness_mm: f64,
    material: &Material,
    security_class: SecurityClass,
) -> Option<HardwareSet> {
    if !panel_type.is_operable() {
        return None;
    }

    let weight = estimate_sash_weight(cell_width_mm, cell_height_mm, glass_thickness_mm, material);
    let perimeter = 2.0 * (cell_width_mm + cell_height_mm);
    let locking_points = calculate_locking_points(perimeter, &security_class);
    let needs_mushroom = security_class.requires_mushroom_cams();

    let cam_type = if needs_mushroom {
        CamType::PaddenstoelNok
    } else {
        CamType::RolNok
    };

    match panel_type {
        PanelType::TurnTilt | PanelType::Turn => {
            let hinge_count = calculate_hinge_count(cell_height_mm, weight);
            let hinge_side = hinge_side_from_direction(opening_direction, panel_type);
            let handle_side = handle_side_from_hinge(hinge_side);

            Some(HardwareSet {
                hinges: Some(HingeConfig {
                    hinge_type: HingeType::Verdekt,
                    count: hinge_count,
                    load_capacity_kg: weight * 1.5, // 50% safety margin
                    side: hinge_side,
                }),
                handle: Some(HandleConfig {
                    handle_type: HandleType::Kruk,
                    side: handle_side,
                    height_mm: 1050.0, // standard handle height
                    lockable: matches!(security_class, SecurityClass::RC2 | SecurityClass::RC3 | SecurityClass::RC4 | SecurityClass::RC5 | SecurityClass::RC6),
                }),
                locking: Some(LockingConfig {
                    lock_type: LockType::Espagnolet,
                    locking_points,
                    cam_type,
                    cylinder: CylinderType::None,
                }),
                ventilation: None,
                closer: None,
                security_class,
                auto_selected: true,
            })
        }

        PanelType::Tilt => {
            Some(HardwareSet {
                hinges: Some(HingeConfig {
                    hinge_type: HingeType::Opleg,
                    count: 2,
                    load_capacity_kg: weight * 1.5,
                    side: HingeSide::Bottom,
                }),
                handle: Some(HandleConfig {
                    handle_type: HandleType::Knop,
                    side: HandleSide::Center,
                    height_mm: cell_height_mm - 100.0,
                    lockable: false,
                }),
                locking: Some(LockingConfig {
                    lock_type: LockType::Espagnolet,
                    locking_points: locking_points.min(3),
                    cam_type,
                    cylinder: CylinderType::None,
                }),
                ventilation: None,
                closer: None,
                security_class,
                auto_selected: true,
            })
        }

        PanelType::Sliding => {
            Some(HardwareSet {
                hinges: None, // sliding has rollers, not hinges
                handle: Some(HandleConfig {
                    handle_type: HandleType::InlaatGreep,
                    side: HandleSide::Right,
                    height_mm: 1050.0,
                    lockable: true,
                }),
                locking: Some(LockingConfig {
                    lock_type: LockType::SlidingLock,
                    locking_points: locking_points.min(3),
                    cam_type: CamType::HaakSluitNok,
                    cylinder: CylinderType::None,
                }),
                ventilation: None,
                closer: None,
                security_class,
                auto_selected: true,
            })
        }

        PanelType::TopHung => {
            Some(HardwareSet {
                hinges: Some(HingeConfig {
                    hinge_type: HingeType::Opleg,
                    count: 2,
                    load_capacity_kg: weight * 1.5,
                    side: HingeSide::Top,
                }),
                handle: Some(HandleConfig {
                    handle_type: HandleType::Knop,
                    side: HandleSide::Center,
                    height_mm: 200.0, // low handle for top-hung
                    lockable: false,
                }),
                locking: Some(LockingConfig {
                    lock_type: LockType::Espagnolet,
                    locking_points: locking_points.min(2),
                    cam_type,
                    cylinder: CylinderType::None,
                }),
                ventilation: None,
                closer: None,
                security_class,
                auto_selected: true,
            })
        }

        PanelType::BottomHung => {
            Some(HardwareSet {
                hinges: Some(HingeConfig {
                    hinge_type: HingeType::Opleg,
                    count: 2,
                    load_capacity_kg: weight * 1.5,
                    side: HingeSide::Bottom,
                }),
                handle: Some(HandleConfig {
                    handle_type: HandleType::Kruk,
                    side: HandleSide::Center,
                    height_mm: cell_height_mm - 100.0,
                    lockable: false,
                }),
                locking: Some(LockingConfig {
                    lock_type: LockType::Espagnolet,
                    locking_points: locking_points.min(3),
                    cam_type,
                    cylinder: CylinderType::None,
                }),
                ventilation: None,
                closer: None,
                security_class,
                auto_selected: true,
            })
        }

        PanelType::LiftSlide => {
            Some(HardwareSet {
                hinges: None, // lift-slide has rollers
                handle: Some(HandleConfig {
                    handle_type: HandleType::InlaatGreep,
                    side: HandleSide::Right,
                    height_mm: 1050.0,
                    lockable: true,
                }),
                locking: Some(LockingConfig {
                    lock_type: LockType::SlidingLock,
                    locking_points: locking_points.min(4),
                    cam_type: CamType::HaakSluitNok,
                    cylinder: CylinderType::None,
                }),
                ventilation: None,
                closer: None,
                security_class,
                auto_selected: true,
            })
        }

        PanelType::Pivot => {
            let hinge_count = calculate_hinge_count(cell_height_mm, weight);

            Some(HardwareSet {
                hinges: Some(HingeConfig {
                    hinge_type: HingeType::Verdekt,
                    count: hinge_count,
                    load_capacity_kg: weight * 1.5,
                    side: HingeSide::Top, // pivot hinges at top and bottom center
                }),
                handle: Some(HandleConfig {
                    handle_type: HandleType::Kruk,
                    side: HandleSide::Right,
                    height_mm: 1050.0,
                    lockable: false,
                }),
                locking: Some(LockingConfig {
                    lock_type: LockType::Espagnolet,
                    locking_points,
                    cam_type,
                    cylinder: CylinderType::None,
                }),
                ventilation: None,
                closer: None,
                security_class,
                auto_selected: true,
            })
        }

        PanelType::Door => {
            let hinge_count = calculate_hinge_count(cell_height_mm, weight).max(3);
            let hinge_side = hinge_side_from_direction(opening_direction, panel_type);
            let handle_side = handle_side_from_hinge(hinge_side);

            let cylinder = match security_class {
                SecurityClass::RC3 | SecurityClass::RC4 | SecurityClass::RC5 | SecurityClass::RC6 => CylinderType::Skg3,
                SecurityClass::RC2 => CylinderType::Skg2,
                _ => CylinderType::EuroProfile,
            };

            Some(HardwareSet {
                hinges: Some(HingeConfig {
                    hinge_type: HingeType::Opleg,
                    count: hinge_count,
                    load_capacity_kg: weight * 1.5,
                    side: hinge_side,
                }),
                handle: Some(HandleConfig {
                    handle_type: HandleType::KrukKruk,
                    side: handle_side,
                    height_mm: 1050.0,
                    lockable: true,
                }),
                locking: Some(LockingConfig {
                    lock_type: LockType::MultiPoint,
                    locking_points: locking_points.max(3),
                    cam_type,
                    cylinder,
                }),
                ventilation: None,
                closer: None,
                security_class,
                auto_selected: true,
            })
        }

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turn_tilt_auto_selection() {
        let hw = default_hardware_set(
            PanelType::TurnTilt,
            Some(OpeningDirection::Left),
            900.0, 1400.0, 24.0,
            &Material::Wood(crate::kozijn::WoodType::Meranti),
            SecurityClass::None,
        );
        let hw = hw.expect("Should generate hardware for TurnTilt");
        assert!(hw.auto_selected);
        let hinges = hw.hinges.expect("Should have hinges");
        assert_eq!(hinges.side, HingeSide::Left);
        assert!(hinges.count >= 2);
        let handle = hw.handle.expect("Should have handle");
        assert_eq!(handle.side, HandleSide::Right);
        assert_eq!(handle.handle_type, HandleType::Kruk);
        let locking = hw.locking.expect("Should have locking");
        assert_eq!(locking.lock_type, LockType::Espagnolet);
        assert!(locking.locking_points >= 2);
    }

    #[test]
    fn test_door_auto_selection() {
        let hw = default_hardware_set(
            PanelType::Door,
            Some(OpeningDirection::Right),
            900.0, 2315.0, 24.0,
            &Material::Wood(crate::kozijn::WoodType::Meranti),
            SecurityClass::RC2,
        );
        let hw = hw.expect("Should generate hardware for Door");
        let hinges = hw.hinges.expect("Should have hinges");
        assert!(hinges.count >= 3, "Door should have at least 3 hinges");
        assert_eq!(hinges.side, HingeSide::Right);
        let locking = hw.locking.expect("Should have locking");
        assert_eq!(locking.lock_type, LockType::MultiPoint);
        assert!(locking.locking_points >= 3);
        assert_eq!(locking.cylinder, CylinderType::Skg2);
        assert!(locking.cam_type == CamType::PaddenstoelNok);
    }

    #[test]
    fn test_fixed_glass_no_hardware() {
        let hw = default_hardware_set(
            PanelType::FixedGlass,
            None, 600.0, 1200.0, 24.0,
            &Material::Aluminum,
            SecurityClass::None,
        );
        assert!(hw.is_none(), "FixedGlass should not have hardware");
    }

    #[test]
    fn test_sliding_auto_selection() {
        let hw = default_hardware_set(
            PanelType::Sliding,
            None, 1800.0, 2150.0, 24.0,
            &Material::Aluminum,
            SecurityClass::RC2,
        );
        let hw = hw.expect("Should generate hardware for Sliding");
        assert!(hw.hinges.is_none(), "Sliding should not have hinges");
        let handle = hw.handle.expect("Should have handle");
        assert_eq!(handle.handle_type, HandleType::InlaatGreep);
        let locking = hw.locking.expect("Should have locking");
        assert_eq!(locking.lock_type, LockType::SlidingLock);
    }

    #[test]
    fn test_security_class_locking_points() {
        assert!(SecurityClass::RC2.min_locking_points() >= 3);
        assert!(SecurityClass::RC3.min_locking_points() >= 5);
        assert!(SecurityClass::None.min_locking_points() >= 1);
    }

    #[test]
    fn test_sash_weight_estimation() {
        let weight = estimate_sash_weight(900.0, 1400.0, 24.0, &Material::Wood(crate::kozijn::WoodType::Meranti));
        // ~75.6 kg glass + ~16.1 kg frame = ~91.7 kg — sanity check
        assert!(weight > 50.0 && weight < 150.0, "Weight {} out of expected range", weight);
    }
}
