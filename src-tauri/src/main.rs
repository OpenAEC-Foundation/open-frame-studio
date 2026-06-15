#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod models;
mod state;
mod blender;
mod cloud;

use state::AppState;

#[tauri::command]
fn get_platform() -> &'static str {
    #[cfg(target_os = "windows")]
    { "windows" }
    #[cfg(target_os = "linux")]
    { "linux" }
    #[cfg(target_os = "macos")]
    { "macos" }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            // Project commands
            commands::project::new_project,
            commands::project::open_project,
            commands::project::save_project,
            commands::project::get_project,
            // Kozijn commands
            commands::kozijn::create_kozijn,
            commands::kozijn::create_kozijn_from_template,
            commands::kozijn::get_kozijn,
            commands::kozijn::get_all_kozijnen,
            commands::kozijn::update_kozijn_dimensions,
            commands::kozijn::update_grid_sizes,
            commands::kozijn::update_frame_profile,
            commands::kozijn::update_sill_profile,
            commands::kozijn::update_divider_profile,
            commands::kozijn::update_member_profile,
            commands::kozijn::update_frame_shape,
            commands::kozijn::update_corner_joints,
            commands::kozijn::add_custom_profile,
            commands::kozijn::update_cell_type,
            commands::kozijn::add_column,
            commands::kozijn::add_row,
            commands::kozijn::remove_kozijn,
            commands::kozijn::get_kozijn_geometry,
            commands::kozijn::update_cell_hardware,
            commands::kozijn::auto_select_hardware,
            commands::kozijn::update_security_class,
            commands::kozijn::update_cell_glazing,
            commands::kozijn::update_frame_colors,
            commands::kozijn::duplicate_kozijn,
            commands::kozijn::calculate_thermal,
            commands::kozijn::get_sjablonen,
            commands::kozijn::save_custom_sjabloon,
            commands::kozijn::delete_custom_sjabloon,
            commands::kozijn::update_cell_sash_profile,
            commands::kozijn::update_edge_config,
            commands::kozijn::add_frame_extension,
            commands::kozijn::remove_frame_extension,
            // Export commands
            commands::export_ifc::export_ifc,
            commands::export_dxf::export_dxf,
            commands::export_pdf::export_kozijnstaat,
            commands::export_workshop::export_workshop_drawing,
            commands::export_gltf::export_gltf,
            // Production commands
            commands::production::get_production_data,
            commands::production::get_production_data_project,
            // Calculation commands
            commands::calculation::get_cost_estimate,
            commands::calculation::get_cost_estimate_project,
            // Production export commands
            commands::export_production::export_production_lists,
            commands::export_production::export_labels_pdf,
            // Vliesgevel commands
            commands::vliesgevel::create_vliesgevel,
            commands::vliesgevel::create_vliesgevel_from_template,
            commands::vliesgevel::get_vliesgevel,
            commands::vliesgevel::get_all_vliesgevels,
            commands::vliesgevel::vliesgevel_add_mullion,
            commands::vliesgevel::vliesgevel_add_transom,
            commands::vliesgevel::vliesgevel_remove_mullion,
            commands::vliesgevel::vliesgevel_remove_transom,
            commands::vliesgevel::vliesgevel_update_panel,
            commands::vliesgevel::get_vliesgevel_geometry,
            commands::vliesgevel::get_vliesgevel_production,
            commands::vliesgevel::remove_vliesgevel,
            // Glass library commands
            commands::glass::get_glass_library,
            // Import commands
            commands::profiles::load_profile_library,
            commands::settings::load_settings,
            commands::settings::save_settings,
            commands::import_profile::import_dxf_profile,
            commands::import_profile::import_catalog,
            // Blender commands
            commands::blender::send_to_blender,
            commands::blender::check_blender_connection,
            // Pricing commands
            commands::pricing::get_pricing_config,
            commands::pricing::update_pricing_config,
            // Optimization commands
            commands::optimization::optimize_project_cut_list,
            // BCF commands
            commands::bcf::get_bcf_topics,
            commands::bcf::create_bcf_topic,
            commands::bcf::update_bcf_topic_status,
            commands::bcf::add_bcf_comment,
            // IDS commands
            commands::ids::validate_project_ids,
            // Quotation commands
            commands::quotation::get_quotations,
            commands::quotation::create_quotation,
            commands::quotation::update_quotation_status,
            commands::quotation::create_quotation_revision,
            // Procurement commands
            commands::procurement::generate_purchase_proposals,
            // Planning commands
            commands::planning::get_production_plan,
            // Energy commands
            commands::energy::get_project_energy,
            // Certification commands
            commands::certification::check_certification,
            // CNC commands
            commands::cnc::export_cnc_gcode,
            commands::cnc::get_cnc_parts,
            // IFC import commands
            commands::import_ifc::import_ifc_file,
            // Combination commands
            commands::combination::create_combination,
            commands::combination::add_to_combination,
            commands::combination::get_combinations,
            commands::combination::remove_combination,
            // IFC roundtrip commands
            commands::ifc_roundtrip::compare_ifc_files,
            commands::ifc_roundtrip::compare_ifc_roundtrip,
            get_platform,
        ])
        .setup(|app| {
            #[cfg(target_os = "windows")]
            {
                use tauri::Manager;
                let window = app.get_webview_window("main").unwrap();
                let _ = window.with_webview(move |webview| unsafe {
                    let core = webview.controller();
                    let core3: webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Controller3 =
                        windows::core::Interface::cast(&core).unwrap();
                    core3.SetBoundsMode(
                        webview2_com::Microsoft::Web::WebView2::Win32::COREWEBVIEW2_BOUNDS_MODE_USE_RAW_PIXELS
                    ).unwrap();
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Open Frame Studio");
}
