<script setup lang="ts">
import PageHeader from "@/components/common/PageHeader.vue";
import AppearanceSettings from "@/components/settings/AppearanceSettings.vue";
import ApiSettings from "@/components/settings/ApiSettings.vue";
import AgentSettings from "@/components/settings/AgentSettings.vue";
import MarketplaceSourceSettings from "@/components/settings/MarketplaceSourceSettings.vue";
import ProjectSettings from "@/components/settings/ProjectSettings.vue";
import MaintenanceSettings from "@/components/settings/MaintenanceSettings.vue";
</script>

<template>
  <div class="settings-page animate-fade-in">
    <PageHeader title="Settings" description="Customize application behavior and manage configurations." />

    <div class="sections">
      <AppearanceSettings />
      <ApiSettings />
      <AgentSettings />
      <MarketplaceSourceSettings />
      <ProjectSettings />
      <MaintenanceSettings />
    </div>
  </div>
</template>

<style scoped>
.settings-page {
  padding: 20px;
  height: 100vh;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.sections {
  display: flex;
  flex-direction: column;
  gap: 40px;
  max-width: 800px;
  padding-bottom: 60px;
}
</style>
