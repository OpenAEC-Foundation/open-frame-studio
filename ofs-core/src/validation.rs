use crate::kozijn::{Kozijn, Material};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Kozijn breedte {0}mm is kleiner dan minimum {1}mm")]
    WidthTooSmall(f64, f64),
    #[error("Kozijn hoogte {0}mm is kleiner dan minimum {1}mm")]
    HeightTooSmall(f64, f64),
    #[error("Kozijn breedte {0}mm overschrijdt maximum {1}mm")]
    WidthTooLarge(f64, f64),
    #[error("Kozijn hoogte {0}mm overschrijdt maximum {1}mm")]
    HeightTooLarge(f64, f64),
    #[error("Cel {0} is te klein: {1}mm x {2}mm (minimum {3}mm)")]
    CellTooSmall(usize, f64, f64, f64),
    #[error("Aantal cellen ({0}) komt niet overeen met grid ({1} kolommen x {2} rijen = {3})")]
    CellCountMismatch(usize, usize, usize, usize),
    #[error("Cel {0}: beweegbaar vak zonder hang- en sluitwerk")]
    MissingHardware(usize),
    #[error("Cel {0}: {1} scharnieren, minimaal {2} vereist voor dit gewicht/hoogte")]
    InsufficientHinges(usize, u8, u8),
    #[error("Cel {0}: glaslat hoogte {1}mm is kleiner dan KVT-minimum {2}mm")]
    GlaslatTooShallow(usize, f64, f64),
    #[error("Glasinval {0}mm is kleiner dan KVT-minimum {1}mm (sponninghoogte dagkant)")]
    GlasinvalTooSmall(f64, f64),
    #[error("Cel {0}: vluchtraam moet beweegbaar (te openen) zijn")]
    EscapeWindowNotOperable(usize),
    #[error("Cel {0}: vluchtraam vrije opening {1:.0}x{2:.0}mm is kleiner dan minimum {3:.0}mm")]
    EscapeWindowTooSmall(usize, f64, f64, f64),
}

/// Minimum free opening (width and height) for an escape window in mm.
const MIN_ESCAPE_OPENING: f64 = 600.0;

/// KVT 12.2 minimum glasinval (sponninghoogte) at the day sides of stiles and
/// head for wood frames, in mm. Tops of bottom/intermediate rails may go down
/// to 14/16 mm (KVT 13.2/14.2), but those sit in separate sill profiles
/// outside the frame snapshot, so they are not judged here.
const MIN_GLASINVAL_WOOD: f64 = 17.0;

const MIN_WIDTH: f64 = 200.0;
const MIN_HEIGHT: f64 = 200.0;
const MAX_WIDTH: f64 = 6000.0;
const MAX_HEIGHT: f64 = 4000.0;
const MIN_CELL_SIZE: f64 = 100.0;

pub fn validate(kozijn: &Kozijn) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    let w = kozijn.frame.outer_width;
    let h = kozijn.frame.outer_height;

    if w < MIN_WIDTH {
        errors.push(ValidationError::WidthTooSmall(w, MIN_WIDTH));
    }
    if h < MIN_HEIGHT {
        errors.push(ValidationError::HeightTooSmall(h, MIN_HEIGHT));
    }
    if w > MAX_WIDTH {
        errors.push(ValidationError::WidthTooLarge(w, MAX_WIDTH));
    }
    if h > MAX_HEIGHT {
        errors.push(ValidationError::HeightTooLarge(h, MAX_HEIGHT));
    }

    // Glasinval plausibility (hout en hout-aluminium): KVT 12.2 stelt 17 mm
    // minimum voor de glasranddekking op stijlen en bovendorpel. PVC en
    // aluminium kennen geen universele norm — onderzochte systemen lopen van
    // 13,5 mm (Reynaers SlimLine 38) tot 27 mm (MasterLine 8) — dus daar
    // wordt niet gevlagd. Alleen expliciete snapshotwaarden worden
    // beoordeeld: kozijnen zonder snapshot (of zonder sponningdata in de
    // snapshot) blijven ongemoeid.
    if matches!(kozijn.frame.material, Material::Wood(_) | Material::WoodAluminum) {
        if let Some(gi) = kozijn
            .frame
            .profile_snapshot
            .as_ref()
            .and_then(|s| s.glasinval.or(s.sponning_hoogte))
        {
            if gi < MIN_GLASINVAL_WOOD {
                errors.push(ValidationError::GlasinvalTooSmall(gi, MIN_GLASINVAL_WOOD));
            }
        }
    }

    let expected_cells = kozijn.grid.columns.len() * kozijn.grid.rows.len();
    if kozijn.cells.len() != expected_cells {
        errors.push(ValidationError::CellCountMismatch(
            kozijn.cells.len(),
            kozijn.grid.columns.len(),
            kozijn.grid.rows.len(),
            expected_cells,
        ));
    }

    for (i, col) in kozijn.grid.columns.iter().enumerate() {
        if col.size < MIN_CELL_SIZE {
            errors.push(ValidationError::CellTooSmall(i, col.size, 0.0, MIN_CELL_SIZE));
        }
    }
    for (i, row) in kozijn.grid.rows.iter().enumerate() {
        if row.size < MIN_CELL_SIZE {
            errors.push(ValidationError::CellTooSmall(i, 0.0, row.size, MIN_CELL_SIZE));
        }
    }

    // HSW validation: operable cells should have hardware
    for (i, cell) in kozijn.cells.iter().enumerate() {
        if cell.panel_type.is_operable() && cell.hardware_set.is_none() {
            errors.push(ValidationError::MissingHardware(i));
        }
        // Glaslat fit: KVT requires a bead height of at least 17 mm.
        if let Some(gl) = cell.glaslat.as_ref() {
            if gl.height_mm < 17.0 {
                errors.push(ValidationError::GlaslatTooShallow(i, gl.height_mm, 17.0));
            }
        }
        // Vluchtraam (escape window): must be operable and meet the minimum free opening.
        if cell.is_escape {
            if !cell.panel_type.is_operable() {
                errors.push(ValidationError::EscapeWindowNotOperable(i));
            } else {
                let cols = kozijn.grid.columns.len().max(1);
                let cw = kozijn.grid.columns.get(i % cols).map(|c| c.size).unwrap_or(0.0);
                let ch = kozijn.grid.rows.get(i / cols).map(|r| r.size).unwrap_or(0.0);
                let sw = cell.sash_width.unwrap_or(69.0);
                let free_w = cw - 2.0 * sw;
                let free_h = ch - 2.0 * sw;
                if free_w < MIN_ESCAPE_OPENING || free_h < MIN_ESCAPE_OPENING {
                    errors.push(ValidationError::EscapeWindowTooSmall(
                        i,
                        free_w,
                        free_h,
                        MIN_ESCAPE_OPENING,
                    ));
                }
            }
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kozijn::Kozijn;
    use crate::profile::ProfileSnapshot;

    fn has_glasinval_error(errors: &[ValidationError]) -> bool {
        errors
            .iter()
            .any(|e| matches!(e, ValidationError::GlasinvalTooSmall(_, _)))
    }

    #[test]
    fn glasinval_below_kvt_minimum_flags_wood() {
        let mut k = Kozijn::new("Test", "T01", 900.0, 1400.0); // hout (meranti)

        // Zonder snapshot: geen oordeel (regressieveilig voor oude projecten).
        assert!(!has_glasinval_error(&validate(&k)));

        // Snapshot zonder sponningdata: geen oordeel.
        k.frame.profile_snapshot = Some(ProfileSnapshot::default());
        assert!(!has_glasinval_error(&validate(&k)));

        // Expliciete sponninghoogte onder de KVT 12.2-grens van 17 mm.
        k.frame.profile_snapshot = Some(ProfileSnapshot {
            sponning_hoogte: Some(12.0),
            ..Default::default()
        });
        let errors = validate(&k);
        assert!(has_glasinval_error(&errors));
        assert!(errors
            .iter()
            .any(|e| e.to_string().contains("Glasinval 12mm")));

        // Op de grens (17) is het goed.
        k.frame.profile_snapshot = Some(ProfileSnapshot {
            sponning_hoogte: Some(17.0),
            ..Default::default()
        });
        assert!(!has_glasinval_error(&validate(&k)));

        // Expliciete glasinval gaat vóór de sponninghoogte.
        k.frame.profile_snapshot = Some(ProfileSnapshot {
            sponning_hoogte: Some(28.0),
            glasinval: Some(12.0),
            ..Default::default()
        });
        assert!(has_glasinval_error(&validate(&k)));
    }

    #[test]
    fn glasinval_check_skips_pvc_and_aluminum() {
        // Reynaers SlimLine 38 heeft legitiem 13,5 mm — geen universele norm
        // voor PVC/alu, dus daar geen melding.
        let mut k = Kozijn::new("Test", "T02", 900.0, 1400.0);
        k.frame.material = crate::kozijn::Material::Aluminum;
        k.frame.profile_snapshot = Some(ProfileSnapshot {
            sponning_hoogte: Some(13.5),
            ..Default::default()
        });
        assert!(!has_glasinval_error(&validate(&k)));

        k.frame.material = crate::kozijn::Material::Pvc;
        assert!(!has_glasinval_error(&validate(&k)));
    }
}
