use crate::vliesgevel::Vliesgevel;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VliesgevalValidationError {
    #[error("Stijlafstand {0:.0}mm overschrijdt maximum {1:.0}mm")]
    MullionSpacingTooLarge(f64, f64),
    #[error("Stijlafstand {0:.0}mm is kleiner dan minimum {1:.0}mm")]
    MullionSpacingTooSmall(f64, f64),
    #[error("Regelafstand {0:.0}mm overschrijdt maximum {1:.0}mm")]
    TransomSpacingTooLarge(f64, f64),
    #[error("Paneel ({0},{1}) te groot: {2:.2} m2 (max {3:.2} m2)")]
    PanelTooLarge(usize, usize, f64, f64),
    #[error("Aantal panelen ({0}) komt niet overeen met grid ({1} kolommen x {2} rijen = {3})")]
    PanelCountMismatch(usize, usize, usize, usize),
}

const MAX_MULLION_SPACING: f64 = 2500.0;
const MIN_MULLION_SPACING: f64 = 300.0;
const MAX_TRANSOM_SPACING: f64 = 4000.0;
const MAX_PANEL_AREA_M2: f64 = 6.0;

pub fn validate_vliesgevel(vg: &Vliesgevel) -> Vec<VliesgevalValidationError> {
    let mut errors = Vec::new();

    // Check mullion spacings
    let mut prev_x = 0.0;
    for m in &vg.mullions {
        let spacing = m.x_position - prev_x;
        if spacing > MAX_MULLION_SPACING {
            errors.push(VliesgevalValidationError::MullionSpacingTooLarge(spacing, MAX_MULLION_SPACING));
        }
        if spacing < MIN_MULLION_SPACING {
            errors.push(VliesgevalValidationError::MullionSpacingTooSmall(spacing, MIN_MULLION_SPACING));
        }
        prev_x = m.x_position;
    }
    let last_spacing = vg.overall_width - prev_x;
    if last_spacing > MAX_MULLION_SPACING {
        errors.push(VliesgevalValidationError::MullionSpacingTooLarge(last_spacing, MAX_MULLION_SPACING));
    }

    // Check transom spacings
    let mut prev_y = 0.0;
    for t in &vg.transoms {
        let spacing = t.y_position - prev_y;
        if spacing > MAX_TRANSOM_SPACING {
            errors.push(VliesgevalValidationError::TransomSpacingTooLarge(spacing, MAX_TRANSOM_SPACING));
        }
        prev_y = t.y_position;
    }
    let last_y_spacing = vg.overall_height - prev_y;
    if last_y_spacing > MAX_TRANSOM_SPACING {
        errors.push(VliesgevalValidationError::TransomSpacingTooLarge(last_y_spacing, MAX_TRANSOM_SPACING));
    }

    // Check panel sizes
    let col_zones = vg.column_zones();
    let row_zones = vg.row_zones();
    for panel in &vg.panels {
        if panel.col < col_zones.len() && panel.row < row_zones.len() {
            let (x1, x2) = col_zones[panel.col];
            let (y1, y2) = row_zones[panel.row];
            let area_m2 = ((x2 - x1) * (y2 - y1)) / 1e6;
            if area_m2 > MAX_PANEL_AREA_M2 {
                errors.push(VliesgevalValidationError::PanelTooLarge(
                    panel.col, panel.row, area_m2, MAX_PANEL_AREA_M2,
                ));
            }
        }
    }

    // Check panel count
    let expected = vg.num_cols() * vg.num_rows();
    if vg.panels.len() != expected {
        errors.push(VliesgevalValidationError::PanelCountMismatch(
            vg.panels.len(), vg.num_cols(), vg.num_rows(), expected,
        ));
    }

    errors
}
