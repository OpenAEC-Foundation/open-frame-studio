use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StockLength {
    pub profile_id: String,
    pub material: String,
    pub available_length_mm: f64,
    pub cost_per_meter: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CutPlan {
    pub allocations: Vec<StockAllocation>,
    /// Positions made from curved stock, excluded from linear bar nesting.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub curved_piece_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unresolved_piece_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub oversize_piece_ids: Vec<String>,
    pub total_waste_mm: f64,
    pub waste_percentage: f64,
    pub total_bars_used: usize,
    pub total_stock_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StockAllocation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<String>,
    #[serde(default)]
    pub profile_name: String,
    #[serde(default)]
    pub material: String,
    /// Supplied surface colours, ordered by face for prefinished stock.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub colors: Option<StockColors>,
    pub stock_length_mm: f64,
    pub cuts: Vec<AllocatedCut>,
    pub remaining_mm: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StockColors {
    pub inside: String,
    pub outside: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocatedCut {
    pub piece_id: String,
    pub kozijn_mark: String,
    pub length_mm: f64,
}

/// Nest the actual straight production blanks. Curved stock is ordered by its
/// developed length and cannot be taken from one of these straight bars.
pub fn optimize_frame_cuts(kozijnen: &[crate::kozijn::Kozijn], stock_length_mm: f64) -> CutPlan {
    use std::collections::BTreeMap;
    let mut groups = BTreeMap::<_, Vec<(String, String, f64)>>::new();
    let mut curved_piece_ids = Vec::new();
    let mut unresolved_piece_ids = Vec::new();
    for kozijn in kozijnen {
        let prod = crate::production::compute_production_data(kozijn);
        for cut in &prod.cut_list {
            if prod.blank_length_mm(cut).is_none() {
                unresolved_piece_ids.push(cut.piece_id.clone());
            } else if cut.arc.is_some() {
                curved_piece_ids.push(cut.piece_id.clone());
            } else {
                let (profile_id, profile_name, section) = cut_profile(kozijn, cut);
                // A two-colour extrusion cannot be flipped to exchange its
                // inside and outside: its rebate and sealing faces are fixed.
                let colors = cut.is_profile_system().then(|| StockColors {
                    inside: kozijn.frame.color_inside.trim().to_owned(),
                    outside: kozijn.frame.color_outside.trim().to_owned(),
                });
                groups
                    .entry((cut.material.clone(), profile_id, profile_name, colors, section))
                    .or_default()
                    .push((
                        cut.piece_id.clone(),
                        prod.kozijn_mark.clone(),
                        cut.gross_length_mm,
                    ));
            }
        }
    }
    let mut plan = empty_plan();
    for ((material, profile_id, profile_name, colors, _section), pieces) in groups {
        // Production already includes the saw kerf once per blank.
        let mut group_plan = optimize_cut_list(pieces, stock_length_mm, 0.0);
        for allocation in &mut group_plan.allocations {
            allocation.profile_id = profile_id.clone();
            allocation.profile_name = profile_name.clone();
            allocation.material = material.clone();
            allocation.colors = colors.clone();
        }
        plan.allocations.extend(group_plan.allocations);
        plan.oversize_piece_ids
            .extend(group_plan.oversize_piece_ids);
    }
    plan.curved_piece_ids = curved_piece_ids;
    plan.unresolved_piece_ids = unresolved_piece_ids;
    plan.total_bars_used = plan.allocations.len();
    plan.total_waste_mm = plan.allocations.iter().map(|a| a.remaining_mm).sum();
    let total_stock = plan.total_bars_used as f64 * stock_length_mm.max(0.0);
    plan.waste_percentage = if total_stock > 0.0 {
        (plan.total_waste_mm / total_stock * 1000.0).round() / 10.0
    } else {
        0.0
    };
    plan
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum StockRole {
    Frame,
    Divider,
    Sash,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SystemSection {
    role: StockRole,
    // Exact saved millimetres are part of the section, including for legacy projects.
    width_bits: Option<u64>,
    depth_bits: Option<u64>,
    custom: bool,
}

fn cut_profile(
    kozijn: &crate::kozijn::Kozijn,
    cut: &crate::production::CutListItem,
) -> (Option<String>, String, Option<SystemSection>) {
    use crate::kozijn::Material;
    use crate::production::MemberType as M;
    let frame = &kozijn.frame;
    let mut role = StockRole::Frame;
    let mut divider_article = None;
    let mut width = Some(frame.frame_width);
    let mut depth = Some(frame.frame_depth);
    let profile = match cut.member_type {
        M::PerimeterStop => return (None, cut.profile_name.clone(), None),
        M::FrameTop => frame.top_profile.as_ref(),
        M::FrameBottom => frame
            .bottom_profile
            .as_ref()
            .or(frame.sill_profile.as_ref()),
        M::FrameLeft => frame.left_profile.as_ref(),
        M::FrameRight => frame.right_profile.as_ref(),
        M::FrameArc => match cut.profile_side {
            Some(crate::curved_timber::TimberProfileSide::Top) => frame.top_profile.as_ref(),
            Some(crate::curved_timber::TimberProfileSide::Bottom) => frame
                .bottom_profile
                .as_ref()
                .or(frame.sill_profile.as_ref()),
            _ => None,
        },
        M::DividerV | M::DividerH => {
            role = StockRole::Divider;
            let marker = if cut.member_type == M::DividerV {
                "-TS"
            } else {
                "-TD"
            };
            let divider = if kozijn.layout.is_some() {
                width = Some(kozijn.layout_divider_width());
                None
            } else {
                let index = cut.piece_id.rsplit_once(marker)
                    .and_then(|(_, suffix)| suffix.split('.').next()?.parse::<usize>().ok());
                let vertical = cut.member_type == M::DividerV;
                let divisions = if vertical { &kozijn.grid.columns } else { &kozijn.grid.rows };
                if let Some(index) = index.and_then(|i| i.checked_sub(1)) {
                    width = Some(kozijn.divider_face_width(vertical, index));
                }
                index.and_then(|i| divisions.get(i))
            };
            depth = divider.and_then(|d| d.divider_depth).or(depth);
            divider_article = width.and_then(|w| kozijn.system_divider_article(divider, w));
            divider.and_then(|d| d.divider_profile.as_ref())
                .filter(|p| crate::glazing_fit::compatible_profile(&p.id, &frame.material))
        }
        M::SashArc | M::SashTop | M::SashBottom | M::SashLeft | M::SashRight => {
            role = StockRole::Sash;
            let cell = cut.piece_id.rsplit_once("-V").and_then(|(_, suffix)| {
                let digits: String = suffix.chars().take_while(|c| c.is_ascii_digit()).collect();
                let cell = digits.parse::<usize>().ok()?.checked_sub(1)?;
                kozijn.cells.get(cell)
            });
            width = cell.and_then(|c| c.sash_width);
            depth = cell.and_then(|c| c.sash_depth);
            cell.and_then(|c| c.sash_profile.as_ref())
        }
    };
    let id = divider_article.map_or_else(|| profile.unwrap_or(&frame.profile).id.as_str(), |a| a.id);
    let mut name = if role == StockRole::Divider && cut.is_profile_system() {
        cut.profile_name.as_str()
    } else {
        profile.map_or(cut.profile_name.as_str(), |p| p.name.as_str())
    };
    let section = cut.is_profile_system().then(|| {
        let custom = frame.member_profiles.contains_key(id);
        // A series-level ID can stand for a frame, a T-profile or a vent.
        // Replace project/preset aliases only for a documented article in its
        // actual role, material and nominal cross-section.
        const ARTICLES: &[(&str, Material, StockRole, f64, f64, &str)] =
            include!(concat!(env!("OUT_DIR"), "/stock_articles.rs"));
        if !custom {
            if let Some(article) = ARTICLES.iter().find(|(key, material, article_role, w, d, _)| {
                *key == id && material.label_nl() == cut.material && *article_role == role
                    && Some(*w) == width && Some(*d) == depth
            }) {
                name = article.5;
            }
        }
        SystemSection {
            role,
            width_bits: width.map(f64::to_bits),
            depth_bits: depth.map(f64::to_bits),
            custom,
        }
    });
    (Some(id.to_owned()), name.to_owned(), section)
}

fn empty_plan() -> CutPlan {
    CutPlan {
        allocations: Vec::new(),
        curved_piece_ids: Vec::new(),
        unresolved_piece_ids: Vec::new(),
        oversize_piece_ids: Vec::new(),
        total_waste_mm: 0.0,
        waste_percentage: 0.0,
        total_bars_used: 0,
        total_stock_cost: 0.0,
    }
}

/// First Fit Decreasing algorithm for 1D cutting stock problem
pub fn optimize_cut_list(
    pieces: Vec<(String, String, f64)>, // (piece_id, kozijn_mark, gross_length_mm)
    stock_length_mm: f64,               // standard bar length (e.g., 5800mm for wood)
    saw_kerf_mm: f64,                   // blade width (typically 4mm)
) -> CutPlan {
    // Sort pieces descending by length; no part may overrun its own bar.
    let mut sorted: Vec<_> = pieces.into_iter().collect();
    sorted.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    let mut allocations: Vec<StockAllocation> = Vec::new();
    let mut oversize_piece_ids = Vec::new();

    for (piece_id, mark, length) in sorted {
        if !stock_length_mm.is_finite()
            || stock_length_mm <= 0.0
            || !length.is_finite()
            || length <= 0.0
            || !saw_kerf_mm.is_finite()
            || saw_kerf_mm < 0.0
            || length + saw_kerf_mm > stock_length_mm
        {
            oversize_piece_ids.push(piece_id);
            continue;
        }
        let cut_length = length + saw_kerf_mm;

        // Try to fit in existing bar
        let mut placed = false;
        for alloc in &mut allocations {
            if alloc.remaining_mm >= cut_length {
                alloc.remaining_mm -= cut_length;
                alloc.cuts.push(AllocatedCut {
                    piece_id: piece_id.clone(),
                    kozijn_mark: mark.clone(),
                    length_mm: length,
                });
                placed = true;
                break;
            }
        }

        // Open new bar
        if !placed {
            allocations.push(StockAllocation {
                profile_id: None,
                profile_name: String::new(),
                material: String::new(),
                colors: None,
                stock_length_mm,
                cuts: vec![AllocatedCut {
                    piece_id,
                    kozijn_mark: mark,
                    length_mm: length,
                }],
                remaining_mm: stock_length_mm - cut_length,
            });
        }
    }

    let total_waste: f64 = allocations.iter().map(|a| a.remaining_mm).sum();
    let total_stock: f64 = allocations.len() as f64 * stock_length_mm;
    let waste_pct = if total_stock > 0.0 {
        total_waste / total_stock * 100.0
    } else {
        0.0
    };

    CutPlan {
        curved_piece_ids: Vec::new(),
        unresolved_piece_ids: Vec::new(),
        oversize_piece_ids,
        total_bars_used: allocations.len(),
        total_waste_mm: total_waste,
        waste_percentage: (waste_pct * 10.0).round() / 10.0,
        total_stock_cost: 0.0, // TODO: calculate from stock prices
        allocations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arched_free_layout_keeps_the_mixed_head_out_of_the_verified_bar_plan() {
        use crate::kozijn::{Kozijn, ShapeType};
        use crate::layout::{glas, split_col, split_row, VakChild};
        use crate::production::{compute_production_data, MemberType};

        let sj = crate::template::get_sjabloon("kunststof-82-veka");
        let radius = 750.0 - sj.frame_width;
        let half_day = 600.0 - sj.frame_width;
        let spring = 750.0 - (radius * radius - half_day * half_day).sqrt();
        let make_frame = |mark: &str, face: f64| {
            let mut k = Kozijn::new_with_sjabloon("Linker bovenlicht", mark, 1200.0, 1500.0, &sj);
            k.frame.shape.shape_type = ShapeType::Arched;
            k.frame.shape.arch_height = Some(300.0);
            let mut snapshot = crate::glazing_fit::catalog_snapshot(&sj.stijl_profile.id, &sj.material).unwrap().clone();
            snapshot.divider_face_width = Some(face);
            k.frame.profile_snapshot = Some(snapshot);
            k.layout = Some(split_row(vec![
                VakChild { size: 1.0, node: split_col(vec![
                    VakChild { size: spring - sj.frame_width - face / 2.0, node: glas() },
                    VakChild { size: 1500.0 - sj.frame_width - spring - face / 2.0, node: glas() },
                ]) },
                VakChild { size: 1.0, node: glas() },
            ]));
            k
        };
        let frames = [make_frame("A", 94.0), make_frame("B", 80.0)];
        let plan = optimize_frame_cuts(&frames, 12000.0);
        for (k, face) in frames.iter().zip([94.0, 80.0]) {
            let p = compute_production_data(k);
            let rail = p.cut_list.iter().find(|c| c.member_type == MemberType::DividerH).unwrap();
            assert!((rail.net_length_mm - (half_day - face / 2.0)).abs() < 1e-6);
            let start = rail.divider_start.as_ref().unwrap();
            assert_eq!(start.radius_mm, Some(radius));
            assert_eq!(start.segments.len(), 2); // The head meets both arc and jamb.
            assert_eq!(rail.end_cope_radius_mm, None);
            assert!(plan.unresolved_piece_ids.contains(&rail.piece_id));
            assert!(!plan.allocations.iter().flat_map(|b| &b.cuts).any(|c| c.piece_id == rail.piece_id));
            if face == 94.0 {
                assert_eq!(rail.profile_name, sj.tussenstijl_profile.name);
            } else {
                assert!(rail.profile_name.contains("80 × 82 mm"));
                assert!(!rail.profile_name.contains("102.310"));
            }
        }
    }

    #[test]
    fn catalog_frame_aliases_share_stock_after_a_material_revision() {
        use crate::{grid::template_single_turn_tilt_sj, profile::ProfileRef, template::get_sjabloon};

        let pvc = get_sjabloon("kunststof-82-veka");
        let mut k = template_single_turn_tilt_sj(1200., 1500., &pvc);
        k.mark = "K02".into();
        let stock = optimize_frame_cuts(std::slice::from_ref(&k), 5800.);
        assert!(stock.allocations.iter().any(|b| b.profile_name.contains("101.290") && b.cuts.len() == 4));
        let standard = ProfileRef {
            id: "reynaers-cs77-standard".into(), name: "Reynaers CS 77 Standard (Functional)".into(),
        };
        k.apply_frame_profile(standard.clone(), Some(51.), Some(68.), None, Some("\"aluminum\"")).unwrap();
        let stock = optimize_frame_cuts(std::slice::from_ref(&k), 5800.);
        assert_eq!(stock.total_bars_used, 2);
        let frame = stock.allocations.iter().find(|b| b.profile_name.contains("008.3136.XX")).unwrap();
        assert_eq!(frame.cuts.iter().map(|c| c.length_mm).collect::<Vec<_>>(), [1504.,1504.,1204.,1204.]);
        assert_eq!(frame.remaining_mm, 384.);
        let sash = stock.allocations.iter().find(|b| b.profile_name.contains("008.3102.XX")).unwrap();
        assert_eq!(sash.cuts.iter().map(|c| c.length_mm).collect::<Vec<_>>(), [1462.,1462.,1162.,1162.]);

        // Older files can reuse the frame reference for an unidentified vent.
        // Even an identical name and envelope do not make the sections equivalent.
        k.cells[0].sash_profile = Some(standard);
        k.cells[0].sash_width = Some(51.);
        k.cells[0].sash_depth = Some(68.);
        let old = optimize_frame_cuts(std::slice::from_ref(&k), 12000.);
        assert_eq!(old.total_bars_used, 2);
        assert!(old.allocations.iter().all(|b| b.cuts.len() == 4));
        let mut deep = k.clone();
        deep.frame.frame_depth = 77.;
        let legacy = optimize_frame_cuts(&[k, deep], 24000.);
        assert_eq!(legacy.allocations.iter().find(|b| b.profile_name.contains("008.3136.XX")).unwrap().cuts.len(), 4);
    }

    #[test]
    fn separate_profile_articles_never_share_a_stock_bar() {
        let mut veka = crate::kozijn::Kozijn::new("VEKA", "V01", 900.0, 1400.0);
        veka.frame.material = crate::kozijn::Material::Pvc;
        veka.frame.profile = crate::profile::ProfileRef {
            id: "veka-softline82".into(),
            name: "VEKA Softline 82".into(),
        };
        veka.frame.sill_profile = None;
        veka.frame.bottom_profile = None;
        let mut kommerling = crate::kozijn::Kozijn::new("Kömmerling", "K01", 900.0, 1400.0);
        kommerling.frame.material = crate::kozijn::Material::Pvc;
        kommerling.frame.profile = crate::profile::ProfileRef {
            id: "kommerling-76ad".into(),
            name: "Kömmerling 76 AD".into(),
        };
        kommerling.frame.sill_profile = None;
        kommerling.frame.bottom_profile = None;

        let plan = optimize_frame_cuts(&[veka, kommerling], 5800.0);
        let articles: Vec<_> = plan
            .allocations
            .iter()
            .filter_map(|bar| bar.profile_id.as_deref())
            .collect();
        assert!(articles.contains(&"veka-softline82"));
        assert!(articles.contains(&"kommerling-76ad"));
        for bar in &plan.allocations {
            let expected_mark = match bar.profile_id.as_deref() {
                Some("veka-softline82") => "V01",
                Some("kommerling-76ad") => "K01",
                _ => panic!("unexpected article {:?}", bar.profile_id),
            };
            assert!(bar.cuts.iter().all(|cut| cut.kozijn_mark == expected_mark));
        }
    }

    #[test]
    fn prefabricated_pvc_and_aluminium_stock_keeps_inside_and_outside_colours_in_order() {
        use crate::kozijn::{Kozijn, Material};
        for (material, profile) in [
            (Material::Pvc, "veka-softline82"),
            (Material::Aluminum, "reynaers-masterline8"),
        ] {
            let mut first = Kozijn::new("Gevel", "A", 500.0, 600.0);
            first.frame.material = material;
            first.frame.profile.id = profile.into();
            first.frame.profile.name = profile.into();
            first.frame.sill_profile = None;
            first.frame.color_inside = "RAL9010".into();
            first.frame.color_outside = "RAL7016".into();
            let mut matching = first.clone();
            matching.mark = "B".into();
            let mut reversed = first.clone();
            reversed.mark = "C".into();
            std::mem::swap(&mut reversed.frame.color_inside, &mut reversed.frame.color_outside);
            // All three frames would fit on one bar by length alone.
            let plan = optimize_frame_cuts(&[first, matching, reversed], 12000.0);
            assert_eq!(plan.total_bars_used, 2);
            for allocation in plan.allocations {
                let colors = allocation.colors.unwrap();
                let mut marks: Vec<_> = allocation.cuts.iter()
                    .map(|c| c.kozijn_mark.as_str())
                    .collect();
                marks.sort_unstable();
                marks.dedup();
                match (colors.inside.as_str(), colors.outside.as_str()) {
                    ("RAL9010", "RAL7016") => assert_eq!(marks, ["A", "B"]),
                    ("RAL7016", "RAL9010") => assert_eq!(marks, ["C"]),
                    _ => panic!("onverwachte kleuren {:?}", colors),
                }
            }
        }
    }

    #[test]
    fn oversize_piece_is_reported_without_a_negative_remainder() {
        let plan = optimize_cut_list(vec![("K01-LANG".into(), "K01".into(), 5801.0)], 5800.0, 0.0);
        assert!(plan.allocations.is_empty());
        assert_eq!(plan.oversize_piece_ids, ["K01-LANG"]);
        assert_eq!(plan.total_bars_used, 0);
    }
}
