<script setup>
import { onMounted, ref, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, emit } from "@tauri-apps/api/event";
import TitleBar from "../components/TitleBar.vue";
import { createToolWindow } from "../utils/window.js";
import { isImagePath, getDeviceIcon } from "../utils/image.js";
import { useRoute } from "vue-router";
import { getCurrentWindow, Window } from "@tauri-apps/api/window";

const route = useRoute();
const deviceId = ref("");
const deviceName = ref("");
const heartRate = ref(null);
const connectionStatus = ref("disconnected");
const deviceIcon = ref({});
const isStreaming = ref(false);
const sensorContact = ref(null);
const heartRateHistory = ref([]);
const toolwindowState = ref({
    isOpen: false,
    text: "开启悬浮窗"
});
let unlistenHeartRate = null;

const getAnimationDuration = () => {
    if (!heartRate.value || heartRate.value <= 0) {
        return '0s';
    }
    return (60 / heartRate.value) + 's';
};

const startMonitoring = async () => {
    try {
        isStreaming.value = true;
        connectionStatus.value = "connecting";
        console.log("Starting heart rate stream for device:", deviceId.value);

        // 启动后端心率流
        await invoke("start_heart_rate_stream", {
            deviceId: deviceId.value
        });
        emit("tool-data-service", { data: { deviceId: deviceId.value, deviceName: deviceName.value } });

        // 监听心率事件
        if (!unlistenHeartRate) {
            unlistenHeartRate = await listen("heart-rate-update", (event) => {
                const rate = event.payload;
                if (rate !== null && rate !== undefined) {
                    connectionStatus.value = "connected";
                    heartRate.value = rate;
                    heartRateHistory.value.push({
                        timestamp: new Date(),
                        rate: rate
                    });
                    if (heartRateHistory.value.length > 100) {
                        heartRateHistory.value.shift();
                    }
                }
            });
        }
    } catch (error) {
        console.error("Error starting heart rate stream:", error);
        alert("启动心率监测失败: " + error);
        isStreaming.value = false;
        connectionStatus.value = "disconnected";
    }
};

const stopMonitoring = async () => {
    try {
        isStreaming.value = false;
        connectionStatus.value = "disconnected";
        console.log("Stopping heart rate stream");
        if (unlistenHeartRate) {
            unlistenHeartRate();
            unlistenHeartRate = null;
        }

        await invoke("stop_heart_rate_stream");
    } catch (error) {
        console.error("Error stopping heart rate stream:", error);
    }
};

const toggleMonitoring = async () => {
    if (isStreaming.value) {
        await stopMonitoring();
    } else {
        await startMonitoring();
    }
};

const openFloatingWidget =async () => {
    let win =await Window.getByLabel("tool");
    console.log("tool window:", win);
    if (toolwindowState.value.isOpen) {
        await win.hide();
        toolwindowState.value.text = "开启悬浮窗";
    } else {
        toolwindowState.value.text = "关闭悬浮窗";
        if(win){
            await win.show();
        }else{
            const params = `device/${encodeURIComponent(deviceId.value)}/${encodeURIComponent(deviceName.value)}/widget`;
            createToolWindow(params, `${deviceName.value} - 心率`, {
                width: 80,
                height: 100,
                alwaysOnTop: true,
                skipTaskbar: true,
                transparent: true,
                decorations: false,
                shadow:false
            });
        }
    };
    toolwindowState.value.isOpen = !toolwindowState.value.isOpen;
}

onMounted(async () => {
    deviceId.value = decodeURIComponent(route.params.id) || '';
    deviceName.value = decodeURIComponent(route.params.name) || 'Unknown Device';
    deviceIcon.value = await getDeviceIcon(deviceName.value);
    connectionStatus.value = "disconnected";
    console.log("Device view opened for:", deviceId.value, deviceName.value);
});

const closeWidget = async () => {
    if (isStreaming.value) {
        await stopMonitoring();
    }
    let win = await Window.getByLabel("main");
    if (win) {
        await win.show();
    }
    let toolWin = await Window.getByLabel("tool");
    if (toolWin) {
        await toolWin.close();
    }
    await getCurrentWindow().close();
};

onUnmounted(() => {
    // 清理事件监听
    if (unlistenHeartRate) {
        unlistenHeartRate();
        unlistenHeartRate = null;
    }
    // 停止监测
    if (isStreaming.value) {
        stopMonitoring();
    }
});
</script>

<template>
    <div class="device-container">
        <TitleBar :title="deviceName" :onClose="closeWidget" />

        <div class="content">
            <!-- 心率显示卡片 -->
            <div class="heart-rate-display">
                <div class="icon-and-status">
                    <div class="device-icon-wrapper">
                        <img v-if="isImagePath(deviceIcon)" :src="deviceIcon" :alt="deviceName" class="device-image" />
                        <div v-else class="device-icon">
                            {{ typeof deviceIcon === 'string' ? deviceIcon : '📱' }}
                        </div>
                    </div>
                    <div class="connection-status" :class="connectionStatus">
                        <span class="status-dot"></span>
                        <span class="status-text">
                            <template v-if="connectionStatus === 'connected'">已连接</template>
                            <template v-else-if="connectionStatus === 'connecting'">正在连接</template>
                            <template v-else>未连接</template>
                        </span>
                    </div>
                </div>
                <div class="heart-icon" :style="{ animationDuration: getAnimationDuration() }">❤️</div>
                <div class="heart-rate-value">
                    <span class="rate">{{ heartRate || "--" }}</span>
                    <span class="unit">BPM</span>
                </div>
                <div class="sensor-status" v-if="sensorContact !== null">
                    <span v-if="sensorContact" class="contact-good">✓ 传感器贴合</span>
                    <span v-else class="contact-bad">✗ 传感器未贴合</span>
                </div>
                <!-- 控制按钮 -->
                <div class="button-group">
                    <button @click="toggleMonitoring" class="btn" :class="isStreaming ? 'btn-stop' : 'btn-start'">
                        {{ isStreaming ? '停止监测' : '开始监测' }}
                    </button>
                    <button @click="openFloatingWidget" class="btn btn-widget">{{toolwindowState.text}}</button>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
* {
    box-sizing: border-box;
}

.device-container {
    width: 100%;
    height: 100vh;
    background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
    display: flex;
    flex-direction: column;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    color: #333;
}

.content {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
}

/* 心率显示卡片 */
.heart-rate-display {
    background: rgba(255, 255, 255, 0.7);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.5);
    padding: 5px 10px;
    text-align: center;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
    transition: all 0.3s ease;
    height: 100%;
    overflow: hidden;
}

.heart-rate-display:hover {
    background: rgba(255, 255, 255, 0.8);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
}

.device-icon-wrapper {
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 12px;
    border-radius: 10px;
    overflow: hidden;
}

.device-image {
    width: 140px;
    height: 140px;
    object-fit: cover;
}

.device-icon {
    font-size: 140px;
}

.icon-and-status {
    position: relative;
    display: flex;
    align-items: flex-end;
    justify-content: center;
    gap: 12px;
    margin-bottom: 12px;
}

.connection-status {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
}

.status-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    display: inline-block;
}

.connection-status.connected .status-dot {
    background: #4caf50;
    animation: pulse 1.5s ease-in-out infinite;
    box-shadow: 0 0 8px rgba(76, 175, 80, 0.5);
}

.connection-status.connecting .status-dot {
    background: #2196f3;
    animation: connecting-pulse 1s ease-in-out infinite;
    box-shadow: 0 0 8px rgba(33, 150, 243, 0.5);
}

.connection-status.disconnected .status-dot {
    background: #ff9800;
}

.connection-status .status-text {
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.3px;
}

.connection-status.connected {
    color: #4caf50;
}

.connection-status.connecting {
    color: #2196f3;
}

.connection-status.disconnected {
    color: #ff9800;
}

@keyframes connecting-pulse {

    0%,
    100% {
        opacity: 1;
    }

    50% {
        opacity: 0.4;
    }
}

@keyframes pulse {

    0%,
    100% {
        opacity: 1;
        transform: scale(1);
    }

    50% {
        opacity: 0.6;
        transform: scale(1.2);
    }
}

.heart-icon {
    font-size: 2.5em;
    margin-bottom: 12px;
    animation: heartbeat 1s ease-in-out infinite;
}

@keyframes heartbeat {

    0%,
    100% {
        transform: scale(1);
    }

    25% {
        transform: scale(1.2);
    }

    50% {
        transform: scale(1);
    }
}

.heart-rate-value {
    display: flex;
    align-items: baseline;
    justify-content: center;
    gap: 8px;
    margin-bottom: 12px;
}

.rate {
    font-size: 2.5em;
    font-weight: 700;
    color: #ff3b30;
}

.unit {
    font-size: 1em;
    color: #999;
    font-weight: 500;
}

.sensor-status {
    font-size: 13px;
    margin-top: 8px;
}

.contact-good {
    color: #2ed573;
    font-weight: 600;
}

.contact-bad {
    color: #ff4757;
    font-weight: 600;
}

.button-group {
    display: flex;
    gap: 12px;
}

.btn {
    flex: 1;
    padding: 10px 12px;
    font-size: 14px;
    font-weight: 600;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.3s ease;
    color: white;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.btn-start {
    background: rgba(76, 175, 80, 0.9);
}

.btn-start:hover {
    background: #4caf50;
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(76, 175, 80, 0.3);
}

.btn-stop {
    background: rgba(255, 59, 48, 0.9);
}

.btn-stop:hover {
    background: #ff3b30;
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(255, 59, 48, 0.3);
}

.btn-widget {
    background: rgba(66, 133, 244, 0.9);
}

.btn-widget:hover {
    background: #4285f4;
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(66, 133, 244, 0.3);
}
</style>
