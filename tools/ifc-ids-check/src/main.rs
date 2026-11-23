//! Check OFS parts exports. Exit 0 = valid, 1 = rejected IFC, 2 = input or tool error.
use bim_validator_core::{validate, SpecificationResult, ValidationStatus};
use ifc_lite_core::{guid::is_valid_global_id, parse_entity, EntityScanner, Token};
use serde_json::json;
use std::{collections::BTreeSet, env, error::Error, fs, process::ExitCode};

fn check() -> Result<u8, Box<dyn Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let [ids_path, ifc_path, expected_json, report_path] = args.as_slice() else {
        return Err(
            "usage: ofs-ifc-ids-check RULES.ids PARTS.ifc '[\"GlobalId\",…]' REPORT.json".into(),
        );
    };
    let expected_ids: Vec<String> = serde_json::from_str(
        expected_json
            .to_str()
            .ok_or("invalid GlobalId JSON argument")?,
    )?;
    let expected: BTreeSet<_> = expected_ids.iter().cloned().collect();
    if expected.is_empty()
        || expected.len() != expected_ids.len()
        || expected.iter().any(|id| !is_valid_global_id(id))
    {
        return Err("expected pane GlobalIds must be nonempty, valid and unique".into());
    }
    let ifc = fs::read(ifc_path)?;
    let report = validate(&ifc, &fs::read_to_string(ids_path)?)?;
    let required_rules = ["OFS-RUIT-KLASSE", "OFS-RUIT-ONDERDEEL"];
    let rules_complete = required_rules.iter().all(|id| {
        report
            .specifications
            .iter()
            .filter(|s| s.identifier.as_deref() == Some(id))
            .count()
            == 1
    });

    // In the parts fixture only glazing panes are IfcWindow occurrences. The
    // Suite reports GlobalIds for failures, so collect the passing occurrences
    // with its STEP parser as well, independently of the OFS model manifest.
    let mut found = BTreeSet::new();
    let mut duplicates = BTreeSet::new();
    let mut invalid = Vec::new();
    let mut scanner = EntityScanner::new(&ifc);
    while let Some((express_id, entity, start, end)) = scanner.next_entity() {
        if !entity.eq_ignore_ascii_case("IFCWINDOW") {
            continue;
        }
        let (_, _, attributes) = parse_entity(&ifc[start..end])?;
        let global_id = match attributes.first() {
            Some(Token::String(bytes)) => std::str::from_utf8(bytes)
                .ok()
                .filter(|s| is_valid_global_id(s)),
            _ => None,
        };
        match global_id {
            Some(id) if !found.insert(id.to_owned()) => {
                duplicates.insert(id.to_owned());
            }
            Some(_) => {}
            None => invalid.push(express_id),
        }
    }
    let missing: Vec<_> = expected.difference(&found).collect();
    let unexpected: Vec<_> = found.difference(&expected).collect();
    let rule_passes = |s: &SpecificationResult| {
        s.status == ValidationStatus::Passed
            && s.applicable_count == expected.len()
            && s.passed_count == expected.len()
            && s.failed_count == 0
    };
    let unresolved = !rules_complete
        || report.status == ValidationStatus::NotCheckable
        || report
            .specifications
            .iter()
            .any(|s| s.status == ValidationStatus::NotCheckable);
    let valid = report.status == ValidationStatus::Passed
        && report.specifications.iter().all(rule_passes)
        && missing.is_empty()
        && unexpected.is_empty()
        && duplicates.is_empty()
        && invalid.is_empty();
    let (status, exit_code) = if unresolved {
        ("not_checkable", 2)
    } else if valid {
        ("passed", 0)
    } else {
        ("failed", 1)
    };
    let result = json!({
        "status": status,
        "paneGlobalIds": found,
        "missingGlobalIds": missing,
        "unexpectedGlobalIds": unexpected,
        "duplicateGlobalIds": duplicates,
        "invalidPaneExpressIds": invalid,
        "report": report,
    });
    fs::write(report_path, serde_json::to_vec_pretty(&result)?)?;
    if exit_code != 0 {
        eprintln!("{ifc_path:?}: {status}, see {report_path:?}");
        if !rules_complete {
            eprintln!("Expected one IDS specification for each of {required_rules:?}");
        }
        if !missing.is_empty()
            || !unexpected.is_empty()
            || !duplicates.is_empty()
            || !invalid.is_empty()
        {
            eprintln!("Pane GlobalIds: missing={missing:?}, unexpected={unexpected:?}, duplicate={duplicates:?}, invalid STEP records={invalid:?}");
        }
        for spec in report.specifications.iter().filter(|s| !rule_passes(s)) {
            eprintln!(
                "{}: {:?}, {} applicable, {} passed (expected {})",
                spec.identifier.as_deref().unwrap_or(&spec.name),
                spec.status,
                spec.applicable_count,
                spec.passed_count,
                expected.len()
            );
            for reason in &spec.reasons {
                eprintln!("  {reason}");
            }
            for finding in &spec.findings {
                eprintln!(
                    "  #{} {} [{}] {}",
                    finding.express_id,
                    finding.global_id.as_deref().unwrap_or("no GlobalId"),
                    finding.facet,
                    finding.reason
                );
            }
        }
    }
    Ok(exit_code)
}

fn main() -> ExitCode {
    match check() {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("IFC release check: {error}");
            ExitCode::from(2)
        }
    }
}
