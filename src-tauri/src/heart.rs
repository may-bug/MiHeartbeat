use std::error::Error;

use bluest::{btuuid::bluetooth_uuid_from_u16, Adapter, Device, Uuid};
use futures_lite::stream::StreamExt;
use serde::Serialize;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tokio::task;
use std::sync::Arc;

const HRS_UUID: Uuid = bluetooth_uuid_from_u16(0x180D);
const HRM_UUID: Uuid = bluetooth_uuid_from_u16(0x2A37);

// 设备信息
#[derive(Serialize)]
pub struct DeviceInfo {
    pub id: String,
    pub mac_address: Option<String>,
    pub name: Option<String>,
    pub connected: bool,
}

// 从设备 ID 中提取 MAC 地址
fn extract_mac_address(device_id: &str) -> Option<String> {
    if let Some(last_part) = device_id.split('#').last() {
        if last_part.len() >= 12 {
            let hex_str = last_part.to_uppercase();
            let hex_only: String = hex_str.chars().filter(|c| c.is_ascii_hexdigit()).collect();
            if hex_only.len() >= 12 {
                let mac_bytes: String = hex_only[..12]
                    .chars()
                    .collect::<Vec<_>>()
                    .chunks(2)
                    .map(|chunk| chunk.iter().collect::<String>())
                    .collect::<Vec<_>>()
                    .join(":");
                return Some(mac_bytes);
            }
        }
    }
    let hex_only: String = device_id
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .collect();

    if hex_only.len() >= 12 {
        let mac_bytes: String = hex_only[..12]
            .chars()
            .collect::<Vec<_>>()
            .chunks(2)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join(":");
        return Some(mac_bytes);
    }

    None
}

#[tauri::command]
pub async fn bluetooth_available() -> Result<bool, String> {
    Ok(Adapter::default().await.is_some())
}

#[tauri::command]
pub async fn list_devices() -> Result<Vec<DeviceInfo>, String> {
    let adapter = Adapter::default()
        .await
        .ok_or("Bluetooth adapter not found")?;
    adapter.wait_available().await.map_err(|e| format!("{e}"))?;

    // 将适配器包装为 Arc 以便跨线程共享
    let adapter = Arc::new(adapter);
    
    // 在独立的 tokio 任务中执行蓝牙扫描
    let scan_task = task::spawn({
        let adapter = Arc::clone(&adapter);
        async move {
            let mut devices = Vec::new();
            let mut device_ids = std::collections::HashSet::new();
            
            eprintln!("Starting device discovery in background thread...");
            
            // 连接设备检查逻辑
            match adapter.connected_devices().await {
                Ok(connected) => {
                    for device in connected {
                        let id: String = device.id().to_string();
                        if !device_ids.contains(&id) {
                            device_ids.insert(id.clone());
                            let name: Option<String> = device.name_async().await.ok();
                            let mac_address: Option<String> = extract_mac_address(&id);
                            devices.push(DeviceInfo {
                                id,
                                mac_address,
                                name,
                                connected: true,
                            });
                        }
                    }
                }
                Err(e) => eprintln!("Error getting connected devices: {}", e),
            }
            
            // 设备扫描部分
            match adapter.discover_devices(&[HRS_UUID]).await {
                Ok(mut scan) => {
                    let start = Instant::now();
                    let timeout = Duration::from_secs(5);
                    let mut count: usize = 0;
                    
                    while start.elapsed() < timeout && count < 100 {
                        match tokio::time::timeout(Duration::from_millis(100), scan.next()).await {
                            Ok(Some(Ok(device))) => {
                                let id: String = device.id().to_string();
                                if !device_ids.contains(&id) {
                                    device_ids.insert(id.clone());
                                    let mac_address: Option<String> = extract_mac_address(&id);
                                    match device.name_async().await {
                                        Ok(name) => {
                                            devices.push(DeviceInfo {
                                                id,
                                                mac_address,
                                                name: Some(name),
                                                connected: false,
                                            });
                                            count += 1;
                                        }
                                        Err(_) => {
                                            devices.push(DeviceInfo {
                                                id,
                                                mac_address,
                                                name: None,
                                                connected: false,
                                            });
                                            count += 1;
                                        }
                                    }
                                }
                            }
                            Ok(Some(Err(e))) => {
                                eprintln!("Discovery error: {}", e);
                                continue;
                            }
                            Ok(None) => {
                                eprintln!("Scan ended");
                                break;
                            }
                            Err(_) => {
                                continue;
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error starting device discovery: {}", e);
                }
            }
            
            eprintln!("Background discovery completed. Found {} devices", devices.len());
            devices
        }
    });
    
    // 等待后台任务完成并获取结果
    match scan_task.await {
        Ok(devices) => Ok(devices),
        Err(e) => Err(format!("Background task failed: {}", e)),
    }
}

#[tauri::command]
pub async fn select_device(id: String) -> Result<(), String> {
    let adapter = Adapter::default()
        .await
        .ok_or("Bluetooth adapter not found")?;
    adapter.wait_available().await.map_err(|e| format!("{e}"))?;

    // check connected devices first
    match adapter.connected_devices_with_services(&[HRS_UUID]).await {
        Ok(connected) => {
            for device in connected.into_iter() {
                if device.id().to_string() == id {
                    match adapter.connect_device(&device).await {
                        Ok(_) => return Ok(()),
                        Err(e) => {
                            eprintln!("Failed to connect to device: {}", e);
                            return Err(format!("Failed to connect: {}", e));
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error getting connected devices: {}", e);
        }
    }

    match adapter.discover_devices(&[HRS_UUID]).await {
        Ok(mut scan) => {
            let start = Instant::now();
            let timeout = Duration::from_secs(5);

            while start.elapsed() < timeout {
                // 关键修改：为每次next()调用添加独立超时
                match tokio::time::timeout(Duration::from_millis(500), scan.next()).await {
                    Ok(Some(Ok(device))) => {
                        if device.id().to_string() == id {
                            match adapter.connect_device(&device).await {
                                Ok(_) => return Ok(()),
                                Err(e) => return Err(format!("Failed to connect: {}", e)),
                            }
                        }
                    }
                    Ok(Some(Err(e))) => {
                        eprintln!("Discovery error: {}", e);
                        continue;
                    }
                    Ok(None) => {
                        eprintln!("Scan ended");
                        break;
                    }
                    Err(_) => {
                        // 500毫秒内没收到设备，继续循环
                        continue;
                    }
                }
            }
        }
        Err(e) => eprintln!("Error starting device discovery: {}", e),
    }

    Err("Device not found".into())
}

#[tauri::command]
pub async fn start_heart_rate_stream(
    app: AppHandle,
    device_id: Option<String>,
) -> Result<(), String> {
    let adapter = Adapter::default()
        .await
        .ok_or("Bluetooth adapter not found")?;
    adapter.wait_available().await.map_err(|e| format!("{e}"))?;

    // try to find a suitable device
    let device = {
        // prefer already connected heart rate devices
        let connected = adapter
            .connected_devices_with_services(&[HRS_UUID])
            .await
            .map_err(|e| format!("{e}"))?;

        if let Some(ref wanted) = device_id {
            if let Some(d) = connected
                .into_iter()
                .find(|d| d.id().to_string() == *wanted)
            {
                Some(d)
            } else {
                None
            }
        } else {
            connected.into_iter().next()
        }
    };

    let device = if let Some(d) = device {
        d
    } else {
        let mut scan = adapter
            .discover_devices(&[HRS_UUID])
            .await
            .map_err(|e| format!("{e}"))?;

        let mut found = None;
        let start = Instant::now();
        let timeout = Duration::from_secs(5);

        while start.elapsed() < timeout {
            match scan.next().await {
                Some(Ok(d)) => {
                    if let Some(ref wanted) = device_id {
                        if d.id().to_string() == *wanted {
                            found = Some(d);
                            break;
                        }
                    } else {
                        found = Some(d);
                        break;
                    }
                }
                Some(Err(e)) => {
                    eprintln!("Discovery error: {}", e);
                    continue;
                }
                None => {
                    eprintln!("Scan ended");
                    break;
                }
            }
        }
        found.ok_or("Device not found".to_string())?
    };

    let app_clone = app.clone();
    let adapter_clone = adapter.clone();

    tauri::async_runtime::spawn(async move {
        if let Err(_e) = handle_device(&adapter_clone, &device, &app_clone).await {
            // let _ = app_clone.emit_to("main", "hr/error", format!("{_e}"));
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_heart_rate_stream() {
    // Implementation for stopping the heart rate stream
}

async fn handle_device(
    adapter: &Adapter,
    device: &Device,
    app: &AppHandle,
) -> Result<(), Box<dyn Error>> {
    if !device.is_connected().await {
        adapter.connect_device(&device).await?;
    }

    let heart_rate_services = device.discover_services_with_uuid(HRS_UUID).await?;
    let heart_rate_service = heart_rate_services
        .first()
        .ok_or("Device should have heart rate service")?;

    let heart_rate_measurements = heart_rate_service
        .discover_characteristics_with_uuid(HRM_UUID)
        .await?;
    let heart_rate_measurement = heart_rate_measurements
        .first()
        .ok_or("No HRM characteristic")?;

    let mut updates = heart_rate_measurement.notify().await?;
    while let Some(Ok(heart_rate)) = updates.next().await {
        let flag = *heart_rate.get(0).ok_or("No flag")?;

        let mut heart_rate_value = *heart_rate.get(1).ok_or("No heart rate u8")? as u16;
        if flag & 0b00001 != 0 {
            heart_rate_value |= (*heart_rate.get(2).ok_or("No heart rate u16")? as u16) << 8;
        }

        let mut _sensor_contact = None;
        if flag & 0b00100 != 0 {
            _sensor_contact = Some(flag & 0b00010 != 0)
        }
        let _ = app.emit_to("device", "heart-rate-update", heart_rate_value);
    }

    Err("No longer heart rate notify".into())
}
