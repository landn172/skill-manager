<script setup lang="ts">
import { onMounted } from "vue";
import { useProjectStore } from "@/stores/project";
import { Folder, Plus, Trash2 } from "lucide-vue-next";
import BaseButton from "@/components/common/BaseButton.vue";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { homeDir } from "@tauri-apps/api/path";

const projectStore = useProjectStore();

onMounted(projectStore.fetchProjects);

const handleAddProject = async () => {
  try {
    const selected = await openDialog({
      directory: true,
      multiple: false,
      defaultPath: await homeDir(),
    });

    if (selected && typeof selected === "string") {
      const name = selected.split("/").pop() || "New Project";
      await projectStore.addProject(name, selected);
    }
  } catch (e) {
    console.error("Failed to add project", e);
  }
};
</script>

<template>
  <section class="section">
    <div class="section-title">
      <Folder :size="20" class="icon" />
      <h2>Managed Projects</h2>
    </div>
    <p class="section-hint">Project directories that have localized skills.</p>

    <div class="list-container">
      <div v-for="project in projectStore.projects" :key="project.id" class="item-card glass-card">
        <div class="item-info">
          <div class="item-text">
            <span class="item-name">{{ project.name }}</span>
            <span class="item-subtext">{{ project.path }}</span>
          </div>
        </div>
        <div class="item-actions">
          <BaseButton variant="ghost" size="icon" class="danger-ghost" @click="projectStore.removeProject(project.id!)">
            <Trash2 :size="16" />
          </BaseButton>
        </div>
      </div>

      <BaseButton variant="outline" class="w-full dashed" @click="handleAddProject">
        <Plus :size="18" />
        Add Project Directory
      </BaseButton>
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

.list-container {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.item-card {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
}

.item-info {
  display: flex;
  align-items: center;
  gap: 16px;
  min-width: 0;
}

.item-text {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.item-name {
  font-weight: 700;
  font-size: 15px;
  color: var(--text-primary);
}

.item-subtext {
  font-size: 12px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 500px;
}

.danger-ghost {
  color: var(--accent-error);
}

.danger-ghost:hover {
  background: rgba(239, 68, 68, 0.1);
}

.dashed {
  border-style: dashed;
  border-width: 2px;
}

.w-full { width: 100%; }
</style>
