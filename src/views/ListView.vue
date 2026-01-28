<script setup>
import { onMounted, onUnmounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { platform } from '@tauri-apps/plugin-os';
import TitleBar from "../components/TitleBar.vue";
import { hideWindow, createDeviceWindow } from "../utils/window";
import { getDeviceIcon, isImagePath, getPlatformIcon } from "../utils/image";
import BLUETOOTH from "../assets/icons/bluetooth.svg";
import { listen } from '@tauri-apps/api/event';

const bluetooth = ref(false);
const deviceList = ref([]);
const isScanning = ref(false);
const selectedDevice = ref(null);
const isConnecting = ref(false);
const deviceIcons = ref({});
const platformInfo = ref({
    icon: '',
    type: ''
});

const MAX_RETRY_COUNT = 10; // 最大重试次数
const RETRY_INTERVAL = 3000; // 重试间隔（毫秒）

let retryCount = 0;
let retryTimer= null;

const getPlatformInfo = async () => {
    try {
        const currentPlatform = platform();
        platformInfo.value = {
            type: currentPlatform
        };
        platformInfo.value.icon = await getPlatformIcon(currentPlatform);
    } catch (error) {
        console.error("Error getting platform info:", error);
        platformInfo.value = {
            type: 'Unknown'
        };
    }
};

const checkBluetooth = async () => {
    try {
        const status = await invoke("bluetooth_available");
        bluetooth.value = status;

        if (status) {
            // 蓝牙可用，清除重试计数器和定时器
            retryCount = 0;
            if (retryTimer) {
                clearTimeout(retryTimer);
                retryTimer = null;
            }
            getDeviceList();
        } else {
            // 蓝牙不可用，启动轮询
            startRetry();
        }
    } catch (error) {
        console.error('Failed to check Bluetooth:', error);
        bluetooth.value = false;
        startRetry();
    }
};

const startRetry = () => {
    // 如果已经有定时器在运行，不要重复启动
    if (retryTimer) return;

    // 检查是否超过最大重试次数
    if (retryCount >= MAX_RETRY_COUNT) {
        console.log('Max retry count reached. Stopping Bluetooth checks.');
        return;
    }

    retryCount++;
    console.log(`Retrying Bluetooth check (${retryCount}/${MAX_RETRY_COUNT})...`);

    retryTimer = setTimeout(() => {
        retryTimer = null;
        checkBluetooth();
    }, RETRY_INTERVAL);
};

// 手动重置重试
const resetRetry = () => {
    retryCount = 0;
    if (retryTimer) {
        clearTimeout(retryTimer);
        retryTimer = null;
    }
    checkBluetooth();
};

// 停止轮询
const stopRetry = () => {
    if (retryTimer) {
        clearTimeout(retryTimer);
        retryTimer = null;
    }
    retryCount = 0;
};

const getDeviceList = async () => {
    try {
        isScanning.value = true;
        let devices = await invoke("list_devices");
        console.log("Discovered devices:", devices);
        deviceList.value = devices;
        for (const device of devices) {
            if (device.name && !deviceIcons.value[device.id]) {
                deviceIcons.value[device.id] = await getDeviceIcon(device.name);
            }
        }
    } catch (error) {
        console.error("Error getting device list:", error);
    } finally {
        isScanning.value = false;
    }
};

const refreshDeviceList = async () => {
    await getDeviceList();
};

const formatDeviceId = (deviceId) => {
   if (!deviceId) return 'Unknown';
   const cleanId = deviceId.replace(/^BluetoothLE#BluetoothLE/, '');
   return cleanId.substring(0, 35) + (cleanId.length > 35 ? '...' : '');
};

const selectDevice = async (device) => {
    try {
        const encodedName = encodeURIComponent(device.name || 'Unknown');
        const encodedId = encodeURIComponent(device.id);
        createDeviceWindow(`/device/${encodedId}/${encodedName}`, device.name);
        selectedDevice.value = device;
        isConnecting.value = true;
        hideWindow("main")
        await invoke("select_device", { id: device.id });
    } catch (error) {
        selectedDevice.value = null;
    } finally {
        isConnecting.value = false;
    }
};

const addListeners = () => {
    listen("main-data-service", (event) => {
        console.log("Device connected:", event.payload);
        isConnecting.value = false;
    });
};

onMounted(async () => {
    await getPlatformInfo();
    await checkBluetooth();
});
onUnmounted(() => {
    stopRetry();
});
</script>

<template>
    <div class="container">
        <TitleBar title="Mi心动" :showSettings="true" />
        <div class="main-content">
            <!-- 平台信息卡片 -->
            <div class="platform-card">
                <div class="platform-left">
                    <img :src="platformInfo.icon" alt="platform" class="platform-icon"></img>
                    <div class="platform-text">
                        <p class="platform-label">设备平台</p>
                        <p class="platform-value">{{ platformInfo.type || 'unkown' }}</p>
                    </div>
                </div>
                <div class="platform-divider"></div>
                <div class="platform-right">
                    <img :src="BLUETOOTH" alt="bluetooth" class="bluetooth-icon">
                    <div class="bluetooth-text">
                        <p class="bluetooth-label">蓝牙状态</p>
                        <p class="bluetooth-value" :class="{ active: bluetooth }">{{ bluetooth ? '已启用' : '未启用' }}</p>
                    </div>
                </div>
            </div>

            <!-- 控制栏卡片 -->
            <div class="control-bar-card">
                <div class="control-left">
                    <p v-if="isScanning" class="scanning-text">正在扫描蓝牙设备...</p>
                    <p v-else class="scanning-text idle">心率设备</p>
                </div>
                <div class="control-right">
                    <button v-if="!isScanning" @click="refreshDeviceList" class="btn-refresh">
                        刷新
                    </button>
                    <button v-else class="btn-refresh" disabled>
                        扫描中
                    </button>
                </div>
            </div>

            <!-- 设备列表 -->
            <div class="devices-container">
                <!-- 空状态 -->
                <div v-if="deviceList.length === 0 && !isScanning" class="empty-state">
                    <div class="empty-icon"></div>
                    <p>未发现设备</p>
                    <small>请确保蓝牙已启用并将设备置于配对模式</small>
                </div>

                <!-- 设备列表 -->
                <div v-else class="device-list">
                    <div v-for="device in deviceList" :key="device.id" class="device-card"
                        :class="{ 'is-connecting': isConnecting && selectedDevice?.id === device.id }">
                        <div class="device-icon-wrapper">
                            <img v-if="isImagePath(deviceIcons[device.id])" :src="deviceIcons[device.id]"
                                :alt="device.name" class="device-image" />
                            <div v-else class="device-icon">
                                {{ typeof deviceIcons[device.id] === 'string' ? deviceIcons[device.id] : '📱' }}
                            </div>
                        </div>

                        <div class="device-info">
                            <div class="name-info">
                               <h3 class="device-name">{{ device.name || "未知设备" }}</h3>
                                <p class="device-mac" v-if="device.mac_address">MAC: {{ device.mac_address }}</p>
                            </div>

                            <!-- 设备ID和操作按钮 -->
                            <div class="device-footer">
                                <span class="device-id">{{ formatDeviceId(device.id) }}</span>
                                <button @click="selectDevice(device)"
                                    :disabled="isConnecting || selectedDevice?.id === device.id" class="btn-connect">
                                    <span v-if="isConnecting && selectedDevice?.id === device.id">
                                        连接中...
                                    </span>
                                    <span v-else>
                                        连接
                                    </span>
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
* {
    box-sizing: border-box;
}

.container {
    width: 100%;
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    color: #333;
}

.bluetooth-status {
    display: flex;
    align-items: center;
    padding: 5px 10px;
    border-radius: 20px;
    background: rgba(0, 0, 0, 0.04);
    font-size: 13px;
    font-weight: 500;
    color: #666;
    transition: all 0.3s ease;
}

.bluetooth-status.active {
    background: rgba(76, 175, 80, 0.1);
    color: #4caf50;
}

.status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #999;
    display: inline-block;
    animation: none;
}

.bluetooth-status.active .status-dot {
    background: #4caf50;
    animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {

    0%,
    100% {
        opacity: 1;
    }

    50% {
        opacity: 0.5;
    }
}

.main-content {
    flex: 1;
    overflow-y: auto;
    padding: 30px 20px;
    display: flex;
    flex-direction: column;
    gap: 20px;
}

.platform-card {
    background: rgba(255, 255, 255, 0.7);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.5);
    border-radius: 12px;
    padding: 16px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
    transition: all 0.3s ease;
}

.platform-card:hover {
    background: rgba(255, 255, 255, 0.8);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
}

.platform-left,
.platform-right {
    display: flex;
    align-items: center;
    gap: 12px;
    flex: 1;
}

.platform-icon,
.bluetooth-icon {
    width: 30px;
    height: 30px;
    font-size: 24px;
    opacity: 0.8;
}

.platform-text,
.bluetooth-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
}

.platform-label,
.bluetooth-label {
    margin: 0;
    font-size: 12px;
    font-weight: 500;
    color: #999;
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

.platform-value,
.bluetooth-value {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: #333;
}

.bluetooth-value.active {
    color: #4caf50;
}

.platform-divider {
    width: 1px;
    height: 40px;
    background: rgba(0, 0, 0, 0.1);
    border-radius: 1px;
}

.control-bar-card {
    background: rgba(255, 255, 255, 0.7);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.5);
    border-radius: 12px;
    padding: 5px 10px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
    transition: all 0.3s ease;
}

.control-bar-card:hover {
    background: rgba(255, 255, 255, 0.8);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
}

.control-center {
    flex: 1;
    text-align: center;
}

.control-right {
    flex: 0 0 auto;
}

.btn-refresh {
    padding: 5px 10px;
    border: none;
    border-radius: 8px;
    background: rgba(76, 175, 80, 0.1);
    color: #4caf50;
    font-weight: 600;
    font-size: 14px;
    cursor: pointer;
    transition: all 0.3s ease;
    border: 1px solid rgba(76, 175, 80, 0.2);
    box-shadow: 0 2px 8px rgba(76, 175, 80, 0.1);
}

.btn-refresh:hover:not(:disabled) {
    background: rgba(76, 175, 80, 0.15);
    box-shadow: 0 4px 12px rgba(76, 175, 80, 0.2);
    transform: translateY(-1px);
    border-color: rgba(76, 175, 80, 0.3);
}

.btn-refresh:disabled {
    opacity: 0.7;
    cursor: not-allowed;
}

.scanning-text {
    margin: 0;
    color: #666;
    font-size: 14px;
    font-weight: 500;
    animation: pulse-text 1.5s ease-in-out infinite;
}

.scanning-text.idle {
    color: #999;
    animation: none;
}

@keyframes pulse-text {

    0%,
    100% {
        opacity: 0.7;
    }

    50% {
        opacity: 1;
    }
}

/* 设备容器 */
.devices-container {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 12px;
}

/* 空状态 */
.empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 60px 20px;
    text-align: center;
    color: #999;
}

.empty-icon {
    font-size: 48px;
    margin-bottom: 16px;
    opacity: 0.5;
}

.empty-state p {
    margin: 0 0 8px 0;
    font-size: 16px;
    color: #666;
}

.empty-state small {
    font-size: 13px;
    color: #999;
}

/* 设备列表 */
.device-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
}

/* 设备卡片 */
.device-card {
    background: rgba(255, 255, 255, 0.7);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.5);
    border-radius: 12px;
    padding: 10px;
    transition: all 0.3s ease;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 12px;
}

.device-card:hover {
    background: rgba(255, 255, 255, 0.85);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.1);
    transform: translateY(-2px);
}

.device-card.is-connecting {
    opacity: 0.7;
}

.device-icon-wrapper {
    flex-shrink: 0;
    width: 70px;
    height: 70px;
    border-radius: 10px;
    background: rgba(0, 0, 0, 0.03);
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
}

.device-image {
    width: 100%;
    height: 100%;
    object-fit: cover;
}

.device-icon {
    font-size: 70px;
}

.device-info {
    flex: 1;
    min-width: 0;
}

.device-name {
    margin: 0 0 4px 0;
    font-size: 16px;
    font-weight: 600;
    color: #333;
    word-break: break-word;
}

.device-mac {
    margin: 0;
    font-size: 12px;
    color: #999;
    font-family: 'Monaco', 'Menlo', monospace;
}

.device-status {
    flex-shrink: 0;
}

.status-badge {
    display: inline-block;
    padding: 4px 10px;
    border-radius: 12px;
    font-size: 12px;
    font-weight: 600;
    color: #999;
    background: rgba(0, 0, 0, 0.04);
}

.status-badge.connected {
    color: #4caf50;
    background: rgba(76, 175, 80, 0.1);
}

/* 设备底部 */
.device-footer {
    display: flex;
    align-items: center;
    gap: 12px;
    justify-content: space-between;
}

.device-id {
    font-size: 12px;
    color: #ccc;
    font-family: 'Monaco', 'Menlo', monospace;
    flex: 1;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.btn-connect {
    padding: 8px 20px;
    border: none;
    border-radius: 6px;
    background: #4caf50;
    color: white;
    font-weight: 600;
    font-size: 13px;
    cursor: pointer;
    transition: all 0.3s ease;
    flex-shrink: 0;
    box-shadow: 0 2px 8px rgba(76, 175, 80, 0.3);
}

.btn-connect:hover:not(:disabled) {
    background: #45a049;
    box-shadow: 0 4px 12px rgba(76, 175, 80, 0.4);
    transform: translateY(-1px);
}

.btn-connect:disabled {
    opacity: 0.6;
    cursor: not-allowed;
}

/* 滚动条美化 */
.devices-container::-webkit-scrollbar {
    width: 6px;
}

.devices-container::-webkit-scrollbar-track {
    background: transparent;
}

.devices-container::-webkit-scrollbar-thumb {
    background: rgba(0, 0, 0, 0.1);
    border-radius: 3px;
}

.devices-container::-webkit-scrollbar-thumb:hover {
    background: rgba(0, 0, 0, 0.15);
}
</style>
