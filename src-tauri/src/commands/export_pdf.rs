use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn export_kozijnstaat(
    state: State<'_, AppState>,
    output_path: String,
    format: String,
) -> Result<String, String> {
    let project = {
        let project = state.project.lock().map_err(|e| e.to_string())?;
        project.clone()
    };

    match format.as_str() {
        "xlsx" => {
            ofs_core::export::xlsx::generate_kozijnstaat_xlsx(&project, &output_path)?;
        }
        _ => {
            ofs_core::export::pdf::generate_kozijnstaat_pdf(&project, &output_path)?;
        }
    }

    Ok(output_path)
}

#[tauri::command]
pub async fn export_quotation_pdf(
    state: State<'_, AppState>,
    output_path: String,
) -> Result<String, String> {
    let project = {
        let project = state.project.lock().map_err(|e| e.to_string())?;
        project.clone()
    };

    // Per-kozijn subtotals from the standard cost estimate.
    let price_table = ofs_core::calculation::PriceTable::default();
    let prices: Vec<(String, f64)> = project
        .kozijnen
        .iter()
        .map(|k| {
            (
                k.mark.clone(),
                ofs_core::calculation::estimate_cost(k, &price_table).total_cost,
            )
        })
        .collect();
    let subtotal: f64 = prices.iter().map(|(_, total)| *total).sum();

    // Use the project pricing config when set; otherwise plain 21% BTW.
    let pricing_config = project.pricing_config.clone().unwrap_or(
        ofs_core::pricing::PricingConfig {
            btw_percentage: 21.0,
            ..Default::default()
        },
    );
    let quotation_price = pricing_config.calculate(subtotal);

    // The project has no company settings (yet) — use neutral defaults.
    let company = ofs_core::export::pdf_quotation::CompanyInfo {
        name: "Open Frame Studio".to_string(),
        address: "-".to_string(),
        phone: "-".to_string(),
        email: "-".to_string(),
        kvk: "-".to_string(),
        btw_id: "-".to_string(),
    };
    let terms = "Geldigheid: 30 dagen na offertedatum.\n\
        Levering conform KVT-richtlijnen.\n\
        Prijzen in euro, inclusief BTW tenzij anders vermeld.";

    let bytes = ofs_core::export::pdf_quotation::generate_quotation_pdf(
        &project,
        &company,
        &prices,
        &quotation_price,
        terms,
    )?;
    std::fs::write(&output_path, bytes).map_err(|e| e.to_string())?;

    Ok(output_path)
}
