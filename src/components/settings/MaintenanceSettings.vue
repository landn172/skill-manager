<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useMarketplaceStore } from "@/stores/marketplace";
import { Download } from "lucide-vue-next";
import BaseButton from "@/components/common/BaseButton.vue";
import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
import { writeTextFile, readTextFile } from "@tauri-apps/plugin-fs";

const marketplaceStore = useMarketplaceStore();
const clearingCache = ref(false);

async function handleClearCache() {
  if (
    !confirm(
      "Are you sure you want to clear the skills cache? This will force a fresh fetch from all sources.",
    )
  )
    return;

  clearingCache.value = true;
  try {
    await invoke<string>("clear_cache");
    await marketplaceStore.fetchSources();
  } catch (e) {
    alert(`Failed to clear cache: ${e}`);
  } finally {
    clearingCache.value = false;
  }
}

async function handleExportConfig() {
  try {
    const json = await invoke<string>("export_config");
    const filePath = await saveDialog({
      filters: [{ name: "JSON", extensions: ["json"] }],
      defaultPath: "skill-manager-config.json",
    });

    if (filePath) {
      await writeTextFile(filePath, json);
    }
  } catch (e) {
    alert(`Failed to export config: ${e}`);
  }
}

async function handleImportConfig() {
  try {
    const filePath = await openDialog({
      filters: [{ name: "JSON", extensions: ["json"] }],
      multiple: false,
    });

    if (filePath && typeof filePath === "string") {
      const json = await readTextFile(filePath);
      await invoke("import_config", { json });
      marketplaceStore.fetchSources();
    }
  } catch (e) {
    alert(`Failed to import config: ${e}`);
  }
}
</script>

<template>
  <section class="section">
    <div class="section-title">
      <Download :size="20" class="icon" />
      <h2>Maintenance & Storage</h2>
    </div>
    <p class="section-hint">System maintenance and configuration management.</p>

    <div class="maintenance-grid">
      <div class="glass-card maintenance-card">
        <div class="text">
          <span class="title">Clear Cache</span>
          <span class="hint">Force a fresh fetch of all marketplace skills.</span>
        </div>
        <BaseButton variant="danger" size="sm" @click="handleClearCache" :loading="clearingCache">
          Clear Cache
        </BaseButton>
      </div>

      <div class="glass-card maintenance-card">
        <div class="text">
          <span class="title">Configuration</span>
          <span class="hint">Backup or restore your settings and sources.</span>
        </div>
        <div class="btns">
          <BaseButton variant="outline" size="sm" @click="handleExportConfig">
            Export
          </BaseButton>
          <BaseButton variant="outline" size="sm" @click="handleImportConfig">
            Import
          </BaseButton>
        </div>
      </div>
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

.maintenance-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.maintenance-card {
  padding: 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.maintenance-card .text {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.maintenance-card .title {
  font-weight: 700;
  font-size: 15px;
}

.maintenance-card .hint {
  font-size: 12px;
  color: var(--text-muted);
}

.maintenance-card .btns {
  display: flex;
  gap: 8px;
}
</style>
