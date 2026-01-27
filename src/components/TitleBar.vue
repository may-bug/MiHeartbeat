<script setup>
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useRouter } from "vue-router";
import { createSettingWindow } from "../utils/window";
import SETTINGS from "../assets/icons/settings.svg";

const props = defineProps({
  title: {
    type: String,
    default: "Mi心动"
  },
  showSettings: {
    type: Boolean,
    default: false
  },
  onClose: {
    type: Function,
    default: null
  }
});

const minimize = async () => {
  const appWindow = getCurrentWindow();
  await appWindow.minimize();
};

const close = async () => {
  if (props.onClose) {
    await props.onClose();
  } else {
    const appWindow = getCurrentWindow();
    await appWindow.close();
  }
};

const openSettings = () => {
  createSettingWindow()
};
</script>

<template>
  <div class="titlebar" data-tauri-drag-region>
    <div class="titlebar-left">
      <h1 class="app-title">{{ title }}</h1>
    </div>
    <div class="titlebar-right">
      <button 
        v-if="showSettings"
        @click="openSettings"
        class="titlebar-btn settings-btn"
        title="设置"
      >
        <img :src="SETTINGS" alt="Settings" width="16" height="16" />
      </button>
      <button 
        @click="minimize"
        class="titlebar-btn minimize-btn"
        title="最小化"
      >
        ─
      </button>
      <button 
        @click="close"
        class="titlebar-btn close-btn"
        title="关闭"
      >
        ✕
      </button>
    </div>
  </div>
</template>

<style scoped>
.titlebar {
  width: 100%;
  height: 50px;
  background: rgba(255, 255, 255, 0.7);
  backdrop-filter: blur(10px);
  border-bottom: 1px solid rgba(255, 255, 255, 0.5);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 10px;
  user-select: none;
  flex-shrink: 0;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
}

.titlebar-left {
  flex: 1;
}

.app-title {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: #333;
  letter-spacing: 0.5px;
}

.titlebar-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.titlebar-btn {
  width: 36px;
  height: 36px;
  border: none;
  background: transparent;
  color: #666;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  border-radius: 6px;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  -webkit-user-select: none;
  user-select: none;
}

.titlebar-btn:hover {
  background: rgba(0, 0, 0, 0.05);
}

.settings-btn {
  font-size: 16px;
}

.settings-btn:hover {
  background: rgba(66, 133, 244, 0.1);
  color: #4285f4;
}.titlebar-btn:hover {
  background: rgba(0, 0, 0, 0.08);
  color: #333;
}

.titlebar-btn.close-btn:hover {
  background: rgba(255, 59, 48, 0.1);
  color: #ff3b30;
}

.titlebar-btn:active {
  background: rgba(0, 0, 0, 0.12);
}

.titlebar-btn.close-btn:active {
  background: rgba(255, 59, 48, 0.2);
}
</style>
