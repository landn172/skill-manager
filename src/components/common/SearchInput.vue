<script setup lang="ts">
import { Search } from "lucide-vue-next";

defineProps<{
  modelValue: string;
  placeholder?: string;
}>();

defineEmits<{
  (e: "update:modelValue", value: string): void;
}>();
</script>

<template>
  <div class="search-box glass">
    <Search :size="18" class="search-icon" />
    <input
      type="text"
      :value="modelValue"
      @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
      :placeholder="placeholder || 'Search...'"
    />
  </div>
</template>

<style scoped>
.search-box {
  position: relative;
  width: 100%;
  max-width: 480px;
  display: flex;
  align-items: center;
  border-radius: 12px;
  overflow: hidden;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.search-box:focus-within {
  border-color: var(--accent-primary);
  box-shadow: 0 0 0 3px rgba(139, 92, 246, 0.15);
  transform: translateY(-1px);
}

.search-icon {
  position: absolute;
  left: 14px;
  color: var(--text-muted);
  pointer-events: none;
}

input {
  width: 100%;
  background: transparent;
  border: none;
  padding: 12px 14px 12px 42px;
  outline: none;
  color: var(--text-primary);
  font-size: 14px;
}

input::placeholder {
  color: var(--text-muted);
}
</style>
