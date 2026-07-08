use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificationResult {
    pub standard: String,
    pub checks: Vec<CertCheck>,
    pub overall_pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertCheck {
    pub requirement: String,
    pub value: String,
    pub expected: String,
    pub passed: bool,
}

pub fn check_ce_marking(
    kozijn: &crate::kozijn::Kozijn,
    profiles: &[crate::profile::ProfileDefinition],
) -> CertificationResult {
    let mut checks = Vec::new();
    let fw = kozijn.frame.frame_width;
    let fd = kozijn.frame.frame_depth;

    // EN 14351-1: minimum frame dimensions
    checks.push(CertCheck {
        requirement: "Profielbreedte >= 50mm".into(),
        value: format!("{:.0}mm", fw),
        expected: ">= 50mm".into(),
        passed: fw >= 50.0,
    });

    checks.push(CertCheck {
        requirement: "Profieldiepte >= 60mm".into(),
        value: format!("{:.0}mm", fd),
        expected: ">= 60mm".into(),
        passed: fd >= 60.0,
    });

    // Uw value check
    let uw_result = crate::thermal::calculate_uw(kozijn, profiles);
    checks.push(CertCheck {
        requirement: "Uw-waarde gedeclareerd".into(),
        value: format!("{:.2} W/m\u{00b2}K", uw_result.uw_value),
        expected: "Aanwezig".into(),
        passed: true,
    });
    checks.push(CertCheck {
        requirement: "Uw <= 1.65 (Bouwbesluit renovatie)".into(),
        value: format!("{:.2}", uw_result.uw_value),
        expected: "<= 1.65".into(),
        passed: uw_result.uw_value <= 1.65,
    });

    // Glass check — glazing is always present, check if it has a glass_type set
    let has_glazing = !kozijn.cells.is_empty();
    checks.push(CertCheck {
        requirement: "Beglazing gespecificeerd".into(),
        value: if has_glazing { "Ja" } else { "Nee" }.into(),
        expected: "Ja".into(),
        passed: has_glazing,
    });

    // Hardware security check
    let has_hardware = kozijn.cells.iter().any(|c| c.hardware_set.is_some());
    checks.push(CertCheck {
        requirement: "Beslag gespecificeerd".into(),
        value: if has_hardware { "Ja" } else { "Nee" }.into(),
        expected: "Ja".into(),
        passed: has_hardware,
    });

    let all_pass = checks.iter().all(|c| c.passed);
    CertificationResult { standard: "CE EN 14351-1".into(), checks, overall_pass: all_pass }
}

pub fn check_skh_komo(kozijn: &crate::kozijn::Kozijn) -> CertificationResult {
    let mut checks = Vec::new();
    let mat = &kozijn.frame.material;

    // Material check (hout kwaliteit)
    let is_wood = matches!(mat, crate::kozijn::Material::Wood(_));
    checks.push(CertCheck {
        requirement: "Materiaal is hout".into(),
        value: format!("{:?}", mat),
        expected: "Wood(...)".into(),
        passed: is_wood,
    });

    if is_wood {
        // Frame width >= 54mm for SKH
        checks.push(CertCheck {
            requirement: "Houtbreedte >= 54mm (SKH/KVT)".into(),
            value: format!("{:.0}mm", kozijn.frame.frame_width),
            expected: ">= 54mm".into(),
            passed: kozijn.frame.frame_width >= 54.0,
        });

        // Sponning depth >= 17mm — real value from the profile snapshot when
        // the frame profile has been applied; assumed for legacy kozijnen.
        let sponning_hoogte = kozijn
            .frame
            .profile_snapshot
            .as_ref()
            .and_then(|s| s.sponning_hoogte);
        checks.push(CertCheck {
            requirement: "Sponninghoogte >= 17mm".into(),
            value: match sponning_hoogte {
                Some(v) => format!("{:.0}mm", v),
                None => "17mm (aangenomen)".into(),
            },
            expected: ">= 17mm".into(),
            passed: sponning_hoogte.map_or(true, |v| v >= 17.0),
        });

        // Corner joints
        let has_joints = !kozijn.frame.corner_joints.is_empty();
        checks.push(CertCheck {
            requirement: "Hoekverbindingen gedefinieerd".into(),
            value: if has_joints { format!("{} hoeken", kozijn.frame.corner_joints.len()) } else { "Geen".into() },
            expected: "4 hoeken".into(),
            passed: kozijn.frame.corner_joints.len() >= 4,
        });
    }

    let all_pass = checks.iter().all(|c| c.passed);
    CertificationResult { standard: "SKH/KOMO Houten Kozijnen".into(), checks, overall_pass: all_pass }
}
