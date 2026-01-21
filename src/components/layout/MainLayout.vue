<script setup lang="ts">
import Sidebar from "./Sidebar.vue";
</script>

<template>
  <div class="layout">
    <Sidebar />
    <main class="content">
      <slot />
    </main>
  </div>
</template>

<style scoped>
.layout {
  display: flex;
  width: 100%;
  height: 100%;
}

.content {
  flex: 1;
  overflow-y: auto;
  padding: 24px 32px;
  background-color: var(--bg-primary);
}
</style>
