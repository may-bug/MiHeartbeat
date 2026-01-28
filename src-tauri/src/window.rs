use tauri::Manager;

#[tauri::command]
pub fn disable_window_operations(app: tauri::AppHandle, label: String) {
    if let Some(window) = app.get_webview_window(&label) {
        let _ = window.set_decorations(false);
        let _ = window.set_resizable(false);
        let _ = window.set_minimizable(false);
        let _ = window.set_maximizable(false);
    }
}