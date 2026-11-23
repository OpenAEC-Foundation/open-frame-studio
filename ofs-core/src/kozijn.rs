use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::glaslat::Glaslat;
use crate::hardware::HardwareSet;
use crate::panel_filling::PanelFilling;
use crate::profile::ProfileRef;

/// Complete kozijn (window frame) definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Kozijn {
    pub id: Uuid,
    pub name: String,
    pub mark: String,
    pub frame: Frame,
    pub grid: Grid,
    pub cells: Vec<Cell>,
    #[serde(default)]
    pub placement: Placement,
    /// Series/batch reference (e.g., "Begane grond", "Verdieping 1")
    #[serde(default)]
    pub series_id: Option<String>,
    /// Free-form extensions beyond the grid (e.g., extended dorpel, extra stijl)
    #[serde(default)]
    pub extensions: Vec<FrameExtension>,
    /// Free subdivision tree (additive). When present, supersedes the rectangular
    /// grid for layout/geometry; old projects (grid only) keep working.
    #[serde(default)]
    pub layout: Option<crate::layout::VakNode>,
    /// Persisted identity of rectangular-grid parts for IFC revisions.
    /// Empty on older projects; their former ordinal IDs remain the fallback.
    #[serde(default, skip_serializing_if = "PartIds::is_empty")]
    part_ids: PartIds,
    /// Basic reconstruction from an IFC exchange model lacks verified joints,
    /// glazing build-up and profile snapshots; machine output must stay blocked.
    #[serde(default, skip_serializing_if = "is_false")]
    pub recovered_from_ifc: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PartIds {
    /// One slot per column/row. Slot zero has no preceding divider.
    columns: Vec<Uuid>,
    rows: Vec<Uuid>,
    cells: Vec<Uuid>,
}

fn is_false(value: &bool) -> bool {
    !*value
}

impl PartIds {
    fn new(columns: usize, rows: usize) -> Self {
        Self {
            columns: (0..columns).map(|_| Uuid::new_v4()).collect(),
            rows: (0..rows).map(|_| Uuid::new_v4()).collect(),
            cells: (0..columns * rows).map(|_| Uuid::new_v4()).collect(),
        }
    }

    fn is_empty(&self) -> bool {
        self.columns.is_empty() && self.rows.is_empty() && self.cells.is_empty()
    }
}

/// A free-form member extension beyond the rectangular grid
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameExtension {
    /// Extension type
    pub extension_type: ExtensionType,
    /// Profile used for this extension
    pub profile: ProfileRef,
    /// Start point (absolute coordinates in mm)
    pub start_x: f64,
    pub start_y: f64,
    /// End point (absolute coordinates in mm)
    pub end_x: f64,
    pub end_y: f64,
    /// Width of the member (face width in mm)
    #[serde(default = "default_ext_width")]
    pub member_width: f64,
    /// Which grid member(s) this connects to
    #[serde(default)]
    pub connects_to: Vec<String>,
}

fn default_ext_width() -> f64 {
    67.0
}

/// Extension member type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionType {
    /// Extended dorpel (wider than frame)
    Dorpel,
    /// Extra stijl (outside the grid)
    Stijl,
    /// Balk (structural beam)
    Balk,
    /// Neut (corner block at sill)
    Neut,
    /// Spouwlat (cavity batten)
    Spouwlat,
}

impl Kozijn {
    fn legacy_part_id(&self, ordinal: u128) -> Uuid {
        Uuid::from_u128(
            (self.id.as_u128() ^ 0x4f46535f50415254535f474c4f42414c_u128).wrapping_add(ordinal),
        )
    }

    pub(crate) fn frame_part_id(&self, index: usize) -> Uuid {
        self.legacy_part_id(index as u128 + 1)
    }

    pub(crate) fn vertical_part_id(&self, index: usize) -> Uuid {
        self.part_ids
            .columns
            .get(index + 1)
            .copied()
            .unwrap_or_else(|| self.legacy_part_id(index as u128 + 5))
    }

    pub(crate) fn horizontal_part_id(&self, index: usize) -> Uuid {
        self.part_ids
            .rows
            .get(index + 1)
            .copied()
            .unwrap_or_else(|| {
                self.legacy_part_id(self.grid.columns.len() as u128 + index as u128 + 4)
            })
    }

    pub(crate) fn glass_part_id(&self, cell_index: usize) -> Uuid {
        self.part_ids
            .cells
            .get(cell_index)
            .copied()
            .unwrap_or_else(|| {
                self.legacy_part_id(
                    self.grid.columns.len() as u128
                        + self.grid.rows.len() as u128
                        + cell_index as u128
                        + 3,
                )
            })
    }

    pub(crate) fn restore_grid_part_ids(
        &mut self,
        vertical: Vec<Uuid>,
        horizontal: Vec<Uuid>,
        glass: Vec<Uuid>,
    ) -> Result<(), String> {
        if vertical.len() + 1 != self.grid.columns.len()
            || horizontal.len() + 1 != self.grid.rows.len()
            || glass.len() != self.cells.len()
        {
            return Err("IFC-herstel: aantal onderdeel-ID's past niet bij de vakindeling".into());
        }
        let mut columns = vec![Uuid::new_v4()];
        columns.extend(vertical);
        let mut rows = vec![Uuid::new_v4()];
        rows.extend(horizontal);
        self.part_ids = PartIds {
            columns,
            rows,
            cells: glass,
        };
        Ok(())
    }

    /// Capture the IDs used by legacy files before the grid order changes.
    fn materialize_part_ids(&mut self) {
        let cols = self.grid.columns.len();
        let rows = self.grid.rows.len();
        while self.part_ids.columns.len() < cols {
            let index = self.part_ids.columns.len();
            self.part_ids
                .columns
                .push(self.legacy_part_id(index as u128 + 4));
        }
        while self.part_ids.rows.len() < rows {
            let index = self.part_ids.rows.len();
            self.part_ids
                .rows
                .push(self.legacy_part_id(cols as u128 + index as u128 + 3));
        }
        while self.part_ids.cells.len() < cols * rows {
            let index = self.part_ids.cells.len();
            self.part_ids
                .cells
                .push(self.legacy_part_id(cols as u128 + rows as u128 + index as u128 + 3));
        }
        self.part_ids.columns.truncate(cols);
        self.part_ids.rows.truncate(rows);
        self.part_ids.cells.truncate(cols * rows);
        self.cells.resize_with(cols * rows, Cell::default);
    }

    pub fn new(name: &str, mark: &str, width: f64, height: f64) -> Self {
        let id = Uuid::new_v4();
        Self {
            id,
            name: name.to_string(),
            mark: mark.to_string(),
            frame: Frame {
                member_profiles: Default::default(),
                outer_width: width,
                outer_height: height,
                material: Material::Wood(WoodType::Meranti),
                profile: ProfileRef::default_frame(),
                sill_profile: Some(ProfileRef::default_sill()),
                color_inside: "RAL9010".into(),
                color_outside: "RAL9010".into(),
                frame_width: 67.0,
                frame_depth: 114.0,
                profile_snapshot: None,
                sill: None,
                perimeter_stop: None,
                top_profile: None,
                bottom_profile: None,
                left_profile: None,
                right_profile: None,
                top_material: None,
                bottom_material: None,
                left_material: None,
                right_material: None,
                shape: FrameShape::default(),
                corner_joints: vec![
                    crate::joint::Joint::default(),
                    crate::joint::Joint::default(),
                    crate::joint::Joint::default(),
                    crate::joint::Joint::default(),
                ],
                joints_configured: false,
                edges: vec![],
            },
            grid: Grid {
                rows_from_day: false,
                columns: vec![GridDivision {
                    size: width - 2.0 * 67.0,
                    divider_profile: None,
                    divider_width: None,
                    divider_depth: None,
                }],
                rows: vec![GridDivision {
                    size: height - 2.0 * 67.0,
                    divider_profile: None,
                    divider_width: None,
                    divider_depth: None,
                }],
            },
            cells: vec![Cell::default()],
            placement: Placement::default(),
            series_id: None,
            extensions: vec![],
            layout: None,
            part_ids: PartIds::new(1, 1),
            recovered_from_ifc: false,
        }
    }

    /// Create a new kozijn using a sjabloon (template) for automatic profile assignment
    pub fn new_with_sjabloon(
        name: &str,
        mark: &str,
        width: f64,
        height: f64,
        sj: &crate::template::KozijnSjabloon,
    ) -> Self {
        let fw = sj.frame_width;
        let id = Uuid::new_v4();
        Self {
            id,
            name: name.to_string(),
            mark: mark.to_string(),
            frame: Frame {
                member_profiles: Default::default(),
                outer_width: width,
                outer_height: height,
                material: sj.material.clone(),
                profile: sj.stijl_profile.clone(),
                sill_profile: Some(sj.onderdorpel_profile.clone()),
                color_inside: sj.color_inside.clone(),
                color_outside: sj.color_outside.clone(),
                frame_width: fw,
                frame_depth: sj.frame_depth,
                profile_snapshot: None,
                sill: None,
                perimeter_stop: None,
                top_profile: Some(sj.bovendorpel_profile.clone()),
                bottom_profile: Some(sj.onderdorpel_profile.clone()),
                left_profile: Some(sj.stijl_profile.clone()),
                right_profile: Some(sj.stijl_profile.clone()),
                top_material: None,
                bottom_material: None,
                left_material: None,
                right_material: None,
                shape: FrameShape::default(),
                corner_joints: vec![
                    crate::joint::Joint::default(),
                    crate::joint::Joint::default(),
                    crate::joint::Joint::default(),
                    crate::joint::Joint::default(),
                ],
                joints_configured: false,
                edges: vec![],
            },
            grid: Grid {
                rows_from_day: false,
                columns: vec![GridDivision {
                    size: width - 2.0 * fw,
                    divider_profile: None,
                    divider_width: None,
                    divider_depth: None,
                }],
                rows: vec![GridDivision {
                    size: height - 2.0 * fw,
                    divider_profile: None,
                    divider_width: None,
                    divider_depth: None,
                }],
            },
            cells: vec![Cell::default_with_glazing(&sj.default_glazing)],
            placement: Placement::default(),
            series_id: None,
            extensions: vec![],
            layout: None,
            part_ids: PartIds::new(1, 1),
            recovered_from_ifc: false,
        }
    }

    /// Duplicate this kozijn with a new mark and UUID
    pub fn duplicate(&self, new_mark: &str) -> Self {
        let mut clone = self.clone();
        clone.id = Uuid::new_v4();
        clone.mark = new_mark.to_string();
        clone.name = format!("{} (kopie)", self.name);
        clone.part_ids = PartIds::new(clone.grid.columns.len(), clone.grid.rows.len());
        clone
    }

    /// Recalculate cells after grid changes
    pub fn rebuild_cells(&mut self) {
        let num_cells = self.grid.columns.len() * self.grid.rows.len();
        self.cells.resize_with(num_cells, Cell::default);
        self.part_ids
            .columns
            .resize_with(self.grid.columns.len(), Uuid::new_v4);
        self.part_ids
            .rows
            .resize_with(self.grid.rows.len(), Uuid::new_v4);
        self.part_ids.cells.resize_with(num_cells, Uuid::new_v4);
    }

    /// Inner width (excluding frame members)
    pub fn inner_width(&self) -> f64 {
        self.frame.outer_width - 2.0 * self.frame.frame_width
    }

    /// Inner height (excluding frame members)
    pub fn inner_height(&self) -> f64 {
        self.frame.outer_height - 2.0 * self.frame.frame_width
    }

    /// Apply one profile selection from the native or browser command.
    /// Decode both optional payloads before changing any part of the model.
    pub fn apply_frame_profile(
        &mut self,
        profile: ProfileRef,
        width: Option<f64>,
        depth: Option<f64>,
        snapshot_json: Option<&str>,
        material_json: Option<&str>,
    ) -> Result<(), String> {
        // Older clients omit the snapshot; an explicit null/empty clears it.
        let snapshot = snapshot_json
            .map(|s| {
                if s.trim().is_empty() {
                    Ok(None)
                } else {
                    serde_json::from_str(s)
                }
            })
            .transpose()
            .map_err(|e| format!("Ongeldig profielsnapshot JSON: {e}"))?;
        let material = material_json
            .filter(|s| !s.trim().is_empty())
            .map(serde_json::from_str::<Option<Material>>)
            .transpose()
            .map_err(|e| format!("Ongeldig profielmateriaal: {e}"))?
            .flatten();

        let previous = std::mem::replace(&mut self.frame.profile, profile);
        let previous_material = self.frame.material;
        if let Some(value) = material {
            self.frame.material = value;
        }
        let material_changed = previous_material != self.frame.material;
        if material_changed {
            self.frame.corner_joints = vec![
                crate::joint::Joint::default(),
                crate::joint::Joint::default(),
                crate::joint::Joint::default(),
                crate::joint::Joint::default(),
            ];
            self.frame.joints_configured = false;
        }
        if let Some(value) = snapshot {
            self.frame.profile_snapshot = value;
        } else if previous.id != self.frame.profile.id {
            self.frame.profile_snapshot = None;
        }
        if let Some(value) = depth {
            self.frame.frame_depth = value;
        }
        if let Some(value) = width {
            self.frame.frame_width = value;
        }
        let system_profile_changed = (self.frame.material.is_profile_system()
            || previous_material.is_profile_system())
            && (previous.id != self.frame.profile.id || previous_material != self.frame.material);
        if system_profile_changed {
            self.update_inherited_profiles(&previous, &previous_material);
        }
        // Update inherited divider articles first, then re-fit the cells using
        // their new face widths. Otherwise the first material/profile switch
        // leaves the grid sized for the previous article.
        if width.is_some() || system_profile_changed {
            self.fit_grid_to_frame();
        }
        Ok(())
    }

    pub fn update_member_profile(
        &mut self,
        member_type: &str,
        member_index: Option<usize>,
        profile: ProfileRef,
        profile_width: Option<f64>,
        profile_depth: Option<f64>,
    ) -> Result<(), String> {
        let mut width = profile_width.filter(|v| v.is_finite() && *v > 0.0);
        let mut depth = profile_depth.filter(|v| v.is_finite() && *v > 0.0);
        if let Some(article) = crate::profile::DividerArticle::CATALOG.iter()
            .find(|a| a.id == profile.id && !self.frame.member_profiles.contains_key(a.id)) {
            if !matches!(member_type, "divider_v" | "divider_h") || article.frame != self.frame.profile.id ||
                article.material != self.frame.material {
                return Err(format!("{} past als tussenprofiel bij kader {}", article.name, article.frame));
            }
            if width.is_some_and(|w| (w - article.width).abs() > 1e-6) ||
                depth.is_some_and(|d| (d - article.depth).abs() > 1e-6) {
                return Err(format!("De catalogusmaat van {} is {} × {} mm", article.name, article.width, article.depth));
            }
            width = Some(article.width);
            depth = Some(article.depth);
        }
        match member_type {
            "frame_top" => self.frame.top_profile = Some(profile),
            "frame_bottom" => self.frame.bottom_profile = Some(profile),
            "frame_left" => self.frame.left_profile = Some(profile),
            "frame_right" => self.frame.right_profile = Some(profile),
            "divider_v" | "divider_h" => {
                let index = member_index.ok_or("Tussenstijl of tussendorpel mist een index")? + 1;
                let divisions = if member_type == "divider_v" {
                    &mut self.grid.columns
                } else {
                    &mut self.grid.rows
                };
                let divider = divisions
                    .get_mut(index)
                    .ok_or("Tussenprofiel niet gevonden")?;
                divider.divider_profile = Some(profile);
                // An explicitly chosen system article can have the frame's
                // width while the standard T-section has another face width.
                divider.divider_width = width.filter(|v| {
                    self.frame.material.is_profile_system() || (*v - self.frame.frame_width).abs() > 1e-6
                });
                divider.divider_depth = depth;
                self.fit_grid_to_frame();
            }
            _ => return Err(format!("Onbekend member type: {member_type}")),
        }
        Ok(())
    }

    pub fn update_cell_sash_profile(
        &mut self,
        cell_index: usize,
        profile: ProfileRef,
        sash_width: f64,
        sash_depth: f64,
        snapshot: Option<crate::profile::ProfileSnapshot>,
    ) -> Result<(), String> {
        let cell = self.cells.get(cell_index).ok_or("Cel niet gevonden")?;
        let inward_article = crate::glazing_fit::inward_window_frame(&profile.id).is_some();
        if inward_article && !cell.accepts_inward_window_sash() {
            return Err(format!("Vleugelprofiel '{}' is voor een binnendraaiend raam", profile.name));
        }
        let frame_snapshot = self.frame.profile_snapshot.as_ref().or_else(|| {
            crate::glazing_fit::catalog_snapshot(&self.frame.profile.id, &self.frame.material)
        });
        if !crate::glazing_fit::system_profile_pair_compatible(
            &self.frame.material,
            &self.frame.profile.id,
            frame_snapshot,
            &profile.id,
            snapshot.as_ref(),
        ) {
            return Err(format!(
                "Vleugelprofiel '{}' hoort niet bij profielsysteem '{}'",
                profile.name, self.frame.profile.name,
            ));
        }
        // A supplier article has one cross section, including when an older
        // client sends the system frame dimensions along with its selection.
        let article = inward_article.then(|| crate::glazing_fit::catalog_snapshot(&profile.id, &self.frame.material)).flatten();
        let sash_width = article.and_then(|p| p.sash_face_width).unwrap_or(sash_width);
        let sash_depth = article.and_then(|p| p.sash_build_depth).unwrap_or(sash_depth);
        let cell = &mut self.cells[cell_index];
        if self.frame.material.is_profile_system()
            && cell.sash_profile.as_ref().unwrap_or(&self.frame.profile).id != profile.id
        {
            cell.glaslat = None;
        }
        cell.sash_profile = Some(profile);
        cell.sash_profile_snapshot = snapshot;
        if sash_width.is_finite() && sash_width > 0.0 {
            cell.sash_width = Some(sash_width);
        }
        if sash_depth.is_finite() && sash_depth > 0.0 {
            cell.sash_depth = Some(sash_depth);
        }
        Ok(())
    }

    pub fn divider_face_width(&self, vertical: bool, index: usize) -> f64 {
        let divisions = if vertical {
            &self.grid.columns
        } else {
            &self.grid.rows
        };
        divisions
            .get(index + 1)
            .and_then(|d| d.divider_width)
            .unwrap_or(self.frame.frame_width)
    }

    /// Face width of the frame system's standard mullion/transom.
    pub fn default_divider_width(&self) -> f64 {
        let frame = &self.frame;
        if frame.material.is_profile_system() {
            let valid = |p: &crate::profile::ProfileSnapshot| {
                p.divider_face_width.filter(|w| w.is_finite() && *w > 0.)
            };
            frame.profile_snapshot.as_ref().and_then(valid)
                .or_else(|| crate::glazing_fit::catalog_snapshot(&frame.profile.id, &frame.material).and_then(valid))
                .unwrap_or(frame.frame_width)
        } else {
            frame.frame_width
        }
    }

    pub fn layout_divider_width(&self) -> f64 {
        match self.frame.shape.shape_type {
            ShapeType::Round | ShapeType::Arched | ShapeType::ArchedTrapezoid => self.default_divider_width(),
            _ => self.frame.frame_width,
        }
    }

    /// A family name alone cannot identify stock of a different section or a project-specific article.
    pub(crate) fn system_divider_article(&self, division: Option<&GridDivision>, width: f64) -> Option<&'static crate::profile::DividerArticle> {
        let frame = &self.frame;
        let stock = division.and_then(|d| d.divider_profile.as_ref());
        let depth = division.and_then(|d| d.divider_depth).unwrap_or(frame.frame_depth);
        crate::profile::DividerArticle::CATALOG.iter().find(|a| {
            a.frame == frame.profile.id && a.material == frame.material
                && (width - a.width).abs() < 1e-6 && (depth - a.depth).abs() < 1e-6
                && !frame.member_profiles.contains_key(a.id) && !frame.member_profiles.contains_key(&frame.profile.id)
                && stock.is_none_or(|p| p.id == a.id || p.id == a.frame || p.id == "wood-meranti-67x114")
        })
    }

    fn new_grid_divider(&self, size: f64, width: f64) -> GridDivision {
        GridDivision {
            size,
            divider_profile: Some(if let Some(article) = self.system_divider_article(None, width) {
                article.reference()
            } else if self.frame.material.is_profile_system() {
                self.frame.profile.clone()
            } else {
                ProfileRef::default_divider()
            }),
            divider_width: (width != self.frame.frame_width).then_some(width),
            divider_depth: None,
        }
    }

    /// Old grid splits stored a Meranti placeholder on system frames and
    /// reserved the frame width in the bay sizes. Restore the corresponding
    /// system section and refit the bays once on opening that project.
    fn restore_system_dividers(&mut self) {
        if !self.frame.material.is_profile_system() || self.recovered_from_ifc {
            return;
        }
        let standard = self.default_divider_width();
        let article = self.system_divider_article(None, standard);
        let placeholder = ProfileRef::default_divider();
        let mut resized = false;
        for d in self.grid.columns.iter_mut().skip(1).chain(self.grid.rows.iter_mut().skip(1)) {
            let inherited = d.divider_profile.as_ref().is_none_or(|p| {
                p.id == self.frame.profile.id || p.id == placeholder.id
            });
            if !inherited {
                continue;
            }
            if let Some(a) = article.filter(|a|
                (d.divider_width.unwrap_or(standard) - a.width).abs() < 1e-6 &&
                (d.divider_depth.unwrap_or(self.frame.frame_depth) - a.depth).abs() < 1e-6
            ) {
                d.divider_profile = Some(a.reference());
            } else if d.divider_profile.as_ref().is_none_or(|p| p.id == placeholder.id) {
                d.divider_profile = Some(self.frame.profile.clone());
            }
            if d.divider_width.is_none() && standard != self.frame.frame_width {
                d.divider_width = Some(standard);
                resized = true;
            }
        }
        if resized {
            self.fit_grid_to_frame();
        }
    }

    fn update_inherited_profiles(&mut self, previous: &ProfileRef, previous_material: &Material) {
        let presets = crate::template::builtin_sjablonen();
        let old = presets
            .iter()
            .find(|s| s.stijl_profile.id == previous.id && s.material == *previous_material);
        let new = presets.iter().find(|s| {
            s.stijl_profile.id == self.frame.profile.id && s.material == self.frame.material
        });
        let frame = &self.frame.profile;
        let replace = |slot: &mut Option<ProfileRef>,
                       before: Option<&ProfileRef>,
                       after: Option<&ProfileRef>| {
            if slot
                .as_ref()
                .is_some_and(|p| p.id == previous.id || before.is_some_and(|b| p.id == b.id))
            {
                *slot = Some(after.unwrap_or(frame).clone());
            }
        };
        if self.frame.top_material.is_none() {
            replace(
                &mut self.frame.top_profile,
                old.map(|s| &s.bovendorpel_profile),
                new.map(|s| &s.bovendorpel_profile),
            );
        }
        if self.frame.bottom_material.is_none() {
            replace(
                &mut self.frame.sill_profile,
                old.map(|s| &s.onderdorpel_profile),
                new.map(|s| &s.onderdorpel_profile),
            );
            replace(
                &mut self.frame.bottom_profile,
                old.map(|s| &s.onderdorpel_profile),
                new.map(|s| &s.onderdorpel_profile),
            );
        }
        if self.frame.left_material.is_none() {
            replace(&mut self.frame.left_profile, None, None);
        }
        if self.frame.right_material.is_none() {
            replace(&mut self.frame.right_profile, None, None);
        }
        for column in &mut self.grid.columns {
            let was_inherited = column.divider_profile.as_ref().is_some_and(|p| {
                p.id == previous.id || old.is_some_and(|s| p.id == s.tussenstijl_profile.id)
            });
            replace(
                &mut column.divider_profile,
                old.map(|s| &s.tussenstijl_profile),
                new.map(|s| &s.tussenstijl_profile),
            );
            if was_inherited {
                if old.is_some_and(|s| column.divider_depth == Some(s.frame_depth)) {
                    column.divider_depth = None;
                }
                let width = new
                    .and_then(|s| {
                        crate::glazing_fit::catalog_snapshot(&s.tussenstijl_profile.id, &s.material)
                    })
                    .and_then(|p| p.divider_face_width)
                    .unwrap_or(self.frame.frame_width);
                column.divider_width =
                    ((width - self.frame.frame_width).abs() > 1e-6).then_some(width);
            }
        }
        for row in &mut self.grid.rows {
            let was_inherited = row.divider_profile.as_ref().is_some_and(|p| {
                p.id == previous.id || old.is_some_and(|s| p.id == s.tussendorpel_profile.id)
            });
            replace(
                &mut row.divider_profile,
                old.map(|s| &s.tussendorpel_profile),
                new.map(|s| &s.tussendorpel_profile),
            );
            if was_inherited {
                if old.is_some_and(|s| row.divider_depth == Some(s.frame_depth)) {
                    row.divider_depth = None;
                }
                let width = new
                    .and_then(|s| {
                        crate::glazing_fit::catalog_snapshot(
                            &s.tussendorpel_profile.id,
                            &s.material,
                        )
                    })
                    .and_then(|p| p.divider_face_width)
                    .unwrap_or(self.frame.frame_width);
                row.divider_width =
                    ((width - self.frame.frame_width).abs() > 1e-6).then_some(width);
            }
        }
        for cell in &mut self.cells {
            let inherited_carrier = cell.sash_profile.as_ref().is_none_or(|p| p.id == previous.id);
            if *previous_material != self.frame.material
                || !cell.panel_type.is_operable()
                || inherited_carrier
            {
                cell.glaslat = None;
            }
            if !cell.panel_type.is_operable() {
                continue;
            }
            let old_sash = old.map(|s| {
                if cell.panel_type == PanelType::Door {
                    &s.deurhout_profile
                } else {
                    &s.raamhout_profile
                }
            });
            let selected = cell.sash_profile.as_ref();
            let belongs_to_old_frame = selected
                .and_then(|p| crate::glazing_fit::inward_window_frame(&p.id))
                .is_some_and(|id| id == previous.id && id != self.frame.profile.id);
            let inherited = cell.sash_profile_snapshot.is_none()
                && selected.is_none_or(|p| p.id == previous.id || old_sash.is_some_and(|s| s.id == p.id));
            if belongs_to_old_frame || inherited {
                cell.glaslat = None;
                cell.assign_default_sash(&self.frame, new);
            }
        }
    }

    /// Re-fit the grid to the current frame (outer size + frame width) while
    /// keeping the invariant Σ(vakken) + (n−1)·frame_width = binnenmaat — the
    /// same one `add_column`/`add_row` establish. Callers change
    /// `outer_*`/`frame_width` first, then call this. Scaling the vakken by
    /// new_inner/old_inner alone ignores the tussenstijlen and drifts (1200 →
    /// 1210 mm is already 0,63 mm off), which the IFC preflight rejects.
    pub fn fit_grid_to_frame(&mut self) {
        let fw = self.frame.frame_width;
        let cols_target = self.inner_width()
            - self
                .grid
                .columns
                .iter()
                .skip(1)
                .map(|c| c.divider_width.unwrap_or(fw))
                .sum::<f64>();
        let rows_target = self.inner_height()
            - self
                .grid
                .rows
                .iter()
                .skip(1)
                .map(|r| r.divider_width.unwrap_or(fw))
                .sum::<f64>();
        let mut cols: Vec<f64> = self.grid.columns.iter().map(|c| c.size).collect();
        let mut rows: Vec<f64> = self.grid.rows.iter().map(|r| r.size).collect();
        fit_sizes(&mut cols, cols_target, &[]);
        fit_sizes(&mut rows, rows_target, &[]);
        for (c, s) in self.grid.columns.iter_mut().zip(cols) {
            c.size = s;
        }
        for (r, s) in self.grid.rows.iter_mut().zip(rows) {
            r.size = s;
        }
    }

    /// Apply explicit vak sizes (typed by the user) and restore the grid
    /// invariant by letting the vakken the user did NOT touch absorb the
    /// difference, so a typed value stays exactly as typed.
    pub fn set_grid_sizes_fitted(&mut self, column_sizes: &[f64], row_sizes: &[f64]) {
        let fw = self.frame.frame_width;
        let cols_target = self.inner_width()
            - self
                .grid
                .columns
                .iter()
                .skip(1)
                .map(|d| d.divider_width.unwrap_or(fw))
                .sum::<f64>();
        let rows_target = self.inner_height()
            - self
                .grid
                .rows
                .iter()
                .skip(1)
                .map(|d| d.divider_width.unwrap_or(fw))
                .sum::<f64>();
        let apply = |current: Vec<f64>, wanted: &[f64], target: f64| -> Vec<f64> {
            let mut sizes = current.clone();
            let mut pinned = vec![false; sizes.len()];
            for (i, s) in sizes.iter_mut().enumerate() {
                if let Some(&w) = wanted.get(i) {
                    let w = w.max(100.0); // minimum 100mm
                    if (w - current[i]).abs() > 1e-6 {
                        pinned[i] = true;
                    }
                    *s = w;
                }
            }
            fit_sizes(&mut sizes, target, &pinned);
            sizes
        };
        let cols = apply(
            self.grid.columns.iter().map(|c| c.size).collect(),
            column_sizes,
            cols_target,
        );
        let rows = apply(
            self.grid.rows.iter().map(|r| r.size).collect(),
            row_sizes,
            rows_target,
        );
        for (c, s) in self.grid.columns.iter_mut().zip(cols) {
            c.size = s;
        }
        for (r, s) in self.grid.rows.iter_mut().zip(rows) {
            r.size = s;
        }
    }

    /// Place a full-width divider with its centre on an elevation coordinate.
    /// Retain the displayed edges of every existing bay, including old grids
    /// whose row weights were distributed below the spring line.
    pub fn add_divider_in_elevation(&mut self, horizontal: bool, position: f64) {
        if !position.is_finite() || self.layout.is_some() {
            return;
        }
        let geometry = crate::geometry::compute_2d_geometry(self);
        let width = self.frame.frame_width;
        let half_divider = self.default_divider_width() / 2.;
        let (bars, sizes, end) = if horizontal {
            (
                &geometry.h_dividers,
                &self.grid.rows,
                self.frame.outer_height - width,
            )
        } else {
            (
                &geometry.v_dividers,
                &self.grid.columns,
                self.frame.outer_width - width,
            )
        };
        if sizes.len() != bars.len() + 1
            || sizes.iter().any(|s| !s.size.is_finite() || s.size <= 0.0)
        {
            return;
        }
        let mut cursor = width;
        let mut split = None;
        let mut spans = Vec::with_capacity(sizes.len() + 1);
        for i in 0..=bars.len() {
            let (next, thickness) = bars.get(i).map_or((end, 0.0), |r| {
                if horizontal {
                    (r.y, r.height)
                } else {
                    (r.x, r.width)
                }
            });
            let before = position - half_divider - cursor;
            let after = next - position - half_divider;
            if before > 0.0 && after > 0.0 {
                split = Some(i);
                spans.extend([before, after]);
            } else {
                spans.push(next - cursor);
            }
            cursor = next + thickness;
        }
        let Some(i) = split else {
            return;
        };
        // Let the existing split preserve cell data, part IDs and profile references.
        let split_at = sizes[..i].iter().map(|s| s.size).sum::<f64>() + sizes[i].size / 2.0;
        let divisions = if horizontal {
            self.add_row(split_at);
            self.grid.rows_from_day = true;
            &mut self.grid.rows
        } else {
            self.add_column(split_at);
            &mut self.grid.columns
        };
        for (division, span) in divisions.iter_mut().zip(spans) {
            division.size = span;
        }
    }

    /// Add a vertical divider (creates a new column). Its article face width
    /// is subtracted proportionally from the
    /// two new column sizes so that Σ(kolommen) + Σ(tussenstijlen) stays
    /// exactly the binnenmaat — the level-1 dimension chain keeps adding up
    /// to the buitenwerkse maat and the last vak no longer runs underneath
    /// the stijl.
    pub fn add_column(&mut self, position: f64) {
        let divider_w = self.default_divider_width();
        let total_existing: f64 = self.grid.columns.iter().map(|c| c.size).sum();

        if position > 0.0 && position < total_existing {
            let mut accumulated = 0.0;
            for i in 0..self.grid.columns.len() {
                let col_end = accumulated + self.grid.columns[i].size;
                if position > accumulated && position < col_end {
                    if self.frame.material.is_profile_system() && self.grid.columns[i].size <= divider_w {
                        return;
                    }
                    self.materialize_part_ids();
                    let old_cols = self.grid.columns.len();
                    let rows = self.grid.rows.len();
                    let mut left = position - accumulated;
                    let mut right = col_end - position;
                    // Reserve room for the divider itself: shrink both halves
                    // proportionally.
                    let span = left + right;
                    if span > divider_w {
                        let scale = (span - divider_w) / span;
                        left *= scale;
                        right *= scale;
                    }
                    self.grid.columns[i].size = left;
                    self.grid.columns.insert(i + 1, self.new_grid_divider(right, divider_w));
                    self.part_ids.columns.insert(i + 1, Uuid::new_v4());
                    let old_cells = std::mem::take(&mut self.cells);
                    let old_ids = std::mem::take(&mut self.part_ids.cells);
                    for row in 0..rows {
                        for col in 0..old_cols {
                            self.cells.push(old_cells[row * old_cols + col].clone());
                            self.part_ids.cells.push(old_ids[row * old_cols + col]);
                            if col == i {
                                self.cells.push(Cell::default());
                                self.part_ids.cells.push(Uuid::new_v4());
                            }
                        }
                    }
                    break;
                }
                accumulated = col_end;
            }
        }
    }

    /// Add a horizontal divider (creates a new row). The new tussendorpel
    /// occupies its article face width, subtracted proportionally from the two new
    /// row sizes — see `add_column` for the invariant.
    pub fn add_row(&mut self, position: f64) {
        let divider_w = self.default_divider_width();
        let total_existing: f64 = self.grid.rows.iter().map(|r| r.size).sum();

        if position > 0.0 && position < total_existing {
            let mut accumulated = 0.0;
            for i in 0..self.grid.rows.len() {
                let row_end = accumulated + self.grid.rows[i].size;
                if position > accumulated && position < row_end {
                    if self.frame.material.is_profile_system() && self.grid.rows[i].size <= divider_w {
                        return;
                    }
                    self.materialize_part_ids();
                    let cols = self.grid.columns.len();
                    let old_rows = self.grid.rows.len();
                    let mut top = position - accumulated;
                    let mut bottom = row_end - position;
                    // Reserve room for the divider itself (see add_column).
                    let span = top + bottom;
                    if span > divider_w {
                        let scale = (span - divider_w) / span;
                        top *= scale;
                        bottom *= scale;
                    }
                    self.grid.rows[i].size = top;
                    self.grid.rows.insert(i + 1, self.new_grid_divider(bottom, divider_w));
                    self.part_ids.rows.insert(i + 1, Uuid::new_v4());
                    let old_cells = std::mem::take(&mut self.cells);
                    let old_ids = std::mem::take(&mut self.part_ids.cells);
                    for row in 0..old_rows {
                        for col in 0..cols {
                            self.cells.push(old_cells[row * cols + col].clone());
                            self.part_ids.cells.push(old_ids[row * cols + col]);
                        }
                        if row == i {
                            for _ in 0..cols {
                                self.cells.push(Cell::default());
                                self.part_ids.cells.push(Uuid::new_v4());
                            }
                        }
                    }
                    break;
                }
                accumulated = row_end;
            }
        }
    }
}

/// Scale `sizes` so they sum to `target`. Entries flagged in `pinned` keep
/// their value; only the others absorb the difference (proportionally). When
/// everything is pinned (or the free entries would go non-positive) all
/// entries are scaled instead, so the result is always positive and exact.
fn fit_sizes(sizes: &mut [f64], target: f64, pinned: &[bool]) {
    if sizes.is_empty() || target <= 0.0 {
        return;
    }
    let is_pinned = |i: usize| pinned.get(i).copied().unwrap_or(false);
    let pinned_sum: f64 = sizes
        .iter()
        .enumerate()
        .filter(|(i, _)| is_pinned(*i))
        .map(|(_, s)| *s)
        .sum();
    let free_sum: f64 = sizes
        .iter()
        .enumerate()
        .filter(|(i, _)| !is_pinned(*i))
        .map(|(_, s)| *s)
        .sum();
    let free_target = target - pinned_sum;
    if free_sum > 0.0 && free_target > 0.0 {
        let scale = free_target / free_sum;
        for (i, s) in sizes.iter_mut().enumerate() {
            if !is_pinned(i) {
                *s *= scale;
            }
        }
    } else {
        let sum: f64 = sizes.iter().sum();
        if sum > 0.0 {
            let scale = target / sum;
            for s in sizes.iter_mut() {
                *s *= scale;
            }
        }
    }
}

/// Frame definition (the outer border of the kozijn)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Frame {
    /// Project-defined sections, shared by all frames and rebuilt on load.
    /// Their single persisted source is Project::custom_profiles.
    #[serde(skip)]
    pub member_profiles:
        std::sync::Arc<std::collections::BTreeMap<String, crate::profile::ProfileDefinition>>,
    /// Overall outer width in mm
    pub outer_width: f64,
    /// Overall outer height in mm
    pub outer_height: f64,
    pub material: Material,
    pub profile: ProfileRef,
    pub sill_profile: Option<ProfileRef>,
    pub color_inside: String,
    pub color_outside: String,
    /// Frame member width (face width) in mm
    pub frame_width: f64,
    /// Frame member depth in mm
    pub frame_depth: f64,
    /// Resolved dimensions of the selected frame profile (sponning, glaslat,
    /// aanzichtbreedte), captured when the profile is chosen. `None` on
    /// pre-snapshot projects — drawing and checks fall back to the classic
    /// defaults. Skipped when absent so old payloads stay byte-identical.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_snapshot: Option<crate::profile::ProfileSnapshot>,
    /// Structured sill configuration (v1.3+)
    #[serde(default)]
    pub sill: Option<crate::sill::Sill>,
    /// Optional separate exterior lip. Frame dimensions exclude its projection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub perimeter_stop: Option<crate::perimeter_stop::StopProfile>,
    /// Per-member profile overrides (None = use default frame profile)
    #[serde(default)]
    pub top_profile: Option<ProfileRef>,
    #[serde(default)]
    pub bottom_profile: Option<ProfileRef>,
    #[serde(default)]
    pub left_profile: Option<ProfileRef>,
    #[serde(default)]
    pub right_profile: Option<ProfileRef>,
    /// Frame shape (rectangular, arched, round)
    #[serde(default)]
    pub shape: FrameShape,
    /// Per-member material overrides (None = use default frame material)
    #[serde(default)]
    pub top_material: Option<Material>,
    #[serde(default)]
    pub bottom_material: Option<Material>,
    #[serde(default)]
    pub left_material: Option<Material>,
    #[serde(default)]
    pub right_material: Option<Material>,
    /// Corner joint configurations [top-left, top-right, bottom-left, bottom-right]
    #[serde(default)]
    pub corner_joints: Vec<crate::joint::Joint>,
    /// `true` once the user has explicitly saved corner joints (set by the
    /// update_corner_joints commands). `Kozijn::new` auto-populates
    /// `corner_joints` with four defaults, so the values alone cannot tell a
    /// deliberate "pen/slis, stijl doorlopend" apart from "never touched";
    /// this flag can. `false` on old .ofs files, which keeps the historic
    /// sentinel fallback (`crate::joint::effective_corner_joints`).
    #[serde(default)]
    pub joints_configured: bool,
    /// Edge configurations [left, right, top, bottom] — wall connection details
    #[serde(default)]
    pub edges: Vec<crate::edge::EdgeConfig>,
}

/// Frame shape definition for arched/round kozijnen
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FrameShape {
    pub shape_type: ShapeType,
    /// Arch radius in mm (for arched shape)
    #[serde(default)]
    pub arch_radius: Option<f64>,
    /// Arch rise height above the rectangular top in mm
    #[serde(default)]
    pub arch_height: Option<f64>,
    /// Trapezoid: top width (can differ from outer_width)
    #[serde(default)]
    pub top_width: Option<f64>,
    /// Trapezoid: left stile angle in degrees (90 = vertical)
    #[serde(default)]
    pub left_angle: Option<f64>,
    /// Trapezoid: right stile angle in degrees (90 = vertical)
    #[serde(default)]
    pub right_angle: Option<f64>,
    /// Elliptical: horizontal radius in mm (default ow/2)
    #[serde(default)]
    pub ellipse_rx: Option<f64>,
    /// Elliptical: vertical radius in mm (default oh/3)
    #[serde(default)]
    pub ellipse_ry: Option<f64>,
    /// Polygon: custom polygon points [[x,y], ...] in mm (for Polygon shape)
    #[serde(default)]
    pub polygon_points: Option<Vec<[f64; 2]>>,
    /// Triangle: apex horizontal offset from center in mm (for Triangle shape)
    #[serde(default)]
    pub apex_offset: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ShapeType {
    #[default]
    Rectangular,
    Arched, // getoogde bovendorpel (segmentboog)
    Round,  // volledig rond (cirkel)
    Elliptical,
    Trapezoid,       // schuine stijl(en) of dorpel
    ArchedTrapezoid, // boog + schuine stijlen gecombineerd (CNCware-stijl)
    Triangle,        // driehoekig kozijn
    Polygon,         // vrije veelhoek
}

/// Grid subdivision — columns (vertical) and rows (horizontal)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Grid {
    /// Absolute row sizes from the top of the day. Older arched grids use
    /// relative sizes below the spring until a divider is placed or dragged.
    #[serde(default, skip_serializing_if = "is_false")]
    pub rows_from_day: bool,
    pub columns: Vec<GridDivision>,
    pub rows: Vec<GridDivision>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridDivision {
    /// Size in mm
    pub size: f64,
    /// Divider profile (None for first column/row since there's no divider before it)
    pub divider_profile: Option<ProfileRef>,
    /// Face width of the selected divider article, in mm.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub divider_width: Option<f64>,
    /// Build depth of the selected divider article, in mm.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub divider_depth: Option<f64>,
}

/// A single cell (vak) in the kozijn grid
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cell {
    pub panel_type: PanelType,
    pub opening_direction: Option<OpeningDirection>,
    pub glazing: Glazing,
    /// Legacy hardware list — kept for backward compatibility with format_version "1.0"
    #[serde(default)]
    pub hardware: Vec<Hardware>,
    /// Structured hardware set (format_version "1.1"+)
    #[serde(default)]
    pub hardware_set: Option<HardwareSet>,
    /// Sash/door profile for operable cells (raamhout/deurhout)
    #[serde(default)]
    pub sash_profile: Option<ProfileRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sash_profile_snapshot: Option<crate::profile::ProfileSnapshot>,
    /// Sash wood width in mm (54/67mm typical)
    #[serde(default)]
    pub sash_width: Option<f64>,
    /// Sash wood depth in mm (67/78/90mm typical)
    #[serde(default)]
    pub sash_depth: Option<f64>,
    /// Infill panel spec for Panel/Ventilation cells (vakvulling). None = use defaults.
    #[serde(default)]
    pub panel_filling: Option<PanelFilling>,
    /// Glazing bead spec for glazed cells (glaslat). None = no explicit bead.
    #[serde(default)]
    pub glaslat: Option<Glaslat>,
    /// Designated as an escape window (vluchtraam) — triggers Bouwbesluit checks.
    #[serde(default)]
    pub is_escape: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            panel_type: PanelType::FixedGlass,
            opening_direction: None,
            glazing: Glazing::default(),
            hardware: vec![],
            hardware_set: None,
            sash_profile: None,
            sash_profile_snapshot: None,
            sash_width: None,
            sash_depth: None,
            panel_filling: None,
            glaslat: None,
            is_escape: false,
        }
    }
}

impl Cell {
    fn accepts_inward_window_sash(&self) -> bool {
        matches!(self.panel_type, PanelType::TurnTilt | PanelType::Turn | PanelType::Tilt)
            && self.opening_direction != Some(OpeningDirection::Outward)
    }

    fn template_sash<'a>(&self, sj: &'a crate::template::KozijnSjabloon) -> &'a ProfileRef {
        if self.panel_type == PanelType::Door {
            &sj.deurhout_profile
        } else if crate::glazing_fit::inward_window_frame(&sj.raamhout_profile.id).is_some()
            && !self.accepts_inward_window_sash()
        {
            &sj.stijl_profile
        } else {
            &sj.raamhout_profile
        }
    }

    fn template_sash_dimensions(&self, sj: &crate::template::KozijnSjabloon) -> (f64, f64) {
        if self.panel_type == PanelType::Door && !sj.material.is_profile_system() {
            return (sj.frame_width, sj.frame_depth);
        }
        if crate::glazing_fit::inward_window_frame(&sj.raamhout_profile.id).is_some() {
            let selected = self.template_sash(sj);
            let spec = crate::glazing_fit::catalog_snapshot(&selected.id, &sj.material);
            // A door, slider or outward sash cannot inherit the physical face
            // of the template's dedicated inward-opening article.
            let width = if selected.id == sj.stijl_profile.id { sj.frame_width } else { sj.sash_width };
            return (
                spec.and_then(|p| p.sash_face_width).unwrap_or(width),
                spec.and_then(|p| p.sash_build_depth).unwrap_or(sj.sash_depth),
            );
        }
        (sj.sash_width, sj.sash_depth)
    }

    /// Create a cell with glazing preset from a sjabloon
    pub fn default_with_glazing(preset: &crate::template::GlazingPreset) -> Self {
        Self {
            glazing: Glazing {
                glass_type: preset.glass_type.clone(),
                thickness_mm: preset.thickness_mm,
                ug_value: preset.ug_value,
                panes: vec![],
                spacer_type: preset.spacer_type.clone(),
            },
            ..Self::default()
        }
    }

    /// Resolve the same sash stock when changing a cell in the desktop or web editor.
    pub fn change_panel_type(&mut self, panel_type: PanelType, frame: &Frame) {
        self.change_panel_opening(panel_type, self.opening_direction, frame);
    }

    pub(crate) fn change_panel_opening(&mut self, panel_type: PanelType, direction: Option<OpeningDirection>, frame: &Frame) {
        let was_inward_window = self.accepts_inward_window_sash();
        let change_stock = (self.panel_type == PanelType::Door) != (panel_type == PanelType::Door);
        let has_stock = self.sash_profile.is_some()
            || self.sash_profile_snapshot.is_some()
            || self.sash_width.is_some()
            || self.sash_depth.is_some();
        self.panel_type = panel_type;
        self.opening_direction = direction;
        if !panel_type.is_operable() {
            self.sash_profile = None;
            self.sash_profile_snapshot = None;
            self.sash_width = None;
            self.sash_depth = None;
        } else if change_stock || !has_stock || self.sash_profile.as_ref().is_some_and(|profile| {
            (!self.accepts_inward_window_sash() && crate::glazing_fit::inward_window_frame(&profile.id).is_some())
                || (!was_inward_window && self.accepts_inward_window_sash() && frame.material.is_profile_system()
                    && profile.id == frame.profile.id && self.sash_profile_snapshot.is_none())
        }) {
            let templates = crate::template::builtin_sjablonen();
            let stock = templates
                .iter()
                .find(|s| s.stijl_profile.id == frame.profile.id && s.material == frame.material);
            self.assign_default_sash(frame, stock);
        }
    }

    fn assign_default_sash(
        &mut self,
        frame: &Frame,
        preset: Option<&crate::template::KozijnSjabloon>,
    ) {
        if frame.material.is_profile_system() {
            let stored = frame.profile_snapshot.as_ref();
            let catalog = crate::glazing_fit::catalog_snapshot(&frame.profile.id, &frame.material);
            let selected = preset.map_or(&frame.profile, |sj| self.template_sash(sj));
            let article = (selected.id != frame.profile.id)
                .then(|| crate::glazing_fit::catalog_snapshot(&selected.id, &frame.material)).flatten();
            let (width, depth) = preset.map(|s| self.template_sash_dimensions(s))
                .unwrap_or((frame.frame_width, frame.frame_depth));
            self.sash_profile = Some(selected.clone());
            self.sash_profile_snapshot = None;
            self.sash_width = Some(
                article.and_then(|p| p.sash_face_width)
                    .or_else(|| stored.and_then(|s| s.sash_face_width))
                    .or_else(|| catalog.and_then(|s| s.sash_face_width))
                    .filter(|v| *v > 0.)
                    .unwrap_or(width),
            );
            self.sash_depth = Some(
                article.and_then(|p| p.sash_build_depth)
                    .or_else(|| stored.and_then(|s| s.sash_build_depth))
                    .or_else(|| catalog.and_then(|s| s.sash_build_depth))
                    .filter(|v| *v > 0.)
                    .unwrap_or(depth),
            );
        } else if let Some(preset) = preset {
            self.assign_sash_from_sjabloon(preset);
        } else {
            let stock = crate::template::builtin_sjablonen()
                .into_iter()
                .find(|s| s.material == frame.material)
                .unwrap_or_else(crate::template::default_sjabloon);
            self.assign_sash_from_sjabloon(&stock);
        }
    }

    /// Assign sash profile for operable cells (raamhout/deurhout)
    pub fn assign_sash_from_sjabloon(&mut self, sj: &crate::template::KozijnSjabloon) {
        self.sash_profile_snapshot = None;
        if self.panel_type.is_operable() {
            let (width, depth) = self.template_sash_dimensions(sj);
            self.sash_profile = Some(self.template_sash(sj).clone());
            self.sash_width = Some(width);
            self.sash_depth = Some(depth);
        } else {
            self.sash_profile = None;
            self.sash_width = None;
            self.sash_depth = None;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PanelType {
    FixedGlass,
    TurnTilt,    // draaikiepraam
    Turn,        // draairaam
    Tilt,        // kiepraam
    Sliding,     // schuifraam
    Door,        // deur
    Panel,       // dicht paneel
    Ventilation, // ventilatierooster
    TopHung,     // klapraam / klep
    BottomHung,  // tuimelraam
    LiftSlide,   // hefschuifpui
    Pivot,       // melkmeisje / pivot raam
}

impl PanelType {
    pub fn label_nl(&self) -> &'static str {
        match self {
            Self::FixedGlass => "Vast glas",
            Self::TurnTilt => "Draaikiepraam",
            Self::Turn => "Draairaam",
            Self::Tilt => "Kiepraam",
            Self::Sliding => "Schuifraam",
            Self::Door => "Deur",
            Self::Panel => "Paneel",
            Self::Ventilation => "Ventilatie",
            Self::TopHung => "Klapraam",
            Self::BottomHung => "Tuimelraam",
            Self::LiftSlide => "Hefschuifpui",
            Self::Pivot => "Pivotraam",
        }
    }

    pub fn is_operable(&self) -> bool {
        matches!(
            self,
            Self::TurnTilt
                | Self::Turn
                | Self::Tilt
                | Self::Sliding
                | Self::Door
                | Self::TopHung
                | Self::BottomHung
                | Self::LiftSlide
                | Self::Pivot
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpeningDirection {
    Left,
    Right,
    Inward,
    Outward,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Glazing {
    pub glass_type: String,
    pub thickness_mm: f64,
    pub ug_value: f64,
    /// Individual pane layers (glass + spacer + glass + ...)
    #[serde(default)]
    pub panes: Vec<Pane>,
    /// Spacer type between panes
    #[serde(default = "default_spacer_type")]
    pub spacer_type: String,
}

fn default_spacer_type() -> String {
    "warm-edge".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pane {
    /// Pane thickness in mm (4, 5, 6, 8, 10)
    pub thickness_mm: f64,
    /// Pane type: "float", "gehard", "gelamineerd", "low-e"
    pub pane_type: String,
}

impl Default for Glazing {
    fn default() -> Self {
        Self {
            glass_type: "HR++".into(),
            thickness_mm: 24.0,
            ug_value: 1.0,
            panes: vec![
                Pane {
                    thickness_mm: 4.0,
                    pane_type: "float".into(),
                },
                Pane {
                    thickness_mm: 4.0,
                    pane_type: "low-e".into(),
                },
            ],
            spacer_type: "warm-edge".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hardware {
    pub hardware_type: HardwareType,
    pub position: String,
    pub brand: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HardwareType {
    Hinge,
    Handle,
    Lock,
    Ventilation,
    Closer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Material {
    Wood(WoodType),
    Aluminum,
    Pvc,
    WoodAluminum,
}

impl Material {
    pub fn is_profile_system(&self) -> bool {
        matches!(self, Self::Pvc | Self::Aluminum)
    }

    pub fn label_nl(&self) -> &'static str {
        match self {
            Self::Wood(WoodType::Meranti) => "Meranti",
            Self::Wood(WoodType::Accoya) => "Accoya",
            Self::Wood(WoodType::Vuren) => "Vuren",
            Self::Wood(WoodType::Eiken) => "Eiken",
            Self::Aluminum => "Aluminium",
            Self::Pvc => "Kunststof",
            Self::WoodAluminum => "Hout-aluminium",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WoodType {
    Meranti,
    Accoya,
    Vuren,
    Eiken,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Placement {
    pub wall_id: Option<String>,
    /// Origin of the lower-left corner in project coordinates, millimetres.
    pub x: f64,
    pub y: f64,
    pub z: f64,
    /// Rotation of the width axis about project Z, in degrees.
    /// At zero degrees the window width points along +X and its depth
    /// points along -Y; its height always points along +Z.
    pub orientation: f64,
}

/// A complete project file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase")]
pub struct Project {
    /// Persistent identity for project-level IFC GlobalIds. Older .ofs files
    /// receive one on load and retain it after the next save.
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub format_version: String,
    pub project_info: ProjectInfo,
    pub kozijnen: Vec<Kozijn>,
    #[serde(default)]
    pub vliesgevels: Vec<crate::vliesgevel::Vliesgevel>,
    #[serde(default)]
    pub custom_profiles: Vec<crate::profile::ProfileDefinition>,
    #[serde(default)]
    pub custom_sjablonen: Vec<crate::template::KozijnSjabloon>,
    /// Stable IFC GUIDs: maps element path (e.g. "kozijn:{id}") to a persistent UUID
    #[serde(default)]
    pub ifc_guid_map: std::collections::HashMap<String, uuid::Uuid>,
    /// Pricing / quotation configuration
    #[serde(default)]
    pub pricing_config: Option<crate::pricing::PricingConfig>,
    /// BCF topics for issue tracking
    #[serde(default)]
    pub bcf_topics: Vec<crate::bcf::BcfTopic>,
    /// Quotation versions
    #[serde(default)]
    pub quotations: Vec<crate::quotation::Quotation>,
    #[serde(default)]
    pub combinations: Vec<crate::combination::CombinationKozijn>,
}

impl Serialize for Project {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Project::serialize(self, serializer)
    }
}

impl<'de> Deserialize<'de> for Project {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut project = Project::deserialize(deserializer)?;
        project.refresh_member_profiles();
        for kozijn in &mut project.kozijnen {
            kozijn.restore_system_dividers();
        }
        Ok(project)
    }
}

impl Project {
    fn refresh_member_profiles(&mut self) {
        if self.kozijnen.is_empty() {
            return;
        }
        let library = std::sync::Arc::new(
            self.custom_profiles
                .iter()
                .map(|p| (p.id.clone(), p.clone()))
                .collect(),
        );
        for k in &mut self.kozijnen {
            k.frame.member_profiles = std::sync::Arc::clone(&library);
        }
    }

    pub fn set_custom_profile(&mut self, profile: crate::profile::ProfileDefinition) {
        self.custom_profiles.retain(|old| old.id != profile.id);
        self.custom_profiles.push(profile);
        self.refresh_member_profiles();
    }

    pub fn add_kozijn(&mut self, mut kozijn: Kozijn) {
        if let Some(existing) = self.kozijnen.first() {
            kozijn.frame.member_profiles = existing.frame.member_profiles.clone();
            self.kozijnen.push(kozijn);
        } else {
            self.kozijnen.push(kozijn);
            self.refresh_member_profiles();
        }
    }

    pub fn new(name: &str, number: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            format_version: "1.3".into(),
            project_info: ProjectInfo {
                name: name.to_string(),
                number: number.to_string(),
                client: String::new(),
                address: String::new(),
                series: vec![],
            },
            kozijnen: vec![],
            vliesgevels: vec![],
            custom_profiles: vec![],
            custom_sjablonen: vec![],
            ifc_guid_map: std::collections::HashMap::new(),
            pricing_config: None,
            bcf_topics: vec![],
            quotations: vec![],
            combinations: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
    pub name: String,
    pub number: String,
    pub client: String,
    pub address: String,
    /// Project series / building phases
    #[serde(default)]
    pub series: Vec<Series>,
}

/// A series / building phase for organizing kozijnen
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selecting_a_pvc_frame_changes_material_and_rejects_partial_updates() {
        let mut k = Kozijn::new("Selectie", "K1", 1000.0, 1200.0);
        let profile = ProfileRef {
            id: "veka-softline82".into(),
            name: "VEKA".into(),
        };
        let before = serde_json::to_value(&k).unwrap();
        for (snapshot, material) in [("{}", "unknown"), ("broken", r#""pvc""#)] {
            assert!(k
                .apply_frame_profile(
                    profile.clone(),
                    Some(73.0),
                    Some(82.0),
                    Some(snapshot),
                    Some(material),
                )
                .is_err());
            assert_eq!(serde_json::to_value(&k).unwrap(), before);
        }
        k.apply_frame_profile(
            profile,
            Some(73.0),
            Some(82.0),
            Some("null"),
            Some(r#""pvc""#),
        )
        .unwrap();
        assert_eq!(k.frame.material, Material::Pvc);
        assert_eq!(k.grid.columns[0].size, 854.0);
        let fit = crate::geometry::compute_2d_geometry(&k)
            .cell_rects
            .remove(0)
            .fit
            .unwrap();
        assert_eq!(fit.day.x - fit.glass.x, 20.0);
    }

    #[test]
    fn frame_system_change_updates_inherited_profiles_and_preserves_explicit_parts() {
        let mut k = crate::grid::template_double_turn_tilt_sj(
            1600.,
            1300.,
            &crate::template::default_sjabloon(),
        );
        k.frame.right_profile = Some(ProfileRef {
            id: "eigen-stijl".into(),
            name: "Eigen stijl".into(),
        });
        k.cells[1].sash_profile = Some(ProfileRef {
            id: "eigen-vleugel".into(),
            name: "Eigen vleugel".into(),
        });
        let profile = ProfileRef {
            id: "reynaers-masterline8".into(),
            name: "MasterLine 8".into(),
        };
        k.apply_frame_profile(
            profile.clone(),
            Some(53.),
            Some(77.),
            None,
            Some(r#""aluminum""#),
        )
        .unwrap();
        for p in [
            &k.frame.top_profile,
            &k.frame.sill_profile,
            &k.frame.bottom_profile,
            &k.frame.left_profile,
            &k.grid.columns[1].divider_profile,
            &k.cells[0].sash_profile,
        ] {
            assert_eq!(p.as_ref().unwrap().id, profile.id);
        }
        assert_eq!(k.frame.right_profile.as_ref().unwrap().id, "eigen-stijl");
        assert_eq!(
            k.cells[1].sash_profile.as_ref().unwrap().id,
            "eigen-vleugel"
        );
        assert_eq!(
            (k.cells[0].sash_width, k.cells[0].sash_depth),
            (Some(50.), Some(87.))
        );
        k.cells[0].change_panel_type(PanelType::FixedGlass, &k.frame);
        k.cells[0].change_panel_type(PanelType::TurnTilt, &k.frame);
        assert_eq!(k.cells[0].sash_profile.as_ref().unwrap().id, profile.id);
        assert_eq!(k.cells[0].sash_depth, Some(87.));
    }

    #[test]
    fn inward_window_articles_keep_stock_glass_and_section_dimensions_together() {
        use crate::production::{compute_production_data, MemberType};

        let template = crate::template::get_sjabloon("kunststof-82-veka");
        let mut frame = crate::grid::template_single_turn_tilt_sj(1200., 1500., &template);
        for (id, width, day_offset, glass_width) in [
            ("veka-103341", 84., 124., 992.),
            ("veka-103340", 74., 114., 1012.),
        ] {
            if id == "veka-103340" {
                let profile = ProfileRef { id: id.into(), name: "VEKA 103.340".into() };
                // Sending the surrounding frame's dimensions must not resize the article.
                frame.update_cell_sash_profile(0, profile, 73., 82., None).unwrap();
            }
            let sash = frame.cells[0].sash_profile.as_ref().unwrap();
            assert_eq!(sash.id, id);
            assert_eq!(frame.cells[0].sash_width, Some(width));
            assert_eq!(frame.cells[0].sash_depth, Some(82.));
            let geometry = crate::geometry::compute_2d_geometry(&frame);
            let fit = geometry.cell_rects[0].fit.as_ref().unwrap();
            assert_eq!(fit.day.x, day_offset);
            assert_eq!(fit.system_seals.as_ref().unwrap().sealing_levels, Some(3));
            assert_eq!(fit.glass.width, glass_width);
            let lists = compute_production_data(&frame);
            assert_eq!(lists.glass_list[0].width_mm, glass_width);
            assert_eq!(lists.glass_list[0].height_mm, glass_width + 300.);
            let top = lists.cut_list.iter().find(|p| p.member_type == MemberType::SashTop).unwrap();
            assert!(top.profile_name.contains(&sash.name));
            assert_eq!(top.net_length_mm, fit.day.width); // short points of the mitred rail
            assert_eq!(fit.sash_outer.as_ref().unwrap().width, 1120.);
            assert_eq!(top.gross_length_mm, 1132.); // long points, 2 × 4 mm weld and 4 mm kerf
            assert_eq!(top.miter_left_deg, 45.);
            assert_eq!(top.miter_right_deg, 45.);
        }
        let narrow = frame.cells[0].sash_profile.clone().unwrap();
        frame.change_cell_opening(0, PanelType::Turn, Some(OpeningDirection::Outward)).unwrap();
        assert_ne!(frame.cells[0].sash_profile.as_ref().unwrap().id, narrow.id);
        assert!(frame.update_cell_sash_profile(0, narrow.clone(), 74., 82., None).is_err());
        frame.change_cell_opening(0, PanelType::TurnTilt, Some(OpeningDirection::Right)).unwrap();
        assert_eq!(frame.cells[0].sash_profile.as_ref().unwrap().id, "veka-103341");

        // An older saved template can still refer to its original profile family.
        let mut legacy = template;
        legacy.raamhout_profile = legacy.stijl_profile.clone();
        let legacy_frame = crate::grid::template_single_turn_tilt_sj(1200., 1500., &legacy);
        assert_eq!(legacy_frame.cells[0].sash_profile.as_ref().unwrap().id, "veka-softline82");
        assert_eq!(compute_production_data(&legacy_frame).glass_list[0].width_mm, 992.);
    }

    #[test]
    fn old_system_splits_restore_article_width_without_changing_selected_dividers_or_part_ids() {
        let sj = crate::template::get_sjabloon("kunststof-82-veka");
        let mut k = Kozijn::new_with_sjabloon("Rond", "R1", 1200., 1200., &sj);
        k.frame.shape.shape_type = ShapeType::Round;
        k.add_column(400.);
        k.add_row(500.);
        k.grid.columns[1].divider_profile = Some(ProfileRef::default_divider());
        k.grid.columns[1].divider_width = None;
        k.fit_grid_to_frame();
        let chosen = ProfileRef { id: "test-t73".into(), name: "T73".into() };
        k.update_member_profile("divider_h", Some(0), chosen, Some(73.), Some(82.)).unwrap();
        let ratio = k.grid.columns[0].size / k.grid.columns[1].size;
        let post_id = k.vertical_part_id(0);
        let mut project = Project::new("Test", "");
        project.kozijnen.push(k);
        let loaded: Project = serde_json::from_value(serde_json::to_value(project).unwrap()).unwrap();
        let k = &loaded.kozijnen[0];
        assert_eq!(k.vertical_part_id(0), post_id);
        assert_eq!(k.grid.columns[1].divider_profile.as_ref().unwrap().id, "veka-102310");
        assert_eq!(k.grid.columns[1].divider_width, Some(94.));
        assert_eq!(k.grid.rows[1].divider_profile.as_ref().unwrap().id, "test-t73");
        assert_eq!(k.grid.rows[1].divider_width, Some(73.));
        assert!((k.grid.columns[0].size / k.grid.columns[1].size - ratio).abs() < 1e-9);
        let geometry = crate::geometry::compute_2d_geometry(k);
        let last = &geometry.cell_rects[3].rect;
        assert!((last.x + last.width - (1200. - sj.frame_width)).abs() < 1e-6);
        assert!((last.y + last.height - (1200. - sj.frame_width)).abs() < 1e-6);
        let once = serde_json::to_value(&loaded).unwrap();
        let reloaded: Project = serde_json::from_value(once.clone()).unwrap();
        assert_eq!(once, serde_json::to_value(&reloaded).unwrap());
    }

    #[test]
    fn a_catalogue_t_section_keeps_its_dimensions_and_changes_article_with_the_frame() {
        let veka = crate::template::get_sjabloon("kunststof-82-veka");
        let cs77 = crate::template::get_sjabloon("aluminium-77-reynaers");
        let mut k = crate::grid::template_double_turn_tilt_sj(1200., 1500., &veka);
        let before = serde_json::to_value(&k).unwrap();
        assert!(k.update_member_profile("divider_v", Some(0), cs77.tussenstijl_profile.clone(), None, None).is_err());
        assert!(k.update_member_profile("divider_v", Some(0), veka.tussenstijl_profile.clone(), Some(76.), None).is_err());
        assert_eq!(serde_json::to_value(&k).unwrap(), before);
        k.update_member_profile("divider_v", Some(0), veka.tussenstijl_profile, None, None).unwrap();
        assert_eq!(k.grid.columns[1].divider_depth, Some(82.0));
        k.apply_frame_profile(cs77.stijl_profile, Some(51.0), Some(68.0), None, Some("\"aluminum\"")).unwrap();
        let d = &k.grid.columns[1];
        let a = k.system_divider_article(Some(d), k.divider_face_width(true, 0)).unwrap();
        assert_eq!(d.divider_profile.as_ref().unwrap().id, "reynaers-0083113");
        assert_eq!((a.width, a.depth), (76.0, 68.0));
    }

    #[test]
    fn cs77_functional_sightline_and_cuts_use_the_inward_sash_article() {
        use crate::geometry::compute_2d_geometry;
        use crate::production::{compute_production_data, MemberType};

        let template = crate::template::get_sjabloon("aluminium-77-reynaers");
        let single = crate::grid::template_single_turn_tilt_sj(1200., 1500., &template);
        let cell = &single.cells[0];
        assert_eq!(cell.sash_profile.as_ref().unwrap().id, "reynaers-0083102");
        assert_eq!((cell.sash_width, cell.sash_depth), (Some(68.), Some(77.)));
        let geometry = compute_2d_geometry(&single);
        let fit = geometry.cell_rects[0].fit.as_ref().unwrap();
        assert_eq!((fit.day.x, fit.day.y), (89., 89.)); // 51 frame + 5 joint + 33 visible sash
        assert_eq!((fit.day.width, fit.day.height), (1022., 1322.));
        assert_eq!(fit.sash_outer.as_ref().unwrap().width, 1158.);
        let production = compute_production_data(&single);
        let pane = &production.glass_list[0];
        assert!(pane.production_ready, "{:?}", pane.production_issue);
        assert_eq!((pane.width_mm, pane.height_mm), (1062., 1362.));
        let rail = production.cut_list.iter().find(|cut| cut.member_type == MemberType::SashTop).unwrap();
        assert!(rail.profile_name.contains("008.3102"));
        assert_eq!((rail.net_length_mm, rail.gross_length_mm), (1022., 1162.));
        assert_eq!((rail.miter_left_deg, rail.miter_right_deg), (45., 45.));

        // A manually chosen article still belongs to its documented system,
        // including when the desktop or web editor persisted its snapshot.
        let pvc = crate::template::get_sjabloon("kunststof-82-veka");
        let mut revision = crate::grid::template_single_turn_tilt_sj(1200., 1500., &pvc);
        let narrow = ProfileRef { id: "veka-103340".into(), name: "VEKA 103.340".into() };
        let stored = crate::glazing_fit::catalog_snapshot(&narrow.id, &Material::Pvc).cloned();
        revision.update_cell_sash_profile(0, narrow.clone(), 74., 82., stored).unwrap();
        revision.apply_frame_profile(template.stijl_profile.clone(), Some(51.), Some(68.), None, Some(r#""aluminum""#)).unwrap();
        assert_eq!(revision.cells[0].sash_profile.as_ref().unwrap().id, "reynaers-0083102");
        assert_eq!(compute_production_data(&revision).glass_list[0].width_mm, pane.width_mm);
        assert!(revision.update_cell_sash_profile(0, narrow, 74., 82., None).is_err());

        revision.change_cell_opening(0, PanelType::Turn, Some(OpeningDirection::Outward)).unwrap();
        assert_eq!(revision.cells[0].sash_profile.as_ref().unwrap().id, "reynaers-cs77-standard");
        assert_eq!(revision.cells[0].sash_width, Some(51.));

        let mut double = crate::grid::template_double_turn_tilt_sj(1600., 1300., &template);
        let geometry = compute_2d_geometry(&double);
        let left = geometry.cell_rects[0].fit.as_ref().unwrap().sash_outer.as_ref().unwrap();
        let right = geometry.cell_rects[1].fit.as_ref().unwrap().sash_outer.as_ref().unwrap();
        assert_eq!(right.x - (left.x + left.width), 16.); // 76 mm divider, two 30 mm overlaps
        assert!(compute_production_data(&double).glass_list.iter().all(|pane| pane.production_ready));
        double.grid.columns[1].divider_width = Some(48.);
        double.fit_grid_to_frame();
        let geometry = compute_2d_geometry(&double);
        for rect in &geometry.cell_rects {
            let seals = rect.fit.as_ref().unwrap().system_seals.as_ref().unwrap();
            assert_eq!(seals.sash_overlap_mm, Some(24.));
            assert_eq!(seals.unmet_required_overlap_mm, Some(30.));
        }
        let production = compute_production_data(&double);
        assert!(production.glass_list.iter().all(|pane| !pane.production_ready
            && pane.production_issue.as_deref().unwrap().contains("30 mm")));
    }

    #[test]
    fn older_project_gets_persistent_id_on_next_save() {
        let project = Project::new("Bestaand", "P-1");
        let mut value = serde_json::to_value(&project).unwrap();
        value.as_object_mut().unwrap().remove("id");
        let migrated: Project = serde_json::from_value(value).unwrap();
        assert_ne!(migrated.id, Uuid::nil());
        let saved = serde_json::to_string(&migrated).unwrap();
        let reopened: Project = serde_json::from_str(&saved).unwrap();
        assert_eq!(reopened.id, migrated.id);
    }

    #[test]
    fn custom_member_sections_survive_project_loading_and_profile_edits() {
        let profiles: Vec<crate::profile::ProfileDefinition> =
            serde_json::from_str(include_str!("../../profiles/wood/meranti.json")).unwrap();
        let mut profile = profiles[0].clone();
        profile.id = "eigen-tussenhout".into();
        let mut project = Project::new("Pen en gat", "P1");
        project.set_custom_profile(profile.clone());
        project.add_kozijn(Kozijn::new("Rond", "R1", 1200., 1200.));
        project.add_kozijn(Kozijn::new("Getoogd", "B1", 1200., 1400.));
        profile.depth = 86.;
        project.set_custom_profile(profile);
        let json = serde_json::to_string(&project).unwrap();
        assert!(!json.contains("memberProfiles"));
        let opened: Project = serde_json::from_str(&json).unwrap();
        assert_eq!(
            opened.kozijnen[0].frame.member_profiles["eigen-tussenhout"].depth,
            86.
        );
        assert!(std::sync::Arc::ptr_eq(
            &opened.kozijnen[0].frame.member_profiles,
            &opened.kozijnen[1].frame.member_profiles
        ));
    }

    #[test]
    fn placing_a_transom_in_the_arch_preserves_existing_heights_on_reopen() {
        use crate::geometry::compute_2d_geometry;
        let mut k = Kozijn::new("Getoogd", "G1", 1200.0, 1500.0);
        k.frame.shape.shape_type = ShapeType::Arched;
        k.frame.shape.arch_height = Some(300.0);
        k.add_row(700.0);
        let original_y = compute_2d_geometry(&k).h_dividers[0].y;
        assert!(!k.grid.rows_from_day);
        k.add_divider_in_elevation(true, 185.0);
        k.add_divider_in_elevation(false, 373.5);
        assert!(k.grid.rows_from_day);
        let saved = serde_json::to_string(&k).unwrap();
        let reopened: Kozijn = serde_json::from_str(&saved).unwrap();
        let g = compute_2d_geometry(&reopened);
        assert_eq!(g.h_dividers.len(), 2);
        assert!((g.h_dividers[0].y + 33.5 - 185.0).abs() < 1e-6);
        assert!((g.h_dividers[1].y - original_y).abs() < 1e-6);
        assert!((g.v_dividers[0].x - 340.0).abs() < 1e-6);
        let head = g
            .curved_divider_timber
            .iter()
            .find(|p| !p.vertical && p.source_index == 0)
            .unwrap();
        assert_eq!(
            (head.start.radius_mm, head.end.radius_mm),
            (Some(683.0), Some(683.0))
        );
        k.add_divider_in_elevation(true, 187.0); // Already inside the transom
        assert_eq!(k.grid.rows.len(), 3);
    }

    #[test]
    fn changing_opening_type_keeps_window_stock_until_its_function_changes() {
        let mut k = Kozijn::new("Getoogd", "R1", 1200.0, 1500.0);
        let cell = &mut k.cells[0];
        cell.change_panel_type(PanelType::TopHung, &k.frame);
        assert_eq!(cell.sash_profile.as_ref().unwrap().id, "raamhout-69x90");
        cell.sash_width = Some(78.0);
        cell.change_panel_type(PanelType::Turn, &k.frame);
        assert_eq!(cell.sash_width, Some(78.0));
        cell.change_panel_type(PanelType::Door, &k.frame);
        assert_eq!(
            cell.sash_profile.as_ref().unwrap().id,
            "deur-meranti-67x114"
        );
        cell.change_panel_type(PanelType::FixedGlass, &k.frame);
        assert!(cell.sash_profile.is_none() && cell.sash_width.is_none());
    }

    #[test]
    fn inserting_column_preserves_existing_part_and_cell_identity() {
        let mut k = Kozijn::new("Test", "K-1", 1600.0, 1400.0);
        k.add_column(500.0);
        k.cells[1].glazing.glass_type = "Rechter ruit".into();
        let old_divider = k.vertical_part_id(0);
        let old_left = k.glass_part_id(0);
        let old_right = k.glass_part_id(1);
        k.add_column(250.0);
        assert_eq!(k.vertical_part_id(1), old_divider);
        assert_eq!(k.glass_part_id(0), old_left);
        assert_eq!(k.glass_part_id(2), old_right);
        assert_ne!(k.glass_part_id(1), old_right);
        assert_eq!(k.cells[2].glazing.glass_type, "Rechter ruit");
        let reopened: Kozijn = serde_json::from_str(&serde_json::to_string(&k).unwrap()).unwrap();
        assert_eq!(reopened.vertical_part_id(1), old_divider);
        assert_eq!(reopened.glass_part_id(2), old_right);
    }

    #[test]
    fn inserting_row_preserves_existing_part_and_cell_identity() {
        let mut k = Kozijn::new("Test", "K-2", 1200.0, 1600.0);
        k.add_row(500.0);
        k.cells[1].glazing.glass_type = "Onderste ruit".into();
        let old_divider = k.horizontal_part_id(0);
        let old_bottom = k.glass_part_id(1);
        k.add_row(250.0);
        assert_eq!(k.horizontal_part_id(1), old_divider);
        assert_eq!(k.glass_part_id(2), old_bottom);
        assert_eq!(k.cells[2].glazing.glass_type, "Onderste ruit");
    }

    #[test]
    fn legacy_ordinal_ids_are_retained_when_grid_is_split() {
        let mut k = Kozijn::new("Oud", "K-3", 1600.0, 1400.0);
        k.add_column(500.0);
        let mut value = serde_json::to_value(&k).unwrap();
        value.as_object_mut().unwrap().remove("partIds");
        let mut legacy: Kozijn = serde_json::from_value(value).unwrap();
        let old_divider = legacy.vertical_part_id(0);
        let old_right = legacy.glass_part_id(1);
        assert_eq!(old_divider, legacy.legacy_part_id(5));
        assert_eq!(old_right, legacy.legacy_part_id(7));
        legacy.add_column(250.0);
        assert_eq!(legacy.vertical_part_id(1), old_divider);
        assert_eq!(legacy.glass_part_id(2), old_right);
    }

    #[test]
    fn multirow_split_preserves_each_existing_cell_and_duplicate_rekeys() {
        let mut k = Kozijn::new("Raster", "K-4", 1600.0, 1600.0);
        k.add_column(500.0);
        k.add_row(500.0);
        k.cells[3].glazing.glass_type = "Rechtsonder".into();
        let before: Vec<_> = (0..4).map(|i| k.glass_part_id(i)).collect();
        k.add_column(250.0);
        let retained: Vec<_> = [0, 2, 3, 5].iter().map(|&i| k.glass_part_id(i)).collect();
        assert_eq!(retained, before);
        assert_eq!(k.cells[5].glazing.glass_type, "Rechtsonder");
        let copy = k.duplicate("K-5");
        assert_ne!(copy.id, k.id);
        assert_ne!(copy.glass_part_id(5), k.glass_part_id(5));
    }

    #[test]
    fn add_column_reserves_divider_space() {
        let mut k = Kozijn::new("Test", "K01", 1200.0, 1500.0);
        let inner_w = k.inner_width(); // 1066
        let fw = k.frame.frame_width; // 67 (= divider width)
        k.add_column(400.0);

        assert_eq!(k.grid.columns.len(), 2);
        assert_eq!(k.cells.len(), 2);
        let sum: f64 = k.grid.columns.iter().map(|c| c.size).sum();
        // Σ kolommen + tussenstijl = binnenmaat: the dimension chain closes.
        assert!(
            (sum + fw - inner_w).abs() < 1e-6,
            "sum {} + fw {} != inner {}",
            sum,
            fw,
            inner_w
        );
        // The divider space is taken proportionally: the 400:666 ratio holds.
        let ratio = k.grid.columns[0].size / k.grid.columns[1].size;
        assert!((ratio - 400.0 / 666.0).abs() < 1e-6);
        // Both columns stay positive.
        assert!(k.grid.columns.iter().all(|c| c.size > 0.0));
    }

    fn grid_closes(k: &Kozijn) {
        let fw = k.frame.frame_width;
        let cols: f64 = k.grid.columns.iter().map(|c| c.size).sum();
        let rows: f64 = k.grid.rows.iter().map(|r| r.size).sum();
        let cols_total = cols + fw * (k.grid.columns.len() - 1) as f64;
        let rows_total = rows + fw * (k.grid.rows.len() - 1) as f64;
        assert!(
            (cols_total - k.inner_width()).abs() < 1e-6,
            "cols {} != inner {}",
            cols_total,
            k.inner_width()
        );
        assert!(
            (rows_total - k.inner_height()).abs() < 1e-6,
            "rows {} != inner {}",
            rows_total,
            k.inner_height()
        );
    }

    #[test]
    fn fit_grid_keeps_chain_closed_after_resize_and_profile_change() {
        let mut k = Kozijn::new("Test", "K01", 1200.0, 1500.0);
        k.add_column(500.0);
        k.add_row(600.0);
        grid_closes(&k);
        // Maatwijziging: 1200 -> 1210 was 0,63 mm off with plain new/old scaling.
        k.frame.outer_width = 1210.0;
        k.fit_grid_to_frame();
        grid_closes(&k);
        k.frame.outer_width = 1500.0;
        k.frame.outer_height = 1700.0;
        k.fit_grid_to_frame();
        grid_closes(&k);
        // Profielbreedte 67 -> 78 verandert ook de dividerbreedte.
        k.frame.frame_width = 78.0;
        k.fit_grid_to_frame();
        grid_closes(&k);
        assert!(k.grid.columns.iter().all(|c| c.size > 0.0));
        assert!(k.grid.rows.iter().all(|r| r.size > 0.0));
    }

    #[test]
    fn typed_vak_size_stays_exact_and_neighbours_absorb() {
        let mut k = Kozijn::new("Test", "K01", 1600.0, 1500.0);
        k.add_column(500.0);
        k.add_column(900.0);
        let before: Vec<f64> = k.grid.columns.iter().map(|c| c.size).collect();
        let rows: Vec<f64> = k.grid.rows.iter().map(|r| r.size).collect();
        let mut wanted = before.clone();
        wanted[0] = 300.0;
        k.set_grid_sizes_fitted(&wanted, &rows);
        assert!(
            (k.grid.columns[0].size - 300.0).abs() < 1e-9,
            "typed value must stay exact"
        );
        grid_closes(&k);
        // The untouched vakken kept their mutual ratio.
        let r_before = before[1] / before[2];
        let r_after = k.grid.columns[1].size / k.grid.columns[2].size;
        assert!((r_before - r_after).abs() < 1e-9);
    }

    #[test]
    fn add_row_reserves_divider_space() {
        let mut k = Kozijn::new("Test", "K01", 1200.0, 1500.0);
        let inner_h = k.inner_height(); // 1366
        let fw = k.frame.frame_width;
        k.add_row(500.0);

        assert_eq!(k.grid.rows.len(), 2);
        let sum: f64 = k.grid.rows.iter().map(|r| r.size).sum();
        assert!(
            (sum + fw - inner_h).abs() < 1e-6,
            "sum {} + fw {} != inner {}",
            sum,
            fw,
            inner_h
        );
        let ratio = k.grid.rows[0].size / k.grid.rows[1].size;
        assert!((ratio - 500.0 / 866.0).abs() < 1e-6);
    }

    #[test]
    fn add_column_on_degenerate_vak_keeps_positive_sizes() {
        // A vak narrower than the divider cannot reserve the space; the split
        // still happens but sizes stay positive (no negative column widths).
        let mut k = Kozijn::new("Test", "K02", 190.0, 400.0);
        // inner width = 190 − 134 = 56 < divider width 67
        assert!(k.inner_width() < k.frame.frame_width);
        k.add_column(28.0);
        assert_eq!(k.grid.columns.len(), 2);
        assert!(k.grid.columns.iter().all(|c| c.size > 0.0));
    }
}
