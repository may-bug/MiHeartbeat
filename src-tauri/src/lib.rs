use std::env;

use crate::system::{init_splash, init_tray};

mod heart;
mod settings;
mod system;
mod window;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 解决linux渲染问题
    #[cfg(target_os = "linux")]
    {
        if env::var("WEBKIT_DISABLE_COMPOSITING_MODE").is_err() {
            env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        }
        if env::var("WEBKIT_DISABLE_DMABUF_RENDERER").is_err() {
            env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            init_tray(&app_handle).expect("init tray failed");
            init_splash(&app_handle).expect("init splash failed");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            heart::bluetooth_available,
            heart::list_devices,
            heart::select_device,
            heart::start_heart_rate_stream,
            heart::stop_heart_rate_stream,
            heart::is_heart_rate_streaming,
            settings::get_settings,
            settings::set_settings,
            settings::reset_to_default,
            system::show_window,
            system::hide_window,
            system::minimize_to_tray,
            window::disable_window_operations,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
