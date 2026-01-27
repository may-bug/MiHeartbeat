<script setup>
import { ref, onMounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import TitleBar from "../components/TitleBar.vue";
import { useRouter } from "vue-router";
import { useDialog } from "../composables/useDialog";

const router = useRouter();
const { alert, success, error, confirm } = useDialog();

// 悬浮窗设置
const floatingWindowSettings = ref({
    opacity: 0.85,
    always_on_top: true,
    auto_start: false,
    show_device_name: true,
    animation_speed: "normal",
});

const isLoading = ref(true);
const isSaving = ref(false);

// 从后端读取设置
const loadSettings = async () => {
    try {
        isLoading.value = true;
        const settings = await invoke("get_settings");
        floatingWindowSettings.value = settings;
    } catch (error) {
        console.error("Error loading settings:", error);
        await error("读取设置失败: " + error, "错误");
    } finally {
        isLoading.value = false;
    }
};

// 保存设置到后端
const saveSettings = async () => {
    try {
        isSaving.value = true;
        const savedSettings = await invoke("set_settings", {
            settings: floatingWindowSettings.value
        });
        floatingWindowSettings.value = savedSettings;
        await success("设置已保存", "成功");
    } catch (err) {
        console.error("Error saving settings:", err);
        await error("保存设置失败: " + err, "错误");
    } finally {
        isSaving.value = false;
    }
};

// 重置为默认设置
const resetSettings = async () => {
    const confirmed = await confirm("确定要重置所有设置吗？", "确认重置");
    if (confirmed) {
        try {
            isSaving.value = true;
            const defaultSettings = await invoke("reset_to_default");
            floatingWindowSettings.value = defaultSettings;
            await success("已重置为默认设置", "成功");
        } catch (err) {
            console.error("Error resetting settings:", err);
            await error("重置设置失败: " + err, "错误");
        } finally {
            isSaving.value = false;
        }
    }
};

onMounted(() => {
    loadSettings();
});
</script>

<template>
    <div class="settings-container">
        <TitleBar title="设置" />

        <div class="content" v-if="!isLoading">
            <!-- 通用设置卡片 -->
            <div class="settings-card">
                <h3 class="card-title">
                    <span>通用设置</span>
                </h3>
            </div>

            <!-- 悬浮窗设置 -->
            <div class="settings-card">
                <h3 class="card-title">
                    <span>悬浮窗设置</span>
                </h3>

                <!-- 透明度设置 -->
                <div class="setting-item">
                    <div class="setting-label">
                        <span class="label-text">窗口透明度</span>
                        <span class="label-value">{{ (floatingWindowSettings.opacity * 100).toFixed(0) }}%</span>
                    </div>
                    <input v-model.number="floatingWindowSettings.opacity" type="range" min="0.1" max="1" step="0.05"
                        class="slider">
                </div>

                <!-- 始终在最前面 -->
                <div class="setting-item">
                    <label class="checkbox-label">
                        <input v-model="floatingWindowSettings.always_on_top" type="checkbox" class="checkbox">
                        <span class="checkbox-text">始终在最前面</span>
                    </label>
                </div>

                <!-- 显示设备名称 -->
                <div class="setting-item">
                    <label class="checkbox-label">
                        <input v-model="floatingWindowSettings.show_device_name" type="checkbox" class="checkbox">
                        <span class="checkbox-text">显示设备名称</span>
                    </label>
                </div>

                <!-- 动画速度 -->
                <div class="setting-item">
                    <div class="setting-label">
                        <span class="label-text">心跳动画速度</span>
                    </div>
                    <div class="radio-group">
                        <label class="radio-label">
                            <input v-model="floatingWindowSettings.animation_speed" type="radio" value="slow"
                                class="radio">
                            <span>缓慢</span>
                        </label>
                        <label class="radio-label">
                            <input v-model="floatingWindowSettings.animation_speed" type="radio" value="normal"
                                class="radio">
                            <span>正常</span>
                        </label>
                        <label class="radio-label">
                            <input v-model="floatingWindowSettings.animation_speed" type="radio" value="fast"
                                class="radio">
                            <span>快速</span>
                        </label>
                    </div>
                </div>
            </div>

            <!-- 系统设置卡片 -->
            <div class="settings-card">
                <h3 class="card-title">
                    <span>系统设置</span>
                </h3>

                <!-- 开机自启 -->
                <div class="setting-item">
                    <label class="checkbox-label">
                        <input v-model="floatingWindowSettings.auto_start" type="checkbox" class="checkbox">
                        <span class="checkbox-text">开机自启悬浮窗</span>
                    </label>
                </div>
            </div>

            <!-- 按钮组 -->
            <div class="button-group">
                <button @click="saveSettings" :disabled="isSaving" class="btn btn-primary">
                    <span v-if="!isSaving">保存设置</span>
                    <span v-else>保存中...</span>
                </button>
                <button @click="resetSettings" :disabled="isSaving" class="btn btn-secondary">重置默认</button>
            </div>

            <!-- 关于卡片 -->
            <div class="settings-card">
                <h3 class="card-title">关于</h3>
                <div class="setting-item">
                    <button @click="router.push('/about')" class="about-btn">
                        查看应用信息
                    </button>
                </div>
            </div>
        </div>

        <!-- 加载状态 -->
        <div v-else class="loading-state">
            <div class="loading-spinner"></div>
            <p>加载设置中...</p>
        </div>
    </div>
</template>

<style scoped>
* {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

.settings-container {
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
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
}

/* 自定义滚动条 */
.content::-webkit-scrollbar {
    width: 8px;
}

.content::-webkit-scrollbar-track {
    background: rgba(255, 255, 255, 0.3);
    border-radius: 10px;
}

.content::-webkit-scrollbar-thumb {
    background: linear-gradient(180deg, rgba(76, 175, 80, 0.6) 0%, rgba(66, 133, 244, 0.6) 100%);
    border-radius: 10px;
    transition: all 0.3s ease;
}

.content::-webkit-scrollbar-thumb:hover {
    background: linear-gradient(180deg, rgba(76, 175, 80, 0.8) 0%, rgba(66, 133, 244, 0.8) 100%);
}

/* 设置卡片 */
.settings-card {
    background: rgba(255, 255, 255, 0.7);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.5);
    border-radius: 12px;
    padding: 14px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
    transition: all 0.3s ease;
}

.settings-card:hover {
    background: rgba(255, 255, 255, 0.8);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
}

.card-title {
    font-size: 14px;
    font-weight: 700;
    margin-bottom: 12px;
    color: #333;
    display: flex;
    align-items: center;
    gap: 8px;
}

.card-icon {
    font-size: 16px;
}

.setting-item {
    margin-bottom: 12px;
    padding-bottom: 12px;
    border-bottom: 1px solid rgba(0, 0, 0, 0.05);
}

.setting-item:last-child {
    margin-bottom: 0;
    padding-bottom: 0;
    border-bottom: none;
}

.setting-label {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
}

.label-text {
    font-size: 13px;
    font-weight: 600;
    color: #333;
}

.label-value {
    font-size: 11px;
    color: #666;
    font-weight: 500;
}

/* 滑块 */
.slider {
    width: 100%;
    height: 5px;
    border-radius: 3px;
    background: linear-gradient(to right, #ddd 0%, #999 100%);
    outline: none;
    -webkit-appearance: none;
    appearance: none;
    cursor: pointer;
}

.slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: linear-gradient(135deg, #4285f4 0%, #1a73e8 100%);
    cursor: pointer;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
    transition: all 0.2s ease;
}

.slider::-webkit-slider-thumb:hover {
    transform: scale(1.2);
    box-shadow: 0 4px 8px rgba(66, 133, 244, 0.3);
}

.slider::-moz-range-thumb {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: linear-gradient(135deg, #4285f4 0%, #1a73e8 100%);
    cursor: pointer;
    border: none;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
    transition: all 0.2s ease;
}

.slider::-moz-range-thumb:hover {
    transform: scale(1.2);
    box-shadow: 0 4px 8px rgba(66, 133, 244, 0.3);
}

/* 复选框 */
.checkbox-label {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    user-select: none;
}

.checkbox {
    width: 16px;
    height: 16px;
    cursor: pointer;
    accent-color: #4285f4;
}

.checkbox-text {
    font-size: 13px;
    color: #333;
}

/* 单选框组 */
.radio-group {
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
}

.radio-label {
    display: flex;
    align-items: center;
    gap: 5px;
    cursor: pointer;
    user-select: none;
}

.radio {
    width: 14px;
    height: 14px;
    cursor: pointer;
    accent-color: #4285f4;
}

.radio-label span {
    font-size: 13px;
    color: #333;
}

/* 按钮组 */
.button-group {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
}

.btn {
    flex: 1;
    min-width: 100px;
    padding: 10px 14px;
    font-size: 13px;
    font-weight: 600;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.3s ease;
    color: white;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.btn-primary {
    background: rgba(76, 175, 80, 0.9);
}

.btn-primary:hover {
    background: #4caf50;
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(76, 175, 80, 0.3);
}

.btn-secondary {
    background: rgba(255, 152, 0, 0.9);
}

.btn-secondary:hover {
    background: #ff9800;
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(255, 152, 0, 0.3);
}

.btn-back {
    background: rgba(66, 133, 244, 0.9);
}

.btn-back:hover {
    background: #4285f4;
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(66, 133, 244, 0.3);
}

.btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
    transform: none;
}

.btn:disabled:hover {
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

/* 加载状态 */
.loading-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
}

.loading-spinner {
    width: 40px;
    height: 40px;
    border: 4px solid rgba(66, 133, 244, 0.2);
    border-top: 4px solid #4285f4;
    border-radius: 50%;
    animation: spin 1s linear infinite;
}

@keyframes spin {
    0% {
        transform: rotate(0deg);
    }

    100% {
        transform: rotate(360deg);
    }
}

.loading-state p {
    font-size: 14px;
    color: #666;
}

/* 关于按钮 */
.about-btn {
    width: 100%;
    padding: 10px 14px;
    border: none;
    background: linear-gradient(135deg, #4285f4 0%, #1a73e8 100%);
    color: white;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.3s ease;
}

.about-btn:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(66, 133, 244, 0.4);
}

@media (max-width: 600px) {
    .button-group {
        flex-direction: column;
    }

    .btn {
        width: 100%;
    }

    .radio-group {
        flex-direction: column;
    }
}
</style>
