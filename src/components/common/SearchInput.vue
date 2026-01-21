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
  <div class="search-box">
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
  max-width: 400px;
}

.search-icon {
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-muted);
}

input {
  width: 100%;
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  padding: 10px 12px 10px 40px;
  outline: none;
  transition: border-color 0.2s;
}

input:focus {
  border-color: var(--accent-primary);
}
</style>
