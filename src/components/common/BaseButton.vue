<script setup lang="ts">
interface Props {
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost' | 'outline';
  size?: 'sm' | 'md' | 'lg' | 'icon';
  disabled?: boolean;
  loading?: boolean;
}

withDefaults(defineProps<Props>(), {
  variant: 'secondary',
  size: 'md',
  disabled: false,
  loading: false,
});
</script>

<template>
  <button
    class="base-button"
    :class="[variant, size, { loading, disabled }]"
    :disabled="disabled || loading"
  >
    <div v-if="loading" class="spinner"></div>
    <slot v-else />
  </button>
</template>

<style scoped>
.base-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  border-radius: var(--border-radius);
  font-weight: 600;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  white-space: nowrap;
  user-select: none;
}

.base-button:active:not(:disabled) {
  transform: scale(0.96);
}

/* Variants */
.primary {
  background: var(--accent-gradient);
  color: white;
  box-shadow: 0 4px 12px rgba(139, 92, 246, 0.3);
}

.primary:hover:not(:disabled) {
  filter: brightness(1.1);
  box-shadow: 0 6px 16px rgba(139, 92, 246, 0.4);
}

.secondary {
  background: var(--bg-tertiary);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
}

.secondary:hover:not(:disabled) {
  background: var(--bg-hover);
  border-color: var(--accent-primary);
}

.outline {
  background: transparent;
  color: var(--text-primary);
  border: 1px solid var(--border-color);
}

.outline:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.05);
  border-color: var(--text-secondary);
}

.danger {
  background: rgba(239, 68, 68, 0.1);
  color: var(--accent-error);
  border: 1px solid rgba(239, 68, 68, 0.2);
}

.danger:hover:not(:disabled) {
  background: var(--accent-error);
  color: white;
}

.ghost {
  background: transparent;
  color: var(--text-secondary);
}

.ghost:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--text-primary);
}

/* Sizes */
.sm {
  padding: 6px 12px;
  font-size: 12px;
}

.md {
  padding: 10px 20px;
  font-size: 14px;
}

.lg {
  padding: 12px 24px;
  font-size: 16px;
}

.icon {
  padding: 8px;
  aspect-ratio: 1;
}

/* States */
.disabled {
  opacity: 0.5;
  cursor: not-allowed;
  filter: grayscale(1);
}

.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid currentColor;
  border-bottom-color: transparent;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
