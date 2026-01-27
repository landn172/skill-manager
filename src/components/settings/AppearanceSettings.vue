<script setup lang="ts">
import { useThemeStore } from "@/stores/theme";
import { Sun, Moon, Monitor, Palette } from "lucide-vue-next";

const themeStore = useThemeStore();
</script>

<template>
  <section class="section">
    <div class="section-title">
      <Palette :size="20" class="icon" />
      <h2>App Appearance</h2>
    </div>
    <p class="section-hint">Choose between light, dark, or system-preferred theme.</p>

    <div class="theme-picker glass">
      <button
        class="theme-opt"
        :class="{ active: themeStore.theme === 'light' }"
        @click="themeStore.setTheme('light')"
      >
        <Sun :size="16" />
        <span>Light</span>
      </button>
      <button
        class="theme-opt"
        :class="{ active: themeStore.theme === 'dark' }"
        @click="themeStore.setTheme('dark')"
      >
        <Moon :size="16" />
        <span>Dark</span>
      </button>
      <button
        class="theme-opt"
        :class="{ active: themeStore.theme === 'system' }"
        @click="themeStore.setTheme('system')"
      >
        <Monitor :size="16" />
        <span>System</span>
      </button>
    </div>
  </section>
</template>

<style scoped>
.section {
  display: flex;
  flex-direction: column;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}

.section-title h2 {
  font-size: 20px;
  font-weight: 700;
  margin: 0;
}

.section-title .icon {
  color: var(--accent-primary);
}

.section-hint {
  font-size: 14px;
  color: var(--text-secondary);
  margin: 0 0 24px;
}

.theme-picker {
  display: flex;
  padding: 4px;
  border-radius: 12px;
  width: fit-content;
}

.theme-opt {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 20px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  transition: all 0.2s;
}

.theme-opt:hover {
  color: var(--text-primary);
  background: rgba(255, 255, 255, 0.05);
}

.theme-opt.active {
  background: var(--bg-primary);
  color: var(--accent-primary);
  box-shadow: var(--shadow-sm);
}
</style>
