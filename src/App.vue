<script setup>
import Dialog from "./components/Dialog.vue";
import { useDialog } from "./composables/useDialog";

const { dialogState } = useDialog();
</script>

<template>
  <RouterView />
  <Dialog 
    :title="dialogState.title"
    :message="dialogState.message"
    :type="dialogState.type"
    :isOpen="dialogState.isOpen"
    :confirmText="dialogState.confirmText"
    :cancelText="dialogState.cancelText"
    @confirm="() => {
      if (dialogState.onConfirm) dialogState.onConfirm();
    }"
    @cancel="() => {
      if (dialogState.onCancel) dialogState.onCancel();
    }"
    @close="() => {
      dialogState.isOpen = false;
    }"
  />
</template>

<style>
html, body {
  margin: 0;
  padding: 0;
  height: 100%;
}

#app {
  height: 100%;
  width: 100%;
}
</style>
