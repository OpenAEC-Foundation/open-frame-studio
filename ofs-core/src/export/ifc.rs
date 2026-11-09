//! IFC4 (STEP Physical File) export for kozijnen.
//!
//! Generates IFC4 files with IfcWindow/IfcDoor entities, geometry
//! (IfcExtrudedAreaSolid), and project property sets for Dutch kozijn data.
//! Export is not yet certified against an IFC Model View Definition or ILS.
//!
//! Writes IFC-SPF (.ifc) text format directly — no external crate needed.

use std::fmt::Write as FmtWrite;
use std::io::Write;

use crate::geometry::{compute_2d_geometry, Rect2D};
use crate::kozijn::{Kozijn, Material, OpeningDirection, PanelType, Project, ShapeType, WoodType};
use crate::production::{compute_production_data, CutListItem, GlassListItem, MemberType};

/// Level of Detail for IFC geometry export
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LodLevel {
    Lod200, // Simple box
    Lod300, // Frame members, dividers and fixed glazing
    Lod400, // Reserved for verified profile and joint detail
}

impl Default for LodLevel {
    fn default() -> Self {
        Self::Lod300
    }
}

/// Generate an IFC4 file from a kozijn definition.
pub fn generate_ifc(kozijn: &Kozijn, output_path: &str) -> Result<(), String> {
    generate_ifc_with_lod(kozijn, output_path, LodLevel::default())
}

/// Generate an IFC4 file from a kozijn definition with a specific LOD level.
pub fn generate_ifc_with_lod(
    kozijn: &Kozijn,
    output_path: &str,
    lod: LodLevel,
) -> Result<(), String> {
    generate_ifc_collection(
        &[kozijn],
        "Open Frame Studio Export",
        "",
        &kozijn.id,
        output_path,
        lod,
        false,
    )
}

/// Generate one IFC4 project containing every kozijn. The IFC element GlobalId
/// remains derived from each kozijn UUID, independent of its position in the
/// project list. Vliesgevels and combination assemblies are not included yet.
pub fn generate_project_ifc_with_lod(
    project: &Project,
    output_path: &str,
    lod: LodLevel,
) -> Result<(), String> {
    if project.kozijnen.is_empty() {
        return Err("IFC-projectexport: het project bevat geen kozijnen".into());
    }
    if !project.vliesgevels.is_empty() || !project.combinations.is_empty() {
        return Err(
            "IFC-projectexport ondersteunt nog geen vliesgevels of kozijncombinaties".into(),
        );
    }
    let kozijnen: Vec<&Kozijn> = project.kozijnen.iter().collect();
    generate_ifc_collection(
        &kozijnen,
        &project.project_info.name,
        &project.project_info.number,
        &project.id,
        output_path,
        lod,
        false,
    )
}

/// Export a project as an IFC4 element assembly with separate frame members
/// and glazing plates. This is an exchange model, not a CNC-ready LOD400 model.
pub fn generate_ifc_parts_project(project: &Project, output_path: &str) -> Result<(), String> {
    if project.kozijnen.is_empty() {
        return Err("IFC-onderdelenexport: het project bevat geen kozijnen".into());
    }
    if !project.vliesgevels.is_empty() || !project.combinations.is_empty() {
        return Err("IFC-onderdelenexport ondersteunt nog geen vliesgevels of kozijncombinaties".into());
    }
    let kozijnen: Vec<&Kozijn> = project.kozijnen.iter().collect();
    generate_ifc_collection(
        &kozijnen,
        &project.project_info.name,
        &project.project_info.number,
        &project.id,
        output_path,
        LodLevel::Lod300,
        true,
    )
}

/// Export reusable IfcWindowType definitions without placed occurrences.
/// The first library scope is rectangular fixed glazing at LOD300.
pub fn generate_ifc_type_library(project: &Project, output_path: &str) -> Result<(), String> {
    if project.kozijnen.is_empty() {
        return Err("IFC-typebibliotheek: het project bevat geen kozijnen".into());
    }
    if !project.vliesgevels.is_empty() || !project.combinations.is_empty() {
        return Err(
            "IFC-typebibliotheek ondersteunt nog geen vliesgevels of kozijncombinaties".into(),
        );
    }
    let mut ids = std::collections::HashSet::new();
    for kozijn in &project.kozijnen {
        if !ids.insert(kozijn.id) {
            return Err(format!(
                "IFC-typebibliotheek: dubbel kozijn-ID bij {}",
                kozijn.mark
            ));
        }
        validate_ifc_export(kozijn, LodLevel::Lod300)
            .map_err(|error| format!("Kozijn {}: {}", kozijn.mark, error))?;
    }
    let mut sorted: Vec<&Kozijn> = project.kozijnen.iter().collect();
    sorted.sort_by_key(|kozijn| kozijn.id);

    let mut ifc = IfcWriter::new(&project.id);
    let owner_history = ifc.add_owner_history();
    let units = ifc.add_si_units();
    let origin = ifc.add_entity("IFCCARTESIANPOINT((0.0,0.0,0.0))");
    let axis_z = ifc.add_entity("IFCDIRECTION((0.0,0.0,1.0))");
    let axis_x = ifc.add_entity("IFCDIRECTION((1.0,0.0,0.0))");
    let mapping_origin = ifc.add_entity(&format!(
        "IFCAXIS2PLACEMENT3D({},{},{})",
        origin, axis_z, axis_x,
    ));
    let north = ifc.add_entity("IFCDIRECTION((0.0,1.0,0.0))");
    let context = ifc.add_entity(&format!(
        "IFCGEOMETRICREPRESENTATIONCONTEXT($,'Model',3,1.0E-5,{},{})",
        mapping_origin, north,
    ));
    let body_context = ifc.add_entity(&format!(
        "IFCGEOMETRICREPRESENTATIONSUBCONTEXT('Body','Model',*,*,*,*,{},$,.MODEL_VIEW.,$)",
        context,
    ));

    let project_guid = ifc.guid();
    let root = ifc.add_entity(&format!(
        "IFCPROJECT('{}',{},'{}',$,$,$,$,({}),{})",
        project_guid,
        owner_history,
        step_str(&project.project_info.name),
        context,
        units,
    ));
    let library_guid = ifc.guid();
    let library = ifc.add_entity(&format!(
        "IFCPROJECTLIBRARY('{}',{},'Kozijnbibliotheek',$,$,$,$,({}),{})",
        library_guid, owner_history, context, units,
    ));
    let declaration_guid = ifc.guid();
    ifc.add_entity(&format!(
        "IFCRELDECLARES('{}',{},$,$,{},({}))",
        declaration_guid, owner_history, root, library,
    ));

    let mut types = Vec::with_capacity(sorted.len());
    for kozijn in sorted {
        let body = add_lod300_body_representation(&mut ifc, kozijn, &body_context);
        let representation_map = ifc.add_entity(&format!(
            "IFCREPRESENTATIONMAP({},{})",
            mapping_origin, body,
        ));
        let properties = add_library_type_properties(&mut ifc, &owner_history, kozijn);
        // A type and its occurrence need different GlobalIds when both files
        // are federated into the same BIM project.
        let type_uuid =
            uuid::Uuid::from_u128(kozijn.id.as_u128() ^ 0x4f46535f57494e444f575f5459504500_u128);
        let type_guid = crate::ifc_guid::uuid_to_ifc_guid(&type_uuid);
        let partition = if kozijn.cells.len() == 1 {
            ".SINGLE_PANEL."
        } else {
            ".NOTDEFINED."
        };
        let window_type = ifc.add_entity(&format!(
            "IFCWINDOWTYPE('{}',{},'{}',$,$,({}),({}),'{}',$,.WINDOW.,{},.F.,$)",
            type_guid,
            owner_history,
            step_str(&kozijn.name),
            properties,
            representation_map,
            step_str(&kozijn.mark),
            partition,
        ));
        types.push(window_type);
    }
    let declaration_guid = ifc.guid();
    ifc.add_entity(&format!(
        "IFCRELDECLARES('{}',{},$,$,{},({}))",
        declaration_guid,
        owner_history,
        library,
        types.join(","),
    ));
    ifc.write_to_file(output_path)
}

fn add_library_type_properties(
    ifc: &mut IfcWriter,
    owner_history: &str,
    kozijn: &Kozijn,
) -> String {
    let mut props = vec![
        ifc.add_prop_real("BuitenwerksBreedte_mm", kozijn.frame.outer_width),
        ifc.add_prop_real("BuitenwerksHoogte_mm", kozijn.frame.outer_height),
        ifc.add_prop_real("Kozijndiepte_mm", kozijn.frame.frame_depth),
        ifc.add_prop_label("Materiaal", material_label(&kozijn.frame.material)),
        ifc.add_prop_int("AantalVakken", kozijn.cells.len() as i64),
    ];
    if let Some(series_id) = &kozijn.series_id {
        props.push(ifc.add_prop_label("SeriesId", series_id));
    }
    for (index, cell) in kozijn.cells.iter().enumerate() {
        props.push(ifc.add_prop_real(
            &format!("Vak_{}_Glasdikte_mm", index + 1),
            cell.glazing.thickness_mm,
        ));
        if cell.glazing.ug_value.is_finite() && cell.glazing.ug_value > 0.0 {
            props.push(ifc.add_prop_real(&format!("Vak_{}_Ug", index + 1), cell.glazing.ug_value));
        }
    }
    let guid = ifc.guid();
    ifc.add_entity(&format!(
        "IFCPROPERTYSET('{}',{},'OFS_KozijnType',$,({}))",
        guid,
        owner_history,
        props.join(","),
    ))
}

fn generate_ifc_collection(
    kozijnen: &[&Kozijn],
    project_name: &str,
    project_number: &str,
    seed: &uuid::Uuid,
    output_path: &str,
    lod: LodLevel,
    parts: bool,
) -> Result<(), String> {
    let mut ids = std::collections::HashSet::new();
    for kozijn in kozijnen {
        if !ids.insert(kozijn.id) {
            return Err(format!("IFC-export: dubbel kozijn-ID bij {}", kozijn.mark));
        }
        validate_ifc_export(kozijn, lod)
            .map_err(|error| format!("Kozijn {}: {}", kozijn.mark, error))?;
        if parts {
            validate_parts_calculation(kozijn)
                .map_err(|error| format!("Kozijn {}: {}", kozijn.mark, error))?;
            for id in part_ids(kozijn) {
                if !ids.insert(id) {
                    return Err(format!("IFC-onderdelen: dubbel onderdeel-ID bij {}", kozijn.mark));
                }
            }
        }
    }
    let mut sorted = kozijnen.to_vec();
    sorted.sort_by_key(|kozijn| kozijn.id);
    let mut ifc = IfcWriter::new(seed);

    let owner_history = ifc.add_owner_history();
    let units = ifc.add_si_units();

    // ── Geometry context ───────────────────────────────────────
    // Created before the project record so IFCPROJECT can reference
    // it in RepresentationContexts (attribute 8).

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

    // ── Spatial hierarchy ──────────────────────────────────────

    let guid_proj = ifc.guid();
    let project_description = if project_number.is_empty() {
        "$".to_string()
    } else {
        format!("'Projectnummer: {}'", step_str(project_number))
    };
    let project = ifc.add_entity(&format!(
        "IFCPROJECT('{}',{},'{}',{},$,$,$,({}),{})",
        guid_proj,
        owner_history,
        step_str(project_name),
        project_description,
        context,
        units
    ));
    let project_props = vec![
        ifc.add_prop_label("ProjectUUID", &seed.to_string()),
        ifc.add_prop_label("ProjectNummer", project_number),
    ];
    ifc.add_property_set(&owner_history, "OFS_Project", &project, &project_props);

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
        "IFCBUILDINGSTOREY('{}',{},'Kozijnen',$,$,$,$,$,.ELEMENT.,0.0)",
        guid_stor, owner_history
    ));

    // Aggregation
    ifc.add_rel_aggregates(&owner_history, "ProjectSite", &project, &[site.clone()]);
    ifc.add_rel_aggregates(&owner_history, "SiteBuilding", &site, &[building.clone()]);
    ifc.add_rel_aggregates(
        &owner_history,
        "BuildingStorey",
        &building,
        &[storey.clone()],
    );

    // ── Elements ───────────────────────────────────────────────
    let mut elements = Vec::with_capacity(sorted.len());
    for kozijn in sorted {
        let frame = &kozijn.frame;
        let cells = &kozijn.cells;
        let has_door = cells.iter().any(|c| c.panel_type == PanelType::Door);
        let ifc_class = if has_door { "IFCDOOR" } else { "IFCWINDOW" };
        let width_m = frame.outer_width / 1000.0;
        let height_m = frame.outer_height / 1000.0;
        let depth_m = frame.frame_depth / 1000.0;

        // ── Frame geometry (LOD-dependent) ──────────────────────────
        let product_shape = if parts {
            "$".to_string()
        } else { match lod {
            LodLevel::Lod200 => {
                let box_extrusion = add_polygon_solid(
                    &mut ifc,
                    &[(0.0, 0.0), (width_m, 0.0), (width_m, height_m), (0.0, height_m)],
                    0.0, depth_m, "BoxProfile",
                );
                let shape_rep = ifc.add_entity(&format!(
                    "IFCSHAPEREPRESENTATION({},'Body','SweptSolid',({}))",
                    body_context, box_extrusion
                ));
                ifc.add_entity(&format!("IFCPRODUCTDEFINITIONSHAPE($,$,({}))", shape_rep))
            }
            LodLevel::Lod300 => {
                let shape_rep = add_lod300_body_representation(&mut ifc, kozijn, &body_context);
                ifc.add_entity(&format!("IFCPRODUCTDEFINITIONSHAPE($,$,({}))", shape_rep))
            }
            LodLevel::Lod400 => unreachable!("LOD400 is rejected by preflight"),
        }};

        // IFC window axes: local X = width, local Y = depth, local Z = up.
        // Placement coordinates are in millimetres; orientation is degrees
        // about world Z. Swept profiles lie in the local XZ plane.
        let angle = kozijn.placement.orientation.to_radians();
        let (sin, cos) = angle.sin_cos();
        let location = ifc.add_entity(&format!(
            "IFCCARTESIANPOINT(({:.6},{:.6},{:.6}))",
            kozijn.placement.x / 1000.0,
            kozijn.placement.y / 1000.0,
            kozijn.placement.z / 1000.0,
        ));
        let horizontal = ifc.add_entity(&format!("IFCDIRECTION(({:.9},{:.9},0.0))", cos, sin,));
        let placement_axis = ifc.add_entity(&format!(
            "IFCAXIS2PLACEMENT3D({},{},{})",
            location, axis_z, horizontal,
        ));
        let local_placement = ifc.add_entity(&format!("IFCLOCALPLACEMENT($,{})", placement_axis));

        if parts {
            let assembly = add_parts_assembly(
                &mut ifc, &owner_history, &body_context, kozijn, &local_placement,
            );
            elements.push(assembly);
            continue;
        }

        // ── Element ────────────────────────────────────────────────

        // Derive the element GlobalId from the kozijn UUID so exports are
        // stable across runs (BCF references, roundtrip re-import).
        //
        // IFC4 IfcWindow/IfcDoor (13 attributes): GlobalId, OwnerHistory,
        // Name, Description, ObjectType, ObjectPlacement, Representation,
        // Tag, OverallHeight, OverallWidth, PredefinedType,
        // PartitioningType/OperationType, UserDefinedPartitioningType/
        // UserDefinedOperationType. The merkteken travels in Tag.
        let guid_elem = crate::ifc_guid::uuid_to_ifc_guid(&kozijn.id);
        let element = ifc.add_entity(&format!(
            "{}('{}',{},'{}',$,$,{},{},'{}',{:.6},{:.6},$,$,$)",
            ifc_class,
            guid_elem,
            owner_history,
            step_str(&kozijn.name),
            local_placement,
            product_shape,
            step_str(&kozijn.mark),
            height_m,
            width_m,
        ));

        // ── Property sets ──────────────────────────────────────────

        add_standard_psets(&mut ifc, &owner_history, &element, kozijn);
        add_ils_psets(&mut ifc, &owner_history, &element, kozijn);
        elements.push(element);
    }

    // Every element belongs to the one neutral storey in this export.
    let guid_rel = ifc.guid();
    ifc.add_entity(&format!(
        "IFCRELCONTAINEDINSPATIALSTRUCTURE('{}',{},$,$,({}),{})",
        guid_rel,
        owner_history,
        elements.join(","),
        storey
    ));

    // Write file
    ifc.write_to_file(output_path)
}

fn validate_parts_calculation(kozijn: &Kozijn) -> Result<(), String> {
    let geometry = compute_2d_geometry(kozijn);
    let production = compute_production_data(kozijn);
    let count = |kind| production.cut_list.iter().filter(|item| item.member_type == kind).count();
    for kind in [MemberType::FrameTop, MemberType::FrameBottom, MemberType::FrameLeft, MemberType::FrameRight] {
        if count(kind) != 1 {
            return Err("IFC-onderdelen: zaaglijst mist een kozijnprofiel".into());
        }
    }
    if count(MemberType::DividerV) != geometry.v_dividers.len()
        || count(MemberType::DividerH) != geometry.h_dividers.len()
        || production.glass_list.len() != geometry.cell_rects.len()
    {
        return Err("IFC-onderdelen: geometrie en calculatielijsten komen niet overeen".into());
    }
    if production.cut_list.iter().any(|item| {
        !item.net_length_mm.is_finite() || item.net_length_mm <= 0.0
        || !item.gross_length_mm.is_finite() || item.gross_length_mm <= 0.0
        || !item.miter_left_deg.is_finite() || !item.miter_right_deg.is_finite()
    }) || production.glass_list.iter().any(|item| {
        !item.width_mm.is_finite() || item.width_mm <= 0.0
        || !item.height_mm.is_finite() || item.height_mm <= 0.0
        || !item.thickness_mm.is_finite() || item.thickness_mm <= 0.0
    }) {
        return Err("IFC-onderdelen: ongeldige indicatieve zaag- of glasmaat".into());
    }
    Ok(())
}

fn part_ids(kozijn: &Kozijn) -> Vec<uuid::Uuid> {
    let geometry = compute_2d_geometry(kozijn);
    let mut ids = (0..4).map(|i| kozijn.frame_part_id(i)).collect::<Vec<_>>();
    ids.extend((0..geometry.v_dividers.len()).map(|i| kozijn.vertical_part_id(i)));
    ids.extend((0..geometry.h_dividers.len()).map(|i| kozijn.horizontal_part_id(i)));
    ids.extend(geometry.cell_rects.iter().map(|r| kozijn.glass_part_id(r.cell_index)));
    ids
}

/// Reject shapes that this IFC representation cannot describe faithfully.
/// Validation runs before the file is created, so a failed export leaves no
/// plausible-looking but incorrect IFC behind.
fn validate_ifc_export(kozijn: &Kozijn, lod: LodLevel) -> Result<(), String> {
    let frame = &kozijn.frame;
    if lod == LodLevel::Lod400 {
        return Err(
            "IFC LOD400 is nog niet beschikbaar: profiel- en verbindingsdetails ontbreken"
                .into(),
        );
    }
    if frame.shape.shape_type != ShapeType::Rectangular {
        return Err("IFC-export ondersteunt momenteel alleen rechthoekige kozijnen".into());
    }
    if kozijn.layout.is_some() || !kozijn.extensions.is_empty() {
        return Err("IFC-export ondersteunt nog geen vrije vakindeling of kozijnaanbouw".into());
    }
    let measures = [
        frame.outer_width,
        frame.outer_height,
        frame.frame_width,
        frame.frame_depth,
        kozijn.placement.x,
        kozijn.placement.y,
        kozijn.placement.z,
        kozijn.placement.orientation,
    ];
    if measures.iter().any(|v| !v.is_finite())
        || frame.frame_width <= 0.0
        || frame.frame_depth <= 0.0
        || frame.outer_width <= 2.0 * frame.frame_width
        || frame.outer_height <= 2.0 * frame.frame_width
    {
        return Err("IFC-export: ongeldige kozijnmaat of plaatsing".into());
    }
    if kozijn.grid.columns.is_empty()
        || kozijn.grid.rows.is_empty()
        || kozijn.cells.len() != kozijn.grid.columns.len() * kozijn.grid.rows.len()
    {
        return Err("IFC-export: vakindeling en aantal cellen komen niet overeen".into());
    }
    let expected_width = frame.outer_width - 2.0 * frame.frame_width;
    let expected_height = frame.outer_height - 2.0 * frame.frame_width;
    let columns = kozijn.grid.columns.iter().map(|c| c.size).sum::<f64>()
        + frame.frame_width * (kozijn.grid.columns.len() - 1) as f64;
    let rows = kozijn.grid.rows.iter().map(|r| r.size).sum::<f64>()
        + frame.frame_width * (kozijn.grid.rows.len() - 1) as f64;
    if kozijn
        .grid
        .columns
        .iter()
        .any(|c| !c.size.is_finite() || c.size <= 0.0)
        || kozijn
            .grid
            .rows
            .iter()
            .any(|r| !r.size.is_finite() || r.size <= 0.0)
        || (columns - expected_width).abs() > 0.01
        || (rows - expected_height).abs() > 0.01
    {
        return Err("IFC-export: vakmaten sluiten niet aan op de buitenmaat".into());
    }
    if lod == LodLevel::Lod300
        && kozijn.cells.iter().any(|cell| {
            cell.panel_type != PanelType::FixedGlass
                || !cell.glazing.thickness_mm.is_finite()
                || cell.glazing.thickness_mm <= 0.0
                || cell.glazing.thickness_mm > frame.frame_depth
        })
    {
        return Err(
            "IFC LOD300 ondersteunt momenteel alleen vaste beglazing met geldige glasdikte".into(),
        );
    }
    Ok(())
}

/// Build the shared local body representation used by an occurrence or a
/// reusable type map. The geometry has no project placement of its own.
fn add_lod300_body_representation(
    ifc: &mut IfcWriter,
    kozijn: &Kozijn,
    body_context: &str,
) -> String {
    let geometry = compute_2d_geometry(kozijn);
    let height_m = kozijn.frame.outer_height / 1000.0;
    let depth_m = kozijn.frame.frame_depth / 1000.0;
    let mut solids = Vec::new();

    // SVG y runs downwards; IFC local y runs upwards.
    if geometry.frame_polygons.is_empty() {
        for rect in &geometry.frame_rects {
            solids.push(add_rect_solid(
                ifc,
                rect,
                height_m,
                0.0,
                depth_m,
                "FrameMember",
            ));
        }
    } else {
        for polygon in &geometry.frame_polygons {
            let points: Vec<(f64, f64)> = polygon
                .iter()
                .map(|p| (p[0] / 1000.0, height_m - p[1] / 1000.0))
                .collect();
            solids.push(add_polygon_solid(ifc, &points, 0.0, depth_m, "FrameMember"));
        }
    }
    for rect in geometry.v_dividers.iter().chain(geometry.h_dividers.iter()) {
        solids.push(add_rect_solid(ifc, rect, height_m, 0.0, depth_m, "Divider"));
    }
    for cell_rect in &geometry.cell_rects {
        let thickness = kozijn.cells[cell_rect.cell_index].glazing.thickness_mm / 1000.0;
        solids.push(add_rect_solid(
            ifc,
            &cell_rect.rect,
            height_m,
            (depth_m - thickness) / 2.0,
            thickness,
            "Glazing",
        ));
    }
    ifc.add_entity(&format!(
        "IFCSHAPEREPRESENTATION({},'Body','SweptSolid',({}))",
        body_context,
        solids.join(","),
    ))
}

fn add_parts_assembly(
    ifc: &mut IfcWriter,
    owner_history: &str,
    body_context: &str,
    kozijn: &Kozijn,
    assembly_placement: &str,
) -> String {
    let assembly_guid = crate::ifc_guid::uuid_to_ifc_guid(&kozijn.id);
    let assembly = ifc.add_entity(&format!(
        "IFCELEMENTASSEMBLY('{}',{},'{}',$,$,{},$,'{}',$,.NOTDEFINED.)",
        assembly_guid,
        owner_history,
        step_str(&kozijn.name),
        assembly_placement,
        step_str(&kozijn.mark),
    ));
    add_ils_psets(ifc, owner_history, &assembly, kozijn);

    // The child profiles use the assembly's local coordinate system. Only
    // the assembly itself is contained in the storey; its parts are aggregated.
    let origin = ifc.add_entity("IFCCARTESIANPOINT((0.0,0.0,0.0))");
    let axis = ifc.add_entity(&format!("IFCAXIS2PLACEMENT3D({},$,$)", origin));
    let part_placement = ifc.add_entity(&format!("IFCLOCALPLACEMENT({},{})", assembly_placement, axis));
    let height_m = kozijn.frame.outer_height / 1000.0;
    let depth_m = kozijn.frame.frame_depth / 1000.0;
    let geometry = compute_2d_geometry(kozijn);
    let roles = ["Bovenregel", "Onderregel", "Linkerstijl", "Rechterstijl"];
    let mut members = Vec::new();
    let mut plates = Vec::new();
    let mut ordinal = 1u128;
    for (index, role) in roles.iter().enumerate() {
        let solid = if geometry.frame_polygons.is_empty() {
            add_rect_solid(ifc, &geometry.frame_rects[index], height_m, 0.0, depth_m, "FrameMember")
        } else {
            let points: Vec<(f64, f64)> = geometry.frame_polygons[index]
                .iter()
                .map(|p| (p[0] / 1000.0, height_m - p[1] / 1000.0))
                .collect();
            add_polygon_solid(ifc, &points, 0.0, depth_m, "FrameMember")
        };
        members.push(add_parts_element(ifc, owner_history, body_context, kozijn, &part_placement,
            "IFCMEMBER", role, ordinal, kozijn.frame_part_id(index), &solid, None));
        ordinal += 1;
    }
    for (index, rect) in geometry.v_dividers.iter().enumerate() {
        let solid = add_rect_solid(ifc, rect, height_m, 0.0, depth_m, "Divider");
        members.push(add_parts_element(ifc, owner_history, body_context, kozijn, &part_placement,
            "IFCMEMBER", &format!("Tussenstijl {}", index + 1), ordinal,
            kozijn.vertical_part_id(index), &solid, None));
        ordinal += 1;
    }
    for (index, rect) in geometry.h_dividers.iter().enumerate() {
        let solid = add_rect_solid(ifc, rect, height_m, 0.0, depth_m, "Divider");
        members.push(add_parts_element(ifc, owner_history, body_context, kozijn, &part_placement,
            "IFCMEMBER", &format!("Tussenregel {}", index + 1), ordinal,
            kozijn.horizontal_part_id(index), &solid, None));
        ordinal += 1;
    }
    for cell_rect in &geometry.cell_rects {
        let cell = &kozijn.cells[cell_rect.cell_index];
        let thickness = cell.glazing.thickness_mm / 1000.0;
        let solid = add_rect_solid(ifc, &cell_rect.rect, height_m,
            (depth_m - thickness) / 2.0, thickness, "Glazing");
        plates.push(add_parts_element(ifc, owner_history, body_context, kozijn, &part_placement,
            "IFCPLATE", &format!("Ruit vak {}", cell_rect.cell_index + 1), ordinal,
            kozijn.glass_part_id(cell_rect.cell_index), &solid,
            Some((cell_rect.rect.width, cell_rect.rect.height, cell.glazing.thickness_mm))));
        ordinal += 1;
    }
    let production = compute_production_data(kozijn);
    let frame_kinds = [
        MemberType::FrameTop, MemberType::FrameBottom,
        MemberType::FrameLeft, MemberType::FrameRight,
    ];
    for (member, kind) in members.iter().take(4).zip(frame_kinds) {
        if let Some(item) = production.cut_list.iter().find(|item| item.member_type == kind) {
            add_indicative_cut_pset(ifc, owner_history, member, item);
        }
    }
    let vertical = production.cut_list.iter().filter(|item| item.member_type == MemberType::DividerV);
    for (member, item) in members.iter().skip(4).take(geometry.v_dividers.len()).zip(vertical) {
        add_indicative_cut_pset(ifc, owner_history, member, item);
    }
    let horizontal = production.cut_list.iter().filter(|item| item.member_type == MemberType::DividerH);
    for (member, item) in members.iter().skip(4 + geometry.v_dividers.len()).zip(horizontal) {
        add_indicative_cut_pset(ifc, owner_history, member, item);
    }
    for (plate, cell_rect) in plates.iter().zip(&geometry.cell_rects) {
        if let Some(item) = production.glass_list.iter().find(|item| item.cell_index == cell_rect.cell_index) {
            add_indicative_glass_pset(ifc, owner_history, plate, item);
        }
    }
    let mut children = members.clone();
    children.extend(plates.iter().cloned());
    ifc.add_rel_aggregates(owner_history, "Kozijnonderdelen", &assembly, &children);

    let frame_material = ifc.add_entity(&format!(
        "IFCMATERIAL('{}',$,'Kozijnprofiel')", step_str(material_label(&kozijn.frame.material)),
    ));
    let glass_material = ifc.add_entity("IFCMATERIAL('Glas',$,'Glas')");
    ifc.add_rel_material(owner_history, &members, &frame_material);
    ifc.add_rel_material(owner_history, &plates, &glass_material);
    assembly
}

fn add_indicative_cut_pset(ifc: &mut IfcWriter, owner_history: &str, element: &str, item: &CutListItem) {
    let props = vec![
        ifc.add_prop_label("Status", "Indicatief; niet voor directe productieaansturing"),
        ifc.add_prop_label("ZaaglijstID", &item.piece_id),
        ifc.add_prop_label("Profielomschrijving", &item.profile_name),
        ifc.add_prop_real("NettoLengte_mm", item.net_length_mm),
        ifc.add_prop_real("BrutoLengte_mm", item.gross_length_mm),
        ifc.add_prop_real("VerstekBegin_graad", item.miter_left_deg),
        ifc.add_prop_real("VerstekEind_graad", item.miter_right_deg),
        ifc.add_prop_int("Aantal", item.quantity as i64),
    ];
    ifc.add_property_set(owner_history, "OFS_CalculatieIndicatief", element, &props);
}

fn add_indicative_glass_pset(ifc: &mut IfcWriter, owner_history: &str, element: &str, item: &GlassListItem) {
    let props = vec![
        ifc.add_prop_label("Status", "Indicatief; glasmaat laten controleren"),
        ifc.add_prop_label("GlaslijstID", &item.piece_id),
        ifc.add_prop_label("Glastype", &item.glass_type),
        ifc.add_prop_real("BestelbreedteIndicatief_mm", item.width_mm),
        ifc.add_prop_real("BestelhoogteIndicatief_mm", item.height_mm),
        ifc.add_prop_real("Dikte_mm", item.thickness_mm),
        ifc.add_prop_int("Aantal", item.quantity as i64),
    ];
    ifc.add_property_set(owner_history, "OFS_CalculatieIndicatief", element, &props);
}

#[allow(clippy::too_many_arguments)]
fn add_parts_element(
    ifc: &mut IfcWriter,
    owner_history: &str,
    body_context: &str,
    kozijn: &Kozijn,
    placement: &str,
    class: &str,
    role: &str,
    ordinal: u128,
    part_uuid: uuid::Uuid,
    solid: &str,
    glazing_mm: Option<(f64, f64, f64)>,
) -> String {
    let shape = ifc.add_entity(&format!(
        "IFCSHAPEREPRESENTATION({},'Body','SweptSolid',({}))", body_context, solid,
    ));
    let product_shape = ifc.add_entity(&format!("IFCPRODUCTDEFINITIONSHAPE($,$,({}))", shape));
    let part_guid = crate::ifc_guid::uuid_to_ifc_guid(&part_uuid);
    let tag = format!("{}-{:03}", kozijn.mark, ordinal);
    // IfcMember and IfcPlate each add one optional PredefinedType to IfcElement.
    let element = ifc.add_entity(&format!(
        "{}('{}',{},'{}',$,$,{},{},'{}',$)",
        class, part_guid, owner_history, step_str(role), placement, product_shape, step_str(&tag),
    ));
    let mut props = vec![
        ifc.add_prop_label("KozijncombinatieID", &kozijn.id.to_string()),
        ifc.add_prop_label("Onderdeel", role),
        ifc.add_prop_label("StukID", &tag),
    ];
    if let Some((width, height, thickness)) = glazing_mm {
        props.push(ifc.add_prop_real("Zichtbreedte_mm", width));
        props.push(ifc.add_prop_real("Zichthoogte_mm", height));
        props.push(ifc.add_prop_real("Glasdikte_mm", thickness));
    }
    ifc.add_property_set(owner_history, "OFS_Onderdeel", &element, &props);
    element
}

fn add_rect_solid(
    ifc: &mut IfcWriter,
    rect: &Rect2D,
    height_m: f64,
    z: f64,
    depth: f64,
    name: &str,
) -> String {
    let left = rect.x / 1000.0;
    let right = (rect.x + rect.width) / 1000.0;
    let top = height_m - rect.y / 1000.0;
    let bottom = height_m - (rect.y + rect.height) / 1000.0;
    add_polygon_solid(
        ifc,
        &[(left, bottom), (right, bottom), (right, top), (left, top)],
        z,
        depth,
        name,
    )
}

fn add_polygon_solid(
    ifc: &mut IfcWriter,
    points: &[(f64, f64)],
    z: f64,
    depth: f64,
    name: &str,
) -> String {
    // IFC profile outer curves use counter-clockwise winding when viewed
    // along the extrusion axis. SVG frame polygons become clockwise when
    // their downward Y is flipped to the IFC upward Y.
    let signed_area_twice: f64 = points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
        .map(|((x1, y1), (x2, y2))| x1 * y2 - x2 * y1)
        .sum();
    let mut ordered = points.to_vec();
    if signed_area_twice < 0.0 {
        ordered.reverse();
    }
    let mut vertices: Vec<String> = ordered
        .iter()
        .map(|(x, y)| ifc.add_entity(&format!("IFCCARTESIANPOINT(({:.6},{:.6}))", x, y)))
        .collect();
    vertices.push(vertices[0].clone());
    let polyline = ifc.add_entity(&format!("IFCPOLYLINE(({}))", vertices.join(",")));
    let profile = ifc.add_entity(&format!(
        "IFCARBITRARYCLOSEDPROFILEDEF(.AREA.,'{}',{})",
        name, polyline,
    ));
    let location = ifc.add_entity(&format!("IFCCARTESIANPOINT((0.0,{:.6},0.0))", -z));
    let profile_normal = ifc.add_entity("IFCDIRECTION((0.0,-1.0,0.0))");
    let horizontal = ifc.add_entity("IFCDIRECTION((1.0,0.0,0.0))");
    let placement = ifc.add_entity(&format!("IFCAXIS2PLACEMENT3D({},{},{})", location, profile_normal, horizontal));
    let extrusion = ifc.add_entity("IFCDIRECTION((0.0,0.0,1.0))");
    ifc.add_entity(&format!(
        "IFCEXTRUDEDAREASOLID({},{},{},{:.6})",
        profile, placement, extrusion, depth,
    ))
}

// ── Standard property sets ─────────────────────────────────────

fn add_standard_psets(ifc: &mut IfcWriter, oh: &str, element: &str, kozijn: &Kozijn) {
    let frame = &kozijn.frame;
    let cells = &kozijn.cells;
    let is_door = cells.iter().any(|c| c.panel_type == PanelType::Door);
    let pset_name = if is_door {
        "Pset_DoorCommon"
    } else {
        "Pset_WindowCommon"
    };

    let ow_mm = frame.outer_width;
    let oh_mm = frame.outer_height;
    let total_area = (ow_mm / 1000.0) * (oh_mm / 1000.0);
    let glass_area = if cells.iter().all(|c| c.panel_type == PanelType::FixedGlass) {
        compute_2d_geometry(kozijn)
            .cell_rects
            .iter()
            .map(|c| c.rect.width * c.rect.height / 1e6)
            .sum::<f64>()
    } else {
        0.0
    };

    let mut props = Vec::new();

    // Reference
    let p = ifc.add_entity(&format!(
        "IFCPROPERTYSINGLEVALUE('Reference',$,IFCLABEL('{}'),$)",
        step_str(&kozijn.mark)
    ));
    props.push(p);

    // GlazingAreaFraction
    if total_area > 0.0 && glass_area > 0.0 {
        let frac = glass_area / total_area;
        let p = ifc.add_entity(&format!(
            "IFCPROPERTYSINGLEVALUE('GlazingAreaFraction',$,IFCPOSITIVERATIOMEASURE({:.4}),$)",
            frac
        ));
        props.push(p);
    }

    // Pset_WindowCommon.ThermalTransmittance means Uw of the complete
    // assembly. A pane's Ug is not a substitute; omit it until a verified
    // assembly value can be supplied.

    ifc.add_property_set(oh, pset_name, element, &props);

    // Pset_OFS_Kozijn
    let mat_str = material_label(&frame.material);
    let mut ofs_props: Vec<String> = vec![
        ifc.add_entity(&format!(
            "IFCPROPERTYSINGLEVALUE('Material',$,IFCLABEL('{}'),$)",
            mat_str
        )),
        ifc.add_entity(&format!(
            "IFCPROPERTYSINGLEVALUE('ColorInside',$,IFCLABEL('{}'),$)",
            step_str(&frame.color_inside)
        )),
        ifc.add_entity(&format!(
            "IFCPROPERTYSINGLEVALUE('ColorOutside',$,IFCLABEL('{}'),$)",
            step_str(&frame.color_outside)
        )),
        ifc.add_entity(&format!(
            "IFCPROPERTYSINGLEVALUE('FrameWidth_mm',$,IFCREAL({:.1}),$)",
            frame.frame_width
        )),
        ifc.add_entity(&format!(
            "IFCPROPERTYSINGLEVALUE('FrameDepth_mm',$,IFCREAL({:.1}),$)",
            frame.frame_depth
        )),
        ifc.add_entity(&format!(
            "IFCPROPERTYSINGLEVALUE('CellCount',$,IFCINTEGER({}),$)",
            cells.len()
        )),
    ];
    if let Some(series_id) = &kozijn.series_id {
        ofs_props.push(ifc.add_prop_label("SeriesId", series_id));
    }
    ifc.add_property_set(oh, "Pset_OFS_Kozijn", element, &ofs_props);
}

// ── Project-specific Dutch kozijn property sets ────────────────

fn add_ils_psets(ifc: &mut IfcWriter, oh: &str, element: &str, kozijn: &Kozijn) {
    let frame = &kozijn.frame;
    let grid = &kozijn.grid;
    let cells = &kozijn.cells;
    let ow = frame.outer_width;
    let oh_mm = frame.outer_height;
    let fw = frame.frame_width;
    let fd = frame.frame_depth;

    // OFS_KozijnAlgemeen
    {
        let kozijn_type = determine_kozijn_type(cells);
        let props = vec![
            ifc.add_prop_label("Merkteken", &kozijn.mark),
            ifc.add_prop_label("Naam", &kozijn.name),
            ifc.add_prop_label("Type", &kozijn_type),
            ifc.add_prop_label("ProfielSysteem", &frame.profile.id),
            ifc.add_prop_label("ProfielSysteemNaam", &frame.profile.name),
        ];
        ifc.add_property_set(oh, "OFS_KozijnAlgemeen", element, &props);
    }

    // OFS_KozijnAfmetingen
    {
        let mut props = vec![
            ifc.add_prop_real("BuitenwerksBreedte_mm", ow),
            ifc.add_prop_real("BuitenwerksHoogte_mm", oh_mm),
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
        ifc.add_property_set(oh, "OFS_KozijnAfmetingen", element, &props);
    }

    // OFS_KozijnMateriaal
    {
        let mat_str = ils_material_label(&frame.material);
        let wood_type = get_wood_type(&frame.material);

        let mut props = vec![
            ifc.add_prop_label("Materiaal", &mat_str),
            ifc.add_prop_label("KleurBinnenzijde", &frame.color_inside),
            ifc.add_prop_label("KleurBuitenzijde", &frame.color_outside),
        ];
        if !wood_type.is_empty() {
            props.push(ifc.add_prop_label("Houtsoort", &wood_type));
        }
        if frame.sill_profile.is_some() {
            let sill_name = frame
                .sill_profile
                .as_ref()
                .map(|s| s.name.as_str())
                .unwrap_or("");
            props.push(ifc.add_prop_label("DorpelType", sill_name));
        }
        ifc.add_property_set(oh, "OFS_KozijnMateriaal", element, &props);
    }

    // OFS_KozijnBeglazing
    {
        let mut props = Vec::new();
        let mut glass_types = std::collections::BTreeSet::new();
        let mut ug_values = Vec::new();

        for (i, cell) in cells
            .iter()
            .enumerate()
            .filter(|(_, cell)| cell.panel_type == PanelType::FixedGlass)
        {
            glass_types.insert(cell.glazing.glass_type.clone());
            if cell.glazing.ug_value.is_finite() && cell.glazing.ug_value > 0.0 {
                ug_values.push(cell.glazing.ug_value);
            }

            props.push(
                ifc.add_prop_label(&format!("Cel_{}_GlasType", i + 1), &cell.glazing.glass_type),
            );
            props.push(ifc.add_prop_real(
                &format!("Cel_{}_GlasDikte_mm", i + 1),
                cell.glazing.thickness_mm,
            ));
            if cell.glazing.ug_value.is_finite() && cell.glazing.ug_value > 0.0 {
                props.push(
                    ifc.add_prop_real(&format!("Cel_{}_UgWaarde", i + 1), cell.glazing.ug_value),
                );
            }
        }

        if !glass_types.is_empty() {
            let types_str: Vec<&str> = glass_types.iter().map(|s| s.as_str()).collect();
            props.push(ifc.add_prop_label("GlasTypen", &types_str.join(", ")));
        }

        if !ug_values.is_empty() {
            let avg = ug_values.iter().sum::<f64>() / ug_values.len() as f64;
            let max = ug_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            props.push(ifc.add_prop_real("UgWaarde_gemiddeld", avg));
            props.push(ifc.add_prop_real("UgWaarde_max", max));
        }

        if !props.is_empty() {
            ifc.add_property_set(oh, "OFS_KozijnBeglazing", element, &props);
        }
    }

    // OFS_KozijnOnderdelen
    {
        let mut props = vec![ifc.add_prop_int("AantalCellen", cells.len() as i64)];

        for (i, cell) in cells.iter().enumerate() {
            let ils_type = ils_panel_type(cell.panel_type);
            props.push(ifc.add_prop_label(&format!("Cel_{}_Type", i + 1), ils_type));

            if let Some(dir) = cell.opening_direction {
                let ils_dir = ils_opening_direction(dir);
                props.push(ifc.add_prop_label(&format!("Cel_{}_Draairichting", i + 1), ils_dir));
            }

            let has_sash = cell.panel_type.is_operable();
            let val = if has_sash { ".T." } else { ".F." };
            props.push(ifc.add_entity(&format!(
                "IFCPROPERTYSINGLEVALUE('Cel_{}_HeeftVleugel',$,IFCBOOLEAN({}),$)",
                i + 1,
                val
            )));
        }

        ifc.add_property_set(oh, "OFS_KozijnOnderdelen", element, &props);
    }

    // OFS_KozijnHangSluitwerk
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
            }
        }

        if !props.is_empty() {
            ifc.add_property_set(oh, "OFS_KozijnHangSluitwerk", element, &props);
        }
    }

    // A verified Uw/Uf/Psi combination is not yet stored on Kozijn. Exporting
    // assumed values here would make an unverified design look compliant.
}

// ── Helper functions ───────────────────────────────────────────

/// Encode a string for ISO 10303-21:2002 (the `2;1` FILE_DESCRIPTION).
/// This edition uses printable ASCII in the exchange file; Unicode is
/// carried through X2/X4 control directives instead of raw UTF-8.
fn step_str(s: &str) -> String {
    let mut encoded = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => encoded.push_str("\\\\"),
            '\'' => encoded.push_str("''"),
            ' '..='~' => encoded.push(ch),
            ch if (ch as u32) <= 0x1f => {
                let _ = write!(encoded, "\\X\\{:02X}", ch as u32);
            }
            ch if (ch as u32) <= 0xffff => {
                let _ = write!(encoded, "\\X2\\{:04X}\\X0\\", ch as u32);
            }
            ch => {
                let _ = write!(encoded, "\\X4\\{:08X}\\X0\\", ch as u32);
            }
        }
    }
    encoded
}

fn determine_kozijn_type(cells: &[crate::kozijn::Cell]) -> String {
    let types: std::collections::HashSet<PanelType> = cells.iter().map(|c| c.panel_type).collect();

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
        PanelType::TopHung => "klapraam",
        PanelType::BottomHung => "tuimelraam",
        PanelType::LiftSlide => "hefschuif",
        PanelType::Pivot => "pivot",
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
        Material::Wood(WoodType::Meranti) => "Meranti".into(),
        Material::Wood(WoodType::Accoya) => "Accoya".into(),
        Material::Wood(WoodType::Vuren) => "Vuren".into(),
        Material::Wood(WoodType::Eiken) => "Eiken".into(),
        _ => String::new(),
    }
}

// ── IFC-SPF writer ─────────────────────────────────────────────

struct IfcWriter {
    entities: Vec<String>,
    next_id: u32,
    guid_counter: u32,
    /// 128-bit seed (the kozijn UUID) from which entity GUIDs derive.
    guid_seed: u128,
}

impl IfcWriter {
    fn new(seed: &uuid::Uuid) -> Self {
        Self {
            entities: Vec::new(),
            next_id: 1,
            guid_counter: 0,
            guid_seed: seed.as_u128(),
        }
    }

    fn add_entity(&mut self, entity: &str) -> String {
        let id = format!("#{}", self.next_id);
        self.entities.push(format!("{}={};", id, entity));
        self.next_id += 1;
        id
    }

    /// Deterministic IFC GlobalId (22 chars, first char always 0-3).
    ///
    /// Mixes the seed UUID with an incrementing counter — multiplying by
    /// an odd constant is a bijection mod 2^128, so distinct counters
    /// yield distinct GUIDs — and encodes the result MSB-first via
    /// `uuid_to_ifc_guid`, which keeps the leading character within the
    /// spec range 0-3. Deterministic, so repeated exports of the same
    /// kozijn produce identical files. (The workspace `uuid` crate has
    /// no `v5` feature, hence no `Uuid::new_v5` here.)
    fn guid(&mut self) -> String {
        self.guid_counter += 1;
        let mixed = (self.guid_seed ^ self.guid_counter as u128)
            .wrapping_mul(0x9E37_79B9_7F4A_7C15_D1B5_4A32_D192_ED03);
        crate::ifc_guid::uuid_to_ifc_guid(&uuid::Uuid::from_u128(mixed))
    }

    fn add_owner_history(&mut self) -> String {
        let person = self.add_entity("IFCPERSON($,$,'Open Frame Studio',$,$,$,$,$)");
        let org = self.add_entity("IFCORGANIZATION($,'OpenAEC Foundation',$,$,$)");
        let person_org =
            self.add_entity(&format!("IFCPERSONANDORGANIZATION({},{},$)", person, org));
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

    fn add_rel_material(&mut self, owner_history: &str, elements: &[String], material: &str) {
        let guid = self.guid();
        self.add_entity(&format!(
            "IFCRELASSOCIATESMATERIAL('{}',{},$,$,({}),{})",
            guid, owner_history, elements.join(","), material,
        ));
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
            name,
            step_str(value)
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

    fn write_to_file(&self, output_path: &str) -> Result<(), String> {
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
            step_str(&output_path.replace('\\', "/"))
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

// ── Tests ──────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Export a kozijn to a temp file and return the IFC text.
    fn export_to_string(kozijn: &Kozijn) -> String {
        let path = std::env::temp_dir().join(format!(
            "ofs_ifc_test_{}_{}.ifc",
            std::process::id(),
            kozijn.id
        ));
        let path_str = path.to_string_lossy().into_owned();
        generate_ifc(kozijn, &path_str).expect("IFC-export hoort te slagen");
        let content = std::fs::read_to_string(&path).expect("IFC-bestand hoort leesbaar te zijn");
        let _ = std::fs::remove_file(&path);
        content
    }

    fn export_project_to_string(project: &Project) -> String {
        let path = std::env::temp_dir().join(format!(
            "ofs_ifc_project_test_{}_{}.ifc",
            std::process::id(),
            project.id,
        ));
        let path_str = path.to_string_lossy().into_owned();
        generate_project_ifc_with_lod(project, &path_str, LodLevel::Lod300).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        content
    }

    fn export_library_to_string(project: &Project) -> String {
        let path = std::env::temp_dir().join(format!(
            "ofs_ifc_library_test_{}_{}.ifc",
            std::process::id(),
            project.id,
        ));
        let path_str = path.to_string_lossy().into_owned();
        generate_ifc_type_library(project, &path_str).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        content
    }

    fn export_parts_to_string(project: &Project) -> String {
        let path = std::env::temp_dir().join(format!(
            "ofs_ifc_parts_test_{}_{}.ifc", std::process::id(), project.id,
        ));
        let path_str = path.to_string_lossy().into_owned();
        generate_ifc_parts_project(project, &path_str).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        content
    }

    /// Split the attribute list of a STEP entity line into top-level
    /// attributes, respecting quoted strings (with doubled apostrophes)
    /// and nested parenthesis groups.
    fn parse_step_attrs(line: &str) -> Vec<String> {
        let start = line.find('(').expect("entity heeft een attributenlijst");
        let end = line.rfind(')').expect("entity heeft een attributenlijst");
        let body = &line[start + 1..end];

        let mut attrs = Vec::new();
        let mut current = String::new();
        let mut depth = 0u32;
        let mut in_string = false;
        let mut chars = body.chars().peekable();
        while let Some(c) = chars.next() {
            if in_string {
                current.push(c);
                if c == '\'' {
                    if chars.peek() == Some(&'\'') {
                        // Doubled apostrophe: escaped quote, stay in string
                        current.push(chars.next().unwrap());
                    } else {
                        in_string = false;
                    }
                }
                continue;
            }
            match c {
                '\'' => {
                    in_string = true;
                    current.push(c);
                }
                '(' => {
                    depth += 1;
                    current.push(c);
                }
                ')' => {
                    depth -= 1;
                    current.push(c);
                }
                ',' if depth == 0 => {
                    attrs.push(current.clone());
                    current.clear();
                }
                _ => current.push(c),
            }
        }
        attrs.push(current);
        attrs
    }

    /// True when `id` (e.g. "#42") refers to an entity of the given class.
    fn entity_is(content: &str, id: &str, ifc_class: &str) -> bool {
        let needle = format!("{}={}(", id, ifc_class);
        content.lines().any(|l| l.starts_with(&needle))
    }

    #[test]
    fn window_record_matches_ifc4_layout() {
        let kozijn = Kozijn::new("Testkozijn", "K-01", 1200.0, 1500.0);
        let content = export_to_string(&kozijn);

        let line = content
            .lines()
            .find(|l| l.contains("=IFCWINDOW("))
            .expect("export hoort een IFCWINDOW-entity te bevatten");
        let attrs = parse_step_attrs(line);

        // IFC4 IfcWindow has 13 attributes
        assert_eq!(
            attrs.len(),
            13,
            "IfcWindow hoort 13 attributen te hebben: {}",
            line
        );

        // Attr 6 (ObjectPlacement) references an IFCLOCALPLACEMENT
        assert!(
            attrs[5].starts_with('#'),
            "ObjectPlacement hoort een referentie te zijn: {}",
            line
        );
        assert!(
            entity_is(&content, &attrs[5], "IFCLOCALPLACEMENT"),
            "ObjectPlacement {} hoort naar IFCLOCALPLACEMENT te verwijzen",
            attrs[5]
        );

        // Attr 7 (Representation) references an IFCPRODUCTDEFINITIONSHAPE
        assert!(
            attrs[6].starts_with('#'),
            "Representation hoort een referentie te zijn: {}",
            line
        );
        assert!(
            entity_is(&content, &attrs[6], "IFCPRODUCTDEFINITIONSHAPE"),
            "Representation {} hoort naar IFCPRODUCTDEFINITIONSHAPE te verwijzen",
            attrs[6]
        );

        // Attr 8 (Tag) carries the merkteken as a string
        assert_eq!(
            attrs[7], "'K-01'",
            "Tag hoort het merkteken te zijn: {}",
            line
        );

        // Attrs 9/10: OverallHeight / OverallWidth in meters
        assert_eq!(attrs[8], "1.500000", "OverallHeight klopt niet: {}", line);
        assert_eq!(attrs[9], "1.200000", "OverallWidth klopt niet: {}", line);
    }

    #[test]
    fn project_references_geometric_context() {
        let kozijn = Kozijn::new("Testkozijn", "K-02", 1000.0, 1000.0);
        let content = export_to_string(&kozijn);

        let line = content
            .lines()
            .find(|l| l.contains("=IFCPROJECT("))
            .expect("export hoort een IFCPROJECT-entity te bevatten");
        let attrs = parse_step_attrs(line);

        assert_eq!(
            attrs.len(),
            9,
            "IfcProject hoort 9 attributen te hebben: {}",
            line
        );

        // Attr 8 (RepresentationContexts) = (#N) with N a geometric context
        let contexts = &attrs[7];
        assert!(
            contexts.starts_with("(#") && contexts.ends_with(')'),
            "RepresentationContexts hoort een set met referentie te zijn: {}",
            line
        );
        let ctx_id = &contexts[1..contexts.len() - 1];
        assert!(
            entity_is(&content, ctx_id, "IFCGEOMETRICREPRESENTATIONCONTEXT"),
            "RepresentationContexts {} hoort naar IFCGEOMETRICREPRESENTATIONCONTEXT te verwijzen",
            ctx_id
        );

        // Attr 9 (UnitsInContext) is a reference
        assert!(
            attrs[8].starts_with('#'),
            "UnitsInContext hoort een referentie te zijn: {}",
            line
        );
    }

    #[test]
    fn user_strings_are_step_escaped() {
        let mut kozijn = Kozijn::new("Kozijn 's-Gravenhage", "K'03\\A", 1000.0, 1000.0);
        kozijn.frame.color_inside = "RAL9010 'zuiver wit'".into();
        let content = export_to_string(&kozijn);

        // Apostrophes doubled per ISO 10303-21
        assert!(
            content.contains("'Kozijn ''s-Gravenhage'"),
            "naam hoort ge-escaped te zijn"
        );
        // Backslash and apostrophe doubled (merkteken in the Tag attribute)
        assert!(
            content.contains("'K''03\\\\A'"),
            "merkteken hoort ge-escaped te zijn"
        );
        assert!(
            content.contains("IFCLABEL('RAL9010 ''zuiver wit''')"),
            "kleur hoort ge-escaped te zijn"
        );

        // The escaped strings must not break the record structure
        let line = content
            .lines()
            .find(|l| l.contains("=IFCWINDOW("))
            .expect("export hoort een IFCWINDOW-entity te bevatten");
        assert_eq!(parse_step_attrs(line).len(), 13);
    }

    #[test]
    fn unicode_names_use_step_control_directives() {
        let kozijn = Kozijn::new("René 😀", "K-É", 900.0, 1200.0);
        let content = export_to_string(&kozijn);
        assert!(content.contains("'Ren\\X2\\00E9\\X0\\ \\X4\\0001F600\\X0\\'"));
        assert!(content.contains("'K-\\X2\\00C9\\X0\\'"));
        assert!(!content.contains("René"));
        assert!(!content.contains("😀"));
    }

    #[test]
    fn global_ids_are_22_chars_within_spec_range() {
        let kozijn = Kozijn::new("Testkozijn", "K-04", 800.0, 1200.0);
        let content = export_to_string(&kozijn);

        // Entity classes rooted in IfcRoot (attribute 1 = GlobalId)
        let rooted = [
            "IFCPROJECT",
            "IFCSITE",
            "IFCBUILDING",
            "IFCBUILDINGSTOREY",
            "IFCWINDOW",
            "IFCDOOR",
            "IFCRELAGGREGATES",
            "IFCRELCONTAINEDINSPATIALSTRUCTURE",
            "IFCPROPERTYSET",
            "IFCRELDEFINESBYPROPERTIES",
        ];

        let mut checked = 0;
        for line in content.lines() {
            if !line.starts_with('#') {
                continue;
            }
            let eq = match line.find('=') {
                Some(i) => i,
                None => continue,
            };
            let class = line[eq + 1..].split('(').next().unwrap_or("");
            if !rooted.contains(&class) {
                continue;
            }
            let attrs = parse_step_attrs(line);
            let guid = attrs[0].trim_matches('\'');
            assert_eq!(guid.len(), 22, "GlobalId hoort 22 tekens te zijn: {}", line);
            let first = guid.chars().next().unwrap();
            assert!(
                matches!(first, '0'..='3'),
                "GlobalId hoort met teken 0-3 te beginnen: {}",
                line
            );
            checked += 1;
        }
        assert!(
            checked >= 10,
            "te weinig GlobalIds gecontroleerd: {}",
            checked
        );
    }

    #[test]
    fn export_is_deterministic() {
        let kozijn = Kozijn::new("Determinisme", "K-05", 900.0, 1200.0);
        let first = export_to_string(&kozijn);
        let second = export_to_string(&kozijn);
        assert_eq!(
            first, second,
            "twee exports van hetzelfde kozijn horen identiek te zijn"
        );
    }

    #[test]
    fn two_cells_have_two_glazing_solids_and_a_divider() {
        let mut kozijn = Kozijn::new("Twee vakken", "K-06", 1200.0, 1500.0);
        kozijn.add_column(500.0);
        kozijn.cells[1].glazing.thickness_mm = 36.0;
        let content = export_to_string(&kozijn);

        assert_eq!(content.matches("'Glazing'").count(), 2);
        assert_eq!(content.matches("'Divider'").count(), 1);
        assert!(content.contains("0.024000)"), "eerste glasdikte ontbreekt");
        assert!(content.contains("0.036000)"), "tweede glasdikte ontbreekt");
        assert!(content.contains("IFCPOSITIVERATIOMEASURE(0.7581)"));
        assert!(!content.contains("IFCARBITRARYPROFILEDEFWITHVOIDS"));
        assert!(!content.contains("'ThermalTransmittance'"));
        assert!(!content.contains("'Uw_kozijn_berekend'"));
        assert!(!content.contains("'DagmaatBreedte_mm'"));
        assert!(content.contains("'OFS_KozijnBeglazing'"));
        assert!(!content.contains("'ILS_KozijnBeglazing'"));
    }

    #[test]
    fn product_placement_uses_millimetres_and_vertical_window_axes() {
        let mut kozijn = Kozijn::new("Geplaatst", "K-07", 900.0, 1200.0);
        kozijn.placement.x = 1200.0;
        kozijn.placement.y = 2500.0;
        kozijn.placement.z = 900.0;
        kozijn.placement.orientation = 30.0;
        let content = export_to_string(&kozijn);

        assert!(content.contains("IFCCARTESIANPOINT((1.200000,2.500000,0.900000))"));
        assert!(content.contains("IFCDIRECTION((0.0,-1.0,0.0))"));
        assert!(content.contains("IFCDIRECTION((0.866025404,0.500000000,0.0))"));
    }

    #[test]
    fn aluminum_mitre_uses_member_polygons() {
        let mut kozijn = Kozijn::new("Aluminium", "K-09", 1200.0, 1500.0);
        kozijn.frame.material = Material::Aluminum;
        let content = export_to_string(&kozijn);
        assert_eq!(content.matches("'FrameMember'").count(), 4);
        assert_eq!(content.matches("'Glazing'").count(), 1);
    }

    #[test]
    fn unsupported_geometry_fails_before_creating_a_file() {
        let mut kozijn = Kozijn::new("Boog", "K-08", 900.0, 1200.0);
        kozijn.frame.shape.shape_type = ShapeType::Arched;
        let path = std::env::temp_dir().join(format!("ofs_rejected_{}.ifc", kozijn.id));
        let path_str = path.to_string_lossy().into_owned();
        let error = generate_ifc(&kozijn, &path_str).unwrap_err();
        assert!(error.contains("rechthoekige"));
        assert!(!path.exists());

        kozijn.frame.shape.shape_type = ShapeType::Rectangular;
        kozijn.cells[0].panel_type = PanelType::Door;
        let error = generate_ifc(&kozijn, &path_str).unwrap_err();
        assert!(error.contains("vaste beglazing"));
        assert!(!path.exists());

        let error = generate_ifc_with_lod(&kozijn, &path_str, LodLevel::Lod400).unwrap_err();
        assert!(error.contains("LOD400"));
        assert!(!path.exists());
    }

    #[test]
    fn coarse_door_export_does_not_invent_glazing_or_hardware() {
        let mut kozijn = Kozijn::new("Dichte deur", "D-01", 1000.0, 2100.0);
        kozijn.cells[0].panel_type = PanelType::Door;
        let path = std::env::temp_dir().join(format!("ofs_door_{}.ifc", kozijn.id));
        let path_str = path.to_string_lossy().into_owned();
        generate_ifc_with_lod(&kozijn, &path_str, LodLevel::Lod200).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        assert!(content.contains("=IFCDOOR("));
        assert!(!content.contains("'OFS_KozijnBeglazing'"));
        assert!(!content.contains("'GlazingAreaFraction'"));
        assert!(!content.contains("'Cel_1_Scharnieren'"));
    }

    #[test]
    fn project_export_has_one_hierarchy_and_two_stable_windows() {
        let mut project = Project::new("Woning 't Park", "P-42");
        let mut first = Kozijn::new("Voorgevel", "K-01", 1200.0, 1500.0);
        first.placement.x = 1000.0;
        first.series_id = Some("fase-a".into());
        let mut second = Kozijn::new("Achtergevel", "K-02", 900.0, 1200.0);
        second.placement.x = 3000.0;
        let first_guid = crate::ifc_guid::uuid_to_ifc_guid(&first.id);
        let second_guid = crate::ifc_guid::uuid_to_ifc_guid(&second.id);
        project.kozijnen = vec![first, second];

        let content = export_project_to_string(&project);
        assert_eq!(content.matches("=IFCPROJECT(").count(), 1);
        assert_eq!(content.matches("=IFCBUILDINGSTOREY(").count(), 1);
        assert_eq!(content.matches("=IFCWINDOW(").count(), 2);
        assert_eq!(
            content
                .matches("=IFCRELCONTAINEDINSPATIALSTRUCTURE(")
                .count(),
            1
        );
        assert!(content.contains("'Woning ''t Park'"));
        assert!(content.contains("'Projectnummer: P-42'"));
        assert!(content.contains("'Kozijnen'"));
        assert!(content.contains(&format!("'{}'", first_guid)));
        assert!(content.contains(&format!("'{}'", second_guid)));
        assert!(content.contains("'SeriesId'"));
        assert!(content.contains("'fase-a'"));
        assert!(content.contains("IFCCARTESIANPOINT((1.000000,0.000000,0.000000))"));
        assert!(content.contains("IFCCARTESIANPOINT((3.000000,0.000000,0.000000))"));

        project.kozijnen.reverse();
        assert_eq!(content, export_project_to_string(&project));

        let original_project_line = content
            .lines()
            .find(|line| line.contains("=IFCPROJECT("))
            .unwrap();
        let original_project_guid = parse_step_attrs(original_project_line)[0].clone();
        project.project_info.name = "Woning Park gewijzigd".into();
        let renamed = export_project_to_string(&project);
        let renamed_project_line = renamed
            .lines()
            .find(|line| line.contains("=IFCPROJECT("))
            .unwrap();
        assert_eq!(
            original_project_guid,
            parse_step_attrs(renamed_project_line)[0]
        );
    }

    #[test]
    fn project_export_preflights_every_element_before_writing() {
        let mut project = Project::new("Preflight", "P-43");
        project
            .kozijnen
            .push(Kozijn::new("Goed", "K-01", 1200.0, 1500.0));
        let mut bad = Kozijn::new("Boog", "K-02", 900.0, 1200.0);
        bad.frame.shape.shape_type = ShapeType::Arched;
        project.kozijnen.push(bad);
        let path = std::env::temp_dir().join(format!(
            "ofs_invalid_project_{}.ifc",
            project.kozijnen[0].id
        ));
        let path_str = path.to_string_lossy().into_owned();

        let error =
            generate_project_ifc_with_lod(&project, &path_str, LodLevel::Lod300).unwrap_err();
        assert!(error.contains("K-02"));
        assert!(!path.exists());

        project.kozijnen[1] = project.kozijnen[0].clone();
        let error =
            generate_project_ifc_with_lod(&project, &path_str, LodLevel::Lod300).unwrap_err();
        assert!(error.contains("dubbel kozijn-ID"));
        assert!(!path.exists());

        project.kozijnen.clear();
        let error =
            generate_project_ifc_with_lod(&project, &path_str, LodLevel::Lod300).unwrap_err();
        assert!(error.contains("geen kozijnen"));
        assert!(!path.exists());
    }

    #[test]
    fn type_library_has_unplaced_types_with_geometry_maps() {
        let mut project = Project::new("Typebibliotheek", "LIB-1");
        let mut first = Kozijn::new("Twee vakken", "T-01", 1200.0, 1500.0);
        first.add_column(500.0);
        first.placement.x = 4500.0;
        first.series_id = Some("serie-a".into());
        let second = Kozijn::new("Een vak", "T-02", 900.0, 1200.0);
        let occurrence_guid = crate::ifc_guid::uuid_to_ifc_guid(&first.id);
        project.kozijnen = vec![first, second];

        let content = export_library_to_string(&project);
        assert_eq!(content.matches("=IFCPROJECT(").count(), 1);
        assert_eq!(content.matches("=IFCPROJECTLIBRARY(").count(), 1);
        assert_eq!(content.matches("=IFCWINDOWTYPE(").count(), 2);
        assert_eq!(content.matches("=IFCREPRESENTATIONMAP(").count(), 2);
        assert_eq!(content.matches("=IFCRELDECLARES(").count(), 2);
        assert_eq!(content.matches("=IFCWINDOW(").count(), 0);
        assert_eq!(content.matches("=IFCLOCALPLACEMENT(").count(), 0);
        assert_eq!(content.matches("'FrameMember'").count(), 8);
        assert_eq!(content.matches("'Divider'").count(), 1);
        assert_eq!(content.matches("'Glazing'").count(), 3);
        assert!(content.contains("'OFS_KozijnType'"));
        assert!(content.contains("'SeriesId'"));
        assert!(!content.contains("IFCCARTESIANPOINT((4.500000,0.000000,0.000000))"));

        let first_type_line = content
            .lines()
            .find(|line| line.contains("'T-01'") && line.contains("=IFCWINDOWTYPE("))
            .unwrap();
        let attrs = parse_step_attrs(first_type_line);
        assert_eq!(attrs.len(), 13);
        assert_ne!(attrs[0], format!("'{}'", occurrence_guid));
        assert!(attrs[5].starts_with("(#")); // direct HasPropertySets
        assert!(attrs[6].starts_with("(#")); // RepresentationMaps
        assert_eq!(attrs[7], "'T-01'");
        assert_eq!(attrs[9], ".WINDOW.");
        assert_eq!(attrs[10], ".NOTDEFINED."); // two vakken

        project.kozijnen.reverse();
        assert_eq!(content, export_library_to_string(&project));
    }

    #[test]
    fn type_library_rejects_unsupported_cells_before_writing() {
        let mut project = Project::new("Ongeschikt", "LIB-2");
        project
            .kozijnen
            .push(Kozijn::new("Goed", "T-01", 900.0, 1200.0));
        let mut bad = Kozijn::new("Draaikiep", "T-02", 900.0, 1200.0);
        bad.cells[0].panel_type = PanelType::TurnTilt;
        project.kozijnen.push(bad);
        let path = std::env::temp_dir().join(format!("ofs_bad_library_{}.ifc", project.id));
        let path_str = path.to_string_lossy().into_owned();

        let error = generate_ifc_type_library(&project, &path_str).unwrap_err();
        assert!(error.contains("T-02"));
        assert!(!path.exists());
    }

    #[test]
    fn parts_export_aggregates_separate_materialized_elements_with_stable_ids() {
        let mut project = Project::new("Onderdelen", "P-44");
        let mut first = Kozijn::new("Twee vakken", "K-01", 1200.0, 1500.0);
        first.add_column(500.0);
        first.cells[1].glazing.thickness_mm = 36.0;
        let second = Kozijn::new("Een vak", "K-02", 900.0, 1200.0);
        project.kozijnen = vec![first, second];
        let content = export_parts_to_string(&project);
        assert_eq!(content.matches("=IFCELEMENTASSEMBLY(").count(), 2);
        assert_eq!(content.matches("=IFCMEMBER(").count(), 9);
        assert_eq!(content.matches("=IFCPLATE(").count(), 3);
        assert_eq!(content.matches("=IFCRELASSOCIATESMATERIAL(").count(), 4);
        assert_eq!(content.matches("=IFCRELCONTAINEDINSPATIALSTRUCTURE(").count(), 1);
        assert_eq!(content.matches("'OFS_Onderdeel'").count(), 12);
        assert!(!content.contains("=IFCWINDOW("));
        assert!(content.contains("'Glasdikte_mm'"));

        let mut part_guids = std::collections::HashSet::new();
        for line in content.lines().filter(|line| {
            line.contains("=IFCMEMBER(") || line.contains("=IFCPLATE(")
        }) {
            let attrs = parse_step_attrs(line);
            assert_eq!(attrs.len(), 9);
            assert!(part_guids.insert(attrs[0].clone()));
        }
        let mut reversed = project.clone();
        reversed.kozijnen.reverse();
        assert_eq!(content, export_parts_to_string(&reversed));
    }

    #[test]
    fn parts_export_keeps_existing_global_ids_after_new_column() {
        let mut project = Project::new("Revisie", "R-1");
        let mut kozijn = Kozijn::new("Drie vakken", "K-01", 1800.0, 1500.0);
        kozijn.add_column(550.0);
        let old_divider_guid = crate::ifc_guid::uuid_to_ifc_guid(&kozijn.vertical_part_id(0));
        let old_right_glass_guid = crate::ifc_guid::uuid_to_ifc_guid(&kozijn.glass_part_id(1));
        project.kozijnen.push(kozijn);
        let before = crate::import::ifc_import::parse_ifc_text(&export_parts_to_string(&project)).unwrap();
        assert!(before.assemblies[0].parts.iter().any(|p| p.guid == old_right_glass_guid));

        project.kozijnen[0].add_column(250.0);
        let after = crate::import::ifc_import::parse_ifc_text(&export_parts_to_string(&project)).unwrap();
        assert!(after.assemblies[0].parts.iter().any(|p| p.guid == old_divider_guid));
        assert!(after.assemblies[0].parts.iter().any(|p| p.guid == old_right_glass_guid));
        assert_eq!(after.assemblies[0].parts.len(), before.assemblies[0].parts.len() + 2);
    }

    #[test]
    fn parts_export_rejects_duplicate_persisted_part_ids() {
        let mut kozijn = Kozijn::new("Dubbel", "K-01", 1200.0, 1500.0);
        kozijn.add_column(500.0);
        let mut value = serde_json::to_value(&kozijn).unwrap();
        let ids = value["partIds"]["cells"].as_array_mut().unwrap();
        ids[1] = ids[0].clone();
        kozijn = serde_json::from_value(value).unwrap();
        let mut project = Project::new("Dubbel", "P-1");
        project.kozijnen.push(kozijn);
        let path = std::env::temp_dir().join(format!("ofs_parts_duplicate_{}.ifc", project.id));
        let error = generate_ifc_parts_project(&project, &path.to_string_lossy()).unwrap_err();
        assert!(error.contains("dubbel onderdeel-ID"));
        assert!(!path.exists());
    }

    #[test]
    fn parts_export_rejects_unsupported_project_before_file_creation() {
        let mut project = Project::new("Preflight", "P-45");
        project.kozijnen.push(Kozijn::new("Goed", "K-01", 900.0, 1200.0));
        let mut bad = Kozijn::new("Draaideel", "K-02", 900.0, 1200.0);
        bad.cells[0].panel_type = PanelType::Door;
        project.kozijnen.push(bad);
        let path = std::env::temp_dir().join(format!("ofs_parts_rejected_{}.ifc", project.id));
        let error = generate_ifc_parts_project(&project, &path.to_string_lossy()).unwrap_err();
        assert!(error.contains("vaste beglazing"));
        assert!(!path.exists());

        let mut tiny = Project::new("Ongeldige glasmaat", "P-46");
        tiny.kozijnen.push(Kozijn::new("Te smal", "K-03", 135.0, 300.0));
        let error = generate_ifc_parts_project(&tiny, &path.to_string_lossy()).unwrap_err();
        assert!(error.contains("ongeldige indicatieve"));
        assert!(!path.exists());
    }
}
