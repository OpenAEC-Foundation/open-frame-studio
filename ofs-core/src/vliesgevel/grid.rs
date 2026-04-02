use uuid::Uuid;

use crate::kozijn::{Material, Placement};
use crate::profile::ProfileRef;
use crate::vliesgevel::*;

/// Create a regular curtain wall grid with uniform mullion and transom spacing.
pub fn create_regular_grid(
    width: f64,
    height: f64,
    mullion_spacing: f64,
    transom_spacing: f64,
) -> Vliesgevel {
    let mullion_width = 50.0;
    let transom_width = 50.0;

    let mut mullions = Vec::new();
    let mut x = mullion_spacing;
    while x < width - mullion_width {
        mullions.push(MullionLine {
            x_position: x,
            profile: None,
            anchors: vec![],
        });
        x += mullion_spacing;
    }

    let mut transoms = Vec::new();
    let mut y = transom_spacing;
    while y < height - transom_width {
        transoms.push(TransomLine {
            y_position: y,
            profile: None,
            anchors: vec![],
        });
        y += transom_spacing;
    }

    let mut vg = Vliesgevel {
        id: Uuid::new_v4(),
        name: "Vliesgevel".into(),
        mark: "VG01".into(),
        overall_width: width,
        overall_height: height,
        material: Material::Aluminum,
        mullion_profile: ProfileRef::default_frame(),
        transom_profile: ProfileRef::default_frame(),
        mullion_width,
        transom_width,
        mullions,
        transoms,
        panels: vec![],
        placement: Placement::default(),
    };
    vg.rebuild_panels();
    vg
}

/// Standard stick system: 1500mm mullion spacing, 1200mm transom spacing.
pub fn template_stick_system(width: f64, height: f64) -> Vliesgevel {
    let mut vg = create_regular_grid(width, height, 1500.0, 1200.0);
    vg.name = "Stijl-regel vliesgevel".into();
    vg
}

/// Unitized system: 1200mm wide, floor-to-floor (typically 3600mm).
pub fn template_unitized(width: f64, height: f64) -> Vliesgevel {
    let mut vg = create_regular_grid(width, height, 1200.0, height);
    vg.name = "Elementgevel".into();
    vg
}

/// Shopfront: wide spacing with a door zone at the bottom.
pub fn template_shopfront(width: f64, height: f64) -> Vliesgevel {
    let mut vg = create_regular_grid(width, height, 2000.0, 2400.0);
    vg.name = "Winkelpui".into();

    // Set bottom-left panel as door
    if let Some(panel) = vg.panel_at_mut(0, 0) {
        panel.panel_type = CurtainPanelType::Door;
    }

    // Set bottom row panels above door height as spandrel
    let num_cols = vg.num_cols();
    for col in 0..num_cols {
        if let Some(panel) = vg.panel_at_mut(col, 1) {
            if panel.panel_type == CurtainPanelType::Glass {
                // keep as glass (vision zone)
            }
        }
    }

    vg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regular_grid() {
        let vg = create_regular_grid(6000.0, 3600.0, 1500.0, 1200.0);
        assert_eq!(vg.mullions.len(), 3); // at 1500, 3000, 4500
        assert_eq!(vg.transoms.len(), 2); // at 1200, 2400
        assert_eq!(vg.panels.len(), 12);  // 4 x 3
    }

    #[test]
    fn test_stick_system_template() {
        let vg = template_stick_system(9000.0, 3600.0);
        assert_eq!(vg.mullions.len(), 5); // at 1500, 3000, 4500, 6000, 7500
        assert_eq!(vg.transoms.len(), 2); // at 1200, 2400
        assert_eq!(vg.panels.len(), 18);  // 6 x 3
    }

    #[test]
    fn test_unitized_template() {
        let vg = template_unitized(6000.0, 3600.0);
        assert_eq!(vg.mullions.len(), 4); // at 1200, 2400, 3600, 4800
        assert_eq!(vg.transoms.len(), 0); // no transoms (floor-to-floor)
        assert_eq!(vg.panels.len(), 5);   // 5 x 1
    }
}
