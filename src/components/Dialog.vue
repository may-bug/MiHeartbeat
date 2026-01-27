<script setup>
import { computed } from "vue";

const props = defineProps({
  title: {
    type: String,
    default: "提示"
  },
  message: {
    type: String,
    default: ""
  },
  type: {
    type: String,
    enum: ["info", "success", "warning", "error", "confirm"],
    default: "info"
  },
  isOpen: {
    type: Boolean,
    default: false
  },
  confirmText: {
    type: String,
    default: "确定"
  },
  cancelText: {
    type: String,
    default: "取消"
  }
});

const emit = defineEmits(["confirm", "cancel", "close"]);

const handleConfirm = () => {
  emit("confirm");
  emit("close");
};

const handleCancel = () => {
  emit("cancel");
  emit("close");
};

const handleBackdropClick = () => {
  emit("close");
};

const typeConfig = {
  info: { icon: "", color: "#4285f4" },
  success: { icon: "", color: "#4caf50" },
  warning: { icon: "", color: "#ff9800" },
  error: { icon: "", color: "#f44336" },
  confirm: { icon: "", color: "#ff9800" }
};

const config = computed(() => typeConfig[props.type] || typeConfig.info);
</script>

<template>
  <teleport to="body">
    <transition name="dialog-fade">
      <div v-if="isOpen" class="dialog-backdrop" @click="handleBackdropClick">
        <transition name="dialog-slide">
          <div v-if="isOpen" class="dialog-container" @click.stop>
            <div class="dialog-header">
              <div class="dialog-icon" :style="{ color: config.color }">
                {{ config.icon }}
              </div>
              <h2 class="dialog-title">{{ title }}</h2>
              <button class="dialog-close" @click="handleBackdropClick">×</button>
            </div>

            <div class="dialog-content">
              {{ message }}
            </div>

            <div class="dialog-footer">
              <button 
                v-if="type === 'confirm'"
                @click="handleCancel" 
                class="btn btn-secondary"
              >
                {{ cancelText }}
              </button>
              <button 
                @click="handleConfirm" 
                class="btn btn-primary"
                :style="{ background: config.color }"
              >
                {{ confirmText }}
              </button>
            </div>
          </div>
        </transition>
      </div>
    </transition>
  </teleport>
</template>

<style scoped>
.dialog-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(135deg, rgba(0, 0, 0, 0.4) 0%, rgba(0, 0, 0, 0.6) 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
}

.dialog-container {
  background: rgba(255, 255, 255, 0.85);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-radius: 16px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15),
              0 0 0 1px rgba(255, 255, 255, 0.8) inset;
  max-width: 380px;
  width: 85%;
  overflow: hidden;
  border: 1px solid rgba(255, 255, 255, 0.6);
  transform: translateZ(0);
}

.dialog-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  background: rgba(255, 255, 255, 0.4);
  backdrop-filter: blur(10px);
  border-bottom: 1px solid rgba(255, 255, 255, 0.5);
  position: relative;
}

.dialog-icon {
  font-size: 24px;
  flex-shrink: 0;
}

.dialog-title {
  font-size: 16px;
  font-weight: 700;
  color: #333;
  margin: 0;
  flex: 1;
}

.dialog-close {
  width: 32px;
  height: 32px;
  border: none;
  background: rgba(0, 0, 0, 0.05);
  color: #999;
  font-size: 24px;
  cursor: pointer;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
  border-radius: 8px;
  backdrop-filter: blur(10px);
}

.dialog-close:hover {
  color: #333;
  background: rgba(0, 0, 0, 0.1);
  transform: scale(1.15);
}

.dialog-content {
  padding: 15px 30px;
  font-size: 14px;
  line-height: 1.6;
  min-height: 40px;
  background-color: #fff;
  word-break: break-word;
}

.dialog-footer {
  display: flex;
  gap: 10px;
  padding: 14px 16px;
  border-top: 1px solid rgba(255, 255, 255, 0.3);
  background: rgba(255, 255, 255, 0.3);
  backdrop-filter: blur(10px);
  justify-content: flex-end;
}

.btn {
  padding: 9px 18px;
  border: none;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
  min-width: 80px;
}

.btn-primary {
  background: linear-gradient(135deg, #4285f4 0%, #1a73e8 100%);
  color: white;
  box-shadow: 0 4px 12px rgba(66, 133, 244, 0.4);
  backdrop-filter: blur(10px);
}

.btn-primary:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(66, 133, 244, 0.5);
  background: linear-gradient(135deg, #5a8cf9 0%, #3261f0 100%);
}

.btn-secondary {
  background: rgba(200, 200, 200, 0.2);
  color: #333;
  border: 1px solid rgba(200, 200, 200, 0.4);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
}

.btn-secondary:hover {
  background: rgba(200, 200, 200, 0.35);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  border-color: rgba(200, 200, 200, 0.6);
}

/* 动画 */
.dialog-fade-enter-active,
.dialog-fade-leave-active {
  transition: opacity 0.3s ease;
}

.dialog-fade-enter-from,
.dialog-fade-leave-to {
  opacity: 0;
}

.dialog-slide-enter-active,
.dialog-slide-leave-active {
  transition: transform 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.dialog-slide-enter-from {
  transform: scale(0.9) translateY(-10px);
}

.dialog-slide-leave-to {
  transform: scale(0.9) translateY(-10px);
}
</style>
