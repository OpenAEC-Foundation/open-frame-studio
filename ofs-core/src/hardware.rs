use serde::{Deserialize, Serialize};

use crate::kozijn::{Material, OpeningDirection, PanelType};

// ── Security classification ────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SecurityClass {
    #[default]
    None,
    RC1,
    RC2,
    RC3,
    RC4,
    RC5,
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
    pub auto_selected: bool,
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

/// Recommend security class based on element type and accessibility.
pub fn recommend_security_class(panel_type: PanelType, is_ground_floor: bool) -> SecurityClass {
    if !panel_type.is_operable() {
        return SecurityClass::None;
    }
    if is_ground_floor {
        // NEN 5096: bereikbare openingen minimaal RC2
        match panel_type {
            PanelType::Door => SecurityClass::RC2,
            _ => SecurityClass::RC2,
        }
    } else {
        SecurityClass::None
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
