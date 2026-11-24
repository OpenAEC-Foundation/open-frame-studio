use ofs_core::calculation::estimate_fixed_frame;

#[repr(C)]
#[derive(Default)]
pub struct OfsEstimateV1 {
    status: i32,
    total_eur: f64,
    labor_hours: f64,
    glass_eur: f64,
}

/// A fixed-size return value keeps allocation and ownership inside each runtime.
#[no_mangle]
pub extern "C" fn ofs_estimate_fixed_v1(width_mm: f64, height_mm: f64) -> OfsEstimateV1 {
    match std::panic::catch_unwind(|| estimate_fixed_frame(width_mm, height_mm)) {
        Ok(Some(estimate)) => OfsEstimateV1 {
            status: 0,
            total_eur: estimate.total_cost,
            labor_hours: estimate.labor_hours,
            glass_eur: estimate.glass_cost,
        },
        Ok(None) => OfsEstimateV1 {
            status: 1,
            ..Default::default()
        },
        Err(_) => OfsEstimateV1 {
            status: 2,
            ..Default::default()
        },
    }
}
