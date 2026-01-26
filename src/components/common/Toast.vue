<script setup lang="ts">
import { useToastStore } from "@/stores/toast";
import { X, CheckCircle2, AlertCircle, Info, AlertTriangle } from "lucide-vue-next";

const toastStore = useToastStore();
</script>

<template>
  <div class="toast-container">
    <TransitionGroup name="toast">
      <div
        v-for="toast in toastStore.toasts"
        :key="toast.id"
        class="toast-item glass-card"
        :class="toast.type"
      >
        <div class="icon-wrap">
          <CheckCircle2 v-if="toast.type === 'success'" :size="18" />
          <AlertCircle v-else-if="toast.type === 'error'" :size="18" />
          <AlertTriangle v-else-if="toast.type === 'warning'" :size="18" />
          <Info v-else :size="18" />
        </div>
        <div class="message">{{ toast.message }}</div>
        <button class="close-btn" @click="toastStore.remove(toast.id)">
          <X :size="14" />
        </button>
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.toast-container {
  position: fixed;
  top: 24px;
  right: 24px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  z-index: 9999;
  pointer-events: none;
}

.toast-item {
  pointer-events: auto;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  min-width: 280px;
  max-width: 420px;
  border-radius: 12px;
  box-shadow: var(--shadow-xl);
  border-left: 4px solid var(--accent-primary);
  animation: slideIn 0.3s ease;
}

.icon-wrap {
  flex-shrink: 0;
}

.message {
  flex: 1;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.close-btn {
  padding: 4px;
  color: var(--text-muted);
  transition: all 0.2s;
}

.close-btn:hover {
  color: var(--text-primary);
  background: var(--bg-hover);
}

/* Types */
.success { border-left-color: var(--accent-success); .icon-wrap { color: var(--accent-success); } }
.error { border-left-color: var(--accent-error); .icon-wrap { color: var(--accent-error); } }
.warning { border-left-color: var(--accent-warning); .icon-wrap { color: var(--accent-warning); } }
.info { border-left-color: var(--accent-primary); .icon-wrap { color: var(--accent-primary); } }

/* Transitions */
.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.toast-enter-from {
  opacity: 0;
  transform: translateX(30px) scale(0.9);
}

.toast-leave-to {
  opacity: 0;
  transform: scale(0.9);
}

@keyframes slideIn {
  from { opacity: 0; transform: translateX(30px); }
  to { opacity: 1; transform: translateX(0); }
}
</style>
