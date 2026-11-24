//! Export modules — replaces Python sidecar for file generation.

pub mod gltf;
pub mod dxf;
pub mod ifc;
pub mod csv_production;
pub mod manufacturer_handoff;
mod arch_joints;

#[cfg(feature = "export")]
pub mod xlsx;
#[cfg(feature = "export")]
pub mod pdf;
#[cfg(feature = "export")]
pub mod pdf_labels;
#[cfg(feature = "export")]
pub mod pdf_quotation;

#[cfg(all(test, feature = "export", feature = "import"))]
mod tests {
    use super::*;
    use crate::kozijn::{Kozijn, ShapeType};
    use crate::production::compute_production_data;
    use calamine::{open_workbook_auto, Reader};
    use std::fs;

    #[test]
    fn sash_cut_exports_name_each_member_for_its_material() {
        let data = [
            ("H", "standaard-67-meranti"), ("K", "kunststof-82-veka"), ("A", "aluminium-77-reynaers"),
        ].map(|(mark, id)| {
            let template = crate::template::get_sjabloon(id);
            let mut k = crate::grid::template_single_turn_tilt_sj(1200., 1500., &template);
            k.mark = mark.into();
            compute_production_data(&k)
        });
        let dir = std::env::temp_dir().join(format!("ofs-vleugelbenamingen-{}", uuid::Uuid::new_v4()));
        csv_production::generate_production_csv(&data, dir.to_str().unwrap()).unwrap();
        let csv = fs::read_to_string(dir.join("kortlijst.csv")).unwrap();
        let xlsx_path = dir.join("productie.xlsx");
        xlsx::generate_production_xlsx(&data, xlsx_path.to_str().unwrap()).unwrap();
        let sheet = open_workbook_auto(xlsx_path).unwrap().worksheet_range("Kortlijst").unwrap();
        let pdf_path = dir.join("productie.pdf");
        pdf::generate_production_pdf(&data, pdf_path.to_str().unwrap()).unwrap();
        let doc = lopdf::Document::load(pdf_path).unwrap();
        let text = doc.extract_text(&doc.get_pages().into_keys().collect::<Vec<_>>()).unwrap();
        for (suffix, timber, system) in [
            ("RSL", "Raamhout links", "Vleugelstijl links"),
            ("RSR", "Raamhout rechts", "Vleugelstijl rechts"),
            ("RDB", "Raamhout boven", "Bovenregel vleugel"),
            ("RDO", "Raamhout onder", "Onderregel vleugel"),
        ] {
            for (mark, label) in [("H", timber), ("K", system), ("A", system)] {
                let id = format!("{mark}-V1-{suffix}");
                let row = csv.lines().map(|line| line.split(';').collect::<Vec<_>>())
                    .find(|row| row[1] == id).unwrap();
                assert_eq!(row[2], label);
                let row = sheet.rows().find(|r| r[1].to_string() == id).unwrap();
                assert_eq!(row[2].to_string(), label);
                assert!(text.contains(label), "PDF mist {label}");
            }
        }
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn system_seal_and_hardware_lists_export_physical_positions_and_measured_lengths() {
        let mut k = Kozijn::new("MD", "V1", 1200.0, 1500.0);
        k.frame.material = crate::kozijn::Material::Pvc;
        k.frame.profile = crate::profile::ProfileRef { id: "veka-softline82".into(), name: "Softline 82 MD".into() };
        k.frame.frame_width = 73.0;
        k.grid.columns[0].size = k.inner_width();
        k.grid.rows[0].size = k.inner_height();
        k.cells[0].panel_type = crate::kozijn::PanelType::TurnTilt;
        let prod = [compute_production_data(&k)];
        let dir = std::env::temp_dir().join(format!("ofs-system-{}", uuid::Uuid::new_v4()));
        csv_production::generate_production_csv(&prod, dir.to_str().unwrap()).unwrap();
        let seals = fs::read_to_string(dir.join("rubberlijst.csv")).unwrap();
        assert!(seals.contains("Middendichting;5080.0;1;V1-V1;Softline 82 MD;Buitenomtrek vleugel"));
        let hardware = fs::read_to_string(dir.join("beslaglijst.csv")).unwrap();
        assert!(hardware.lines().any(|s| s.contains(";Sluitplaat;") && s.ends_with(";V1-V1")));
        let book_path = dir.join("productie.xlsx");
        xlsx::generate_production_xlsx(&prod, book_path.to_str().unwrap()).unwrap();
        let mut book = open_workbook_auto(&book_path).unwrap();
        let rubber = book.worksheet_range("Rubberlijst").unwrap();
        let centre = rubber.rows().find(|r| r[1].to_string() == "Middendichting").unwrap();
        assert_eq!((centre[2].to_string(), centre[4].to_string(), centre[6].to_string()),
            ("5080".into(), "V1-V1".into(), "Buitenomtrek vleugel".into()));
        let parts = book.worksheet_range("Beslaglijst").unwrap();
        assert!(parts.rows().any(|r| r[2].to_string() == "Sluitplaat" && r[5].to_string() == "V1-V1"));
        let pdf_path = dir.join("productie.pdf");
        pdf::generate_production_pdf(&prod, pdf_path.to_str().unwrap()).unwrap();
        let doc = lopdf::Document::load(pdf_path).unwrap();
        let text = doc.extract_text(&doc.get_pages().into_keys().collect::<Vec<_>>()).unwrap();
        for label in ["Rubberlijst", "Netto modelomtrek", "Middendichting", "5080.0", "Beslag per vleugel", "Sluitplaat"] {
            assert!(text.contains(label), "{label}");
        }
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn arched_system_exports_keep_fractional_sizes_for_straight_and_curved_panes() {
        let sj = crate::template::get_sjabloon("kunststof-82-veka");
        let mut k = Kozijn::new_with_sjabloon("Boog met onderlichten", "K02", 1200., 1500., &sj);
        k.frame.shape.shape_type = ShapeType::Arched;
        k.frame.shape.arch_height = Some(300.);
        k.add_divider_in_elevation(false, 520.5);
        k.add_divider_in_elevation(true, 650.4);
        let lists = [compute_production_data(&k)];
        let dir = std::env::temp_dir().join(format!("ofs-boogvakken-{}", uuid::Uuid::new_v4()));
        csv_production::generate_production_csv(&lists, dir.to_str().unwrap()).unwrap();
        let csv = fs::read_to_string(dir.join("glaslijst.csv")).unwrap();
        let book = dir.join("lijsten.xlsx");
        xlsx::generate_production_xlsx(&lists, book.to_str().unwrap()).unwrap();
        let sheet = open_workbook_auto(book).unwrap().worksheet_range("Glaslijst").unwrap();
        let path = dir.join("lijsten.pdf");
        pdf::generate_production_pdf(&lists, path.to_str().unwrap()).unwrap();
        let pdf = lopdf::Document::load(path).unwrap();
        let text = pdf.extract_text(&pdf.get_pages().into_keys().collect::<Vec<_>>()).unwrap();
        for (id, width, height, shape) in [
            ("K02-V1", "440.5", "562.2", "Boog R697.0"),
            ("K02-V4", "599.5", "769.6", "Rechthoek"),
        ] {
            let row = csv.lines().map(|s| s.split(';').collect::<Vec<_>>())
                .find(|r| r[1] == id).unwrap();
            assert_eq!((&row[3..5], row[9]), ([width, height].as_slice(), shape));
            let row = sheet.rows().find(|r| r[1].to_string() == id).unwrap();
            assert_eq!([row[3].to_string(), row[4].to_string(), row[9].to_string()], [width, height, shape]);
            assert!(text.contains(width) && text.contains(height) && text.contains(shape));
        }
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn production_files_include_the_formed_panes_cutting_instructions() {
        let mut frame = Kozijn::new("Getoogd", "A1", 1200.0, 1500.0);
        frame.frame.shape.shape_type = ShapeType::Arched;
        frame.frame.shape.arch_height = Some(300.0);
        let prod = [compute_production_data(&frame)];
        let dir = std::env::temp_dir().join(format!("ofs-arch-{}", uuid::Uuid::new_v4()));
        csv_production::generate_production_csv(&prod, dir.to_str().unwrap()).unwrap();
        let csv = fs::read_to_string(dir.join("glaslijst.csv")).unwrap();
        assert!(csv.contains(";1090.0;1390.0;"));
        assert!(csv.contains(";Boog R695.0;"));
        assert!(csv.contains("R695.0 M(545.0,695.0)"));
        let cut_csv = fs::read_to_string(dir.join("kortlijst.csv")).unwrap();
        let head: Vec<_> = cut_csv.lines().find(|l| l.contains(";A1-BO1;")).unwrap().split(';').collect();
        assert_eq!(head[2], "Boogstuk boven");
        assert_eq!(&head[5..12], ["1390.9", "1394.9", "72.01", "72.01", "1", "750.0", "1390.9"]);
        assert_eq!(head[14], "106.26");

        let xlsx_path = dir.join("productie.xlsx");
        xlsx::generate_production_xlsx(&prod, xlsx_path.to_str().unwrap()).unwrap();
        let sheet = open_workbook_auto(&xlsx_path).unwrap().worksheet_range("Glaslijst").unwrap();
        assert_eq!(sheet[(1, 9)].to_string(), "Boog R695.0");
        assert!(sheet[(1, 10)].to_string().contains("R695.0 M(545.0,695.0)"));
        let cuts = open_workbook_auto(&xlsx_path).unwrap().worksheet_range("Kortlijst").unwrap();
        assert_eq!(cuts[(1, 10)].to_string(), "750");
        assert!((cuts[(1, 7)].to_string().parse::<f64>().unwrap() - 72.01298753).abs() < 1e-6);

        let pdf_path = dir.join("productie.pdf");
        pdf::generate_production_pdf(&prod, pdf_path.to_str().unwrap()).unwrap();
        let bytes = fs::read(&pdf_path).unwrap();
        let pdf = printpdf::PdfDocument::parse(&bytes, &Default::default(), &mut vec![]).unwrap();
        assert_eq!(pdf.pages.len(), 2);
        assert!(!pdf.pages[1].ops.is_empty());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn rejected_system_glass_is_marked_in_csv_xlsx_and_pdf() {
        let mut frame = Kozijn::new("VEKA", "V80", 900.0, 1400.0);
        frame.frame.material = crate::kozijn::Material::Pvc;
        let snapshot = crate::glazing_fit::catalog_snapshot(
            "veka-softline82",
            &crate::kozijn::Material::Pvc,
        )
        .unwrap();
        let snapshot_json = serde_json::to_string(snapshot).unwrap();
        frame
            .apply_frame_profile(
                crate::profile::ProfileRef {
                    id: "veka-softline82".into(),
                    name: "VEKA Softline 82 MD".into(),
                },
                Some(73.0),
                Some(82.0),
                Some(&snapshot_json),
                None,
            )
            .unwrap();
        frame.cells[0].glazing.thickness_mm = 80.0;
        let production = [compute_production_data(&frame)];
        let glass = &production[0].glass_list[0];
        assert!(!glass.production_ready);

        let dir = std::env::temp_dir().join(format!("ofs-rejected-glass-{}", uuid::Uuid::new_v4()));
        csv_production::generate_production_csv(&production, dir.to_str().unwrap()).unwrap();
        let csv = fs::read_to_string(dir.join("glaslijst.csv")).unwrap();
        let row = csv
            .lines()
            .find(|line| line.contains(&glass.piece_id))
            .expect("afgekeurde ruit blijft herkenbaar in de glaslijst");
        assert!(row.contains(";Nee;"));
        assert!(row.contains("52"));

        let xlsx_path = dir.join("productie.xlsx");
        xlsx::generate_production_xlsx(&production, xlsx_path.to_str().unwrap()).unwrap();
        let mut workbook = open_workbook_auto(&xlsx_path).unwrap();
        let sheet = workbook.worksheet_range("Glaslijst").unwrap();
        assert_eq!(sheet[(1, 9)].to_string(), "Nee");
        assert!(sheet[(1, 10)].to_string().contains("52"));

        let pdf_path = dir.join("productie.pdf");
        pdf::generate_production_pdf(&production, pdf_path.to_str().unwrap()).unwrap();
        let document = lopdf::Document::load(&pdf_path).unwrap();
        let text = document
            .extract_text(&document.get_pages().into_keys().collect::<Vec<_>>())
            .unwrap();
        assert!(text.contains("AFGEKEURD"));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn revised_round_system_exports_head_geometry_and_identifies_missing_machining_data() {
        let pvc = crate::template::get_sjabloon("kunststof-82-veka");
        let alu = crate::template::get_sjabloon("aluminium-77-reynaers");
        let mut k = Kozijn::new_with_sjabloon("Rond kruis", "P", 1200., 1200., &pvc);
        k.frame.shape.shape_type = ShapeType::Round;
        k.add_divider_in_elevation(false, 600.);
        k.add_divider_in_elevation(true, 600.);
        let pvc_data = compute_production_data(&k);
        k.apply_frame_profile(alu.stijl_profile.clone(), Some(51.), Some(68.), None, Some(r#""aluminum""#)).unwrap();
        k.mark = "A".into();
        let data = [pvc_data, compute_production_data(&k)];
        let dir = std::env::temp_dir().join(format!("ofs-ronde-t-profielen-{}", uuid::Uuid::new_v4()));
        csv_production::generate_production_csv(&data, dir.to_str().unwrap()).unwrap();
        let csv = fs::read_to_string(dir.join("kortlijst.csv")).unwrap();
        let path = dir.join("productie.xlsx");
        xlsx::generate_production_xlsx(&data, path.to_str().unwrap()).unwrap();
        let mut book = open_workbook_auto(&path).unwrap();
        let sheet = book.worksheet_range("Kortlijst").unwrap();
        let path = dir.join("productie.pdf");
        pdf::generate_production_pdf(&data, path.to_str().unwrap()).unwrap();
        let doc = lopdf::Document::load(&path).unwrap();
        let pdf = doc.extract_text(&doc.get_pages().into_keys().collect::<Vec<_>>()).unwrap();
        for (mark, system, radius, half_face) in [("P", "VEKA", 527., 47.), ("A", "Reynaers", 549., 38.)] {
            for (suffix, net, cope) in [("TS1", 2.0 * radius, 15), ("TD1.1", radius - half_face, 15),
                                        ("TD1.2", radius - half_face, 16)] {
                let id = format!("{mark}-{suffix}");
                let row = csv.lines().map(|l| l.split(';').collect::<Vec<_>>()).find(|r| r[1] == id).unwrap();
                assert!(row[3].contains(system));
                assert_eq!(row[5], format!("{net:.1}"));
                assert!(row[6].is_empty()); // The curved counterprofile has no confirmed blank length.
                assert!(row[10..15].iter().all(|col| col.is_empty())); // No developed length for straight stock.
                assert_eq!(row[cope], format!("{radius:.1}"));
                assert!(row[if cope == 15 { 19 } else { 22 }].contains(&format!("R{radius:.1}")));
                let xl = sheet.rows().find(|r| r[1].to_string() == id).unwrap();
                assert_eq!(xl[3].to_string(), row[3]);
                assert!(xl[6].to_string().is_empty());
                let status = row.len() - 3 + (cope - 15);
                assert_eq!(row[status], "Fabrieksblad boogverbinding vereist");
                assert_eq!(xl[status].to_string(), row[status]);
                for (col, expected) in [(5, net), (cope, radius)] {
                    let actual = xl[col].to_string().parse::<f64>().unwrap();
                    assert!((actual - expected).abs() < 1e-6);
                }
                assert!(pdf.contains(&id));
            }
        }
        assert!(pdf.contains("Kopprofielen van tussenstijlen en tussendorpels"));
        assert!(pdf.contains("Fabrieksblad boogverbinding") && pdf.contains("vereist"));
        assert!(csv.lines().next().unwrap().contains("Kopradius_begin_in_aanzicht_mm"));
        assert!(!book.sheet_names().iter().any(|name| name == "Pen en gat"));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn veka_standard_straight_heads_export_glm_without_kerf_and_gross_with_kerf() {
        let sj = crate::template::get_sjabloon("kunststof-82-veka");
        let mut frame = Kozijn::new_with_sjabloon("Getoogd", "K", 1200., 1500., &sj);
        frame.frame.shape.shape_type = ShapeType::Arched;
        frame.frame.shape.arch_height = Some(300.);
        frame.add_divider_in_elevation(false, 600.);
        frame.add_divider_in_elevation(true, 800.);
        let lists = [compute_production_data(&frame)];
        let p = &lists[0];
        let cut = |id: &str| p.cut_list.iter().find(|r| r.piece_id == id).unwrap();
        for id in ["K-TD1.1", "K-TD1.2"] {
            let part = cut(id);
            assert_eq!(part.net_length_mm, 480.);
            assert_eq!(part.system_saw_cut_mm(), Some(530.));
            assert_eq!(p.blank_length_mm(part), Some(534.));
        }
        let stile = cut("K-TS1");
        assert_eq!(stile.divider_end.as_ref().unwrap().saw_allowance_mm, Some(25.));
        assert!(p.blank_length_mm(stile).is_none() && stile.system_saw_cut_mm().is_none());
        let dir = std::env::temp_dir().join(format!("ofs-standaard-glm-{}", uuid::Uuid::new_v4()));
        csv_production::generate_production_csv(&lists, dir.to_str().unwrap()).unwrap();
        let csv = fs::read_to_string(dir.join("kortlijst.csv")).unwrap();
        let rail: Vec<_> = csv.lines().find(|row| row.contains(";K-TD1.1;")).unwrap().split(';').collect();
        assert_eq!(&rail[5..7], ["480.0", "534.0"]);
        assert_eq!(rail.last().copied(), Some("530.0"));
        assert!(rail[rail.len()-2].contains("GLM +25"));
        let path = dir.join("lijsten.xlsx");
        xlsx::generate_production_xlsx(&lists, path.to_str().unwrap()).unwrap();
        let sheet = open_workbook_auto(path).unwrap().worksheet_range("Kortlijst").unwrap();
        let rail = sheet.rows().find(|r| r[1].to_string() == "K-TD1.1").unwrap();
        assert_eq!(rail.last().unwrap().to_string(), "530");
        let path = dir.join("lijsten.pdf");
        pdf::generate_production_pdf(&lists, path.to_str().unwrap()).unwrap();
        let doc = lopdf::Document::load(path).unwrap();
        let text = doc.extract_text(&doc.get_pages().into_keys().collect::<Vec<_>>()).unwrap();
        assert!(text.contains("530.0") && text.contains("534.0") && text.contains("Bewerking T-profielen"));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn workshop_pdf_prints_grid_and_local_dividers_with_labels_in_elevation_order() {
        use crate::kozijn::PanelType;
        use crate::layout::{glas, paneel, raam, split_col, split_row, VakChild};

        let mut grid = Kozijn::new("Raster", "G", 1200., 1200.);
        grid.add_column(380.);
        grid.add_row(410.);
        grid.cells[0].panel_type = PanelType::TurnTilt;
        grid.cells[2].panel_type = PanelType::Panel;

        let mut local = Kozijn::new("Plaatselijke tussenstijl", "L", 1200., 1200.);
        local.layout = Some(split_col(vec![
            VakChild { size: 410., node: split_row(vec![
                VakChild { size: 380., node: raam("draaikiep") },
                VakChild { size: 619., node: glas() },
            ]) },
            VakChild { size: 589., node: paneel() },
        ]));

        for (frame, top_label, bottom_label) in [
            (grid, "V1  DK", "V3  P"), (local, "V1  Raam", "V3  PN"),
        ] {
            let path = std::env::temp_dir().join(format!("ofs-werkplaats-{}.pdf", uuid::Uuid::new_v4()));
            pdf::generate_workshop_pdf(&frame, &crate::kozijn::Project::new("Afdrukproef", ""), path.to_str().unwrap()).unwrap();
            let doc = lopdf::Document::load(&path).unwrap();
            fs::remove_file(path).unwrap();
            let content = doc.get_page_content(doc.get_pages()[&1]).unwrap();
            let ops = lopdf::content::Content::decode(&content).unwrap().operations;
            let coords = |op: &lopdf::content::Operation| {
                [f64::from(op.operands[0].as_float().unwrap()), f64::from(op.operands[1].as_float().unwrap())]
            };
            let mut path = Vec::new();
            let mut filled = Vec::new();
            let mut cursor = [0.; 2];
            let mut captions = std::collections::HashMap::new();
            for op in &ops {
                match op.operator.as_str() {
                    "m" => { path.clear(); path.push(coords(op)); }
                    "l" => path.push(coords(op)),
                    "B" | "b" | "f" | "f*" => filled.push(std::mem::take(&mut path)),
                    "Td" => cursor = coords(op),
                    "Tj" => { captions.insert(op.operands[0].as_str().unwrap(), cursor); }
                    _ => {}
                }
            }
            // A 1200 mm square fits at 1:8 on the A3 elevation centred at (210, 168.5) mm.
            let pt = 72. / 25.4;
            let to_pdf = |x, y| [(210. + (x - 600.) / 8.) * pt, (168.5 - (y - 600.) / 8.) * pt];
            let near = |a: &[f64; 2], b: &[f64; 2]| (a[0]-b[0]).abs() < 0.02 && (a[1]-b[1]).abs() < 0.02;
            let geometry = crate::geometry::compute_2d_geometry(&frame);
            assert_eq!((geometry.h_dividers.len(), geometry.v_dividers.len()), (1, 1));
            for r in geometry.h_dividers.iter().chain(&geometry.v_dividers) {
                let corners = [to_pdf(r.x, r.y), to_pdf(r.x + r.width, r.y),
                    to_pdf(r.x + r.width, r.y + r.height), to_pdf(r.x, r.y + r.height)];
                assert!(filled.iter().any(|ring| ring.len() == 4 &&
                    corners.iter().all(|corner| ring.iter().any(|point| near(point, corner)))),
                    "{}: divider missing from filled PDF paths: {r:?}", frame.mark);
            }
            let top = captions[top_label.as_bytes()];
            let bottom = captions[bottom_label.as_bytes()];
            let transom = &geometry.h_dividers[0];
            assert!(top[1] > to_pdf(0., transom.y)[1]);
            assert!(bottom[1] < to_pdf(0., transom.y + transom.height)[1]);
        }
    }

    #[test]
    fn workshop_pdf_draws_the_head_intersections_at_print_scale() {
        let sj = crate::template::get_sjabloon("kunststof-82-veka");
        let mut k = Kozijn::new_with_sjabloon("Rond", "R", 1200., 1200., &sj);
        k.frame.shape.shape_type = ShapeType::Round;
        k.add_divider_in_elevation(false, 455.);
        k.add_divider_in_elevation(true, 480.);
        let path = std::env::temp_dir().join(format!("ofs-boogkop-{}.pdf", uuid::Uuid::new_v4()));
        pdf::generate_workshop_pdf(&k, &crate::kozijn::Project::new("Kopproef", "T"), path.to_str().unwrap()).unwrap();
        let doc = lopdf::Document::load(&path).unwrap();
        let page = doc.get_pages()[&1];
        let stream = lopdf::content::Content::decode(&doc.get_page_content(page).unwrap()).unwrap();
        let vertices: Vec<_> = stream.operations.iter().filter(|op| matches!(op.operator.as_str(), "m" | "l"))
            .map(|op| [op.operands[0].as_float().unwrap(), op.operands[1].as_float().unwrap()])
            .collect();
        let radius: f64 = 600. - 73.;
        let half: f64 = 94. / 2.;
        let x = 455. - half;
        let y = 480. - half;
        for (x, y) in [(x, 600. - (radius*radius - (600.-x).powi(2)).sqrt()),
                       (600. - (radius*radius - (600.-y).powi(2)).sqrt(), y)] {
            // The A3 drawing is centred at (210, 168.5) mm at a scale of 1:8.
            let pdf_x = (210. + (x-600.)/8.)*72./25.4;
            let pdf_y = (168.5 - (y-600.)/8.)*72./25.4;
            assert!(vertices.iter().any(|v| (f64::from(v[0])-pdf_x).abs()<0.02 &&
                (f64::from(v[1])-pdf_y).abs()<0.02), "missing end point ({x}, {y}) on printed contour");
        }
        let text = doc.extract_text(&[1]).unwrap();
        assert!(text.contains("Koppen in aanzicht op de daglijn") && text.contains("1:8"));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn arch_spring_exports_the_flat_shoulder_and_the_cope_with_its_chord_angle() {
        let mut frame = Kozijn::new("Toog met hoge tussendorpel", "B1", 1200.0, 1500.0);
        frame.frame.shape.shape_type = ShapeType::Arched;
        frame.frame.shape.arch_height = Some(300.0);
        use crate::layout::{glas, split_col, VakChild};
        frame.layout = Some(split_col(vec![
            VakChild { size: 233.0, node: glas() },
            VakChild { size: 1066.0, node: glas() },
        ]));
        let prod = [compute_production_data(&frame)];
        let td = prod[0].cut_list.iter().find(|i| i.member_type == crate::production::MemberType::DividerH).unwrap();
        let offset = 533.0 - (683.0_f64.powi(2) - 450.0_f64.powi(2)).sqrt();
        let angle = 67.0_f64.atan2(offset).to_degrees();
        let end = td.divider_start.as_ref().unwrap();
        assert_eq!(end.segments.len(), 2);
        assert_eq!(end.radius_mm, Some(683.0));
        assert!((end.angle_deg - angle).abs() < 1e-7);
        assert!((td.net_length_mm - 1066.0).abs() < 1e-7);

        let dir = std::env::temp_dir().join(format!("ofs-divider-{}", uuid::Uuid::new_v4()));
        csv_production::generate_production_csv(&prod, dir.to_str().unwrap()).unwrap();
        let cut_csv = fs::read_to_string(dir.join("kortlijst.csv")).unwrap();
        let row: Vec<_> = cut_csv.lines().find(|l| l.split(';').nth(1) == Some(&td.piece_id)).unwrap().split(';').collect();
        assert_eq!(row[7], format!("{angle:.2}"));
        assert_eq!(row[8], row[7]);
        assert_eq!(&row[15..17], ["683.0", "683.0"]);
        assert_eq!(row[17], format!("{offset:.1}"));
        assert_eq!(row[18], "0.0");
        assert!(row[19].contains("R683.0 M(600.0,750.0)") && row[19].contains("L ("));
        assert_ne!(row[19], row[22]);

        let xlsx_path = dir.join("tussenhout.xlsx");
        xlsx::generate_production_xlsx(&prod, xlsx_path.to_str().unwrap()).unwrap();
        let sheet = open_workbook_auto(&xlsx_path).unwrap().worksheet_range("Kortlijst").unwrap();
        let idx = prod[0].cut_list.iter().position(|i| i.piece_id == td.piece_id).unwrap() + 1;
        assert!((sheet[(idx, 7)].to_string().parse::<f64>().unwrap() - angle).abs() < 1e-7);
        assert_eq!(sheet[(idx, 15)].to_string(), "683");
        assert!(sheet[(idx, 22)].to_string().contains("R683.0 M(600.0,750.0)"));

        let pdf_path = dir.join("tussenhout.pdf");
        pdf::generate_production_pdf(&prod, pdf_path.to_str().unwrap()).unwrap();
        let pdf = lopdf::Document::load(pdf_path).unwrap();
        let text = pdf.extract_text(&pdf.get_pages().into_keys().collect::<Vec<_>>()).unwrap();
        assert!(text.contains("Kopprofielen van tussenstijlen en tussendorpels") && text.contains("Middelpunt X, Y"));
        assert!(text.contains(&format!("{angle:.2} / R683.0")));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn joint_exports_pair_each_pen_with_the_receiving_bow_and_its_actual_depth() {
        let mut frame = Kozijn::new("Rond met tussenstijl", "R1", 1200., 1200.);
        frame.frame.shape.shape_type = ShapeType::Round;
        frame.add_column(533.);
        let prod = [compute_production_data(&frame)];
        let j = prod[0].divider_joints.iter().find(|j| j.end == crate::production::DividerJointEnd::Start).unwrap();
        let t = j.tenon.as_ref().unwrap();
        assert_eq!(j.receiver_piece_id.as_deref(), Some("R1-BO1"));
        assert!((t.mortise_depth_mm - t.tenon_length_mm - t.bottom_clearance_mm).abs() < 1e-7);
        let dir = std::env::temp_dir().join(format!("ofs-joints-{}", uuid::Uuid::new_v4()));
        csv_production::generate_production_csv(&prod, dir.to_str().unwrap()).unwrap();
        let csv = fs::read_to_string(dir.join("kortlijst.csv")).unwrap();
        let headers: Vec<_> = csv.lines().next().unwrap().split(';').collect();
        let get = |pos: &str, name| {
            let col = headers.iter().position(|s| *s == name).unwrap();
            csv.lines().skip(1).map(|l| l.split(';').collect::<Vec<_>>()).find(|r| r[1] == pos).unwrap()[col]
        };
        assert_eq!(get(&j.piece_id, "Begin_penlengte_mm"), format!("{:.1}", t.tenon_length_mm));
        assert_eq!(get(&j.piece_id, "Begin_gatdiepte_mm"), format!("{:.1}", t.mortise_depth_mm));
        assert_eq!(get(&j.piece_id, "Begin_gatstuk"), "R1-BO1");
        assert_eq!(get(&j.piece_id, "Eind_gatstuk"), "R1-BO2");
        assert!(get("R1-BO1", "Ontvangt_gaten_van_kop_aantal_diepte_mm").contains("R1-TS1 Begin"));

        let excel = dir.join("pen-en-gat.xlsx");
        xlsx::generate_production_xlsx(&prod, excel.to_str().unwrap()).unwrap();
        let sheet = open_workbook_auto(excel).unwrap().worksheet_range("Pen en gat").unwrap();
        assert_eq!(sheet.height(), 3);
        assert_eq!(sheet[(1, 3)].to_string(), "R1-BO1");
        assert!((sheet[(1, 6)].to_string().parse::<f64>().unwrap()-t.mortise_depth_mm).abs()<1e-7);
        assert_eq!(sheet[(1, 12)].to_string(), j.z_bands());
        let path = dir.join("pen-en-gat.pdf");
        pdf::generate_production_pdf(&prod, path.to_str().unwrap()).unwrap();
        let doc = lopdf::Document::load(path).unwrap();
        let text = doc.extract_text(&doc.get_pages().into_keys().collect::<Vec<_>>()).unwrap();
        let joinery = text.split_once("Pen-en-gatverbindingen tussenhout").unwrap().1;
        for label in ["Gatdiepte", "Bodemvrij", "R1-TS1", "R1-BO1", "R1-BO2", &format!("{:.1}", t.mortise_depth_mm)] {
            assert!(joinery.contains(label), "Missing {label}");
        }
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn arch_foot_exports_identify_both_cuts_and_the_mortise_setting() {
        let mut k = Kozijn::new("Boogvoeten", "B1", 1200., 1500.);
        k.frame.shape.shape_type = ShapeType::Arched;
        k.frame.shape.arch_height = Some(300.);
        k.frame.joints_configured = true;
        for j in &mut k.frame.corner_joints { j.through_member = crate::joint::ThroughMember::Dorpel; }
        k.frame.corner_joints[1].through_member = crate::joint::ThroughMember::Stijl;
        k.frame.corner_joints[1].pen_length = 24.;
        let mut bad = k.clone();
        bad.mark = "X1".into();
        bad.frame.corner_joints[1].pen_length = 1000.;
        let data = [compute_production_data(&k), compute_production_data(&bad)];
        let dir = std::env::temp_dir().join(format!("ofs-boogvoeten-{}", uuid::Uuid::new_v4()));
        csv_production::generate_production_csv(&data, dir.to_str().unwrap()).unwrap();
        let csv = fs::read_to_string(dir.join("boogvoeten.csv")).unwrap();
        let rows: Vec<_> = csv.lines().map(|l| l.split(';').collect::<Vec<_>>()).collect();
        assert_eq!(rows.len(), 5);
        assert!(rows.iter().all(|r| r.len() == rows[0].len()));
        assert_eq!(&rows[1][1..6], ["Links", "B1-SL", "Begin", "B1-BO1", "Begin"]);
        assert_eq!(&rows[2][1..6], ["Rechts", "B1-BO1", "Eind", "B1-SR", "Begin"]);
        assert_eq!(&rows[2][13..17], ["2", "24.0", "26.0", "2.0"]);
        assert_eq!(rows[4][2], "X1-BO1");
        assert!(rows[4][13..26].iter().all(|s| s.is_empty()));
        assert_eq!(rows[4][26], data[1].arch_joints[1].joint.issue.unwrap().to_string());
        let cuts = fs::read_to_string(dir.join("kortlijst.csv")).unwrap();
        let unknown = cuts.lines().find(|l| l.split(';').nth(1) == Some("X1-BO1")).unwrap();
        assert_eq!(unknown.split(';').nth(6), Some(""));

        let path = dir.join("boogvoeten.xlsx");
        xlsx::generate_production_xlsx(&data, path.to_str().unwrap()).unwrap();
        let ws = open_workbook_auto(path).unwrap().worksheet_range("Boogvoeten").unwrap();
        assert_eq!(ws.height(), 5);
        for (i, row) in data[0].arch_joints.iter().enumerate() {
            let t = row.joint.tenon.as_ref().unwrap();
            let cell = |c| ws[(i + 1, c)].to_string().parse::<f64>().unwrap();
            for (col, expected) in [(6, row.joint.source_angle_deg), (7, row.joint.receiver_angle_deg),
                (17, t.blank_extension_mm), (19, t.shoulder_center[0]), (20, t.shoulder_center[1]),
                (22, t.direction[1].atan2(t.direction[0]).to_degrees().rem_euclid(360.))] {
                assert!((cell(col) - expected).abs() < 1e-8);
            }
            assert_eq!(ws[(i + 1, 23)].to_string(), crate::production::z_band_text(&t.z_from_inside_mm));
        }
        assert!(ws[(4, 14)].to_string().is_empty());
        let path = dir.join("boogvoeten.pdf");
        pdf::generate_production_pdf(&data, path.to_str().unwrap()).unwrap();
        let pdf = lopdf::Document::load(path).unwrap();
        let text = pdf.extract_text(&pdf.get_pages().into_keys().collect::<Vec<_>>()).unwrap();
        for label in ["Boogvoeten - pen en gat", "Schouderhoek", "Hart gatmond", "B1-BO1 / Eind",
            "B1-SR / Begin", "Onbepaald", "opgegeven pen 1000.0 mm"] { assert!(text.contains(label), "{label}"); }
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn ring_seams_keep_lower_bow_ends_and_report_both_halves_separately_from_arch_feet() {
        use crate::joint::ThroughMember;
        let mut k = Kozijn::new("Rond", "R1", 1200., 1200.);
        k.frame.shape.shape_type = ShapeType::Round;
        k.frame.joints_configured = true;
        k.frame.corner_joints[0].through_member = ThroughMember::Dorpel;
        k.frame.corner_joints[0].pen_length = 18.;
        k.frame.corner_joints[1].through_member = ThroughMember::Stijl;
        k.frame.corner_joints[1].pen_length = 26.;
        let mut arch = Kozijn::new("Toog", "A1", 1200., 1500.);
        arch.frame.shape.shape_type = ShapeType::Arched;
        arch.frame.shape.arch_height = Some(300.);
        let mut bad = k.clone();
        bad.mark = "X1".into();
        bad.frame.corner_joints[0].pen_length = 1000.;
        let data = [compute_production_data(&k), compute_production_data(&arch), compute_production_data(&bad)];
        let dir = std::env::temp_dir().join(format!("ofs-ringnaden-{}", uuid::Uuid::new_v4()));
        csv_production::generate_production_csv(&data, dir.to_str().unwrap()).unwrap();
        let csv = fs::read_to_string(dir.join("ringnaden.csv")).unwrap();
        let rows: Vec<_> = csv.lines().map(|l| l.split(';').collect::<Vec<_>>()).collect();
        assert_eq!(rows.len(), 5);
        assert_eq!(rows[0][1], "Ringnaad");
        assert_eq!(&rows[1][1..6], ["Links", "R1-BO2", "Eind", "R1-BO1", "Begin"]);
        assert_eq!(&rows[2][1..6], ["Rechts", "R1-BO1", "Eind", "R1-BO2", "Begin"]);
        assert_eq!(&rows[1][13..17], ["2", "18.0", "20.0", "2.0"]);
        assert_eq!(&rows[2][13..17], ["2", "26.0", "28.0", "2.0"]);
        assert!(rows[3][13..26].iter().all(|s| s.is_empty()));
        assert!(!rows[3][26].is_empty());
        let feet = fs::read_to_string(dir.join("boogvoeten.csv")).unwrap();
        assert_eq!(feet.lines().count(), 3);
        assert!(!feet.contains("R1-"));
        let cuts = fs::read_to_string(dir.join("kortlijst.csv")).unwrap();
        let headers: Vec<_> = cuts.lines().next().unwrap().split(';').collect();
        let col = |name: &str| headers.iter().position(|s| s == &name).unwrap();
        for (i, (begin, end)) in [("Gat 20.0 mm", "Pen 26.0 mm"), ("Gat 28.0 mm", "Pen 18.0 mm")].into_iter().enumerate() {
            let row: Vec<_> = cuts.lines().nth(i + 1).unwrap().split(';').collect();
            assert!(row[col("Ringnaad_begin")].starts_with(begin));
            assert!(row[col("Ringnaad_eind")].starts_with(end));
            assert!(row[col("Boogvoet_begin")].is_empty());
        }
        let path = dir.join("productie.xlsx");
        xlsx::generate_production_xlsx(&data, path.to_str().unwrap()).unwrap();
        let mut wb = open_workbook_auto(path).unwrap();
        let ws = wb.worksheet_range("Ringnaden").unwrap();
        assert_eq!((ws.height(), wb.worksheet_range("Boogvoeten").unwrap().height()), (5, 3));
        assert_eq!(ws[(1, 2)].to_string(), "R1-BO2");
        assert_eq!(ws[(1, 3)].to_string(), "Eind");
        assert_eq!(ws[(1, 4)].to_string(), "R1-BO1");
        for (n, joint) in data[0].arch_joints.iter().enumerate() {
            let t = joint.joint.tenon.as_ref().unwrap();
            let cell = |col| ws[(n + 1, col)].to_string().parse::<f64>().unwrap();
            assert!((cell(17) - t.blank_extension_mm).abs() < 1e-8);
            assert!((cell(15) - t.mortise_depth_mm).abs() < 1e-8);
        }
        let path = dir.join("productie.pdf");
        pdf::generate_production_pdf(&data, path.to_str().unwrap()).unwrap();
        let pdf = lopdf::Document::load(path).unwrap();
        let text = pdf.extract_text(&pdf.get_pages().into_keys().collect::<Vec<_>>()).unwrap();
        for label in ["Ringnaden - pen en gat", "onderboog van rechts naar links", "R1-BO2 / Eind",
            "R1-BO1 / Begin", "R1-BO1 / Eind", "R1-BO2 / Begin", "Boogvoeten - pen en gat", "Onbepaald"] {
            assert!(text.contains(label), "{label}");
        }
        csv_production::generate_production_csv(&data[1..2], dir.to_str().unwrap()).unwrap();
        assert_eq!(fs::read_to_string(dir.join("ringnaden.csv")).unwrap().lines().count(), 1);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn divided_operable_arch_exports_every_sash_and_its_own_bending_dimensions() {
        let mut k = Kozijn::new("Getoogd met draaiende ramen", "B2", 2400.0, 1800.0);
        k.frame.shape.shape_type = ShapeType::Arched;
        k.frame.shape.arch_height = Some(400.0);
        for x in [600.0, 1200.0, 1800.0] { k.add_column(x); }
        k.add_row(850.0);
        for cell in &mut k.cells { cell.panel_type = crate::kozijn::PanelType::Turn; }
        let data = [compute_production_data(&k)];
        let cuts = &data[0].cut_list;
        assert!(cuts.len() > 35);
        let first_arc = cuts.iter().find(|i| i.member_type == crate::production::MemberType::SashArc).unwrap();
        let dir = std::env::temp_dir().join(format!("ofs-sash-cuts-{}", uuid::Uuid::new_v4()));
        csv_production::generate_production_csv(&data, dir.to_str().unwrap()).unwrap();
        let csv = fs::read_to_string(dir.join("kortlijst.csv")).unwrap();
        let csv_arc: Vec<_> = csv.lines().find(|line| line.split(';').nth(1) == Some(&first_arc.piece_id)).unwrap().split(';').collect();
        assert_eq!(csv_arc[2], "Vleugelboog boven");
        assert_eq!(csv_arc[10], "1947.0"); // outside R2000, 67 mm frame face and 14 mm sash overlap
        assert_eq!(csv_arc[11], format!("{:.1}", first_arc.net_length_mm));

        let xlsx_path = dir.join("raamhout.xlsx");
        xlsx::generate_production_xlsx(&data, xlsx_path.to_str().unwrap()).unwrap();
        let sheet = open_workbook_auto(&xlsx_path).unwrap().worksheet_range("Kortlijst").unwrap();
        assert_eq!(sheet.height(), cuts.len() + 1);
        let position = cuts.iter().position(|i| i.piece_id == first_arc.piece_id).unwrap() + 1;
        assert_eq!(sheet[(position, 10)].to_string(), "1947");
        assert!((sheet[(position, 12)].to_string().parse::<f64>().unwrap() - 1912.5).abs() < 1e-7);

        let pdf_path = dir.join("raamhout.pdf");
        pdf::generate_production_pdf(&data, pdf_path.to_str().unwrap()).unwrap();
        let pdf = lopdf::Document::load(pdf_path).unwrap();
        let pages: Vec<_> = pdf.get_pages().into_keys().collect();
        let text = pdf.extract_text(&pages).unwrap();
        for item in cuts { assert!(text.contains(&item.piece_id), "Missing {}", item.piece_id); }
        assert!(text.contains("1947.0") && text.contains("Vleugelboog boven"));
        fs::remove_dir_all(dir).unwrap();
    }
}
