//! IFC4 (STEP Physical File) export for kozijnen.
//!
//! Generates IFC4 files with IfcWindow/IfcDoor entities, geometry
//! (IfcExtrudedAreaSolid), and property sets including ILS Houten
//! Kozijnen v2.0 compliance.
//!
//! Writes IFC-SPF (.ifc) text format directly — no external crate needed.

use std::fmt::Write as FmtWrite;
use std::io::Write;

use crate::kozijn::{Kozijn, Material, PanelType, OpeningDirection, WoodType};

/// Generate an IFC4 file from a kozijn definition.
pub fn generate_ifc(kozijn: &Kozijn, output_path: &str) -> Result<(), String> {
    let mut ifc = IfcWriter::new();

    let frame = &kozijn.frame;
    let cells = &kozijn.cells;

    let has_door = cells.iter().any(|c| c.panel_type == PanelType::Door);
    let ifc_class = if has_door { "IFCDOOR" } else { "IFCWINDOW" };

    // Dimensions in meters
    let width_m = frame.outer_width / 1000.0;
    let height_m = frame.outer_height / 1000.0;
    let depth_m = frame.frame_depth / 1000.0;
    let fw_m = frame.frame_width / 1000.0;

    // ── Spatial hierarchy ──────────────────────────────────────

    let owner_history = ifc.add_owner_history();
    let units = ifc.add_si_units();
    let guid_proj = ifc.guid();
    let project = ifc.add_entity(&format!(
        "IFCPROJECT('{}',{},'{}',$,$,$,$,$,{})",
        guid_proj, owner_history, "Open Frame Studio Export", units
    ));

    let guid_site = ifc.guid();
    let site = ifc.add_entity(&format!(
        "IFCSITE('{}',{},'Bouwlocatie',$,$,$,$,$,.ELEMENT.,$,$,$,$,$)",
        guid_site, owner_history
    ));
    let guid_bldg = ifc.guid();
    let building = ifc.add_entity(&format!(
        "IFCBUILDING('{}',{},'Gebouw',$,$,$,$,$,.ELEMENT.,$,$,$)",
        guid_bldg, owner_history
    ));
    let guid_stor = ifc.guid();
    let storey = ifc.add_entity(&format!(
        "IFCBUILDINGSTOREY('{}',{},'Begane grond',$,$,$,$,$,.ELEMENT.,0.0)",
        guid_stor, owner_history
    ));

    // Aggregation
    ifc.add_rel_aggregates(&owner_history, "ProjectSite", &project, &[site.clone()]);
    ifc.add_rel_aggregates(&owner_history, "SiteBuilding", &site, &[building.clone()]);
    ifc.add_rel_aggregates(&owner_history, "BuildingStorey", &building, &[storey.clone()]);

    // ── Geometry context ───────────────────────────────────────

    let origin_3d = ifc.add_entity("IFCCARTESIANPOINT((0.0,0.0,0.0))");
    let axis_z = ifc.add_entity("IFCDIRECTION((0.0,0.0,1.0))");
    let axis_x = ifc.add_entity("IFCDIRECTION((1.0,0.0,0.0))");
    let placement_3d = ifc.add_entity(&format!(
        "IFCAXIS2PLACEMENT3D({},{},{})",
        origin_3d, axis_z, axis_x
    ));

    let true_north = ifc.add_entity("IFCDIRECTION((0.0,1.0,0.0))");
    let context = ifc.add_entity(&format!(
        "IFCGEOMETRICREPRESENTATIONCONTEXT($,'Model',3,1.0E-5,{},{})",
        placement_3d, true_north
    ));

    let body_context = ifc.add_entity(&format!(
        "IFCGEOMETRICREPRESENTATIONSUBCONTEXT('Body','Model',*,*,*,*,{},{},.MODEL_VIEW.,$)",
        context, "$"
    ));

    // ── Frame geometry ─────────────────────────────────────────

    // Outer profile polyline
    let outer_pts: Vec<String> = [
        (0.0, 0.0),
        (width_m, 0.0),
        (width_m, height_m),
        (0.0, height_m),
        (0.0, 0.0),
    ]
    .iter()
    .map(|(x, y)| ifc.add_entity(&format!("IFCCARTESIANPOINT(({:.6},{:.6}))", x, y)))
    .collect();
    let outer_polyline = ifc.add_entity(&format!(
        "IFCPOLYLINE(({}))",
        outer_pts.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(",")
    ));

    // Inner void polyline
    let inner_pts: Vec<String> = [
        (fw_m, fw_m),
        (width_m - fw_m, fw_m),
        (width_m - fw_m, height_m - fw_m),
        (fw_m, height_m - fw_m),
        (fw_m, fw_m),
    ]
    .iter()
    .map(|(x, y)| ifc.add_entity(&format!("IFCCARTESIANPOINT(({:.6},{:.6}))", x, y)))
    .collect();
    let inner_polyline = ifc.add_entity(&format!(
        "IFCPOLYLINE(({}))",
        inner_pts.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(",")
    ));

    let profile = ifc.add_entity(&format!(
        "IFCARBITRARYPROFILEDEFWITHVOIDS(.AREA.,'FrameProfile',{},({}))",
        outer_polyline, inner_polyline
    ));

    let extrusion_placement = ifc.add_entity(&format!(
        "IFCAXIS2PLACEMENT3D({},{},{})",
        origin_3d, axis_z, axis_x
    ));
    let extrusion = ifc.add_entity(&format!(
        "IFCEXTRUDEDAREASOLID({},{},{},{:.6})",
        profile, extrusion_placement, axis_z, depth_m
    ));

    // Glass panel geometry
    let glass_pts: Vec<String> = [
        (fw_m, fw_m),
        (width_m - fw_m, fw_m),
        (width_m - fw_m, height_m - fw_m),
        (fw_m, height_m - fw_m),
        (fw_m, fw_m),
    ]
    .iter()
    .map(|(x, y)| ifc.add_entity(&format!("IFCCARTESIANPOINT(({:.6},{:.6}))", x, y)))
    .collect();
    let glass_polyline = ifc.add_entity(&format!(
        "IFCPOLYLINE(({}))",
        glass_pts.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(",")
    ));
    let glass_profile = ifc.add_entity(&format!(
        "IFCARBITRARYCLOSEDPROFILEDEF(.AREA.,'GlassProfile',{})",
        glass_polyline
    ));

    let glass_thickness = 0.024; // 24mm
    let glass_offset = (depth_m - glass_thickness) / 2.0;
    let glass_origin = ifc.add_entity(&format!(
        "IFCCARTESIANPOINT((0.0,0.0,{:.6}))",
        glass_offset
    ));
    let glass_placement = ifc.add_entity(&format!(
        "IFCAXIS2PLACEMENT3D({},{},{})",
        glass_origin, axis_z, axis_x
    ));
    let glass_extrusion = ifc.add_entity(&format!(
        "IFCEXTRUDEDAREASOLID({},{},{},{:.6})",
        glass_profile, glass_placement, axis_z, glass_thickness
    ));

    // Shape representation
    let shape_rep = ifc.add_entity(&format!(
        "IFCSHAPEREPRESENTATION({},'Body','SweptSolid',({},{}))",
        body_context, extrusion, glass_extrusion
    ));
    let product_shape = ifc.add_entity(&format!(
        "IFCPRODUCTDEFINITIONSHAPE($,$,({}))",
        shape_rep
    ));

    // Placement
    let placement_axis = ifc.add_entity(&format!(
        "IFCAXIS2PLACEMENT3D({},{},{})",
        origin_3d, axis_z, axis_x
    ));
    let local_placement = ifc.add_entity(&format!(
        "IFCLOCALPLACEMENT($,{})",
        placement_axis
    ));

    // ── Element ────────────────────────────────────────────────

    let guid_elem = ifc.guid();
    let element = ifc.add_entity(&format!(
        "{}('{}',{},'{}','{}','{}',$,{},{},{:.6},{:.6})",
        ifc_class, guid_elem, owner_history,
        kozijn.name, kozijn.mark, "",
        local_placement, product_shape, height_m, width_m,
    ));

    // Assign to storey
    let guid_rel = ifc.guid();
    ifc.add_entity(&format!(
        "IFCRELCONTAINEDINSPATIALSTRUCTURE('{}',{},$,$,({}),{})",
        guid_rel, owner_history, element, storey
    ));

    // ── Property sets ──────────────────────────────────────────

    add_standard_psets(&mut ifc, &owner_history, &element, kozijn);
    add_ils_psets(&mut ifc, &owner_history, &element, kozijn);

    // Write file
    ifc.write_to_file(output_path, project, context)
}

// ── Standard property sets ─────────────────────────────────────

fn add_standard_psets(ifc: &mut IfcWriter, oh: &str, element: &str, kozijn: &Kozijn) {
    let frame = &kozijn.frame;
    let cells = &kozijn.cells;
    let is_door = cells.iter().any(|c| c.panel_type == PanelType::Door);
    let pset_name = if is_door { "Pset_DoorCommon" } else { "Pset_WindowCommon" };

    let ow_mm = frame.outer_width;
    let oh_mm = frame.outer_height;
    let fw_mm = frame.frame_width;
    let total_area = (ow_mm / 1000.0) * (oh_mm / 1000.0);
    let inner_w = ow_mm / 1000.0 - 2.0 * fw_mm / 1000.0;
    let inner_h = oh_mm / 1000.0 - 2.0 * fw_mm / 1000.0;
    let glass_area = inner_w * inner_h;

    let mut props = Vec::new();

    // Reference
    let p = ifc.add_entity(&format!(
        "IFCPROPERTYSINGLEVALUE('Reference',$,IFCLABEL('{}'),$)",
        kozijn.mark
    ));
    props.push(p);

    // IsExternal
    let p = ifc.add_entity(
        "IFCPROPERTYSINGLEVALUE('IsExternal',$,IFCBOOLEAN(.T.),$)"
    );
    props.push(p);

    // GlazingAreaFraction
    if total_area > 0.0 {
        let frac = glass_area / total_area;
        let p = ifc.add_entity(&format!(
            "IFCPROPERTYSINGLEVALUE('GlazingAreaFraction',$,IFCPOSITIVERATIOMEASURE({:.4}),$)",
            frac
        ));
        props.push(p);
    }

    // ThermalTransmittance from first cell
    if let Some(cell) = cells.first() {
        if cell.glazing.ug_value > 0.0 {
            let p = ifc.add_entity(&format!(
                "IFCPROPERTYSINGLEVALUE('ThermalTransmittance',$,IFCTHERMALTRANSMITTANCEMEASURE({:.2}),$)",
                cell.glazing.ug_value
            ));
            props.push(p);
        }
    }

    ifc.add_property_set(oh, pset_name, element, &props);

    // Pset_OFS_Kozijn
    let mat_str = material_label(&frame.material);
    let ofs_props: Vec<String> = vec![
        ifc.add_entity(&format!(
            "IFCPROPERTYSINGLEVALUE('Material',$,IFCLABEL('{}'),$)", mat_str
        )),
        ifc.add_entity(&format!(
            "IFCPROPERTYSINGLEVALUE('ColorInside',$,IFCLABEL('{}'),$)", frame.color_inside
        )),
        ifc.add_entity(&format!(
            "IFCPROPERTYSINGLEVALUE('ColorOutside',$,IFCLABEL('{}'),$)", frame.color_outside
        )),
        ifc.add_entity(&format!(
            "IFCPROPERTYSINGLEVALUE('FrameWidth_mm',$,IFCLENGTHMEASURE({:.1}),$)", frame.frame_width
        )),
        ifc.add_entity(&format!(
            "IFCPROPERTYSINGLEVALUE('FrameDepth_mm',$,IFCLENGTHMEASURE({:.1}),$)", frame.frame_depth
        )),
        ifc.add_entity(&format!(
            "IFCPROPERTYSINGLEVALUE('CellCount',$,IFCINTEGER({}),$)", cells.len()
        )),
    ];
    ifc.add_property_set(oh, "Pset_OFS_Kozijn", element, &ofs_props);
}

// ── ILS Houten Kozijnen v2.0 property sets ─────────────────────

fn add_ils_psets(ifc: &mut IfcWriter, oh: &str, element: &str, kozijn: &Kozijn) {
    let frame = &kozijn.frame;
    let grid = &kozijn.grid;
    let cells = &kozijn.cells;
    let ow = frame.outer_width;
    let oh_mm = frame.outer_height;
    let fw = frame.frame_width;
    let fd = frame.frame_depth;

    // ILS_KozijnAlgemeen
    {
        let kozijn_type = determine_kozijn_type(cells);
        let props = vec![
            ifc.add_prop_label("Merkteken", &kozijn.mark),
            ifc.add_prop_label("Naam", &kozijn.name),
            ifc.add_prop_label("Type", &kozijn_type),
            ifc.add_entity("IFCPROPERTYSINGLEVALUE('IsUitwendig',$,IFCBOOLEAN(.T.),$)"),
            ifc.add_prop_label("Leverancier", &frame.profile.name),
            ifc.add_prop_label("ProfielSysteem", &frame.profile.id),
        ];
        ifc.add_property_set(oh, "ILS_KozijnAlgemeen", element, &props);
    }

    // ILS_KozijnAfmetingen
    {
        let mut props = vec![
            ifc.add_prop_real("BuitenwerksBreedte_mm", ow),
            ifc.add_prop_real("BuitenwerksHoogte_mm", oh_mm),
            ifc.add_prop_real("DagmaatBreedte_mm", ow - 4.0),
            ifc.add_prop_real("DagmaatHoogte_mm", oh_mm - 2.0),
            ifc.add_prop_real("KozijnprofielBreedte_mm", fw),
            ifc.add_prop_real("KozijnprofielDiepte_mm", fd),
            ifc.add_prop_int("AantalKolommen", grid.columns.len() as i64),
            ifc.add_prop_int("AantalRijen", grid.rows.len() as i64),
            ifc.add_prop_real("BinnenwerksBreedte_mm", ow - 2.0 * fw),
            ifc.add_prop_real("BinnenwerksHoogte_mm", oh_mm - 2.0 * fw),
        ];
        for (i, col) in grid.columns.iter().enumerate() {
            props.push(ifc.add_prop_real(&format!("KolomBreedte_{}_mm", i + 1), col.size));
        }
        for (i, row) in grid.rows.iter().enumerate() {
            props.push(ifc.add_prop_real(&format!("RijHoogte_{}_mm", i + 1), row.size));
        }
        ifc.add_property_set(oh, "ILS_KozijnAfmetingen", element, &props);
    }

    // ILS_KozijnMateriaal
    {
        let mat_str = ils_material_label(&frame.material);
        let wood_type = get_wood_type(&frame.material);
        let durability = get_durability_class(&frame.material);

        let mut props = vec![
            ifc.add_prop_label("Materiaal", &mat_str),
            ifc.add_prop_label("KleurBinnenzijde", &frame.color_inside),
            ifc.add_prop_label("KleurBuitenzijde", &frame.color_outside),
            ifc.add_prop_label("Afwerking", "dekkend gelakt"),
            ifc.add_prop_label("Houtsoort", &wood_type),
            ifc.add_prop_label("Duurzaamheidsklasse", &durability),
        ];
        if frame.sill_profile.is_some() {
            let sill_name = frame
                .sill_profile
                .as_ref()
                .map(|s| s.name.as_str())
                .unwrap_or("");
            props.push(ifc.add_prop_label("DorpelType", sill_name));
        }
        ifc.add_property_set(oh, "ILS_KozijnMateriaal", element, &props);
    }

    // ILS_KozijnBeglazing
    {
        let mut props = Vec::new();
        let mut glass_types = std::collections::BTreeSet::new();
        let mut ug_values = Vec::new();

        for (i, cell) in cells.iter().enumerate() {
            glass_types.insert(cell.glazing.glass_type.clone());
            ug_values.push(cell.glazing.ug_value);

            props.push(ifc.add_prop_label(
                &format!("Cel_{}_GlasType", i + 1),
                &cell.glazing.glass_type,
            ));
            props.push(ifc.add_prop_real(
                &format!("Cel_{}_GlasDikte_mm", i + 1),
                cell.glazing.thickness_mm,
            ));
            props.push(ifc.add_prop_real(
                &format!("Cel_{}_UgWaarde", i + 1),
                cell.glazing.ug_value,
            ));
        }

        let types_str: Vec<&str> = glass_types.iter().map(|s| s.as_str()).collect();
        props.push(ifc.add_prop_label("GlasTypen", &types_str.join(", ")));

        if !ug_values.is_empty() {
            let avg = ug_values.iter().sum::<f64>() / ug_values.len() as f64;
            let max = ug_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            props.push(ifc.add_prop_real("UgWaarde_gemiddeld", avg));
            props.push(ifc.add_prop_real("UgWaarde_max", max));
        }

        ifc.add_property_set(oh, "ILS_KozijnBeglazing", element, &props);
    }

    // ILS_KozijnOnderdelen
    {
        let mut props = vec![ifc.add_prop_int("AantalCellen", cells.len() as i64)];

        for (i, cell) in cells.iter().enumerate() {
            let ils_type = ils_panel_type(cell.panel_type);
            props.push(ifc.add_prop_label(&format!("Cel_{}_Type", i + 1), ils_type));

            if let Some(dir) = cell.opening_direction {
                let ils_dir = ils_opening_direction(dir);
                props.push(ifc.add_prop_label(
                    &format!("Cel_{}_Draairichting", i + 1),
                    ils_dir,
                ));
            }

            let has_sash = cell.panel_type.is_operable();
            let val = if has_sash { ".T." } else { ".F." };
            props.push(ifc.add_entity(&format!(
                "IFCPROPERTYSINGLEVALUE('Cel_{}_HeeftVleugel',$,IFCBOOLEAN({}),$)",
                i + 1,
                val
            )));
        }

        ifc.add_property_set(oh, "ILS_KozijnOnderdelen", element, &props);
    }

    // ILS_KozijnHangSluitwerk
    {
        let mut props = Vec::new();

        for (i, cell) in cells.iter().enumerate() {
            if let Some(ref hw) = cell.hardware_set {
                props.push(ifc.add_prop_label(
                    &format!("Cel_{}_Beveiligingsklasse", i + 1),
                    &format!("{:?}", hw.security_class),
                ));
                if let Some(ref hinges) = hw.hinges {
                    props.push(ifc.add_prop_label(
                        &format!("Cel_{}_ScharnierType", i + 1),
                        &format!("{:?}", hinges.hinge_type),
                    ));
                    props.push(ifc.add_prop_int(
                        &format!("Cel_{}_ScharnierAantal", i + 1),
                        hinges.count as i64,
                    ));
                    props.push(ifc.add_prop_label(
                        &format!("Cel_{}_ScharnierZijde", i + 1),
                        &format!("{:?}", hinges.side),
                    ));
                    props.push(ifc.add_prop_real(
                        &format!("Cel_{}_ScharnierDraagkracht_kg", i + 1),
                        hinges.load_capacity_kg,
                    ));
                }
                if let Some(ref handle) = hw.handle {
                    props.push(ifc.add_prop_label(
                        &format!("Cel_{}_GreepType", i + 1),
                        &format!("{:?}", handle.handle_type),
                    ));
                    props.push(ifc.add_prop_label(
                        &format!("Cel_{}_GreepZijde", i + 1),
                        &format!("{:?}", handle.side),
                    ));
                    props.push(ifc.add_prop_int(
                        &format!("Cel_{}_GreepHoogte_mm", i + 1),
                        handle.height_mm as i64,
                    ));
                }
                if let Some(ref locking) = hw.locking {
                    props.push(ifc.add_prop_label(
                        &format!("Cel_{}_SlotType", i + 1),
                        &format!("{:?}", locking.lock_type),
                    ));
                    props.push(ifc.add_prop_int(
                        &format!("Cel_{}_Sluitpunten", i + 1),
                        locking.locking_points as i64,
                    ));
                    props.push(ifc.add_prop_label(
                        &format!("Cel_{}_NokType", i + 1),
                        &format!("{:?}", locking.cam_type),
                    ));
                }
                if let Some(ref vent) = hw.ventilation {
                    props.push(ifc.add_prop_label(
                        &format!("Cel_{}_VentilatieType", i + 1),
                        &format!("{:?}", vent.vent_type),
                    ));
                    props.push(ifc.add_prop_real(
                        &format!("Cel_{}_VentilatieCapaciteit_dm3s", i + 1),
                        vent.capacity_dm3s,
                    ));
                }
                if let Some(ref closer) = hw.closer {
                    props.push(ifc.add_prop_label(
                        &format!("Cel_{}_DrangType", i + 1),
                        &format!("{:?}", closer.closer_type),
                    ));
                    props.push(ifc.add_prop_int(
                        &format!("Cel_{}_DrangKlasse", i + 1),
                        closer.force_class as i64,
                    ));
                }
            } else {
                // Legacy fallback
                match cell.panel_type {
                    PanelType::TurnTilt | PanelType::Turn | PanelType::Tilt => {
                        props.push(ifc.add_prop_label(
                            &format!("Cel_{}_Beslag", i + 1),
                            "draai-kiep beslag",
                        ));
                        props.push(ifc.add_prop_label(
                            &format!("Cel_{}_Scharnieren", i + 1),
                            "verborgen",
                        ));
                        props.push(ifc.add_prop_label(
                            &format!("Cel_{}_GreepType", i + 1),
                            "kruk",
                        ));
                        let points = calc_locking_points(kozijn, i);
                        props.push(ifc.add_prop_int(
                            &format!("Cel_{}_Sluitpunten", i + 1),
                            points,
                        ));
                    }
                    PanelType::Door => {
                        props.push(ifc.add_prop_label(
                            &format!("Cel_{}_Beslag", i + 1),
                            "deurbeslag",
                        ));
                        props.push(ifc.add_prop_label(
                            &format!("Cel_{}_Scharnieren", i + 1),
                            "opleg 3-delig",
                        ));
                        props.push(ifc.add_prop_label(
                            &format!("Cel_{}_GreepType", i + 1),
                            "kruk-kruk",
                        ));
                        props.push(ifc.add_prop_int(
                            &format!("Cel_{}_Sluitpunten", i + 1),
                            3,
                        ));
                        props.push(ifc.add_prop_label(
                            &format!("Cel_{}_SlotType", i + 1),
                            "meerpuntssluiting",
                        ));
                    }
                    PanelType::Sliding => {
                        props.push(ifc.add_prop_label(
                            &format!("Cel_{}_Beslag", i + 1),
                            "schuifbeslag",
                        ));
                        props.push(ifc.add_prop_label(
                            &format!("Cel_{}_GreepType", i + 1),
                            "inlaat greep",
                        ));
                    }
                    _ => {}
                }
            }
        }

        if !props.is_empty() {
            ifc.add_property_set(oh, "ILS_KozijnHangSluitwerk", element, &props);
        }
    }

    // ILS_KozijnThermisch
    {
        // We don't have ufValue on ProfileRef, use default
        let uf = 1.8;

        let ug_values: Vec<f64> = cells.iter().map(|c| c.glazing.ug_value).collect();
        let ug_avg = if ug_values.is_empty() {
            1.0
        } else {
            ug_values.iter().sum::<f64>() / ug_values.len() as f64
        };

        let frame_area =
            2.0 * (ow * fw + oh_mm * fw - 4.0 * fw * fw) / 1e6;
        let glass_area =
            (ow - 2.0 * fw) * (oh_mm - 2.0 * fw) / 1e6;
        let total_area = ow * oh_mm / 1e6;
        let psi_g = 0.06;
        let glass_perimeter =
            2.0 * ((ow - 2.0 * fw) + (oh_mm - 2.0 * fw)) / 1000.0;

        let uw = if total_area > 0.0 {
            (uf * frame_area + ug_avg * glass_area + psi_g * glass_perimeter) / total_area
        } else {
            1.5
        };

        let glass_frac = if total_area > 0.0 {
            glass_area / total_area
        } else {
            0.0
        };

        let props = vec![
            ifc.add_prop_real("Uf_kozijnprofiel", uf),
            ifc.add_prop_real("Ug_glas_gemiddeld", ug_avg),
            ifc.add_prop_real("Uw_kozijn_berekend", (uw * 100.0).round() / 100.0),
            ifc.add_prop_real("Psi_glasrand", psi_g),
            ifc.add_prop_real("GlasOppervlak_m2", (glass_area * 1000.0).round() / 1000.0),
            ifc.add_prop_real("KozijnOppervlak_m2", (frame_area * 1000.0).round() / 1000.0),
            ifc.add_prop_real("TotaalOppervlak_m2", (total_area * 1000.0).round() / 1000.0),
            ifc.add_prop_real("GlasFractie", (glass_frac * 1000.0).round() / 1000.0),
        ];
        ifc.add_property_set(oh, "ILS_KozijnThermisch", element, &props);
    }
}

// ── Helper functions ───────────────────────────────────────────

fn determine_kozijn_type(cells: &[crate::kozijn::Cell]) -> String {
    let types: std::collections::HashSet<PanelType> =
        cells.iter().map(|c| c.panel_type).collect();

    if types.contains(&PanelType::Door) {
        if types.contains(&PanelType::FixedGlass) || types.contains(&PanelType::TurnTilt) {
            return "deurkozijn met bovenlicht".into();
        }
        return "deurkozijn".into();
    }

    if types.contains(&PanelType::Sliding) {
        return "schuifpui".into();
    }

    if types.contains(&PanelType::TurnTilt)
        || types.contains(&PanelType::Turn)
        || types.contains(&PanelType::Tilt)
    {
        if types.contains(&PanelType::FixedGlass) {
            return "raamkozijn met vast glas".into();
        }
        if cells.len() > 1 {
            return "raamkozijn meerdelig".into();
        }
        return "raamkozijn".into();
    }

    if cells.iter().all(|c| c.panel_type == PanelType::FixedGlass) {
        return "vast kozijn".into();
    }

    if types.contains(&PanelType::Panel) {
        return "paneelkozijn".into();
    }

    "kozijn".into()
}

fn ils_panel_type(pt: PanelType) -> &'static str {
    match pt {
        PanelType::FixedGlass => "vast",
        PanelType::TurnTilt => "draai-kiep",
        PanelType::Turn => "draai",
        PanelType::Tilt => "kiep",
        PanelType::Sliding => "schuif",
        PanelType::Door => "deur",
        PanelType::Panel => "paneel",
        PanelType::Ventilation => "ventilatie",
    }
}

fn ils_opening_direction(dir: OpeningDirection) -> &'static str {
    match dir {
        OpeningDirection::Left => "links",
        OpeningDirection::Right => "rechts",
        OpeningDirection::Inward => "naar binnen",
        OpeningDirection::Outward => "naar buiten",
    }
}

fn material_label(mat: &Material) -> &'static str {
    match mat {
        Material::Wood(WoodType::Meranti) => "Meranti",
        Material::Wood(WoodType::Accoya) => "Accoya",
        Material::Wood(WoodType::Vuren) => "Vuren",
        Material::Wood(WoodType::Eiken) => "Eiken",
        Material::Aluminum => "Aluminium",
        Material::Pvc => "Kunststof",
        Material::WoodAluminum => "Hout-aluminium",
    }
}

fn ils_material_label(mat: &Material) -> String {
    match mat {
        Material::Wood(wt) => {
            let wt_str = match wt {
                WoodType::Meranti => "meranti",
                WoodType::Accoya => "accoya",
                WoodType::Vuren => "vuren",
                WoodType::Eiken => "eiken",
            };
            format!("hout ({})", wt_str)
        }
        Material::Aluminum => "aluminium".into(),
        Material::Pvc => "kunststof".into(),
        Material::WoodAluminum => "hout-aluminium".into(),
    }
}

fn get_wood_type(mat: &Material) -> String {
    match mat {
        Material::Wood(WoodType::Meranti) => "Meranti (Dark Red Meranti)".into(),
        Material::Wood(WoodType::Accoya) => "Accoya (geacetyleerd hout)".into(),
        Material::Wood(WoodType::Vuren) => "Naaldhout (vuren/grenen)".into(),
        Material::Wood(WoodType::Eiken) => "Eiken".into(),
        _ => String::new(),
    }
}

fn get_durability_class(mat: &Material) -> String {
    match mat {
        Material::Wood(WoodType::Meranti) => "Klasse 2 (duurzaam)".into(),
        Material::Wood(WoodType::Accoya) => "Klasse 1 (zeer duurzaam)".into(),
        Material::Wood(WoodType::Eiken) => "Klasse 1-2".into(),
        Material::Wood(WoodType::Vuren) => {
            "Klasse 4-5 (niet duurzaam, verduurzaming nodig)".into()
        }
        _ => String::new(),
    }
}

fn calc_locking_points(kozijn: &Kozijn, cell_index: usize) -> i64 {
    let num_cols = kozijn.grid.columns.len();
    let col_idx = cell_index % num_cols;
    let row_idx = cell_index / num_cols;

    let cell_w = kozijn
        .grid
        .columns
        .get(col_idx)
        .map(|c| c.size)
        .unwrap_or(500.0);
    let cell_h = kozijn
        .grid
        .rows
        .get(row_idx)
        .map(|r| r.size)
        .unwrap_or(800.0);

    let perimeter = 2.0 * (cell_w + cell_h);
    (perimeter / 400.0).round().max(2.0) as i64
}

// ── IFC-SPF writer ─────────────────────────────────────────────

struct IfcWriter {
    entities: Vec<String>,
    next_id: u32,
    guid_counter: u32,
}

impl IfcWriter {
    fn new() -> Self {
        Self {
            entities: Vec::new(),
            next_id: 1,
            guid_counter: 0,
        }
    }

    fn add_entity(&mut self, entity: &str) -> String {
        let id = format!("#{}", self.next_id);
        self.entities.push(format!("{}={};", id, entity));
        self.next_id += 1;
        id
    }

    fn guid(&mut self) -> String {
        self.guid_counter += 1;
        // Generate a deterministic IFC-compatible compressed GUID (22 chars)
        let mut guid = String::with_capacity(22);
        let chars = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_$";
        let chars: Vec<char> = chars.chars().collect();

        // Use counter bytes + padding to generate 22-char GUID
        let mut val: u128 = self.guid_counter as u128;
        // Mix in a seed for uniqueness
        val = val.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        for _ in 0..22 {
            guid.push(chars[(val % 64) as usize]);
            val /= 64;
        }
        guid
    }

    fn add_owner_history(&mut self) -> String {
        let person = self.add_entity("IFCPERSON($,$,'Open Frame Studio',$,$,$,$,$)");
        let org = self.add_entity("IFCORGANIZATION($,'OpenAEC Foundation',$,$,$)");
        let person_org = self.add_entity(&format!(
            "IFCPERSONANDORGANIZATION({},{},$)",
            person, org
        ));
        let app = self.add_entity(&format!(
            "IFCAPPLICATION({},'0.1.0','Open Frame Studio','OFS')",
            org
        ));
        self.add_entity(&format!(
            "IFCOWNERHISTORY({},{},$,.NOCHANGE.,$,$,$,0)",
            person_org, app
        ))
    }

    fn add_si_units(&mut self) -> String {
        let length = self.add_entity("IFCSIUNIT(*,.LENGTHUNIT.,$,.METRE.)");
        let area = self.add_entity("IFCSIUNIT(*,.AREAUNIT.,$,.SQUARE_METRE.)");
        let volume = self.add_entity("IFCSIUNIT(*,.VOLUMEUNIT.,$,.CUBIC_METRE.)");
        let angle = self.add_entity("IFCSIUNIT(*,.PLANEANGLEUNIT.,$,.RADIAN.)");
        self.add_entity(&format!(
            "IFCUNITASSIGNMENT(({},{},{},{}))",
            length, area, volume, angle
        ))
    }

    fn add_rel_aggregates(
        &mut self,
        owner_history: &str,
        name: &str,
        relating: &str,
        related: &[String],
    ) -> String {
        let related_str = related.join(",");
        let guid = self.guid();
        self.add_entity(&format!(
            "IFCRELAGGREGATES('{}',{},'{}','{}',{},({}))",
            guid, owner_history, name, "", relating, related_str,
        ))
    }

    fn add_property_set(
        &mut self,
        owner_history: &str,
        name: &str,
        element: &str,
        props: &[String],
    ) {
        let props_str = props.join(",");
        let guid1 = self.guid();
        let pset = self.add_entity(&format!(
            "IFCPROPERTYSET('{}',{},'{}','{}',({}))",
            guid1, owner_history, name, "", props_str,
        ));
        let guid2 = self.guid();
        self.add_entity(&format!(
            "IFCRELDEFINESBYPROPERTIES('{}',{},$,$,({}),{})",
            guid2, owner_history, element, pset,
        ));
    }

    fn add_prop_label(&mut self, name: &str, value: &str) -> String {
        self.add_entity(&format!(
            "IFCPROPERTYSINGLEVALUE('{}','',IFCLABEL('{}'),$)",
            name, value
        ))
    }

    fn add_prop_real(&mut self, name: &str, value: f64) -> String {
        self.add_entity(&format!(
            "IFCPROPERTYSINGLEVALUE('{}','',IFCREAL({:.6}),$)",
            name, value
        ))
    }

    fn add_prop_int(&mut self, name: &str, value: i64) -> String {
        self.add_entity(&format!(
            "IFCPROPERTYSINGLEVALUE('{}','',IFCINTEGER({}),$)",
            name, value
        ))
    }

    fn write_to_file(
        &self,
        output_path: &str,
        _project: String,
        _context: String,
    ) -> Result<(), String> {
        let mut out = String::with_capacity(self.entities.len() * 100);

        // ISO header
        let _ = writeln!(out, "ISO-10303-21;");
        let _ = writeln!(out, "HEADER;");
        let _ = writeln!(
            out,
            "FILE_DESCRIPTION(('ViewDefinition [CoordinationView]'),'2;1');"
        );
        let _ = writeln!(
            out,
            "FILE_NAME('{}','',('Open Frame Studio'),('OpenAEC Foundation'),'','Open Frame Studio','');",
            output_path.replace('\\', "/")
        );
        let _ = writeln!(out, "FILE_SCHEMA(('IFC4'));");
        let _ = writeln!(out, "ENDSEC;");
        let _ = writeln!(out, "DATA;");

        for entity in &self.entities {
            let _ = writeln!(out, "{}", entity);
        }

        let _ = writeln!(out, "ENDSEC;");
        let _ = writeln!(out, "END-ISO-10303-21;");

        let mut file = std::fs::File::create(output_path)
            .map_err(|e| format!("Kan IFC bestand niet aanmaken: {}", e))?;
        file.write_all(out.as_bytes())
            .map_err(|e| format!("Kan IFC niet schrijven: {}", e))?;

        Ok(())
    }
}
