use tauri::{
    menu::{Menu, MenuItem },
    tray::TrayIconBuilder,
    AppHandle, Manager, Window,
};

/// 初始化系统托盘图标
pub fn init_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // 创建菜单项
    let show_item = MenuItem::with_id(app, "show", "显示", true, None::<&str>)?;
    let hide_item = MenuItem::with_id(app, "hide", "隐藏", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    // 创建菜单
    let menu = Menu::with_items(
        app,
        &[
            &show_item,
            &hide_item,
            &quit_item,
        ],
    )?;

    // 创建托盘图标
    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .icon(app.default_window_icon().unwrap().clone())
        .on_menu_event(move |app, event| {
            match event.id.as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                    }
                }
                "hide" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.hide();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}

pub fn init_splash(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>>  {
    #[cfg(not(target_os = "macos"))]
    {
        // 获取主窗口
        let main_window = app.get_webview_window("main").unwrap();
        main_window.hide()?;
        // 创建启动屏窗口
        let _splash_window = tauri::WebviewWindowBuilder::new(
            app,
            "splash",
            tauri::WebviewUrl::App("index.html#/splash".into()),
        )
            .title("加载中...")
            .inner_size(600.0, 400.0)
            .resizable(false)
            .decorations(false)
            .transparent(true)
            .always_on_top(true)
            .visible(true)
            .shadow(false)
            .center()
            .build()?;
        std::thread::spawn({
            let app_handle = app.clone();
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
        if let Err(e) = init_splash(&app.clone()) {
            eprintln!("Failed to initialize system tray: {}", e)
        }
    }
    Ok(())
}

/// 显示主窗口
#[tauri::command]
pub fn show_window(window: Window) {
    let _ = window.show();
}

/// 隐藏主窗口
#[tauri::command]
pub fn hide_window(window: Window) {
    let _ = window.hide();
}

/// 最小化到托盘
#[tauri::command]
pub fn minimize_to_tray(window: Window) {
    let _ = window.hide();
}