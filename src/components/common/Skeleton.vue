<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  width?: string;
  height?: string;
  borderRadius?: string;
  variant?: 'text' | 'circular' | 'rectangular';
}>();

const style = computed(() => ({
  width: props.width || (props.variant === 'circular' ? '40px' : '100%'),
  height: props.height || (props.variant === 'text' ? '1em' : '20px'),
  borderRadius: props.variant === 'circular' ? '50%' : (props.borderRadius || '4px'),
}));
</script>

<template>
  <div class="skeleton" :class="variant" :style="style"></div>
</template>

<style scoped>
.skeleton {
  background: var(--bg-tertiary);
  position: relative;
  overflow: hidden;
  display: inline-block;
}

.skeleton::after {
  content: "";
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  left: 0;
  transform: translateX(-100%);
  background-image: linear-gradient(
    90deg,
    rgba(255, 255, 255, 0) 0,
    rgba(255, 255, 255, 0.05) 20%,
    rgba(255, 255, 255, 0.1) 60%,
    rgba(255, 255, 255, 0)
  );
  animation: shimmer 2s infinite;
}

@keyframes shimmer {
  100% {
    transform: translateX(100%);
  }
}
</style>
