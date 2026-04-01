"""ILS Houten Kozijnen v2.0 property sets for IFC export.

Implements the Dutch BIM standard 'Informatie Leverings Specificatie (ILS)
Houten Kozijnen versie 2.0' property sets. These property sets ensure
interoperability with Dutch BIM workflows and kozijn manufacturers.

Reference: ILS Houten Kozijnen v2.0 (BIM Loket / SBR)
"""

import ifcopenshell.api


# ILS property set definitions based on ILS Houten Kozijnen v2.0
ILS_PANEL_TYPE_MAP = {
    "fixed_glass": "vast",
    "turn_tilt": "draai-kiep",
    "turn": "draai",
    "tilt": "kiep",
    "sliding": "schuif",
    "door": "deur",
    "panel": "paneel",
    "ventilation": "ventilatie",
}

ILS_MATERIAL_MAP = {
    "wood": "hout",
    "aluminum": "aluminium",
    "pvc": "kunststof",
    "wood_aluminum": "hout-aluminium",
}

ILS_OPENING_DIRECTION_MAP = {
    "left": "links",
    "right": "rechts",
    "inward": "naar binnen",
    "outward": "naar buiten",
}


def add_ils_property_sets(model, element, kozijn_data):
    """Add ILS Houten Kozijnen v2.0 compliant property sets.

    Adds the following property sets:
    - ILS_KozijnAlgemeen: General kozijn properties
    - ILS_KozijnAfmetingen: Dimensional properties
    - ILS_KozijnMateriaal: Material and finish properties
    - ILS_KozijnBeglazing: Glazing properties
    - ILS_KozijnOnderdelen: Component/cell properties
    - ILS_KozijnHangSluitwerk: Hardware properties
    - ILS_KozijnThermisch: Thermal performance
    """
    frame = kozijn_data.get("frame", {})
    grid = kozijn_data.get("grid", {})
    cells = kozijn_data.get("cells", [])

    # --- ILS_KozijnAlgemeen ---
    algemeen_props = {
        "Merkteken": kozijn_data.get("mark", ""),
        "Naam": kozijn_data.get("name", ""),
        "Type": _determine_kozijn_type(cells),
        "IsUitwendig": True,
        "Leverancier": frame.get("profile", {}).get("name", ""),
        "ProfielSysteem": frame.get("profile", {}).get("id", ""),
    }
    _add_pset(model, element, "ILS_KozijnAlgemeen", algemeen_props)

    # --- ILS_KozijnAfmetingen ---
    ow = frame.get("outerWidth", 1200)
    oh = frame.get("outerHeight", 1500)
    fw = frame.get("frameWidth", 67)
    fd = frame.get("frameDepth", 114)

    afmetingen_props = {
        "BuitenwerksBreedte_mm": float(ow),
        "BuitenwerksHoogte_mm": float(oh),
        "DagmaatBreedte_mm": float(ow - 4),  # 2mm speling per zijde
        "DagmaatHoogte_mm": float(oh - 2),   # 2mm speling boven
        "KozijnprofielBreedte_mm": float(fw),
        "KozijnprofielDiepte_mm": float(fd),
        "AantalKolommen": len(grid.get("columns", [1])),
        "AantalRijen": len(grid.get("rows", [1])),
        "BinnenwerksBreedte_mm": float(ow - 2 * fw),
        "BinnenwerksHoogte_mm": float(oh - 2 * fw),
    }

    # Add individual column widths
    columns = grid.get("columns", [])
    for i, col in enumerate(columns):
        afmetingen_props[f"KolomBreedte_{i+1}_mm"] = float(col.get("size", 0))

    # Add individual row heights
    rows = grid.get("rows", [])
    for i, row in enumerate(rows):
        afmetingen_props[f"RijHoogte_{i+1}_mm"] = float(row.get("size", 0))

    _add_pset(model, element, "ILS_KozijnAfmetingen", afmetingen_props)

    # --- ILS_KozijnMateriaal ---
    material = frame.get("material", {})
    material_str = ""
    if isinstance(material, dict):
        if "wood" in material:
            material_str = f"hout ({material['wood']})"
        elif "aluminum" in material:
            material_str = "aluminium"
        elif "pvc" in material:
            material_str = "kunststof"
        elif "woodAluminum" in material:
            material_str = "hout-aluminium"
    elif isinstance(material, str):
        material_str = ILS_MATERIAL_MAP.get(material, material)

    materiaal_props = {
        "Materiaal": material_str,
        "KleurBinnenzijde": frame.get("colorInside", "RAL9010"),
        "KleurBuitenzijde": frame.get("colorOutside", "RAL9010"),
        "Afwerking": "dekkend gelakt",
        "Houtsoort": _get_wood_type(material),
        "Duurzaamheidsklasse": _get_durability_class(material),
    }

    sill = frame.get("sillProfile", {})
    if sill:
        materiaal_props["DorpelType"] = sill.get("name", "")
        materiaal_props["DorpelBreedte_mm"] = float(sill.get("width", fw) if isinstance(sill, dict) else fw)

    _add_pset(model, element, "ILS_KozijnMateriaal", materiaal_props)

    # --- ILS_KozijnBeglazing ---
    glazing_props = {}
    glass_types = set()
    ug_values = []

    for i, cell in enumerate(cells):
        glazing = cell.get("glazing", {})
        glass_type = glazing.get("glassType", "HR++")
        glass_types.add(glass_type)

        ug = glazing.get("ugValue", 1.0)
        ug_values.append(ug)

        glazing_props[f"Cel_{i+1}_GlasType"] = glass_type
        glazing_props[f"Cel_{i+1}_GlasDikte_mm"] = float(glazing.get("thicknessMm", 24))
        glazing_props[f"Cel_{i+1}_UgWaarde"] = float(ug)

    glazing_props["GlasTypen"] = ", ".join(sorted(glass_types))
    if ug_values:
        glazing_props["UgWaarde_gemiddeld"] = sum(ug_values) / len(ug_values)
        glazing_props["UgWaarde_max"] = max(ug_values)

    _add_pset(model, element, "ILS_KozijnBeglazing", glazing_props)

    # --- ILS_KozijnOnderdelen ---
    onderdelen_props = {
        "AantalCellen": len(cells),
    }

    for i, cell in enumerate(cells):
        panel_type = cell.get("panelType", "fixed_glass")
        ils_type = ILS_PANEL_TYPE_MAP.get(panel_type, panel_type)
        onderdelen_props[f"Cel_{i+1}_Type"] = ils_type

        direction = cell.get("openingDirection")
        if direction:
            ils_dir = ILS_OPENING_DIRECTION_MAP.get(direction, direction)
            onderdelen_props[f"Cel_{i+1}_Draairichting"] = ils_dir

        # Determine if cell has a sash (raamvleugel)
        has_sash = panel_type in ("turn_tilt", "turn", "tilt", "sliding", "door")
        onderdelen_props[f"Cel_{i+1}_HeeftVleugel"] = has_sash

    _add_pset(model, element, "ILS_KozijnOnderdelen", onderdelen_props)

    # --- ILS_KozijnHangSluitwerk ---
    hardware_props = {}
    for i, cell in enumerate(cells):
        panel_type = cell.get("panelType", "fixed_glass")
        hw_set = cell.get("hardwareSet")

        if hw_set:
            # Read from structured HardwareSet (format_version 1.1+)
            security = hw_set.get("securityClass", "none")
            hardware_props[f"Cel_{i+1}_Beveiligingsklasse"] = security

            hinges = hw_set.get("hinges")
            if hinges:
                hardware_props[f"Cel_{i+1}_ScharnierType"] = hinges.get("hingeType", "")
                hardware_props[f"Cel_{i+1}_ScharnierAantal"] = hinges.get("count", 0)
                hardware_props[f"Cel_{i+1}_ScharnierZijde"] = hinges.get("side", "")
                hardware_props[f"Cel_{i+1}_ScharnierDraagkracht_kg"] = round(hinges.get("loadCapacityKg", 0), 1)

            handle = hw_set.get("handle")
            if handle:
                hardware_props[f"Cel_{i+1}_GreepType"] = handle.get("handleType", "")
                hardware_props[f"Cel_{i+1}_GreepZijde"] = handle.get("side", "")
                hardware_props[f"Cel_{i+1}_GreepHoogte_mm"] = handle.get("heightMm", 1050)
                hardware_props[f"Cel_{i+1}_GreepAfsluitbaar"] = handle.get("lockable", False)

            locking = hw_set.get("locking")
            if locking:
                hardware_props[f"Cel_{i+1}_SlotType"] = locking.get("lockType", "")
                hardware_props[f"Cel_{i+1}_Sluitpunten"] = locking.get("lockingPoints", 0)
                hardware_props[f"Cel_{i+1}_NokType"] = locking.get("camType", "")
                cylinder = locking.get("cylinder", "none")
                if cylinder != "none":
                    hardware_props[f"Cel_{i+1}_CilinderType"] = cylinder

            vent = hw_set.get("ventilation")
            if vent:
                hardware_props[f"Cel_{i+1}_VentilatieType"] = vent.get("ventType", "")
                hardware_props[f"Cel_{i+1}_VentilatieCapaciteit_dm3s"] = vent.get("capacityDm3s", 0)

            closer = hw_set.get("closer")
            if closer:
                hardware_props[f"Cel_{i+1}_DrangType"] = closer.get("closerType", "")
                hardware_props[f"Cel_{i+1}_DrangKlasse"] = closer.get("forceClass", 3)
        else:
            # Fallback: infer from panel type (format_version 1.0 backward compat)
            hw_list = cell.get("hardware", [])

            if panel_type in ("turn_tilt", "turn", "tilt"):
                hardware_props[f"Cel_{i+1}_Beslag"] = "draai-kiep beslag"
                hardware_props[f"Cel_{i+1}_Scharnieren"] = "verborgen"
                hardware_props[f"Cel_{i+1}_GreepType"] = "kruk"
                hardware_props[f"Cel_{i+1}_Sluitpunten"] = _calc_locking_points(cell, kozijn_data, i)
            elif panel_type == "door":
                hardware_props[f"Cel_{i+1}_Beslag"] = "deurbeslag"
                hardware_props[f"Cel_{i+1}_Scharnieren"] = "opleg 3-delig"
                hardware_props[f"Cel_{i+1}_GreepType"] = "kruk-kruk"
                hardware_props[f"Cel_{i+1}_Sluitpunten"] = 3
                hardware_props[f"Cel_{i+1}_SlotType"] = "meerpuntssluiting"
            elif panel_type == "sliding":
                hardware_props[f"Cel_{i+1}_Beslag"] = "schuifbeslag"
                hardware_props[f"Cel_{i+1}_GreepType"] = "inlaat greep"

            for hw in hw_list:
                if hw.get("type") == "ventilation":
                    hardware_props[f"Cel_{i+1}_VentilatieType"] = hw.get("subtype", "rooster")
                    hardware_props[f"Cel_{i+1}_VentilatieKleur"] = hw.get("color", "RAL9010")

    if hardware_props:
        _add_pset(model, element, "ILS_KozijnHangSluitwerk", hardware_props)

    # --- ILS_KozijnThermisch ---
    uf_frame = _get_uf_value(frame)
    ug_avg = sum(ug_values) / len(ug_values) if ug_values else 1.0

    # Simplified Uw calculation (frame area weighted)
    frame_area = 2 * (ow * fw + oh * fw - 4 * fw * fw) / 1e6  # m²
    glass_area = (ow - 2 * fw) * (oh - 2 * fw) / 1e6  # m²
    total_area = ow * oh / 1e6  # m²

    # Linear thermal transmittance (simplified)
    psi_g = 0.06  # W/(m·K) typical spacer bar
    glass_perimeter = 2 * ((ow - 2 * fw) + (oh - 2 * fw)) / 1000  # m

    if total_area > 0:
        uw = (uf_frame * frame_area + ug_avg * glass_area + psi_g * glass_perimeter) / total_area
    else:
        uw = 1.5

    thermisch_props = {
        "Uf_kozijnprofiel": uf_frame,
        "Ug_glas_gemiddeld": ug_avg,
        "Uw_kozijn_berekend": round(uw, 2),
        "Psi_glasrand": psi_g,
        "GlasOppervlak_m2": round(glass_area, 3),
        "KozijnOppervlak_m2": round(frame_area, 3),
        "TotaalOppervlak_m2": round(total_area, 3),
        "GlasFractie": round(glass_area / total_area, 3) if total_area > 0 else 0,
    }
    _add_pset(model, element, "ILS_KozijnThermisch", thermisch_props)


def _add_pset(model, element, pset_name, properties):
    """Helper to add a property set to an IFC element."""
    # Filter out None values
    clean_props = {k: v for k, v in properties.items() if v is not None}
    if not clean_props:
        return

    pset = ifcopenshell.api.run("pset.add_pset", model, product=element, name=pset_name)
    ifcopenshell.api.run("pset.edit_pset", model, pset=pset, properties=clean_props)


def _determine_kozijn_type(cells):
    """Determine the overall kozijn type for ILS classification."""
    types = set(c.get("panelType", "fixed_glass") for c in cells)

    if "door" in types:
        if "fixed_glass" in types or "turn_tilt" in types:
            return "deurkozijn met bovenlicht"
        return "deurkozijn"

    if "sliding" in types:
        return "schuifpui"

    if "turn_tilt" in types or "turn" in types or "tilt" in types:
        if "fixed_glass" in types:
            return "raamkozijn met vast glas"
        if len(cells) > 1:
            return "raamkozijn meerdelig"
        return "raamkozijn"

    if all(c.get("panelType") == "fixed_glass" for c in cells):
        return "vast kozijn"

    if "panel" in types:
        return "paneelkozijn"

    return "kozijn"


def _get_wood_type(material):
    """Extract wood type from material specification."""
    if isinstance(material, dict):
        wood = material.get("wood", "")
        wood_types = {
            "meranti": "Meranti (Dark Red Meranti)",
            "accoya": "Accoya (geacetyleerd hout)",
            "fsc_hardhout": "FSC Hardhout",
            "naaldhout": "Naaldhout (vuren/grenen)",
        }
        return wood_types.get(wood, wood)
    return ""


def _get_durability_class(material):
    """Determine NEN-EN 350 durability class."""
    if isinstance(material, dict):
        wood = material.get("wood", "")
        classes = {
            "meranti": "Klasse 2 (duurzaam)",
            "accoya": "Klasse 1 (zeer duurzaam)",
            "fsc_hardhout": "Klasse 1-2",
            "naaldhout": "Klasse 4-5 (niet duurzaam, verduurzaming nodig)",
        }
        return classes.get(wood, "")
    return ""


def _get_uf_value(frame):
    """Get thermal transmittance of the frame profile."""
    profile = frame.get("profile", {})
    if isinstance(profile, dict):
        return profile.get("ufValue", 1.8)
    return 1.8


def _calc_locking_points(cell, kozijn_data, cell_index):
    """Calculate recommended number of locking points based on cell size."""
    grid = kozijn_data.get("grid", {})
    columns = grid.get("columns", [])
    rows = grid.get("rows", [])

    num_cols = len(columns)
    col_idx = cell_index % num_cols
    row_idx = cell_index // num_cols

    cell_w = columns[col_idx].get("size", 500) if col_idx < len(columns) else 500
    cell_h = rows[row_idx].get("size", 800) if row_idx < len(rows) else 800

    perimeter = 2 * (cell_w + cell_h)

    # Rule of thumb: 1 locking point per ~400mm of perimeter
    points = max(2, round(perimeter / 400))
    return points
