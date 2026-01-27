<script setup lang="ts">
interface Props {
  title: string;
  description?: string;
}

defineProps<Props>();
</script>

<template>
  <header class="page-header">
    <div class="content">
      <h1 class="title">{{ title }}</h1>
      <p v-if="description" class="description">{{ description }}</p>
    </div>
    <div class="actions">
      <slot name="actions" />
    </div>
  </header>
</template>

<style scoped>
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  margin-bottom: 32px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--glass-border);
}

.title {
  font-size: 32px;
  font-weight: 800;
  margin: 0;
  letter-spacing: -0.04em;
  background: var(--accent-gradient);
  background-clip: text;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.description {
  margin: 8px 0 0;
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
}

.actions {
  display: flex;
  gap: 12px;
  align-items: center;
}
</style>
