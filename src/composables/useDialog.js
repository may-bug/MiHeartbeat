import { ref } from "vue";

const dialogState = ref({
  isOpen: false,
  title: "提示",
  message: "",
  type: "info",
  confirmText: "确定",
  cancelText: "取消",
  onConfirm: null,
  onCancel: null
});

export const useDialog = () => {
  const showDialog = (options = {}) => {
    dialogState.value = {
      isOpen: true,
      title: options.title || "提示",
      message: options.message || "",
      type: options.type || "info",
      confirmText: options.confirmText || "确定",
      cancelText: options.cancelText || "取消",
      onConfirm: options.onConfirm || null,
      onCancel: options.onCancel || null
    };
  };

  const closeDialog = () => {
    dialogState.value.isOpen = false;
  };

  const confirm = (message, title = "确认") => {
    return new Promise((resolve) => {
      showDialog({
        type: "confirm",
        title,
        message,
        confirmText: "确定",
        cancelText: "取消",
        onConfirm: () => {
          closeDialog();
          resolve(true);
        },
        onCancel: () => {
          closeDialog();
          resolve(false);
        }
      });
    });
  };

  const alert = (message, title = "提示", type = "info") => {
    return new Promise((resolve) => {
      showDialog({
        type,
        title,
        message,
        confirmText: "确定",
        onConfirm: () => {
          closeDialog();
          resolve();
        }
      });
    });
  };

  const success = (message, title = "成功") => {
    return alert(message, title, "success");
  };

  const error = (message, title = "错误") => {
    return alert(message, title, "error");
  };

  const warning = (message, title = "警告") => {
    return alert(message, title, "warning");
  };

  return {
    dialogState,
    showDialog,
    closeDialog,
    confirm,
    alert,
    success,
    error,
    warning
  };
};
