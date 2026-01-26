<script setup lang="ts">
import { onMounted, computed, ref } from "vue";
import type { InstalledSkill } from "@/types";
import SkillList from "@/components/skill/SkillList.vue";
import SkillEditor from "@/components/skill/SkillEditor.vue";
import PageHeader from "@/components/common/PageHeader.vue";
import InstallModal from "@/components/skill/InstallModal.vue";
import BaseButton from "@/components/common/BaseButton.vue";
import { RefreshCw, Package, Plus } from "lucide-vue-next";
import { useSkillsStore } from "@/stores/skills";

const skillsStore = useSkillsStore();

// Install Modal State
const showInstallModal = ref(false);
const selectedSkill = ref<InstalledSkill | null>(null);

const scope = computed({
  get: () => skillsStore.scope,
  set: (val) => skillsStore.fetchInstalledSkills(val),
});

const skills = computed(() => skillsStore.installedSkills);
const loading = computed(() => skillsStore.loading);

const editingSkill = ref<InstalledSkill | null>(null);

onMounted(() => {
  skillsStore.fetchInstalledSkills();
});

function handleRefresh() {
  skillsStore.fetchInstalledSkills();
}

function handleEdit(skill: InstalledSkill) {
  editingSkill.value = skill;
}

function closeEditor() {
  editingSkill.value = null;
  handleRefresh();
}

// Open modal for supplemental install
function openInstallModal(skill: InstalledSkill) {
  selectedSkill.value = skill;
  showInstallModal.value = true;
}

async function handleUninstall(skill: InstalledSkill) {
  if (!confirm(`Are you sure you want to uninstall ${skill.name}?`)) return;
  try {
    for (const agent of skill.agents) {
      await skillsStore.uninstallSkill(skill.name, agent);
    }
  } catch (e) {
    alert(`Failed to uninstall: ${e}`);
  }
}

function handleScopeChange(newScope: "project" | "global") {
  skillsStore.fetchInstalledSkills(newScope);
}
</script>

<template>
  <div class="installed-page animate-fade-in">
    <PageHeader title="Installed Skills" description="Manage the skills currently installed on your agents.">
      <template #actions>
        <BaseButton variant="ghost" size="icon" @click="handleRefresh" :disabled="loading">
          <RefreshCw :size="20" :class="{ spinning: loading }" />
        </BaseButton>
        <router-link to="/create">
          <BaseButton variant="primary">
            <Plus :size="18" />
            Create Skill
          </BaseButton>
        </router-link>
      </template>
    </PageHeader>

    <div class="controls-bar glass">
      <div class="scope-picker">
        <button
          class="picker-opt"
          :class="{ active: scope === 'project' }"
          @click="handleScopeChange('project')"
        >
          Current Project
        </button>
        <button
          class="picker-opt"
          :class="{ active: scope === 'global' }"
          @click="handleScopeChange('global')"
        >
          Global Scope
        </button>
      </div>
      <div class="stats">
        {{ skills.length }} skill{{ skills.length !== 1 ? 's' : '' }} installed
      </div>
    </div>

    <div class="content-area">
      <div v-if="loading && skills.length === 0" class="loading-state">
        <div class="loader"></div>
        <p>Scanning for installed skills...</p>
      </div>

      <div v-else-if="skills.length === 0" class="empty-state glass-card">
        <Package :size="64" class="empty-icon" />
        <h3>No skills found</h3>
        <p>No skills are installed in the <strong>{{ scope }}</strong> scope.</p>
        <BaseButton variant="primary" @click="handleRefresh">Refresh</BaseButton>
      </div>

      <div v-else>
        <SkillList
          :skills="skills"
          @edit="handleEdit"
          @uninstall="handleUninstall"
          @install="openInstallModal"
        />
      </div>
    </div>

    <!-- Skill Editor Overlay -->
    <SkillEditor
      v-if="editingSkill"
      :skill-path="editingSkill.path"
      :skill-name="editingSkill.name"
      @close="closeEditor"
      @save="handleRefresh"
    />

    <!-- Supplemental Install Modal -->
    <InstallModal
      :show="showInstallModal"
      :skill="selectedSkill"
      @close="showInstallModal = false"
      @success="handleRefresh"
    />
  </div>
</template>

<style scoped>
.installed-page {
  padding: 20px;
  height: 100vh;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.controls-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 24px;
  border-radius: 16px;
  margin-bottom: 32px;
}

.scope-picker {
  display: flex;
  background: var(--bg-tertiary);
  padding: 4px;
  border-radius: 12px;
  border: 1px solid var(--border-color);
}

.picker-opt {
  padding: 8px 16px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  transition: all 0.2s;
}

.picker-opt.active {
  background: var(--bg-primary);
  color: var(--accent-primary);
  box-shadow: var(--shadow-sm);
}

.stats {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-muted);
}

.content-area {
  flex: 1;
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 100px 0;
  color: var(--text-secondary);
}

.loader {
  width: 48px;
  height: 48px;
  border: 3px solid var(--border-color);
  border-top-color: var(--accent-primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 20px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px 40px;
  text-align: center;
  max-width: 460px;
  margin: 40px auto;
  gap: 16px;
}

.empty-icon {
  color: var(--border-color);
  margin-bottom: 8px;
}

.empty-state h3 {
  margin: 0;
  font-size: 20px;
}

.empty-state p {
  color: var(--text-secondary);
  margin-bottom: 12px;
}
</style>
