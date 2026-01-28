use std::error::Error;
use std::time::{Duration, Instant};

use bluest::{btuuid::bluetooth_uuid_from_u16, Adapter, Device, Uuid};
use futures_lite::stream::StreamExt;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;
use tokio::time::timeout;

// 常量定义
const HRS_UUID: Uuid = bluetooth_uuid_from_u16(0x180D);
const HRM_UUID: Uuid = bluetooth_uuid_from_u16(0x2A37);
const SCAN_TIMEOUT: Duration = Duration::from_secs(5);
const SCAN_INTERVAL: Duration = Duration::from_millis(100);
const _DEVICE_TIMEOUT: Duration = Duration::from_millis(500);
const MAX_DEVICES: usize = 100;
const MAC_ADDRESS_LENGTH: usize = 12;

// Linux D-Bus 优化常量
const DBUS_OPERATION_TIMEOUT: Duration = Duration::from_secs(30);
const OPERATION_DELAY: Duration = Duration::from_millis(300);
const MAX_RETRIES: u32 = 5;
const RECONNECT_DELAY: Duration = Duration::from_secs(2);

/// 设备信息
#[derive(Debug, Clone, Serialize)]
pub struct DeviceInfo {
    pub id: String,
    pub mac_address: Option<String>,
    pub name: Option<String>,
    pub connected: bool,
}

/// 全局心率流状态
struct HeartRateStreamState {
    task: Option<tokio::task::JoinHandle<()>>,
    selected_device_id: Option<String>,
    is_running: bool,
}

impl HeartRateStreamState {
    const fn new() -> Self {
        Self {
            task: None,
            selected_device_id: None,
            is_running: false,
        }
    }
}

/// 心率流状态管理
static HEART_RATE_STATE: RwLock<HeartRateStreamState> = RwLock::const_new(HeartRateStreamState::new());

/// 检查蓝牙适配器是否可用
#[tauri::command]
pub async fn bluetooth_available() -> Result<bool, String> {
    match Adapter::default().await {
        Some(adapter) => {
            #[cfg(target_os = "linux")]
            let check_timeout = Duration::from_secs(10);

            #[cfg(target_os = "windows")]
            let check_timeout = std::time::Duration::from_secs(3);

            #[cfg(target_os = "macos")]
            let check_timeout = std::time::Duration::from_secs(2);

            match timeout(check_timeout, adapter.wait_available()).await {
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

    #[cfg(target_os = "linux")]
    {
        timeout(DBUS_OPERATION_TIMEOUT, adapter.wait_available())
            .await
            .map_err(|_| "Adapter availability check timeout".to_string())?
            .map_err(|e| format!("Adapter not available: {e}"))?;
    }

    #[cfg(not(target_os = "linux"))]
    {
        adapter
            .wait_available()
            .await
            .map_err(|e| format!("Adapter not available: {e}"))?;
    }

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
    let mut all_devices = collect_connected_devices(&adapter, &mut device_ids).await;
    let scanned_devices = scan_for_devices(&adapter, &mut device_ids).await;
    all_devices.extend(scanned_devices);
    eprintln!("Discovery completed. Found {} devices", all_devices.len());
    Ok(all_devices)
}

/// 选择设备
#[tauri::command]
pub async fn select_device(id: String) -> Result<(), String> {
    let mut state = HEART_RATE_STATE.write().await;
    // 如果正在运行，先停止
    if state.is_running {
        if let Some(task) = state.task.take() {
            task.abort();
            eprintln!("Stopped existing heart rate stream before selecting new device");
        }
        state.is_running = false;
    }
    state.selected_device_id = Some(id.clone());
    eprintln!("Selected device: {id}");

    Ok(())
}

/// 获取当前选中的设备 ID
#[tauri::command]
pub async fn _get_selected_device() -> Result<Option<String>, String> {
    let state = HEART_RATE_STATE.read().await;
    Ok(state.selected_device_id.clone())
}

/// 检查心率流是否正在运行
#[tauri::command]
pub async fn is_heart_rate_streaming() -> Result<bool, String> {
    let state = HEART_RATE_STATE.read().await;
    Ok(state.is_running)
}

#[cfg(target_os = "linux")]
async fn connect_device_linux(
    adapter: &Adapter,
    device: &Device,
) -> Result<(), Box<dyn Error>> {
    eprintln!("Linux: Starting device connection with D-Bus optimizations...");

    // 配对（失败不致命）
    let _ = timeout(Duration::from_secs(15), device.pair()).await;

    tokio::time::sleep(OPERATION_DELAY).await;

    // 连接（失败才返回）
    if let Err(e) = timeout(DBUS_OPERATION_TIMEOUT, adapter.connect_device(device)).await {
        eprintln!("Connect timeout: {e}");
    }

    tokio::time::sleep(Duration::from_secs(1)).await;

    // 发现服务（核心修复点）
    for attempt in 1..=MAX_RETRIES {
        eprintln!("Discovering services (attempt {}/{})", attempt, MAX_RETRIES);

        match timeout(DBUS_OPERATION_TIMEOUT, device.discover_services()).await {
            Ok(Ok(_)) => return Ok(()),

            // ★ 关键：Linux 下这些错误不能 panic
            Ok(Err(e)) => {
                let msg = e.to_string();
                if msg.contains("not found")
                    || msg.contains("removed")
                    || msg.contains("Page Timeout")
                {
                    tokio::time::sleep(RECONNECT_DELAY).await;
                    continue;
                }
                return Err(e.into());
            }

            Err(_) => {
                tokio::time::sleep(RECONNECT_DELAY).await;
            }
        }
    }
    eprintln!("Service discovery skipped/failed, continuing anyway");
    Ok(())

}

/// 非 Linux 平台的标准连接方法
#[cfg(not(target_os = "linux"))]
async fn connect_device_standard(adapter: &Adapter, device: &Device) -> Result<(), Box<dyn Error>> {
    if !device.is_connected().await {
        adapter.connect_device(device).await?;
        eprintln!("Device connected");
    }
    Ok(())
}

/// 查找心率设备
async fn find_heart_rate_device(
    adapter: &Adapter,
    device_id: Option<&str>,
) -> Result<Device, String> {
    // 优先检查已连接的设备
    if let Ok(connected) = adapter.connected_devices_with_services(&[HRS_UUID]).await {
        if let Some(device) = find_device_in_list(&connected, device_id) {
            eprintln!("Found device in connected devices");
            return Ok(device);
        }
    }

    // 扫描新设备
    eprintln!("Scanning for device...");
    scan_for_device(adapter, device_id).await
}

/// 在设备列表中查找指定设备
fn find_device_in_list(devices: &[Device], target_id: Option<&str>) -> Option<Device> {
    match target_id {
        Some(id) => devices.iter().find(|d| d.id().to_string() == id).cloned(),
        None => devices.first().cloned(),
    }
}

/// 扫描并查找设备
async fn scan_for_device(adapter: &Adapter, device_id: Option<&str>) -> Result<Device, String> {
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
                    eprintln!("Found target device");
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

    Err("Device not found during scan".to_string())
}

/// 开始心率数据流
#[tauri::command]
pub async fn start_heart_rate_stream(app: AppHandle) -> Result<(), String> {
    let mut state = HEART_RATE_STATE.write().await;

    // 检查是否已经在运行
    if state.is_running {
        return Err("Heart rate stream is already running".to_string());
    }

    // 检查是否有选中的设备
    let device_id = state
        .selected_device_id
        .clone()
        .ok_or("No device selected. Please select a device first.")?;

    eprintln!("Starting heart rate stream for device: {device_id}");

    let adapter = get_adapter().await?;
    let device = find_heart_rate_device(&adapter, Some(&device_id)).await?;

    // 启动新任务
    let task = tokio::task::spawn(async move {
        if let Err(e) = handle_heart_rate_stream(&adapter, &device, &app).await {
            eprintln!("Heart rate stream error: {e}");
            let _ = app.emit("heart-rate-error", format!("{e}"));
            let _ = app.emit("heart-rate-stopped", ());
        }
    });

    state.task = Some(task);
    state.is_running = true;

    eprintln!("✓ Heart rate stream started");

    Ok(())
}

/// 停止心率数据流
#[tauri::command]
pub async fn stop_heart_rate_stream(app: AppHandle) -> Result<(), String> {
    let mut state = HEART_RATE_STATE.write().await;

    if let Some(task) = state.task.take() {
        task.abort();
        state.is_running = false;
        eprintln!("Heart rate stream stopped");

        // 全局广播停止事件
        let _ = app.emit("heart-rate-stopped", ());

        Ok(())
    } else {
        Err("No heart rate stream is running".to_string())
    }
}

/// 处理心率数据流
async fn handle_heart_rate_stream(
    adapter: &Adapter,
    device: &Device,
    app: &AppHandle,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut consecutive_errors = 0;
    const MAX_CONSECUTIVE_ERRORS: u32 = 3;

    loop {
        match process_heart_rate_notifications(adapter, device, app).await {
            Ok(_) => {
                eprintln!("Heart rate notifications ended normally");
                break;
            }
            Err(e) => {
                consecutive_errors += 1;
                eprintln!("Error in heart rate stream (attempt {}/{}): {e}",
                          consecutive_errors, MAX_CONSECUTIVE_ERRORS);

                if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                    return Err(format!("Failed after {} consecutive errors", MAX_CONSECUTIVE_ERRORS).into());
                }

                // 等待后重试
                eprintln!("Waiting before retry...");
                tokio::time::sleep(RECONNECT_DELAY).await;

                // 尝试重新连接
                eprintln!("Attempting to reconnect...");
                #[cfg(target_os = "linux")]
                {
                    if let Err(e) = connect_device_linux(adapter, device).await {
                        eprintln!("Reconnection failed: {e}");
                        continue;
                    }
                }
                #[cfg(not(target_os = "linux"))]
                {
                    if let Err(e) = connect_device_standard(adapter, device).await {
                        eprintln!("Reconnection failed: {e}");
                        continue;
                    }
                }

                eprintln!("Reconnected, retrying stream...");
            }
        }
    }

    Ok(())
}

/// 处理心率通知
async fn process_heart_rate_notifications(
    adapter: &Adapter,
    device: &Device,
    app: &AppHandle,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // 确保设备已连接
    #[cfg(target_os = "linux")]
    connect_device_linux(adapter, device).await.expect("linux connect");

    #[cfg(not(target_os = "linux"))]
    let _=connect_device_standard(adapter, device).await;

    // 查找心率服务和特征
    let heart_rate_measurement = find_heart_rate_characteristic_with_retry(device).await?;

    eprintln!("Starting heart rate notifications...");

    // 订阅通知
    #[cfg(target_os = "linux")]
    let mut updates = {
        timeout(DBUS_OPERATION_TIMEOUT, heart_rate_measurement.notify())
            .await
            .map_err(|_| "Notification subscription timeout")??
    };

    #[cfg(not(target_os = "linux"))]
    let mut updates = heart_rate_measurement.notify().await?;

    eprintln!("Successfully subscribed to heart rate notifications");

    // 处理通知流
    while let Some(update_result) = updates.next().await {
        match update_result {
            Ok(heart_rate_data) => {
                match parse_heart_rate(&heart_rate_data) {
                    Ok(heart_rate_value) => {
                        // 全局广播心率更新
                        let _ = app.emit("heart-rate-update", heart_rate_value);
                    }
                    Err(e) => {
                        eprintln!("Failed to parse heart rate data: {e}");
                    }
                }
            }
            Err(e) => {
                return Err(format!("Heart rate update error: {e}").into());
            }
        }
    }

    Err("Notification stream ended".into())
}

/// 查找心率特征
async fn find_heart_rate_characteristic_with_retry(
    device: &Device,
) -> Result<bluest::Characteristic, Box<dyn Error + Send + Sync>> {
    for attempt in 1..=MAX_RETRIES {
        eprintln!("Finding heart rate characteristic (attempt {}/{})", attempt, MAX_RETRIES);

        match find_heart_rate_characteristic(device).await {
            Ok(char) => {
                eprintln!("Found heart rate characteristic");
                return Ok(char);
            }
            Err(e) => {
                eprintln!("Failed to find characteristic: {e}");
                if attempt < MAX_RETRIES {
                    tokio::time::sleep(RECONNECT_DELAY).await;

                    #[cfg(target_os = "linux")]
                    {
                        if let Err(e) = timeout(DBUS_OPERATION_TIMEOUT, device.discover_services()).await {
                            eprintln!("Service rediscovery failed: {:?}", e);
                        }
                        tokio::time::sleep(OPERATION_DELAY).await;
                    }

                    #[cfg(not(target_os = "linux"))]
                    {
                        let _ = device.discover_services().await;
                    }
                } else {
                    return Err(e);
                }
            }
        }
    }

    Err("Failed to find heart rate characteristic after all retries".into())
}

/// 查找心率特征
async fn find_heart_rate_characteristic(
    device: &Device,
) -> Result<bluest::Characteristic, Box<dyn Error + Send + Sync>> {
    #[cfg(target_os = "linux")]
    let heart_rate_services = {
        timeout(DBUS_OPERATION_TIMEOUT, device.discover_services_with_uuid(HRS_UUID))
            .await
            .map_err(|_| "Service discovery timeout")??
    };

    #[cfg(not(target_os = "linux"))]
    let heart_rate_services = device.discover_services_with_uuid(HRS_UUID).await?;

    let heart_rate_service = heart_rate_services
        .first()
        .ok_or("Device does not have heart rate service")?;

    #[cfg(target_os = "linux")]
    tokio::time::sleep(OPERATION_DELAY).await;

    #[cfg(target_os = "linux")]
    let heart_rate_measurements = {
        timeout(
            DBUS_OPERATION_TIMEOUT,
            heart_rate_service.discover_characteristics_with_uuid(HRM_UUID)
        )
            .await
            .map_err(|_| "Characteristic discovery timeout")??
    };

    #[cfg(not(target_os = "linux"))]
    let heart_rate_measurements = heart_rate_service
        .discover_characteristics_with_uuid(HRM_UUID)
        .await?;

    let heart_rate_measurement = heart_rate_measurements
        .first()
        .ok_or("No heart rate measurement characteristic found")?;

    Ok(heart_rate_measurement.clone())
}

/// 解析心率数据
fn parse_heart_rate(data: &[u8]) -> Result<u16, Box<dyn Error>> {
    if data.is_empty() {
        return Err("Empty heart rate data".into());
    }

    let flags = data[0];

    let heart_rate_value = if flags & 0b0000_0001 != 0 {
        if data.len() < 3 {
            return Err("Insufficient data for 16-bit heart rate".into());
        }
        u16::from_le_bytes([data[1], data[2]])
    } else {
        if data.len() < 2 {
            return Err("Insufficient data for 8-bit heart rate".into());
        }
        data[1] as u16
    };

    if flags & 0b0000_0100 != 0 {
        let sensor_contact = flags & 0b0000_0010 != 0;
        if !sensor_contact {
            eprintln!("Warning: Sensor contact lost");
        }
    }

    Ok(heart_rate_value)
}


/// 从设备 ID 中提取 MAC 地址
fn extract_mac_address(device_id: &str) -> Option<String> {
    let target = device_id.split('#').last().unwrap_or(device_id);

    let hex_only: String = target
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .take(MAC_ADDRESS_LENGTH)
        .collect();

    if hex_only.len() < MAC_ADDRESS_LENGTH {
        return None;
    }
    Some(
        hex_only
            .as_bytes()
            .chunks(2)
            .map(|chunk| std::str::from_utf8(chunk).unwrap_or("").to_uppercase())
            .collect::<Vec<_>>()
            .join(":")
    )
}