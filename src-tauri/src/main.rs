#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod models;
mod state;
mod blender;

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
            // Import commands
            commands::profiles::load_profile_library,
            commands::settings::load_settings,
            commands::settings::save_settings,
            commands::import_profile::import_dxf_profile,
            commands::import_profile::import_catalog,
            // Blender commands
            commands::blender::send_to_blender,
            commands::blender::check_blender_connection,
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
