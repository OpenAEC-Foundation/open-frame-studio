use crate::kozijn::Kozijn;
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
}

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
    }

    errors
}
