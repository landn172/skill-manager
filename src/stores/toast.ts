import { defineStore } from "pinia";
import { ref } from "vue";

export type ToastType = "info" | "success" | "warning" | "error";

export interface Toast {
  id: string;
  message: string;
  type: ToastType;
  duration?: number;
}

export const useToastStore = defineStore("toast", () => {
  const toasts = ref<Toast[]>([]);

  function add(message: string, type: ToastType = "info", duration = 5000) {
    const id = Math.random().toString(36).substring(2, 9);
    toasts.value.push({ id, message, type, duration });

    if (duration > 0) {
      setTimeout(() => {
        remove(id);
      }, duration);
    }
  }

  function remove(id: string) {
    toasts.value = toasts.value.filter((t) => t.id !== id);
  }

  return {
    toasts,
    add,
    remove,
    success: (msg: string, dur?: number) => add(msg, "success", dur),
    error: (msg: string, dur?: number) => add(msg, "error", dur),
    info: (msg: string, dur?: number) => add(msg, "info", dur),
    warning: (msg: string, dur?: number) => add(msg, "warning", dur),
  };
});
