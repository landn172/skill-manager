<script setup lang="ts">
import { onMounted, computed, ref } from 'vue'
import type { InstalledSkill } from '@/types'
import SkillList from '@/components/skill/SkillList.vue'
import SkillEditor from '@/components/skill/SkillEditor.vue'
import { RefreshCw, Package, Plus } from 'lucide-vue-next'
import { useSkillsStore } from '@/stores/skills'

const skillsStore = useSkillsStore()

const scope = computed({
  get: () => skillsStore.scope,
  set: (val) => skillsStore.fetchInstalledSkills(val),
})

const skills = computed(() => skillsStore.installedSkills)
const loading = computed(() => skillsStore.loading)

const editingSkill = ref<InstalledSkill | null>(null)

onMounted(() => {
  skillsStore.fetchInstalledSkills()
})

function handleRefresh() {
  skillsStore.fetchInstalledSkills()
}

function handleEdit(skill: InstalledSkill) {
  editingSkill.value = skill
}

function closeEditor() {
  editingSkill.value = null
  handleRefresh()
}

async function handleUninstall(skill: InstalledSkill) {
  if (!confirm(`Are you sure you want to uninstall ${skill.name}?`)) return

  try {
    for (const agent of skill.agents) {
      await skillsStore.uninstallSkill(skill.name, agent)
    }
  } catch (e) {
    alert(`Failed to uninstall: ${e}`)
  }
}

function handleScopeChange(newScope: 'project' | 'global') {
  skillsStore.fetchInstalledSkills(newScope)
}
</script>

<template>
  <div class="installed-page">
    <header class="header">
      <div class="title-row">
        <h1>Installed Skills</h1>
        <button class="icon-btn" @click="handleRefresh" :disabled="loading">
          <RefreshCw :size="20" :class="{ spinning: loading }" />
        </button>
        <router-link to="/create" class="primary-btn small">
          <Plus :size="16" />
          Create Skill
        </router-link>
      </div>

      <div class="scope-toggle">
        <button
          class="toggle-btn"
          :class="{ active: scope === 'project' }"
          @click="handleScopeChange('project')"
        >
          Project
        </button>
        <button
          class="toggle-btn"
          :class="{ active: scope === 'global' }"
          @click="handleScopeChange('global')"
        >
          Global
        </button>
      </div>
    </header>

    <div v-if="loading && skills.length === 0" class="empty-state">
      <div class="loader"></div>
      <p>Scanning for installed skills...</p>
    </div>

    <div v-else-if="skills.length === 0" class="empty-state">
      <Package :size="48" class="empty-icon" />
      <p>No skills installed in {{ scope }} scope.</p>
    </div>

    <div v-else>
      <SkillList
        :skills="skills"
        @edit="handleEdit"
        @uninstall="handleUninstall"
      />
    </div>

    <!-- Skill Editor Overlay -->
    <SkillEditor
      v-if="editingSkill"
      :skill-path="editingSkill.path"
      :skill-name="editingSkill.name"
      @close="closeEditor"
      @save="handleRefresh"
    />
  </div>
</template>

<style scoped>
.installed-page {
  display: flex;
  flex-direction: column;
  gap: 32px;
}

.title-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  gap: 16px;
}

h1 {
  font-size: 28px;
  font-weight: 700;
  margin: 0;
}

.scope-toggle {
  display: flex;
  background-color: var(--bg-secondary);
  padding: 4px;
  border-radius: 10px;
  width: fit-content;
}

.toggle-btn {
  padding: 8px 20px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-secondary);
  transition: all 0.2s;
}

.toggle-btn.active {
  background-color: var(--bg-tertiary);
  color: var(--accent-primary);
  box-shadow: var(--shadow-sm);
}

.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px 0;
  color: var(--text-secondary);
}

.empty-icon {
  margin-bottom: 16px;
  color: var(--border-color);
}

.icon-btn {
  padding: 8px;
  border-radius: 50%;
  color: var(--text-muted);
  transition: all 0.2s;
}

.icon-btn:hover:not(:disabled) {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.loader {
  width: 48px;
  height: 48px;
  border: 3px solid var(--border-color);
  border-radius: 50%;
  border-top-color: var(--accent-primary);
  animation: spin 1s ease-in-out infinite;
  margin-bottom: 16px;
}

.primary-btn.small {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  background-color: var(--accent-primary);
  color: white;
  border-radius: var(--border-radius);
  font-size: 14px;
  font-weight: 600;
  transition: all 0.2s;
}

.primary-btn.small:hover {
  filter: brightness(1.1);
}
</style>
