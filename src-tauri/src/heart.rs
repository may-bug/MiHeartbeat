use std::error::Error;
use std::time::{Duration, Instant};

use bluest::{btuuid::bluetooth_uuid_from_u16, Adapter, Device, Uuid};
use futures_lite::stream::StreamExt;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;
use tokio::task;

// 常量定义
const HRS_UUID: Uuid = bluetooth_uuid_from_u16(0x180D);
const HRM_UUID: Uuid = bluetooth_uuid_from_u16(0x2A37);
const SCAN_TIMEOUT: Duration = Duration::from_secs(5);
const SCAN_INTERVAL: Duration = Duration::from_millis(100);
const DEVICE_TIMEOUT: Duration = Duration::from_millis(500);
const MAX_DEVICES: usize = 100;
const MAC_ADDRESS_LENGTH: usize = 12;

/// 设备信息
#[derive(Debug, Clone, Serialize)]
pub struct DeviceInfo {
    pub id: String,
    pub mac_address: Option<String>,
    pub name: Option<String>,
    pub connected: bool,
}

/// 心率流状态管理
static HEART_RATE_TASK: RwLock<Option<tokio::task::JoinHandle<()>>> = RwLock::const_new(None);

/// 从设备 ID 中提取 MAC 地址
fn extract_mac_address(device_id: &str) -> Option<String> {
    // 优先尝试从分隔符后提取
    let target = device_id
        .split('#')
        .last()
        .unwrap_or(device_id);

    // 提取十六进制字符
    let hex_only: String = target
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .take(MAC_ADDRESS_LENGTH)
        .collect();

    // 检查长度是否足够
    if hex_only.len() < MAC_ADDRESS_LENGTH {
        return None;
    }

    // 格式化为 MAC 地址格式
    Some(
        hex_only
            .as_bytes()
            .chunks(2)
            .map(|chunk| {
                std::str::from_utf8(chunk)
                    .unwrap_or("")
                    .to_uppercase()
            })
            .collect::<Vec<_>>()
            .join(":")
    )
}

/// 检查蓝牙适配器是否可用
#[tauri::command]
pub async fn bluetooth_available() -> Result<bool, String> {
    // Windows/macOS/Linux 通用检查
    match Adapter::default().await {
        Some(adapter) => {
            // 针对不同平台可能需要不同的超时设置
            #[cfg(target_os = "windows")]
            let timeout = std::time::Duration::from_secs(3);
            
            #[cfg(target_os = "macos")]
            let timeout = std::time::Duration::from_secs(2);
            
            #[cfg(target_os = "linux")]
            let timeout = std::time::Duration::from_secs(5);
            
            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            let timeout = std::time::Duration::from_secs(3);
            
            // 使用超时检查可用性
            match tokio::time::timeout(timeout, adapter.wait_available()).await {
                Ok(Ok(_)) => Ok(true),
                Ok(Err(e)) => {
                    eprintln!("Bluetooth adapter error: {e}");
                    Ok(false)
                }
                Err(_) => {
                    eprintln!("Bluetooth availability check timeout");
                    Ok(false)
                }
            }
        }
        None => Ok(false),
    }
}

/// 获取适配器并等待可用
async fn get_adapter() -> Result<Adapter, String> {
    let adapter = Adapter::default()
        .await
        .ok_or_else(|| "Bluetooth adapter not found".to_string())?;
    
    adapter
        .wait_available()
        .await
        .map_err(|e| format!("Adapter not available: {e}"))?;
    
    Ok(adapter)
}

/// 从设备构建设备信息
async fn device_to_info(device: &Device, connected: bool) -> DeviceInfo {
    let id = device.id().to_string();
    let mac_address = extract_mac_address(&id);
    let name = device.name_async().await.ok();
    
    DeviceInfo {
        id,
        mac_address,
        name,
        connected,
    }
}

/// 收集已连接的设备
async fn collect_connected_devices(
    adapter: &Adapter,
    device_ids: &mut std::collections::HashSet<String>,
) -> Vec<DeviceInfo> {
    let mut devices = Vec::new();
    
    match adapter.connected_devices().await {
        Ok(connected) => {
            for device in connected {
                let id = device.id().to_string();
                if device_ids.insert(id.clone()) {
                    devices.push(device_to_info(&device, true).await);
                }
            }
        }
        Err(e) => eprintln!("Error getting connected devices: {e}"),
    }
    
    devices
}

/// 扫描并收集新设备
async fn scan_for_devices(
    adapter: &Adapter,
    device_ids: &mut std::collections::HashSet<String>,
) -> Vec<DeviceInfo> {
    let mut devices = Vec::new();
    
    let Ok(mut scan) = adapter.discover_devices(&[HRS_UUID]).await else {
        eprintln!("Failed to start device discovery");
        return devices;
    };
    
    let start = Instant::now();
    let mut count = 0;
    
    while start.elapsed() < SCAN_TIMEOUT && count < MAX_DEVICES {
        let timeout_result = tokio::time::timeout(SCAN_INTERVAL, scan.next()).await;
        
        match timeout_result {
            Ok(Some(Ok(device))) => {
                let id = device.id().to_string();
                if device_ids.insert(id.clone()) {
                    devices.push(device_to_info(&device, false).await);
                    count += 1;
                }
            }
            Ok(Some(Err(e))) => {
                eprintln!("Discovery error: {e}");
            }
            Ok(None) => {
                eprintln!("Scan completed");
                break;
            }
            Err(_) => {
                // Timeout, continue scanning
                continue;
            }
        }
    }
    
    devices
}

/// 列出所有可用的蓝牙设备
#[tauri::command]
pub async fn list_devices() -> Result<Vec<DeviceInfo>, String> {
    let adapter = get_adapter().await?;
    
    let mut device_ids = std::collections::HashSet::new();
    
    eprintln!("Starting device discovery...");
    
    // 收集已连接的设备
    let mut all_devices = collect_connected_devices(&adapter, &mut device_ids).await;
    
    // 扫描新设备
    let scanned_devices = scan_for_devices(&adapter, &mut device_ids).await;
    all_devices.extend(scanned_devices);
    
    eprintln!("Discovery completed. Found {} devices", all_devices.len());
    
    Ok(all_devices)
}

/// 查找并连接指定设备
async fn find_and_connect_device(
    adapter: &Adapter,
    target_id: &str,
) -> Result<(), String> {
    // 首先检查已连接的设备
    if let Ok(connected) = adapter.connected_devices_with_services(&[HRS_UUID]).await {
        for device in connected {
            if device.id().to_string() == target_id {
                return adapter
                    .connect_device(&device)
                    .await
                    .map_err(|e| format!("Failed to connect: {e}"));
            }
        }
    }
    
    // 如果未找到，开始扫描
    let Ok(mut scan) = adapter.discover_devices(&[HRS_UUID]).await else {
        return Err("Failed to start device discovery".to_string());
    };
    
    let start = Instant::now();
    
    while start.elapsed() < SCAN_TIMEOUT {
        let timeout_result = tokio::time::timeout(DEVICE_TIMEOUT, scan.next()).await;
        
        match timeout_result {
            Ok(Some(Ok(device))) => {
                if device.id().to_string() == target_id {
                    return adapter
                        .connect_device(&device)
                        .await
                        .map_err(|e| format!("Failed to connect: {e}"));
                }
            }
            Ok(Some(Err(e))) => {
                eprintln!("Discovery error: {e}");
            }
            Ok(None) => {
                break;
            }
            Err(_) => {
                // Timeout, continue scanning
                continue;
            }
        }
    }
    
    Err("Device not found".to_string())
}

/// 选择并连接到指定设备
#[tauri::command]
pub async fn select_device(id: String) -> Result<(), String> {
    let adapter = get_adapter().await?;
    find_and_connect_device(&adapter, &id).await
}

/// 查找合适的心率设备
async fn find_heart_rate_device(
    adapter: &Adapter,
    device_id: Option<&str>,
) -> Result<Device, String> {
    // 优先使用已连接的设备
    let connected = adapter
        .connected_devices_with_services(&[HRS_UUID])
        .await
        .map_err(|e| format!("Failed to get connected devices: {e}"))?;
    
    if let Some(device) = find_device_in_list(&connected, device_id) {
        return Ok(device);
    }
    
    // 如果没有找到，开始扫描
    scan_for_device(adapter, device_id).await
}

/// 在设备列表中查找指定设备
fn find_device_in_list(devices: &[Device], target_id: Option<&str>) -> Option<Device> {
    match target_id {
        Some(id) => devices
            .iter()
            .find(|d| d.id().to_string() == id)
            .cloned(),
        None => devices.first().cloned(),
    }
}

/// 扫描并查找设备
async fn scan_for_device(
    adapter: &Adapter,
    device_id: Option<&str>,
) -> Result<Device, String> {
    let mut scan = adapter
        .discover_devices(&[HRS_UUID])
        .await
        .map_err(|e| format!("Failed to start scan: {e}"))?;
    
    let start = Instant::now();
    
    while start.elapsed() < SCAN_TIMEOUT {
        match scan.next().await {
            Some(Ok(device)) => {
                let is_target = match device_id {
                    Some(id) => device.id().to_string() == id,
                    None => true,
                };
                
                if is_target {
                    return Ok(device);
                }
            }
            Some(Err(e)) => {
                eprintln!("Discovery error: {e}");
            }
            None => {
                break;
            }
        }
    }
    
    Err("Device not found".to_string())
}

/// 开始心率数据流
#[tauri::command]
pub async fn start_heart_rate_stream(
    app: AppHandle,
    device_id: Option<String>,
) -> Result<(), String> {
    // 取消现有任务
    stop_heart_rate_stream().await;
    
    let adapter = get_adapter().await?;
    let device = find_heart_rate_device(&adapter, device_id.as_deref()).await?;
    
    // 启动新任务
    let task = task::spawn(async move {
        if let Err(e) = handle_device(&adapter, &device, &app).await {
            eprintln!("Heart rate stream error: {e}");
            let _ = app.emit_to("device", "heart-rate-error", format!("{e}"));
        }
    });
    
    // 保存任务句柄
    *HEART_RATE_TASK.write().await = Some(task);
    
    Ok(())
}

/// 停止心率数据流
#[tauri::command]
pub async fn stop_heart_rate_stream() {
    let mut task_lock = HEART_RATE_TASK.write().await;
    
    if let Some(task) = task_lock.take() {
        task.abort();
        eprintln!("Heart rate stream stopped");
    }
}

/// 处理设备心率数据
async fn handle_device(
    adapter: &Adapter,
    device: &Device,
    app: &AppHandle,
) -> Result<(), Box<dyn Error>> {
    // 确保设备已连接
    if !device.is_connected().await {
        adapter.connect_device(device).await?;
    }
    
    // 发现心率服务
    let heart_rate_services = device
        .discover_services_with_uuid(HRS_UUID)
        .await?;
    
    let heart_rate_service = heart_rate_services
        .first()
        .ok_or("Device does not have heart rate service")?;
    
    // 发现心率测量特征
    let heart_rate_measurements = heart_rate_service
        .discover_characteristics_with_uuid(HRM_UUID)
        .await?;
    
    let heart_rate_measurement = heart_rate_measurements
        .first()
        .ok_or("No heart rate measurement characteristic found")?;
    
    // 开始接收通知
    let mut updates = heart_rate_measurement.notify().await?;
    
    while let Some(update_result) = updates.next().await {
        match update_result {
            Ok(heart_rate_data) => {
                match parse_heart_rate(&heart_rate_data) {
                    Ok(heart_rate_value) => {
                        let _ = app.emit_to("device", "heart-rate-update", heart_rate_value);
                    }
                    Err(e) => {
                        eprintln!("Failed to parse heart rate data: {e}");
                    }
                }
            }
            Err(e) => {
                eprintln!("Heart rate update error: {e}");
            }
        }
    }
    
    Err("Heart rate notifications ended".into())
}

/// 解析心率数据
fn parse_heart_rate(data: &[u8]) -> Result<u16, Box<dyn Error>> {
    // 检查数据长度
    if data.is_empty() {
        return Err("Empty heart rate data".into());
    }
    
    let flags = data[0];
    
    // 检查心率值格式（bit 0: 0=uint8, 1=uint16）
    let heart_rate_value = if flags & 0b0000_0001 != 0 {
        // 16-bit heart rate value
        if data.len() < 3 {
            return Err("Insufficient data for 16-bit heart rate".into());
        }
        u16::from_le_bytes([data[1], data[2]])
    } else {
        // 8-bit heart rate value
        if data.len() < 2 {
            return Err("Insufficient data for 8-bit heart rate".into());
        }
        data[1] as u16
    };
    
    // 检查传感器接触状态（bits 1-2）
    if flags & 0b0000_0100 != 0 {
        let sensor_contact = flags & 0b0000_0010 != 0;
        if !sensor_contact {
            eprintln!("Warning: Sensor contact lost");
        }
    }
    
    Ok(heart_rate_value)
}