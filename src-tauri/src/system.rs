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