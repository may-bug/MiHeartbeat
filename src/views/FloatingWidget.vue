<script setup>
import { onMounted, ref, onUnmounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useRoute } from "vue-router";

const route = useRoute();
const deviceId = ref("");
const deviceName = ref("");
const heartRate = ref(0);
const isStreaming = ref(false);
const eventRef=ref(null);
const settings = ref({
  opacity: 0.85,
  always_on_top: true,
  auto_start: false,
  show_device_name: false,
  animation_speed: "normal"
});

let unlistenHeartRate = null;

// 从Rust端读取设置
const loadSettings = async () => {
  try {
    const savedSettings = await invoke("get_settings");
    settings.value = savedSettings;
  } catch (error) {
    console.error("Failed to load settings:", error);
  }
};

const toggleMonitoring = async () => {
  if (isStreaming.value) {
    await stopMonitoring();
  } else {
    await startMonitoring();
  }
};

const startMonitoring = async () => {
  try {
    isStreaming.value = true;
    await invoke("start_heart_rate_stream", { 
      deviceId: deviceId.value 
    });
    
    if (!unlistenHeartRate) {
      unlistenHeartRate = await listen("heart-rate-update", (event) => {
        const rate = event.payload;
        if (rate !== null && rate !== undefined) {
          heartRate.value = rate;
        }
      });
    }
  } catch (error) {
    console.error("Error starting heart rate stream:", error);
    isStreaming.value = false;
  }
};

const stopMonitoring = async () => {
  try {
    isStreaming.value = false;
    if (unlistenHeartRate) {
      unlistenHeartRate();
      unlistenHeartRate = null;
    }
    await invoke("stop_heart_rate_stream");
  } catch (error) {
    console.error("Error stopping heart rate stream:", error);
  }
};

// 计算动画速度
const animationDuration = computed(() => {
  const speed = settings.value.animation_speed;
  switch (speed) {
    case "fast": return "0.6s";
    case "slow": return "1.4s";
    default: return "1s";
  }
});
const installListen=()=>{
    eventRef.value=listen("tool-data-service",(event)=>{
        console.log("tool-data-service event:",event.payload);
    });
}
onMounted(async () => {
  deviceId.value = decodeURIComponent(route.params.id) || '';
  deviceName.value = decodeURIComponent(route.params.name) || 'Device';
  await loadSettings();
  installListen();
  // 自动开始监测
  await startMonitoring();
});

onUnmounted(() => {
  if (unlistenHeartRate) {
    unlistenHeartRate();
    unlistenHeartRate = null;
  }
  if (isStreaming.value) {
    stopMonitoring();
  }
});
</script>

<template>
  <div class="app-container">
    <div 
      class="floating-widget"
    >
      <div class="widget-content">
        <div class="heart-icon">❤️</div>
        <div class="heart-rate">{{ heartRate || "--" }}</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

.app-container {
  width: 100%;
  height: 100%;
  background: transparent;
  display: flex;
  align-items: center;
  justify-content: center;
  -webkit-app-region: drag;
}

.floating-widget {
  width: 64px;
  height: 72px;
  /* background: rgba(255, 255, 255, v-bind('settings.opacity * 0.9')); */
  backdrop-filter: blur(12px);
  border-radius: 10px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  /* border: 1px solid rgba(255, 255, 255, 0.3); */
  /* box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1); */
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  cursor: default;
  transition: background 0.3s ease;
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
}

.floating-widget:hover {
  /* background: rgba(255, 255, 255, v-bind('Math.min(settings.opacity * 0.9 + 0.1, 1)')); */
}

.widget-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  width: 100%;
  pointer-events: none;
}

.heart-icon {
  font-size: 1.6em;
  width: 1.6em;
  height: 1.6em;
  display: flex;
  align-items: center;
  justify-content: center;
  animation: heartbeat v-bind('animationDuration') ease-in-out infinite;
}

@keyframes heartbeat {
  0%, 100% { transform: scale(1); }
  20% { transform: scale(1.12); }
  40% { transform: scale(1); }
  60% { transform: scale(1.08); }
  80% { transform: scale(1); }
}

.heart-rate {
  font-size: 1.2em;
  font-weight: 700;
  color: #ff3b30;
  line-height: 1;
  letter-spacing: -0.5px;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}
</style>

