import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";

const windows = new Map();

const createDeviceWindow = (params,name,options) => {
    let label = 'device';
    let deviceWindow = new WebviewWindow(label,{
        url: 'index.html#' + params,
        width: 400,
        height: 400,
        title: name || '设备详情',
        decorations: false,
        devtools:true,
        center: true,
        ...options
    });
    windows.set(label, deviceWindow);
    console.log('index.html#' + params)
    deviceWindow.once('tauri://error', function (e) {
        console.error('Error creating device window:', e);
    });
}

const createToolWindow = (params,name,options) => {
    const label= 'tool';
    let toolWindow = new WebviewWindow(label,{
        url: 'index.html#' + params,
        width: options.width || 80,
        height: options.height || 100,
        title: name || '悬浮窗',
        decorations: false,
        devtools:true,
        transparent: true,
        alwaysOnTop: true,
        skipTaskbar: true,
        resizable: false,
        ...options
    });
    windows.set(label, toolWindow);
    console.log('index.html#' + params)
    toolWindow.once('tauri://error', function (e) {
        console.error('Error creating tool window:', e);
    });
}

const createSettingWindow = (params, name, options) => {
    const label = 'settings';
    let settingWindow = new WebviewWindow(label, {
        url: 'index.html#/settings',
        width: 400,
        height: 600,
        center: true,
        title: name || '设置',
        decorations: false,
        devtools: true,
        ...options
    });
    windows.set(label, settingWindow);
    console.log('index.html#' + params)
    settingWindow.once('tauri://error', function (e) {
        console.error('Error creating setting window:', e);
    });
}

// 最小化窗口
const minimizeWindow = async (label) => {
    const window = windows.get(label);
    if (window || label === "main") {
        await getCurrentWindow().minimize();
    }
}

// 关闭窗口
const closeWindow = async (label) => {
    const window = windows.get(label);
    if (window || label === "main") {
        await getCurrentWindow().close();
        windows.delete(label);
    }
}

// 隐藏窗口
const hideWindow = async (label) => {
    const window = windows.get(label);
    if (window || label === "main") {
        await getCurrentWindow().hide();
    }
}

// 显示窗口
const showWindow = async (label) => {
    const window = windows.get(label);
    if (window || label === "main") {
        await getCurrentWindow().show();
    }
}

// 禁用窗口操作
const disableWindowOperations = async () => {
    try {
        const win = getCurrentWindow();
        await invoke('disable_window_operations', {
            label: win.label
        });
    } catch (error) {
        console.error('Failed to disable window operations:', error);
    }
};

// 根据窗口标签禁用窗口操作
const disableWindowByLabel = async (windowLabel) => {
    await invoke('disable_window_operations', {
        label: windowLabel
    });
};

export { createDeviceWindow,createToolWindow,createSettingWindow, minimizeWindow, closeWindow, hideWindow, showWindow,disableWindowOperations, disableWindowByLabel };