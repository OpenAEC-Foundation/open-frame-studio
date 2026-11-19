use crate::kozijn::{Kozijn, Material, ShapeType};
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
    #[error("Sponninghoogte {0}mm is kleiner dan KVT-minimum {1}mm")]
    GlasinvalTooSmall(f64, f64),
    #[error("Cel {0}: glaslatvoet {1}mm is smaller dan de benodigde oplegging van {2}mm")]
    BeadBearingTooSmall(usize, f64, f64),
    #[error("Cel {0}: sponningbreedte {1}mm, benodigd {2}mm voor glas, afdichtingen en oplegging")]
    GlazingPocketTooNarrow(usize, f64, f64),
    #[error("De opgegeven boogpijl van {0:.1} mm valt buiten de maakbare booghoogte (groter dan 0 en maximaal {1:.1} mm).")]
    InvalidArchRise(f64, f64),
    #[error("De boog en de stijlen kunnen bij deze afmetingen geen gesloten buitencontour vormen.")]
    CurvedFrameNoOutline,
    #[error("Binnen het gebogen kozijn blijft geen dagopening over. Controleer de profielbreedte en de vakverdeling.")]
    CurvedFrameNoOpening,
    #[error("Onder de aanzet van de boog is {0:.1} mm beschikbaar voor de rijen. Alleen de tussendorpels vragen al {1:.1} mm.")]
    ArchRowsDoNotFit(f64, f64),
    #[error("Vak {0}: voor {1} van de opgegeven ruiten blijft geen glasopening over binnen de gebogen contour.")]
    CurvedGlassMissing(usize, usize),
    #[error("{piece}, kop {end}: {reason}.")]
    DividerJoint { piece: String, end: &'static str, reason: crate::curved_timber::TenonIssue },
    #[error("{location}, gevraagde pen {length:.1} mm: {reason}.")]
    ArchJoint { location: &'static str, length: f64, reason: crate::curved_timber::TenonIssue },
    #[error("Cel {0}: vluchtraam moet beweegbaar (te openen) zijn")]
    EscapeWindowNotOperable(usize),
    #[error("Cel {0}: vluchtraam vrije opening {1:.0}x{2:.0}mm is kleiner dan minimum {3:.0}mm")]
    EscapeWindowTooSmall(usize, f64, f64, f64),
}

/// Minimum free opening (width and height) for an escape window in mm.
const MIN_ESCAPE_OPENING: f64 = 600.0;

/// KVT 12.2 minimum rebate height at the day sides of stiles and
/// head for wood frames, in mm. Tops of bottom/intermediate rails may go down
/// to 14/16 mm (KVT 13.2/14.2), but those sit in separate sill profiles
/// outside the frame snapshot, so they are not judged here.
const MIN_REBATE_HEIGHT_WOOD: f64 = 17.0;

const MIN_WIDTH: f64 = 200.0;
const MIN_HEIGHT: f64 = 200.0;
const MAX_WIDTH: f64 = 6000.0;
const MAX_HEIGHT: f64 = 4000.0;
const MIN_CELL_SIZE: f64 = 100.0;

fn validate_curved_frame(k: &Kozijn, geometry: &crate::geometry::KozijnGeometry2D, errors: &mut Vec<ValidationError>) {
    use crate::curved_contour::FrameCurve;
    let Some(frame) = &geometry.curved_frame else { return };
    if matches!(k.frame.shape.shape_type, ShapeType::Arched | ShapeType::ArchedTrapezoid) {
        let maximum = FrameCurve::max_rise(&k.frame);
        if let Some(rise) = k.frame.shape.arch_height {
            if !rise.is_finite() || rise <= 0.0 || rise > maximum + 1e-7 {
                errors.push(ValidationError::InvalidArchRise(rise, maximum));
            }
        }
        if k.layout.is_none() && !k.grid.rows_from_day && k.grid.rows.len() > 1 {
            let available = (k.frame.outer_height - k.frame.frame_width - FrameCurve::rise(&k.frame)).max(0.0);
            let dividers = (k.grid.rows.len() - 1) as f64 * k.frame.frame_width;
            if available <= dividers {
                errors.push(ValidationError::ArchRowsDoNotFit(available, dividers));
            }
        }
    }
    for joint in &geometry.arch_joints {
        if let Some(reason) = joint.issue {
            use crate::curved_timber::ArchJointKind;
            let location = match (joint.kind, joint.corner) {
                (ArchJointKind::RingSeam, 0) => "Linker ringnaad",
                (ArchJointKind::RingSeam, _) => "Rechter ringnaad",
                (ArchJointKind::Springing, 0) => "Linker boogvoet",
                (ArchJointKind::Springing, _) => "Rechter boogvoet",
            };
            errors.push(ValidationError::ArchJoint {
                location,
                length: joint.requested_length_mm,
                reason,
            });
        }
    }
    for part in &geometry.curved_divider_timber {
        let (name, start, end) = if part.vertical { ("Tussenstijl", "boven", "onder") } else { ("Tussendorpel", "links", "rechts") };
        for (profile, location) in [(&part.start, start), (&part.end, end)] {
            if let Some(reason) = profile.tenon_issue {
                errors.push(ValidationError::DividerJoint {
                    piece: format!("{name} {}, deel {}", part.source_index + 1, part.piece_index + 1),
                    end: location, reason,
                });
            }
        }
    }
    if frame.outer.segments.is_empty() {
        errors.push(ValidationError::CurvedFrameNoOutline);
    } else if frame.day.segments.is_empty() || geometry.cell_rects.iter().all(|cr| {
        cr.opening_contour.as_ref().map_or(true, |p| p.segments.is_empty())
    }) {
        errors.push(ValidationError::CurvedFrameNoOpening);
    } else {
        for cr in &geometry.cell_rects {
            if cr.missing_glass_panes > 0 {
                errors.push(ValidationError::CurvedGlassMissing(cr.cell_index + 1, cr.missing_glass_panes));
            }
        }
    }
}

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

    let timber = matches!(kozijn.frame.material, Material::Wood(_) | Material::WoodAluminum);
    let curved = matches!(kozijn.frame.shape.shape_type, ShapeType::Round | ShapeType::Arched | ShapeType::ArchedTrapezoid);
    if timber || curved {
        let geometry = crate::geometry::compute_2d_geometry(kozijn);
        validate_curved_frame(kozijn, &geometry, &mut errors);
        for cr in geometry.cell_rects.iter().filter(|_| timber) {
            // Paired door leaves use the same section and glazing specification.
            let Some(fit) = &cr.fit else { continue };
            let height = fit.contours.as_ref().map_or(fit.day.x - fit.rebate.x, |c| c.rebate_height);
            let min_height = MIN_REBATE_HEIGHT_WOOD.max(height);
            if height + 1e-6 < MIN_REBATE_HEIGHT_WOOD {
                errors.push(ValidationError::GlasinvalTooSmall(height, MIN_REBATE_HEIGHT_WOOD));
            }
            if fit.bead_height + 1e-6 < min_height {
                errors.push(ValidationError::GlaslatTooShallow(cr.cell_index, fit.bead_height, min_height));
            }
            let min_bearing = match fit.bead_position {
                crate::glaslat::GlaslatPosition::Binnen => 13.0,
                crate::glaslat::GlaslatPosition::Buiten => 15.0,
            };
            if fit.bead_width < min_bearing {
                errors.push(ValidationError::BeadBearingTooSmall(cr.cell_index, fit.bead_width, min_bearing));
            }
            if let Some(depth) = fit.rebate_depth {
                let needed = fit.glass_thickness + 2.0 * fit.seal_thickness + min_bearing;
                if depth + 1e-6 < needed {
                    errors.push(ValidationError::GlazingPocketTooNarrow(cr.cell_index, depth, needed));
                }
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
    fn unknown_left_arch_profile_flags_only_that_foot() {
        let mut k = Kozijn::new("Boog", "B1", 1200., 1500.);
        k.frame.shape.shape_type = ShapeType::Arched;
        k.frame.shape.arch_height = Some(300.);
        k.frame.left_profile = Some(crate::profile::ProfileRef {
            id: "onbekende-stijl".into(), name: "Eigen stijl".into(),
        });
        let feet: Vec<_> = validate(&k).into_iter().filter_map(|error| match error {
            ValidationError::ArchJoint { location, reason: crate::curved_timber::TenonIssue::UnknownProfile, .. } => Some(location),
            _ => None,
        }).collect();
        assert_eq!(feet, ["Linker boogvoet"]);
    }

    #[test]
    fn unresolved_half_ring_identifies_each_radial_seam() {
        let mut k = Kozijn::new("Rond", "R1", 1000., 1000.);
        k.frame.shape.shape_type = ShapeType::Round;
        k.frame.top_profile = Some(crate::profile::ProfileRef {
            id: "onbekende-bovenboog".into(), name: "Eigen bovenboog".into(),
        });
        let locations: Vec<_> = validate(&k).into_iter().filter_map(|error| match error {
            ValidationError::ArchJoint { location, reason: crate::curved_timber::TenonIssue::UnknownProfile, .. } => Some(location),
            _ => None,
        }).collect();
        assert_eq!(locations, ["Linker ringnaad", "Rechter ringnaad"]);
    }

    #[test]
    fn unresolved_divider_profile_identifies_both_physical_ends() {
        let mut k = Kozijn::new("Rond", "R1", 1200., 1200.);
        k.frame.shape.shape_type = ShapeType::Round;
        k.add_divider_in_elevation(false, 450.);
        k.grid.columns[1].divider_profile = Some(crate::profile::ProfileRef {
            id: "onbekend-profiel".into(), name: "Onbekend".into(),
        });
        let locations: Vec<_> = validate(&k).into_iter().filter_map(|error| match error {
            ValidationError::DividerJoint { end, reason: crate::curved_timber::TenonIssue::UnknownProfile, .. } => Some(end),
            _ => None,
        }).collect();
        assert_eq!(locations, ["boven", "onder"]);
    }

    #[test]
    fn invalid_curved_frames_and_missing_glass_are_reported_for_all_materials() {
        use crate::kozijn::PanelType;
        let mut k = Kozijn::new("Rond", "R01", 240.0, 240.0);
        k.frame.material = Material::Aluminum;
        k.frame.shape.shape_type = ShapeType::Round;
        assert!(validate(&k).is_empty());
        k.cells[0].panel_type = PanelType::TurnTilt;
        assert!(validate(&k).iter().any(|e| matches!(e, ValidationError::CurvedGlassMissing(1, 1))));

        k.frame.frame_width = 125.0;
        assert!(validate(&k).iter().any(|e| matches!(e, ValidationError::CurvedFrameNoOpening)));
        k.frame.shape.shape_type = ShapeType::ArchedTrapezoid;
        k.frame.shape.left_angle = Some(45.0);
        k.frame.shape.right_angle = Some(45.0);
        assert!(validate(&k).iter().any(|e| matches!(e, ValidationError::CurvedFrameNoOutline)));
    }

    #[test]
    fn oversized_arch_and_rows_report_their_limits_without_fabricating_a_transom() {
        let mut k = Kozijn::new("Boog", "B01", 1200.0, 400.0);
        k.frame.shape.shape_type = ShapeType::Arched;
        k.frame.shape.arch_height = Some(900.0);
        k.add_row(150.0);
        let g = crate::geometry::compute_2d_geometry(&k);
        let sill_top = k.frame.outer_height - k.frame.frame_width;
        assert_eq!(g.cell_rects[0].rect.y, sill_top);
        assert!(g.cell_rects[0].fit.is_some());
        assert!(g.curved_dividers[0].segments.is_empty());
        assert!(g.cell_rects[1].fit.is_none());
        assert!(g.cell_rects.iter().all(|cr| cr.rect.y + cr.rect.height <= sill_top));
        let issues = validate(&k);
        assert!(issues.iter().any(|e| matches!(e, ValidationError::InvalidArchRise(900.0, 333.0))));
        assert!(issues.iter().any(|e| matches!(e, ValidationError::ArchRowsDoNotFit(0.0, 67.0))));
        assert!(issues.iter().any(|e| matches!(e, ValidationError::CurvedGlassMissing(2, 1))));
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
            .any(|e| e.to_string().contains("Sponninghoogte 12mm")));

        // Op de grens (17) is het goed.
        k.frame.profile_snapshot = Some(ProfileSnapshot {
            sponning_hoogte: Some(17.0),
            ..Default::default()
        });
        assert!(!has_glasinval_error(&validate(&k)));

        // Netto glasranddekking heeft een andere referentielijn.
        k.frame.profile_snapshot = Some(ProfileSnapshot {
            sponning_hoogte: Some(28.0),
            glasinval: Some(12.0),
            ..Default::default()
        });
        assert!(!has_glasinval_error(&validate(&k)));
    }

    #[test]
    fn timber_bead_needs_height_bearing_and_space_for_the_glazing() {
        let mut k = Kozijn::new("Test", "T03", 900.0, 1400.0);
        k.frame.profile_snapshot = Some(ProfileSnapshot {
            sponning_hoogte: Some(20.0),
            sponning_diepte: Some(51.0),
            ..Default::default()
        });
        k.cells[0].glaslat = Some(crate::glaslat::Glaslat {
            position: crate::glaslat::GlaslatPosition::Buiten, width_mm: 13.0,
            ..Default::default()
        });
        k.cells[0].glazing.thickness_mm = 44.0;
        let errors = validate(&k);
        assert!(errors.iter().any(|e| matches!(e, ValidationError::GlaslatTooShallow(0, _, _))));
        assert!(errors.iter().any(|e| matches!(e, ValidationError::BeadBearingTooSmall(0, _, _))));
        assert!(errors.iter().any(|e| matches!(e, ValidationError::GlazingPocketTooNarrow(0, _, _))));
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
