use super::postprocessor::CncPostProcessor;
use super::CncPart;

pub struct GenericGCode;

impl CncPostProcessor for GenericGCode {
    fn name(&self) -> &str { "Generic G-Code (niet vrijgegeven)" }
    fn extension(&self) -> &str { "nc" }

    fn generate(&self, _parts: &[CncPart]) -> Result<Vec<(String, String)>, String> {
        Err("Generieke G-code is niet vrijgegeven: machineassen, gereedschappen, profielen, verbindingen en bewerkingsmaten zijn niet voor een producent gevalideerd".into())
    }
}

#[cfg(test)]
mod tests {
    use super::GenericGCode;
    use crate::cnc::{generate_cnc_parts, postprocessor::CncPostProcessor};
    use crate::kozijn::Kozijn;

    #[test]
    fn generic_postprocessor_never_emits_machine_files() {
        let kozijn = Kozijn::new("Kozijn", "K01", 1200.0, 1500.0);
        let parts = generate_cnc_parts(&kozijn);
        let error = GenericGCode.generate(&parts).unwrap_err();
        assert!(error.contains("niet vrijgegeven"));
    }
}
