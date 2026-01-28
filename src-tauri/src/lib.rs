use std::env;

use tauri::Manager;
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
            // 获取主窗口
            let main_window = app.get_webview_window("main").unwrap();
            main_window.hide().unwrap();
            // 创建启动屏窗口
            let _splash_window = tauri::WebviewWindowBuilder::new(
                app,
                "splash",
                tauri::WebviewUrl::App("index.html#/splash".into()),
            )
            .title("加载中...")
            .inner_size(400.0, 300.0)
            .resizable(false)
            .decorations(false)
            .transparent(true)
            .always_on_top(true)
            .visible(true)
            .shadow(false)
            .center()
            .build()
            .unwrap();
            std::thread::spawn({
                let app_handle = app.handle().clone();
                let main_window = main_window.clone();
                move || {
                    println!("初始化应用...");
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    
                    println!("加载配置...");
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    
                    println!("准备界面...");
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    
                    // 完成后关闭启动屏并显示主窗口
                    // app_handle.emit_to("splash", "splash-close", {}).unwrap();
                    
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    
                    // 显示主窗口
                    let _ = main_window.show();
                    let _ = main_window.set_focus();
                    
                    // 关闭启动屏
                    if let Some(splash) = app_handle.get_webview_window("splash") {
                        let _ = splash.close();
                    }
                }
            });
            if let Err(e) = system::init_tray(app.handle()) {
                eprintln!("Failed to initialize system tray: {}", e);
            }
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
