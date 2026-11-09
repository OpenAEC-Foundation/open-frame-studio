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

    // Internal design screening only. These dimensions do not constitute an
    // EN 14351-1, Bbl, CE or producer approval.
    checks.push(CertCheck {
        requirement: "Indicatieve profielbreedte >= 50mm".into(),
        value: format!("{:.0}mm", fw),
        expected: ">= 50mm".into(),
        passed: fw >= 50.0,
    });

    checks.push(CertCheck {
        requirement: "Indicatieve profieldiepte >= 60mm".into(),
        value: format!("{:.0}mm", fd),
        expected: ">= 60mm".into(),
        passed: fd >= 60.0,
    });

    // A calculated value is not a producer-declared performance. The model
    // currently has no declaration evidence or building context, so neither
    // declaration nor statutory compliance can pass automatically.
    let uw_result = crate::thermal::calculate_uw(kozijn, profiles);
    checks.push(CertCheck {
        requirement: "Uw-waarde gedeclareerd".into(),
        value: format!("Niet gedeclareerd (indicatief berekend: {:.2} W/m\u{00b2}K)", uw_result.uw_value),
        expected: "Geverifieerde producentenverklaring".into(),
        passed: false,
    });
    checks.push(CertCheck {
        requirement: "Bbl-Uw-eis voor dit bouwwerk bepaald".into(),
        value: format!("Onbekend (indicatieve Uw: {:.2} W/m\u{00b2}K)", uw_result.uw_value),
        expected: "Gebouwsituatie, toepassing en rechtens verkregen niveau vastgesteld".into(),
        passed: false,
    });

    // Check actual glass input only for glazed cells. A populated cell list
    // alone says nothing about whether glazing has been specified.
    let glass_cells: Vec<_> = kozijn.cells.iter().filter(|c| {
        !matches!(c.panel_type, crate::kozijn::PanelType::Door
            | crate::kozijn::PanelType::Panel
            | crate::kozijn::PanelType::Ventilation)
    }).collect();
    if !glass_cells.is_empty() {
        let specified = glass_cells.iter().all(|c| {
            !c.glazing.glass_type.trim().is_empty()
                && c.glazing.thickness_mm.is_finite()
                && c.glazing.thickness_mm > 0.0
                && c.glazing.ug_value.is_finite()
                && c.glazing.ug_value > 0.0
        });
        checks.push(CertCheck {
            requirement: "Glasgegevens voor glasvakken ingevuld".into(),
            value: if specified { "Ja" } else { "Onvolledig" }.into(),
            expected: "Glastype, dikte en Ug ingevuld".into(),
            passed: specified,
        });
    }

    let operable_cells: Vec<_> = kozijn.cells.iter().filter(|c| c.panel_type.is_operable()).collect();
    if !operable_cells.is_empty() {
        let has_hardware = operable_cells.iter().all(|c| c.hardware_set.is_some() || !c.hardware.is_empty());
        checks.push(CertCheck {
            requirement: "Beslag voor alle beweegbare vakken gespecificeerd".into(),
            value: if has_hardware { "Ja" } else { "Onvolledig" }.into(),
            expected: "Ja".into(),
            passed: has_hardware,
        });
    }

    let all_pass = checks.iter().all(|c| c.passed);
    CertificationResult { standard: "Indicatieve CE-voorcontrole (geen certificering)".into(), checks, overall_pass: all_pass }
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

        // A missing profile snapshot cannot satisfy this check.
        let sponning_hoogte = kozijn
            .frame
            .profile_snapshot
            .as_ref()
            .and_then(|s| s.sponning_hoogte);
        checks.push(CertCheck {
            requirement: "Sponninghoogte >= 17mm".into(),
            value: match sponning_hoogte {
                Some(v) => format!("{:.0}mm", v),
                None => "Onbekend (profielsnapshot ontbreekt)".into(),
            },
            expected: ">= 17mm".into(),
            passed: sponning_hoogte.map_or(false, |v| v.is_finite() && v >= 17.0),
        });

        // Corner joints
        let has_joints = kozijn.frame.joints_configured && kozijn.frame.corner_joints.len() >= 4;
        checks.push(CertCheck {
            requirement: "Hoekverbindingen expliciet vastgelegd".into(),
            value: if has_joints { "4 hoeken bevestigd" } else { "Niet bevestigd" }.into(),
            expected: "4 hoeken bevestigd".into(),
            passed: has_joints,
        });
    }

    // No certificate or approved system reference is stored in the kozijn.
    // A complete set of geometric inputs must not be reported as approval.
    checks.push(CertCheck {
        requirement: "SKH/KOMO-systeem en certificaat bevestigd".into(),
        value: "Niet vastgelegd".into(),
        expected: "Geverifieerde producent- en certificaatreferentie".into(),
        passed: false,
    });

    let all_pass = checks.iter().all(|c| c.passed);
    CertificationResult { standard: "Indicatieve houtcontrole (geen SKH/KOMO-toets)".into(), checks, overall_pass: all_pass }
}

#[cfg(test)]
mod tests {
    use super::{check_ce_marking, check_skh_komo, CertificationResult};
    use crate::kozijn::Kozijn;
    use crate::profile::ProfileSnapshot;

    fn check<'a>(result: &'a CertificationResult, requirement: &str) -> &'a super::CertCheck {
        result.checks.iter().find(|c| c.requirement == requirement).unwrap()
    }

    #[test]
    fn a_calculated_uw_is_not_a_declaration_or_bbl_approval() {
        let kozijn = Kozijn::new("Kozijn", "K01", 1200.0, 1500.0);
        let result = check_ce_marking(&kozijn, &[]);
        assert!(!check(&result, "Uw-waarde gedeclareerd").passed);
        assert!(!check(&result, "Bbl-Uw-eis voor dit bouwwerk bepaald").passed);
        assert!(!result.overall_pass);
    }

    #[test]
    fn empty_glazing_input_is_not_treated_as_specified() {
        let mut kozijn = Kozijn::new("Kozijn", "K01", 1200.0, 1500.0);
        kozijn.cells[0].glazing.glass_type = " ".into();
        let result = check_ce_marking(&kozijn, &[]);
        assert!(!check(&result, "Glasgegevens voor glasvakken ingevuld").passed);
    }

    #[test]
    fn unmeasured_rebate_and_default_joints_cannot_pass() {
        let mut kozijn = Kozijn::new("Kozijn", "K01", 1200.0, 1500.0);
        let result = check_skh_komo(&kozijn);
        assert!(!check(&result, "Sponninghoogte >= 17mm").passed);
        assert!(!check(&result, "Hoekverbindingen expliciet vastgelegd").passed);

        kozijn.frame.profile_snapshot = Some(ProfileSnapshot {
            sponning_hoogte: Some(18.0),
            ..ProfileSnapshot::default()
        });
        kozijn.frame.joints_configured = true;
        let result = check_skh_komo(&kozijn);
        assert!(check(&result, "Sponninghoogte >= 17mm").passed);
        assert!(check(&result, "Hoekverbindingen expliciet vastgelegd").passed);
        assert!(!check(&result, "SKH/KOMO-systeem en certificaat bevestigd").passed);
        assert!(!result.overall_pass);
    }
}
