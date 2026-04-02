use crate::kozijn::{GridDivision, Kozijn};
use crate::profile::ProfileRef;

/// Distribute grid divisions evenly across available space
pub fn distribute_evenly(count: usize, total_size: f64, divider_width: f64) -> Vec<GridDivision> {
    let total_divider_space = if count > 1 {
        (count - 1) as f64 * divider_width
    } else {
        0.0
    };
    let cell_size = (total_size - total_divider_space) / count as f64;

    (0..count)
        .map(|i| GridDivision {
            size: cell_size,
            divider_profile: if i > 0 {
                Some(ProfileRef::default_divider())
            } else {
                None
            },
        })
        .collect()
}

/// Create a kozijn from a common template
pub fn template_single_turn_tilt(width: f64, height: f64) -> Kozijn {
    let mut k = Kozijn::new("Draaikiepraam", "K01", width, height);
    k.cells[0].panel_type = crate::kozijn::PanelType::TurnTilt;
    k.cells[0].opening_direction = Some(crate::kozijn::OpeningDirection::Left);
    k
}

pub fn template_double_turn_tilt(width: f64, height: f64) -> Kozijn {
    let fw = 67.0;
    let inner_w = width - 2.0 * fw;
    let half = (inner_w - fw) / 2.0; // minus one divider

    let mut k = Kozijn::new("Dubbel draaikiepraam", "K01", width, height);
    k.grid.columns = vec![
        GridDivision {
            size: half,
            divider_profile: None,
        },
        GridDivision {
            size: half,
            divider_profile: Some(ProfileRef::default_divider()),
        },
    ];
    k.rebuild_cells();
    k.cells[0].panel_type = crate::kozijn::PanelType::TurnTilt;
    k.cells[0].opening_direction = Some(crate::kozijn::OpeningDirection::Left);
    k.cells[1].panel_type = crate::kozijn::PanelType::TurnTilt;
    k.cells[1].opening_direction = Some(crate::kozijn::OpeningDirection::Right);
    k
}

pub fn template_sliding_door(width: f64, height: f64) -> Kozijn {
    let fw = 67.0;
    let inner_w = width - 2.0 * fw;
    let half = (inner_w - fw) / 2.0;

    let mut k = Kozijn::new("Schuifpui", "P01", width, height);
    k.grid.columns = vec![
        GridDivision {
            size: half,
            divider_profile: None,
        },
        GridDivision {
            size: half,
            divider_profile: Some(ProfileRef::default_divider()),
        },
    ];
    k.rebuild_cells();
    k.cells[0].panel_type = crate::kozijn::PanelType::FixedGlass;
    k.cells[1].panel_type = crate::kozijn::PanelType::Sliding;
    k
}

pub fn template_front_door(width: f64, height: f64) -> Kozijn {
    let fw = 67.0;
    let inner_h = height - 2.0 * fw;
    let top_light = 400.0;
    let door_h = inner_h - top_light - fw;

    let mut k = Kozijn::new("Voordeur", "D01", width, height);
    k.grid.rows = vec![
        GridDivision {
            size: top_light,
            divider_profile: None,
        },
        GridDivision {
            size: door_h,
            divider_profile: Some(ProfileRef::default_divider()),
        },
    ];
    k.rebuild_cells();
    k.cells[0].panel_type = crate::kozijn::PanelType::FixedGlass;
    k.cells[1].panel_type = crate::kozijn::PanelType::Door;
    k.cells[1].opening_direction = Some(crate::kozijn::OpeningDirection::Inward);
    k
}
