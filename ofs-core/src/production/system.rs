use super::{GasketListItem, GasketType, HardwareListItem, SealMeasureLine, SealReference};
use crate::curved_contour::{Contour, SashLeaf};
use crate::geometry::{CellRect, Rect2D};
use crate::glaslat::GlaslatPosition;
use crate::glazing_fit::GlazingFit;
use crate::hardware::{FittingKind, HardwareMount, HardwareSet};
use crate::kozijn::{Cell, HardwareType};
use std::collections::BTreeMap;

pub(super) fn gaskets(
    fit: &GlazingFit,
    piece: &str,
    profile: &str,
    rows: &mut Vec<GasketListItem>,
) {
    let Some(system) = &fit.system_seals else {
        return;
    };
    let contours = fit.contours.as_ref();
    let rect_perimeter = |r: &Rect2D| 2.0 * (r.width + r.height);
    let day = (
        SealMeasureLine::GlazingDay,
        contours.map_or_else(|| rect_perimeter(&fit.day), |c| c.day.perimeter()),
    );
    let bead = (
        SealMeasureLine::BeadOpening,
        contours.map_or_else(
            || rect_perimeter(&fit.rebate.inset(fit.bead_height)),
            |c| c.bead_inner.perimeter(),
        ),
    );
    let mut add = |kind, (line, length_mm), quantity| {
        rows.push(GasketListItem {
            gasket_type: kind,
            length_mm,
            quantity,
            reference: Some(SealReference {
                piece_id: piece.into(),
                profile_name: profile.into(),
                line,
            }),
        })
    };
    let (inside, outside) = match fit.bead_position {
        GlaslatPosition::Binnen => (bead, day),
        GlaslatPosition::Buiten => (day, bead),
    };
    add(GasketType::GlazingInner, inside, 1);
    add(GasketType::GlazingOuter, outside, 1);
    let Some(sash) = &fit.sash_outer else { return };
    let length = contours
        .and_then(|c| c.sash_outer.as_ref())
        .map_or_else(|| rect_perimeter(sash), Contour::perimeter);
    let measure = (SealMeasureLine::SashOuter, length);
    match system.sealing_levels {
        Some(1) => add(GasketType::ClosingSingle, measure, 1),
        Some(2 | 3) => {
            add(GasketType::ClosingInner, measure, 1);
            if system.sealing_levels == Some(3) {
                add(GasketType::ClosingCentre, measure, 1);
            }
            add(GasketType::ClosingOuter, measure, 1);
        }
        _ => add(GasketType::ClosingUnspecified, measure, 0),
    }
}

pub(super) fn hardware(
    cr: &CellRect,
    cell: Option<&Cell>,
    mark: &str,
    rows: &mut Vec<HardwareListItem>,
) {
    let cell_piece = format!("{mark}-V{}", cr.cell_index + 1);
    let mut legacy_written = false;
    for leaf in &cr.hardware_leaves {
        let piece = match leaf.leaf {
            Some(SashLeaf::Left) => format!("{cell_piece}-L"),
            Some(SashLeaf::Right) => format!("{cell_piece}-R"),
            None => cell_piece.clone(),
        };
        if leaf.set.is_none() && cell.is_some_and(|c| c.hardware_set.is_none()) && !legacy_written {
            for item in cell.into_iter().flat_map(|c| &c.hardware) {
                let kind = match item.hardware_type {
                    HardwareType::Hinge => "Scharnier",
                    HardwareType::Handle => "Greep",
                    HardwareType::Lock => "Sluiting",
                    HardwareType::Ventilation => "Ventilatierooster",
                    HardwareType::Closer => "Deurdranger",
                };
                let detail = [
                    item.brand.as_deref(),
                    item.model.as_deref(),
                    Some(&item.position),
                ]
                .into_iter()
                .flatten()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
                rows.push(HardwareListItem {
                    piece_id: Some(cell_piece.clone()),
                    cell_index: cr.cell_index,
                    component: kind.into(),
                    description: detail,
                    quantity: 1,
                });
            }
            legacy_written = true;
        }
        if let Some(issue) = leaf.issue {
            rows.push(HardwareListItem {
                piece_id: Some(piece),
                cell_index: cr.cell_index,
                component: "Beslaguitwerking".into(),
                description: issue.label_nl().into(),
                quantity: 0,
            });
            continue;
        }
        let mut counts = BTreeMap::new();
        for part in &leaf.components {
            let mounting = match part.mount {
                HardwareMount::Sash => "Vleugel",
                HardwareMount::Frame => "Kader",
                HardwareMount::MeetingLeaf => "Aansluitende vleugel",
            };
            let detail = leaf
                .set
                .as_ref()
                .and_then(|set| fitting_detail(part.kind, set));
            let description =
                detail.map_or_else(|| mounting.to_owned(), |d| format!("{mounting} · {d}"));
            *counts
                .entry((part.kind.label_nl(), description))
                .or_insert(0) += 1;
        }
        rows.extend(
            counts
                .into_iter()
                .map(|((name, mounting), quantity)| HardwareListItem {
                    piece_id: Some(piece.clone()),
                    cell_index: cr.cell_index,
                    component: name.into(),
                    description: mounting,
                    quantity,
                }),
        );
    }
}

fn fitting_detail(kind: FittingKind, set: &HardwareSet) -> Option<String> {
    match kind {
        FittingKind::Hinge => set
            .hinges
            .as_ref()
            .map(|h| format!("{:?}, {:.0} kg", h.hinge_type, h.load_capacity_kg)),
        FittingKind::Handle => set
            .handle
            .as_ref()
            .map(|h| format!("{:?}, {:.0} mm", h.handle_type, h.height_mm)),
        FittingKind::Lock => set.locking.as_ref().map(|l| format!("{:?}", l.lock_type)),
        FittingKind::LockingPoint => set.locking.as_ref().map(|l| format!("{:?}", l.cam_type)),
        FittingKind::Closer => set
            .closer
            .as_ref()
            .map(|c| format!("{:?}, EN {}", c.closer_type, c.force_class)),
        FittingKind::Vent => set
            .ventilation
            .as_ref()
            .map(|v| format!("{} {:.1} dm³/s", v.vent_type, v.capacity_dm3s)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::compute_2d_geometry;
    use crate::kozijn::{Kozijn, Material, PanelType, ShapeType};
    use crate::production::{compute_production_data, MemberType};
    use crate::profile::{ProfileRef, ProfileSnapshot};

    fn frame(id: &str, material: Material, face: f64) -> Kozijn {
        let mut k = Kozijn::new("Systeem", "S1", 1200.0, 1500.0);
        k.frame.material = material;
        k.frame.frame_width = face;
        k.frame.profile = ProfileRef {
            id: id.into(),
            name: id.into(),
        };
        k.grid.columns[0].size = k.inner_width();
        k.grid.rows[0].size = k.inner_height();
        k
    }

    #[test]
    fn veka_fixed_and_moving_panes_use_their_own_bead_stop_and_sealing_levels() {
        let mut k = frame("veka-softline82", Material::Pvc, 73.0);
        let fixed = compute_production_data(&k);
        assert_eq!(fixed.gasket_list.len(), 2);
        assert!(fixed.hardware_list.is_empty());
        let seal = |rows: &[GasketListItem], kind| {
            rows.iter()
                .find(|g| g.gasket_type == kind)
                .unwrap()
                .length_mm
        };
        // 1054 × 1354 mm daylight; the 25 mm bead on a 28 mm rebate is 3 mm back per edge.
        assert_eq!(seal(&fixed.gasket_list, GasketType::GlazingOuter), 4816.0);
        assert_eq!(seal(&fixed.gasket_list, GasketType::GlazingInner), 4840.0);
        k.cells[0].panel_type = PanelType::TurnTilt;
        k.cells[0].sash_profile = Some(ProfileRef::default_frame());
        k.cells[0].sash_width = Some(69.0);
        k.cells[0].sash_profile_snapshot = Some(ProfileSnapshot {
            sash_overlap: Some(14.0),
            ..Default::default()
        });
        let shape = compute_2d_geometry(&k);
        let fit = shape.cell_rects[0].fit.as_ref().unwrap();
        let s = fit.system_seals.as_ref().unwrap();
        assert_eq!(
            (s.sash_overlap_mm, s.sash_clearance_mm, s.sealing_levels),
            (Some(33.0), Some(12.0), Some(3))
        );
        assert_eq!(fit.day.x, 124.0);
        let p = compute_production_data(&k);
        assert_eq!(p.gasket_list.len(), 5);
        for kind in [
            GasketType::ClosingInner,
            GasketType::ClosingCentre,
            GasketType::ClosingOuter,
        ] {
            assert_eq!(seal(&p.gasket_list, kind), 5080.0);
        }
        assert!(p
            .gasket_list
            .iter()
            .all(|r| r.quantity == 1 && r.reference.as_ref().unwrap().piece_id == "S1-V1"));
        assert!(p
            .cut_list
            .iter()
            .all(|r| !r.profile_name.contains("Meranti")));
        let placed: usize = shape.cell_rects[0]
            .hardware_leaves
            .iter()
            .map(|h| h.components.len())
            .sum();
        assert!(placed > 4);
        assert_eq!(
            p.hardware_list
                .iter()
                .map(|h| h.quantity as usize)
                .sum::<usize>(),
            placed
        );
        assert!(p
            .hardware_list
            .iter()
            .all(|h| h.piece_id.as_deref() == Some("S1-V1")));
        k.frame.profile.id = "veka-softline82-ad".into();
        let ad = compute_production_data(&k);
        assert_eq!(ad.gasket_list.len(), 4);
        assert!(!ad
            .gasket_list
            .iter()
            .any(|g| g.gasket_type == GasketType::ClosingCentre));
    }

    #[test]
    fn paired_doors_and_a_fixed_ring_get_individual_reference_contours() {
        let mut k = frame("veka-softline82", Material::Pvc, 73.0);
        k.layout = Some(crate::layout::deur("dubbel"));
        let g = compute_2d_geometry(&k);
        let left = g.cell_rects[0]
            .fit
            .as_ref()
            .unwrap()
            .sash_outer
            .as_ref()
            .unwrap();
        let right = g.cell_rects[0]
            .second_fit
            .as_ref()
            .unwrap()
            .sash_outer
            .as_ref()
            .unwrap();
        assert_eq!(right.x - left.x - left.width, 8.0);
        let rows = compute_production_data(&k).gasket_list;
        for id in ["S1-V1-L", "S1-V1-R"] {
            let leaf: Vec<_> = rows
                .iter()
                .filter(|r| r.reference.as_ref().unwrap().piece_id == id)
                .collect();
            assert_eq!(leaf.len(), 5);
            assert_eq!(
                leaf.iter()
                    .find(|r| r.gasket_type == GasketType::ClosingCentre)
                    .unwrap()
                    .length_mm,
                2.0 * (left.height + left.width)
            );
        }
        k.layout = None;
        k.frame.shape.shape_type = ShapeType::Round;
        k.frame.outer_height = 1200.0;
        k.grid.rows[0].size = k.inner_height();
        let round = compute_production_data(&k);
        assert_eq!(round.gasket_list.len(), 2);
        for r in &round.gasket_list {
            let radius = if r.gasket_type == GasketType::GlazingInner {
                530.0
            } else {
                527.0
            };
            assert!((r.length_mm - std::f64::consts::TAU * radius).abs() < 1e-7);
        }
    }

    #[test]
    fn aluminum_combined_sightline_and_unspecified_seals_do_not_create_stock() {
        let mut k = frame("reynaers-masterline8", Material::Aluminum, 53.0);
        k.cells[0].panel_type = PanelType::Turn;
        let fit = compute_2d_geometry(&k).cell_rects.remove(0).fit.unwrap();
        assert_eq!(fit.day.x, 97.0);
        assert_eq!(fit.sash_outer.unwrap().x, 47.0);
        let p = compute_production_data(&k);
        let unknown = p
            .gasket_list
            .iter()
            .find(|r| r.gasket_type == GasketType::ClosingUnspecified)
            .unwrap();
        assert_eq!(unknown.quantity, 0);
        assert_eq!(p.bom.iter().filter(|b| b.category == "Rubber").count(), 2);
        assert!(!p.bom.iter().any(|b| b.description.contains("EPDM")));
        k.frame.profile_snapshot = Some(ProfileSnapshot {
            sealing_levels: Some(2),
            ..Default::default()
        });
        k.cells[0].panel_type = PanelType::Sliding;
        assert!(compute_production_data(&k)
            .gasket_list
            .iter()
            .any(|g| g.gasket_type == GasketType::ClosingUnspecified));
    }

    #[test]
    fn system_grid_closes_transoms_against_continuous_mullions_without_weld_overlength() {
        let mut k = frame("veka-softline82", Material::Pvc, 73.0);
        k.add_column(420.0);
        k.add_row(650.0);
        k.grid.rows[1].divider_profile = Some(ProfileRef {
            id: "custom-transom".into(),
            name: "Dwarsprofiel".into(),
        });
        let p = compute_production_data(&k);
        let cross: Vec<_> = p
            .cut_list
            .iter()
            .filter(|c| c.member_type == MemberType::DividerH)
            .collect();
        assert_eq!(cross.len(), 2);
        for (i, c) in cross.iter().enumerate() {
            assert_eq!(c.piece_id, format!("S1-TD1.{}", i + 1));
            assert_eq!(c.profile_name, "Dwarsprofiel");
            assert_eq!(c.net_length_mm, k.grid.columns[i].size);
            assert_eq!(
                (c.gross_length_mm, c.miter_left_deg, c.miter_right_deg),
                (c.net_length_mm + 4.0, 90.0, 90.0)
            );
        }
        let stock = p
            .bom
            .iter()
            .find(|b| b.description.starts_with("Dwarsprofiel"))
            .unwrap();
        assert!(
            (stock.quantity * 1000.0 - cross.iter().map(|r| r.gross_length_mm).sum::<f64>()).abs()
                < 1e-7
        );
    }

    #[test]
    fn legacy_door_fittings_stay_on_the_cell_and_external_beads_swap_the_measurement_lines() {
        let mut k = frame("veka-softline82", Material::Pvc, 73.0);
        k.layout = Some(crate::layout::deur("dubbel"));
        k.cells[0].hardware.push(crate::kozijn::Hardware {
            hardware_type: HardwareType::Handle,
            position: "Actieve deur".into(),
            brand: Some("Eigen merk".into()),
            model: Some("21".into()),
        });
        let p = compute_production_data(&k);
        assert_eq!(p.hardware_list.len(), 1);
        let h = &p.hardware_list[0];
        assert_eq!(
            (h.piece_id.as_deref(), h.component.as_str(), h.quantity),
            (Some("S1-V1"), "Greep", 1)
        );
        assert!(h.description.contains("Eigen merk 21"));
        k.layout = None;
        k.cells[0].glaslat = Some(crate::glaslat::Glaslat {
            position: GlaslatPosition::Buiten,
            material: "PVC".into(),
            width_mm: 19.0,
            height_mm: 25.0,
            mitered: false,
        });
        let p = compute_production_data(&k);
        assert!(p.hardware_list.is_empty());
        let outer = p
            .gasket_list
            .iter()
            .find(|g| g.gasket_type == GasketType::GlazingOuter)
            .unwrap();
        assert!(matches!(
            outer.reference.as_ref().unwrap().line,
            SealMeasureLine::BeadOpening
        ));
        assert_eq!(outer.length_mm, 4840.0);
    }
}
