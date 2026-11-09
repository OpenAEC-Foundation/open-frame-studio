//! Make the same small IFC set for manual acceptance in BIM applications.
//! Usage: cargo run -p ofs-core --example ifc_acceptance_fixture -- <output-dir>

use ofs_core::export::ifc::{
    generate_ifc_parts_project, generate_ifc_type_library, generate_ifc_with_lod,
    generate_project_ifc_with_lod, LodLevel,
};
use ofs_core::export::manufacturer_handoff::generate_manufacturer_handoff;
use ofs_core::kozijn::{Kozijn, Project};
use std::path::Path;

fn path_string(dir: &Path, name: &str) -> Result<String, String> {
    dir.join(name)
        .into_os_string()
        .into_string()
        .map_err(|_| format!("Ongeldig uitvoerpad voor {name}"))
}

fn main() -> Result<(), String> {
    let output = std::env::args()
        .nth(1)
        .ok_or("Geef een uitvoermap mee: -- <output-dir>")?;
    let dir = Path::new(&output);
    std::fs::create_dir_all(dir).map_err(|e| format!("Kan uitvoermap niet maken: {e}"))?;

    let mut project = Project::new("IFC acceptatieproject", "OFS-QA-01");
    let mut front = Kozijn::new("Voorgevel René", "K-01", 1200.0, 1500.0);
    front.add_column(500.0);
    front.placement.x = 1000.0;
    let mut back = Kozijn::new("Achtergevel", "K-02", 900.0, 1200.0);
    back.placement.x = 3000.0;
    project.kozijnen = vec![front, back];

    generate_ifc_with_lod(
        &project.kozijnen[0],
        &path_string(dir, "01-los-kozijn.ifc")?,
        LodLevel::Lod300,
    )?;
    generate_project_ifc_with_lod(
        &project,
        &path_string(dir, "02-project.ifc")?,
        LodLevel::Lod300,
    )?;
    generate_ifc_parts_project(&project, &path_string(dir, "03-onderdelen.ifc")?)?;
    generate_ifc_type_library(&project, &path_string(dir, "04-typebibliotheek.ifc")?)?;
    let handoff = generate_manufacturer_handoff(&project, dir)?;
    let mut revised = project.clone();
    revised.kozijnen[0].add_column(250.0);
    generate_ifc_parts_project(&revised, &path_string(dir, "05-onderdelen-revisie.ifc")?)?;
    println!(
        "Vier IFC4-acceptatiebestanden geschreven naar {}",
        dir.display()
    );
    println!(
        "Indicatieve producentenoverdracht geschreven naar {}",
        handoff.display()
    );
    println!(
        "Revisie-IFC geschreven naar {}",
        dir.join("05-onderdelen-revisie.ifc").display()
    );
    Ok(())
}
