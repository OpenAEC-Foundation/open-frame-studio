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
        assert!(text.contains("Kopprofielen tussenhout") && text.contains("Middelpunt X, Y"));
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
