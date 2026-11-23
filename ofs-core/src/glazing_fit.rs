use crate::geometry::Rect2D;
use crate::glaslat::GlaslatPosition;
use crate::kozijn::{Cell, Frame, Material, OpeningDirection, PanelType};
use crate::layout::Vakvulling;
use crate::profile::{norm_glasinval, norm_glaslat_hoogte, ProfileSnapshot};
use serde::{Deserialize, Serialize};

/// Front-elevation assembly, all measured in the same coordinate system as
/// the opening. The day line is the fixed stop; the rebate is its pocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlazingFit {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_seals: Option<SystemSeals>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contours: Option<crate::curved_contour::FitContours>,
    pub sash_outer: Option<Rect2D>,
    pub day: Rect2D,
    pub rebate: Rect2D,
    pub glass: Rect2D,
    pub glass_thickness: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_glass_thickness: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_glass_thickness: Option<f64>,
    /// Measured build-direction depth of the receiving glazing pocket, if known.
    pub rebate_depth: Option<f64>,
    /// Modelled compressed bedding thickness on each face of the pane.
    pub seal_thickness: f64,
    pub bead_height: f64,
    pub bead_width: f64,
    #[serde(default)]
    pub bead_material: String,
    pub bead_mitered: bool,
    pub bead_position: GlaslatPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemSeals {
    pub profile_id: String,
    pub profile_name: String,
    pub hinged: bool,
    pub sealing_levels: Option<u8>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub closing_seals: Vec<ClosingSeal>,
    pub sash_overlap_mm: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unmet_required_overlap_mm: Option<f64>,
    pub sash_clearance_mm: Option<f64>,
    pub meeting_gap_mm: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SealPosition { Inner, Middle, Outer }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClosingSeal {
    pub position: SealPosition,
    pub article_number: String,
}

pub(crate) fn compatible_profile(id: &str, material: &Material) -> bool {
    if !material.is_profile_system() {
        return true;
    }
    // Older templates stored a generic timber sash/sill even on a system frame.
    if id == "deur-meranti-67x114" || id.starts_with("raamhout-") || id.starts_with("wood-") {
        return false;
    }
    let Some((_, expected, _)) = catalog_entry(id) else {
        return true;
    };
    matches!(
        (material, *expected),
        (Material::Pvc, "pvc") | (Material::Aluminum, "aluminum")
    )
}

impl Rect2D {
    /// Positive values move the boundary inward; negative values expand it.
    pub(crate) fn inset(&self, offset: f64) -> Self {
        let dx = offset.min(self.width.max(0.0) / 2.0);
        let dy = offset.min(self.height.max(0.0) / 2.0);
        Self {
            x: self.x + dx,
            y: self.y + dy,
            width: (self.width - 2.0 * dx).max(0.0),
            height: (self.height - 2.0 * dy).max(0.0),
        }
    }
}

#[derive(Default)]
struct SashSpec {
    door: bool,
    pair: bool,
    hinged: bool,
    outward: bool,
}

pub(crate) struct SashMount {
    pub clearance: f64,
    pub seat_depth: Option<f64>,
    pub outward: bool,
    pub pair: bool,
    pub timber: bool,
}

/// The profile index and its dimensions are compiled into the binary for the
/// desktop and WASM. The lookup never opens files or parses JSON at runtime.
pub(crate) fn catalog_snapshot(id: &str, material: &Material) -> Option<&'static ProfileSnapshot> {
    let material_key = match material {
        Material::Wood(_) => "wood",
        Material::WoodAluminum => "wood_aluminum",
        Material::Aluminum => "aluminum",
        Material::Pvc => "pvc",
    };
    let (_, catalog_material, snapshot) = catalog_entry(id)?;
    (*catalog_material == material_key).then_some(snapshot)
}

fn catalog_system_id(id: &str) -> Option<&'static str> {
    static SYSTEMS: &[(&str, &str)] =
        include!(concat!(env!("OUT_DIR"), "/profile_system_ids.rs"));
    SYSTEMS.binary_search_by(|row| row.0.cmp(id)).ok().map(|i| SYSTEMS[i].1)
}

/// Reference frame used for the documented inward-opening sash assembly.
type InwardPair = (&'static str, &'static str, &'static [(SealPosition, &'static str)]);
fn inward_window_pair(sash_id: &str) -> Option<&'static InwardPair> {
    static PAIRS: &[InwardPair] = include!(concat!(env!("OUT_DIR"), "/inward_window_frames.rs"));
    PAIRS.binary_search_by_key(&sash_id, |row| row.0).ok().map(|i| &PAIRS[i])
}

pub(crate) fn inward_window_frame(sash_id: &str) -> Option<&'static str> {
    inward_window_pair(sash_id).map(|pair| pair.1)
}

/// System frame and sash articles may only be paired when their manufacturer
/// and series are verified to match. Legacy/custom entries without a known
/// series are accepted only when they are the same selected article.
pub fn system_profile_pair_compatible(
    material: &Material,
    frame_id: &str,
    frame_snapshot: Option<&ProfileSnapshot>,
    sash_id: &str,
    sash_snapshot: Option<&ProfileSnapshot>,
) -> bool {
    if let Some(frame) = inward_window_frame(sash_id) {
        return frame_id == frame && catalog_snapshot(sash_id, material).is_some();
    }
    if !material.is_profile_system() || frame_id == sash_id {
        return true;
    }
    let frame_catalog = catalog_snapshot(frame_id, material);
    let sash_catalog = catalog_snapshot(sash_id, material);
    let frame_system = frame_snapshot
        .and_then(|s| s.system_id.as_deref())
        .or_else(|| frame_catalog.and_then(|s| s.system_id.as_deref()))
        .or_else(|| catalog_system_id(frame_id));
    let sash_system = sash_snapshot
        .and_then(|s| s.system_id.as_deref())
        .or_else(|| sash_catalog.and_then(|s| s.system_id.as_deref()))
        .or_else(|| catalog_system_id(sash_id));
    frame_system.is_some() && frame_system == sash_system
}

fn catalog_entry(id: &str) -> Option<&'static (&'static str, &'static str, ProfileSnapshot)> {
    static TABLE: &[(&str, &str, ProfileSnapshot)] =
        include!(concat!(env!("OUT_DIR"), "/profile_snapshots.rs"));
    TABLE
        .binary_search_by(|row| row.0.cmp(id))
        .ok()
        .map(|i| &TABLE[i])
}

/// Older snapshots store one range for the whole series. Apply any more
/// restrictive frame/vent limit for the same article while retaining narrower
/// saved limits. An explicitly saved role-specific range takes precedence.
fn glazing_limits(
    saved: Option<&ProfileSnapshot>,
    catalog: Option<&ProfileSnapshot>,
    vent: bool,
) -> (Option<f64>, Option<f64>) {
    let role = |s: &ProfileSnapshot| if vent {
        (s.min_glass_thickness_vent, s.max_glass_thickness_vent)
    } else {
        (s.min_glass_thickness_fixed, s.max_glass_thickness_fixed)
    };
    let (saved_min, saved_max) = saved.map(role).unwrap_or_default();
    let (article_min, article_max) = catalog.map(role).unwrap_or_default();
    let min = saved.and_then(|s| s.min_glass_thickness)
        .or_else(|| catalog.and_then(|s| s.min_glass_thickness));
    let max = saved.and_then(|s| s.max_glass_thickness)
        .or_else(|| catalog.and_then(|s| s.max_glass_thickness));
    (
        saved_min.or_else(|| min.into_iter().chain(article_min).reduce(f64::max)),
        saved_max.or_else(|| max.into_iter().chain(article_max).reduce(f64::min)),
    )
}

/// Resolve the layout leaf first. A grid cell supplies saved profile and bead
/// settings only when the caller has established that its index matches.
pub(crate) fn glazing_fits(
    opening: &Rect2D,
    frame: &Frame,
    cell: Option<&Cell>,
    vulling: Option<&Vakvulling>,
    article_overlap_limit: Option<f64>,
) -> (Option<GlazingFit>, Option<GlazingFit>, Option<SashMount>) {
    let (sash, bead_side) = match vulling {
        Some(Vakvulling::Glas { glaslat }) => (None, glaslat.as_deref()),
        Some(Vakvulling::Raam {
            open_type,
            opens_outward,
            glaslat,
            ..
        }) => (
            Some(SashSpec {
                hinged: !matches!(
                    open_type.as_str(),
                    "schuif" | "hefschuif" | "taats" | "tuimel"
                ),
                outward: opens_outward.unwrap_or(open_type == "uitzet"),
                ..Default::default()
            }),
            glaslat.as_deref(),
        ),
        Some(Vakvulling::Deur {
            door_kind,
            opens_outward,
            glaslat,
            ..
        }) => (
            Some(SashSpec {
                door: true,
                pair: door_kind == "dubbel",
                hinged: !matches!(
                    door_kind.as_str(),
                    "schuif" | "hefschuif" | "vouw" | "taats"
                ),
                outward: opens_outward.unwrap_or(false),
            }),
            glaslat.as_deref(),
        ),
        Some(_) => return (None, None, None),
        None => {
            let pt = cell.map_or(PanelType::FixedGlass, |c| c.panel_type);
            if pt != PanelType::FixedGlass && !pt.is_operable() {
                return (None, None, None);
            }
            let outward = match cell.and_then(|c| c.opening_direction) {
                Some(OpeningDirection::Outward) => true,
                Some(OpeningDirection::Inward) => false,
                _ => pt == PanelType::TopHung,
            };
            let sash = pt.is_operable().then_some(SashSpec {
                door: pt == PanelType::Door,
                hinged: !matches!(
                    pt,
                    PanelType::Sliding
                        | PanelType::LiftSlide
                        | PanelType::Pivot
                        | PanelType::BottomHung
                ),
                outward,
                ..Default::default()
            });
            (sash, None)
        }
    };

    let system = frame.material.is_profile_system();
    let frame_ok = compatible_profile(&frame.profile.id, &frame.material);
    let frame_catalog = catalog_snapshot(&frame.profile.id, &frame.material);
    // Projects may have explicit system dimensions alongside a historic
    // stock reference. The saved dimensions describe the installed section.
    let frame_spec = frame.profile_snapshot.as_ref().or(frame_catalog);
    let sash_cell = cell.filter(|c| {
        c.sash_profile
            .as_ref()
            .is_none_or(|p| compatible_profile(&p.id, &frame.material))
    });
    let sash_id = sash_cell
        .and_then(|c| c.sash_profile.as_ref())
        .map(|p| p.id.as_str());
    let sash_catalog = sash_id.and_then(|id| catalog_snapshot(id, &frame.material));
    let sash_spec = sash_cell
        .and_then(|c| c.sash_profile_snapshot.as_ref())
        .or_else(|| {
            (system && sash_id.is_none_or(|id| id == frame.profile.id))
                .then_some(frame_spec)
                .flatten()
        })
        .or_else(|| {
            // Older timber templates use the frame stock as a door profile.
            let id = match sash_id {
                Some("deur-meranti-67x114") => "wood-meranti-67x114",
                Some(id) => id,
                None if system || sash.as_ref().is_some_and(|s| s.door) => &frame.profile.id,
                None => "raamhout-69x90",
            };
            catalog_snapshot(id, &frame.material)
        });
    let glazing_spec = if sash.is_some() {
        sash_spec.or(frame_spec)
    } else {
        frame_spec
    };
    let glazing_catalog = if sash.is_some() {
        sash_id.map_or(frame_catalog, |_| sash_catalog)
    } else {
        frame_catalog
    };
    let (min_glass_thickness, max_glass_thickness) =
        glazing_limits(glazing_spec, glazing_catalog, sash.is_some());
    let positive = |v: &f64| v.is_finite() && *v > 0.0;
    let carrier = if sash.is_some() {
        sash_spec
    } else {
        frame_spec
    };
    let rebate_depth = carrier.and_then(|s| s.sponning_diepte).filter(positive);
    let glass_thickness = cell.map_or_else(
        || crate::kozijn::Glazing::default().thickness_mm,
        |c| c.glazing.thickness_mm,
    );
    let glass_cover = glazing_spec.and_then(|s| s.glasinval).filter(positive);
    // An explicit Glaseinstand already excludes edge clearance (e.g. 20
    // inside a 28 mm Falz). Without it, allow 5 mm at each glass edge.
    let nominal_height = glazing_spec
        .and_then(|s| s.sponning_hoogte)
        .filter(positive)
        .unwrap_or_else(|| {
            glass_cover.map_or_else(|| norm_glasinval(&frame.material), |v| v + 5.0)
        });
    let system_width = sash_spec
        .and_then(|p| p.sash_face_width)
        .or_else(|| sash_catalog.and_then(|p| p.sash_face_width))
        .or_else(|| frame_spec.and_then(|p| p.sash_face_width))
        .or_else(|| frame_catalog.and_then(|p| p.sash_face_width))
        .filter(|n| system && positive(n));
    let sash_width = sash_cell
        .and_then(|c| c.sash_width)
        .filter(positive)
        .or(system_width)
        .unwrap_or(if system || sash.as_ref().is_some_and(|s| s.door) {
            frame.frame_width
        } else {
            69.0
        });
    let system_overlap = sash_spec
        .and_then(|s| s.sash_overlap)
        .or_else(|| sash_catalog.and_then(|s| s.sash_overlap))
        .or_else(|| frame_spec.and_then(|s| s.sash_overlap))
        .or_else(|| frame_catalog.and_then(|s| s.sash_overlap))
        .filter(|v| v.is_finite() && *v >= 0.0);
    let overlap = if let Some(s) = sash.as_ref().filter(|s| s.hinged) {
        let wood = matches!(frame.material, Material::Wood(_) | Material::WoodAluminum);
        // 69 mm stock with a 55 mm visible face gives a 14 mm interior
        // overlap. Pocket depth in the build direction cannot set this size.
        let fallback = match (wood, s.outward) {
            (true, true) => 17.0,
            (true, false) => 14.0,
            _ => 0.0,
        };
        system_overlap
            .unwrap_or(fallback)
            .min(article_overlap_limit.unwrap_or(frame.frame_width / 2.0).max(0.0))
            .min(sash_width)
    } else {
        0.0
    };
    let bead = cell
        .and_then(|c| c.glaslat.as_ref())
        .filter(|b| b.fits_carrier(&frame.material));
    let bead_position = match bead_side {
        Some("binnen") => GlaslatPosition::Binnen,
        Some("buiten") => GlaslatPosition::Buiten,
        _ => bead
            .map(|b| b.position)
            .or_else(|| glazing_spec.and_then(|s| s.bead_position))
            .unwrap_or(GlaslatPosition::Binnen),
    };
    let bead_height = bead
        .map(|b| b.height_mm)
        .filter(positive)
        .or_else(|| glazing_spec.and_then(|s| s.glaslat_hoogte).filter(positive))
        .unwrap_or_else(|| norm_glaslat_hoogte(&frame.material, nominal_height));
    let bead_width = bead
        .map(|b| b.width_mm)
        .filter(positive)
        .or_else(|| {
            glazing_spec
                .and_then(|s| s.glaslat_breedte)
                .filter(positive)
        })
        .or_else(|| {
            glazing_spec
                .and_then(|s| s.bead_width_base)
                .map(|base| base - glass_thickness)
                .filter(positive)
        })
        .unwrap_or(15.0);
    let known_clearance = frame_spec
        .and_then(|p| p.sash_clearance)
        .or_else(|| frame_catalog.and_then(|p| p.sash_clearance))
        .filter(positive);
    let known_gap = frame_spec
        .and_then(|p| p.meeting_gap)
        .or_else(|| frame_catalog.and_then(|p| p.meeting_gap))
        .filter(positive);
    let hinged = sash.as_ref().is_some_and(|s| s.hinged);
    let closing_seals: Vec<_> = sash_id.and_then(inward_window_pair)
        .filter(|pair| pair.1 == frame.profile.id
            && sash.as_ref().is_some_and(|s| s.hinged && !s.outward && !s.door))
        .into_iter().flat_map(|pair| pair.2.iter())
        .map(|&(position, article)| ClosingSeal { position, article_number: article.to_owned() })
        .collect();
    let system_seals = system.then(|| SystemSeals {
        profile_id: if frame_ok {
            frame.profile.id.clone()
        } else {
            String::new()
        },
        profile_name: if frame_ok {
            frame.profile.name.clone()
        } else {
            String::new()
        },
        hinged,
        sealing_levels: if !closing_seals.is_empty() { Some(closing_seals.len() as u8) } else { frame_spec
            .and_then(|p| p.sealing_levels)
            .or_else(|| frame_catalog.and_then(|p| p.sealing_levels))
            .filter(|v| {
                hinged && (1..=3).contains(v) && sash_id.is_none_or(|id| {
                    id == frame.profile.id || inward_window_frame(id) == Some(frame.profile.id.as_str())
                })
            }) },
        closing_seals,
        sash_overlap_mm: system_overlap.filter(|_| hinged).map(|_| overlap),
        unmet_required_overlap_mm: system_overlap.filter(|required| {
            hinged && article_overlap_limit.is_some() && *required > overlap + 1e-6
        }),
        sash_clearance_mm: known_clearance.filter(|_| hinged),
        meeting_gap_mm: known_gap.filter(|_| sash.as_ref().is_some_and(|s| s.pair)),
    });
    let make_fit = |host: Rect2D| {
        let day = if sash.is_some() {
            host.inset(sash_width)
        } else {
            host.clone()
        };
        let available = if sash.is_some() {
            (day.x - host.x).min(day.y - host.y)
        } else {
            frame.frame_width.max(0.0)
        };
        let height = nominal_height.min(available);
        let cover = glass_cover.unwrap_or(height - 5.0).min(height);
        GlazingFit {
            system_seals: system_seals.clone(),
            contours: None,
            sash_outer: sash.is_some().then_some(host),
            rebate: day.inset(-height),
            glass: day.inset(-cover),
            day,
            glass_thickness,
            min_glass_thickness,
            max_glass_thickness,
            rebate_depth,
            seal_thickness: 2.0,
            bead_height,
            bead_width,
            bead_material: bead
                .map(|b| b.material.trim())
                .filter(|name| !name.is_empty())
                .unwrap_or(frame.material.label_nl())
                .to_owned(),
            bead_mitered: bead.map_or(true, |b| b.mitered),
            bead_position,
        }
    };
    let outer = opening.inset(-overlap);
    let mount = sash.as_ref().map(|s| {
        // Older snapshots already contain the glazing data but not the
        // operating-sash shoulder dimensions. Recover those from its profile.
        let catalog = catalog_snapshot(&frame.profile.id, &frame.material);
        let clearance = known_clearance.unwrap_or(4.0);
        SashMount {
            clearance: clearance.min((frame.frame_width - overlap - 1.).max(0.)),
            seat_depth: frame_spec
                .and_then(|p| p.sash_seat_depth)
                .filter(positive)
                .or_else(|| catalog.and_then(|p| p.sash_seat_depth)),
            outward: s.outward,
            pair: s.pair,
            timber: matches!(frame.material, Material::Wood(_) | Material::WoodAluminum),
        }
    });
    if sash.as_ref().map_or(false, |s| s.pair) {
        // Two independent leaves meet at the centre; the frame overlap
        // applies only at the jambs. A 6 mm meeting gap is the model default.
        let gap = known_gap.unwrap_or(6.0).min(outer.width);
        let half = (outer.width - gap) / 2.0;
        let left = Rect2D {
            width: half,
            ..outer
        };
        let right = Rect2D {
            x: left.x + half + gap,
            ..left
        };
        (Some(make_fit(left)), Some(make_fit(right)), mount)
    } else {
        (Some(make_fit(outer)), None, mount)
    }
}

#[cfg(test)]
mod glass_range_tests {
    use super::*;
    use crate::{kozijn::Kozijn, production::compute_production_data};

    #[test]
    fn fixed_and_opening_glass_follow_their_own_catalog_limits() {
        let mut k = Kozijn::new("Glasbereik", "AL1", 1200., 1500.);
        k.frame.material = Material::Aluminum;
        k.frame.profile.id = "schuco-aws70hi".into();
        // Projects from before the split stored the common 11–62 mm range.
        k.frame.profile_snapshot = serde_json::from_str(r#"{"minGlassThickness":11,"maxGlassThickness":62}"#).unwrap();
        for (thickness, fixed, vent) in [(11., true, false), (21., true, true),
            (52., true, true), (53., false, true), (62., false, true), (63., false, false)] {
            for (kind, expected) in [(PanelType::FixedGlass, fixed), (PanelType::TurnTilt, vent)] {
                k.cells[0].change_panel_type(kind, &k.frame);
                k.cells[0].glazing.thickness_mm = thickness;
                let row = compute_production_data(&k).glass_list.remove(0);
                assert_eq!(row.production_ready, expected, "{kind:?}, {thickness} mm: {:?}", row.production_issue);
            }
        }
    }

    #[test]
    fn saved_narrower_article_and_explicit_role_ranges_survive_reopening() {
        let catalog = catalog_snapshot("reynaers-masterline8", &Material::Aluminum);
        let saved: ProfileSnapshot = serde_json::from_str(r#"{"maxGlassThickness":48}"#).unwrap();
        assert_eq!(glazing_limits(Some(&saved), catalog, false).1, Some(48.));
        let mut own = saved;
        own.max_glass_thickness_fixed = Some(40.);
        own.max_glass_thickness_vent = Some(46.);
        let reopened = serde_json::from_value(serde_json::to_value(own).unwrap()).unwrap();
        assert_eq!(glazing_limits(Some(&reopened), None, false).1, Some(40.));
        assert_eq!(glazing_limits(Some(&reopened), None, true).1, Some(46.));
        assert_eq!(glazing_limits(None, catalog, false).1, Some(62.));
        assert_eq!(glazing_limits(None, catalog, true).1, Some(72.));
    }
}
