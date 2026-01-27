<script setup lang="ts">
import { onMounted, computed, ref } from "vue";
import type { InstalledSkill } from "@/types";
import SkillList from "@/components/skill/SkillList.vue";
import SkillEditor from "@/components/skill/SkillEditor.vue";
import SkillCardSkeleton from "@/components/skill/SkillCardSkeleton.vue";
import PageHeader from "@/components/common/PageHeader.vue";
import InstallModal from "@/components/skill/InstallModal.vue";
import Modal from "@/components/common/Modal.vue";
import BaseButton from "@/components/common/BaseButton.vue";
import { RefreshCw, Package, Plus, Trash2 } from "lucide-vue-next";
import { useSkillsStore } from "@/stores/skills";
import { useAgentsStore } from "@/stores/agents";

const skillsStore = useSkillsStore();
const agentsStore = useAgentsStore();

// Install Modal State
const showInstallModal = ref(false);
const selectedSkill = ref<InstalledSkill | null>(null);

const scope = computed({
  get: () => skillsStore.scope,
  set: (val) => skillsStore.fetchInstalledSkills(val),
});

const skills = computed(() => {
  return [...skillsStore.installedSkills].sort((a, b) =>
    a.name.localeCompare(b.name)
  );
});
const loading = computed(() => skillsStore.loading);

const editingSkill = ref<InstalledSkill | null>(null);

// Selection State
const isSelectionMode = ref(false);
const selectedSkills = ref<string[]>([]);

onMounted(() => {
  skillsStore.fetchInstalledSkills();
  agentsStore.fetchAgents();
});

function handleRefresh() {
  skillsStore.fetchInstalledSkills();
}

function toggleSelectionMode() {
  isSelectionMode.value = !isSelectionMode.value;
  if (!isSelectionMode.value) {
    selectedSkills.value = [];
  }
}

function handleToggleSelection(skillName: string) {
  const index = selectedSkills.value.indexOf(skillName);
  if (index > -1) {
    selectedSkills.value.splice(index, 1);
  } else {
    selectedSkills.value.push(skillName);
  }
}

async function handleBulkUninstall() {
  if (selectedSkills.value.length === 0) return;
  if (!confirm(`Are you sure you want to uninstall ${selectedSkills.value.length} selected skill(s)?`)) return;

  try {
    for (const skillName of selectedSkills.value) {
      const skill = skillsStore.getSkillByName(skillName);
      if (skill) {
        for (const agent of skill.agents) {
          await skillsStore.uninstallSkill(skillName, agent);
        }
      }
    }
    isSelectionMode.value = false;
    selectedSkills.value = [];
    handleRefresh();
  } catch (e) {
    alert(`Failed to uninstall some skills: ${e}`);
  }
}

function handleEdit(skill: InstalledSkill) {
  editingSkill.value = skill;
}

function closeEditor() {
  editingSkill.value = null;
  handleRefresh();
}

function openInstallModal(skill: InstalledSkill) {
  selectedSkill.value = skill;
  showInstallModal.value = true;
}

const showUninstallModal = ref(false);
const skillToUninstall = ref<InstalledSkill | null>(null);
const uninstalling = ref(false);

function confirmUninstall(skill: InstalledSkill) {
  skillToUninstall.value = skill;
  showUninstallModal.value = true;
}

async function handleUninstallConfirm() {
  if (!skillToUninstall.value) return;
  
  uninstalling.value = true;
  try {
    const skill = skillToUninstall.value;
    for (const agent of skill.agents) {
      await skillsStore.uninstallSkill(skill.name, agent);
    }
    showUninstallModal.value = false;
    skillToUninstall.value = null;
    handleRefresh();
  } catch (e) {
    alert(`Failed to uninstall: ${e}`);
  } finally {
    uninstalling.value = false;
  }
}

function handleScopeChange(newScope: "project" | "global") {
  skillsStore.fetchInstalledSkills(newScope);
}
</script>

<template>
  <div class="installed-page animate-fade-in" :class="{ 'has-bulk-bar': isSelectionMode && selectedSkills.length > 0 }">
    <PageHeader title="Installed Skills" description="Manage the skills currently installed on your agents.">
      <template #actions>
        <BaseButton
          :variant="isSelectionMode ? 'primary' : 'outline'"
          @click="toggleSelectionMode"
          :disabled="skills.length === 0"
        >
          {{ isSelectionMode ? 'Cancel Selection' : 'Manage' }}
        </BaseButton>

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
      <div v-if="loading && skills.length === 0" class="skills-grid">
        <SkillCardSkeleton v-for="i in 6" :key="i" />
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
          :is-selection-mode="isSelectionMode"
          :selected-skills="selectedSkills"
          @edit="handleEdit"
          @uninstall="confirmUninstall"
          @install="openInstallModal"
          @toggle-selection="handleToggleSelection"
        />
      </div>
    </div>

    <!-- Bulk Action Bar -->
    <transition name="slide-up">
      <div v-if="isSelectionMode && selectedSkills.length > 0" class="bulk-action-bar glass">
        <div class="bulk-info">
          <span class="count">{{ selectedSkills.length }}</span>
          <span>skills selected</span>
        </div>
        <div class="bulk-ops">
          <BaseButton variant="ghost" @click="selectedSkills = []">Clear</BaseButton>
          <BaseButton variant="danger" @click="handleBulkUninstall">
            <Trash2 :size="18" />
            <span>Uninstall Selected</span>
          </BaseButton>
        </div>
      </div>
    </transition>

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

    <!-- Uninstall Confirmation Modal -->
    <Modal
      :show="showUninstallModal"
      title="Uninstall Skill"
      @close="showUninstallModal = false"
    >
      <div class="confirm-content">
        <p>Are you sure you want to uninstall <strong>{{ skillToUninstall?.name }}</strong>?</p>
        <p class="sub-text">This will remove the skill from all installed agents.</p>
      </div>
      <template #footer>
        <BaseButton variant="ghost" @click="showUninstallModal = false">Cancel</BaseButton>
        <BaseButton variant="primary" class="danger-btn" :loading="uninstalling" @click="handleUninstallConfirm">Uninstall</BaseButton>
      </template>
    </Modal>
  </div>
</template>

<style scoped>
.confirm-content {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.confirm-content strong {
  color: var(--text-primary);
}

.sub-text {
  font-size: 13px;
  color: var(--text-muted);
  line-height: 1.5;
}

.danger-btn {
  background-color: var(--accent-error) !important;
  color: white !important;
  border: none !important;
}

.danger-btn:hover {
  background-color: #dc2626 !important;
}
.skills-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 24px;
  padding: 20px;
}
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
  padding-bottom: 20px;
}

.installed-page.has-bulk-bar .content-area {
  padding-bottom: 120px;
}

.bulk-action-bar {
  position: fixed;
  bottom: 32px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 32px;
  min-width: 500px;
  border-radius: 20px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.4);
  z-index: 100;
  border: 1px solid rgba(255, 255, 255, 0.15) !important;
}

.bulk-info {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  color: var(--text-primary);
}

.bulk-info .count {
  background: var(--accent-primary);
  color: white;
  padding: 2px 8px;
  border-radius: 6px;
  font-size: 14px;
}

.bulk-ops {
  display: flex;
  gap: 12px;
}

/* Slide Up Transition */
.slide-up-enter-active,
.slide-up-leave-active {
  transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1);
}

.slide-up-enter-from,
.slide-up-leave-to {
  transform: translate(-50%, 100%) scale(0.9);
  opacity: 0;
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
