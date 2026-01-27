use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloatingWindowSettings {
    pub opacity: f32,
    pub always_on_top: bool,
    pub auto_start: bool,
    pub show_device_name: bool,
    pub animation_speed: String,
}

impl Default for FloatingWindowSettings {
    fn default() -> Self {
        Self {
            opacity: 0.85,
            always_on_top: true,
            auto_start: false,
            show_device_name: true,
            animation_speed: "normal".to_string(),
        }
    }
}

fn get_settings_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .expect("Failed to get config directory")
        .join("heart");
    
    // 确保目录存在
    let _ = fs::create_dir_all(&config_dir);
    
    config_dir.join("settings.json")
}

pub fn load_settings() -> Result<FloatingWindowSettings, String> {
    let path = get_settings_path();
    
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(content) => {
                match serde_json::from_str(&content) {
                    Ok(settings) => Ok(settings),
                    Err(e) => {
                        eprintln!("Failed to parse settings: {}", e);
                        Ok(FloatingWindowSettings::default())
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read settings file: {}", e);
                Ok(FloatingWindowSettings::default())
            }
        }
    } else {
        Ok(FloatingWindowSettings::default())
    }
}

pub fn save_settings(settings: &FloatingWindowSettings) -> Result<(), String> {
    let path = get_settings_path();
    
    match serde_json::to_string_pretty(&settings) {
        Ok(content) => {
            match fs::write(&path, content) {
                Ok(_) => {
                    eprintln!("Settings saved to: {:?}", path);
                    Ok(())
                }
                Err(e) => Err(format!("Failed to write settings file: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to serialize settings: {}", e)),
    }
}

pub fn update_settings(updates: FloatingWindowSettings) -> Result<FloatingWindowSettings, String> {
    save_settings(&updates)?;
    Ok(updates)
}

pub fn reset_settings() -> Result<FloatingWindowSettings, String> {
    let default = FloatingWindowSettings::default();
    save_settings(&default)?;
    Ok(default)
}

#[tauri::command]
pub async fn get_settings() -> Result<FloatingWindowSettings, String> {
    load_settings()
}

#[tauri::command]
pub async fn set_settings(settings: FloatingWindowSettings) -> Result<FloatingWindowSettings, String> {
    update_settings(settings)
}

#[tauri::command]
pub async fn reset_to_default() -> Result<FloatingWindowSettings, String> {
    reset_settings()
}
