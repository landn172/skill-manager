<script setup lang="ts">
import { onMounted, computed, ref, watch } from 'vue'
import { useMarketplaceStore } from '@/stores/marketplace'
import SkillCard from '@/components/skill/SkillCard.vue'
import SearchInput from '@/components/common/SearchInput.vue'
import Modal from '@/components/common/Modal.vue'
import {
  RefreshCw,
  Filter,
  CheckCircle2,
  Sparkles,
  Search,
  AlertCircle,
} from 'lucide-vue-next'
import { useAgentsStore } from '@/stores/agents'
import { useSkillsStore } from '@/stores/skills'
import { invoke } from '@tauri-apps/api/core'
import type { Skill, SearchMode } from '@/types'

const store = useMarketplaceStore()
const agentsStore = useAgentsStore()
const skillsStore = useSkillsStore()

const showInstallModal = ref(false)
const selectedSkill = ref<Skill | null>(null)
const selectedAgents = ref<string[]>([])
const installScope = ref<'project' | 'global'>('global')
const installing = ref(false)
const searchDebounceTimer = ref<ReturnType<typeof setTimeout> | null>(null)

onMounted(() => {
  store.fetchSources()
  store.fetchSkills()
  agentsStore.fetchAgents()
  skillsStore.fetchInstalledSkills()
})

const skills = computed(() => store.filteredSkills)
const isSkillsmpSelected = computed(() => store.selectedSource === 'skillsmp')
const showNoApiKeyWarning = computed(
  () => isSkillsmpSelected.value && !store.hasApiKey
)

// Debounced search for API
watch(
  () => store.searchQuery,
  (query) => {
    if (!isSkillsmpSelected.value) return

    if (searchDebounceTimer.value) {
      clearTimeout(searchDebounceTimer.value)
    }

    searchDebounceTimer.value = setTimeout(() => {
      store.searchSkillsmp(query)
    }, 500)
  }
)

async function handleRefresh() {
  await store.fetchSkills(store.selectedSource || undefined, true)
}

function toggleSearchMode() {
  const newMode: SearchMode = store.searchMode === 'keyword' ? 'ai' : 'keyword'
  store.setSearchMode(newMode)
  // Re-search if there's an existing query
  if (store.searchQuery && isSkillsmpSelected.value) {
    store.searchSkillsmp(store.searchQuery)
  }
}

function openInstallModal(skill: Skill) {
  selectedSkill.value = skill
  selectedAgents.value = agentsStore.agents
    .filter((a) => a.installed)
    .map((a) => a.agent_type)
  showInstallModal.value = true
  console.log(
    'Opening install modal for skill:',
    skill,
    'Available agents:',
    agentsStore.agents
  )
}

async function handleInstall() {
  if (!selectedSkill.value || selectedAgents.value.length === 0) return

  installing.value = true
  console.log(
    'Installing skill:',
    selectedSkill.value,
    'to agents:',
    selectedAgents.value,
    'scope:',
    installScope.value
  )
  try {
    const result = await invoke('install_skill', {
      skill: selectedSkill.value,
      agents: selectedAgents.value,
      scope: installScope.value,
    })
    console.log('Install result:', result)
    alert('Skill installed successfully!')
    showInstallModal.value = false
    skillsStore.fetchInstalledSkills()
  } catch (e) {
    console.error('Installation failed:', e)
    alert(`Installation failed: ${e}`)
  } finally {
    installing.value = false
  }
}

async function handleUninstall(skillName: string) {
  if (!confirm(`Are you sure you want to uninstall ${skillName}?`)) return

  try {
    // Uninstall from all agents that have it installed
    const installedSkill = skillsStore.getSkillByName(skillName)
    if (installedSkill) {
      for (const agent of installedSkill.agents) {
        await skillsStore.uninstallSkill(skillName, agent)
      }
    } else {
      // Fallback: try to uninstall from all agents with global scope
      for (const agent of agentsStore.agents.filter((a) => a.installed)) {
        try {
          await invoke('uninstall_skill', {
            skillName,
            agent: agent.agent_type,
            scope: 'global',
          })
        } catch (e) {
          // Ignore errors for agents that don't have this skill
        }
      }
    }
    skillsStore.fetchInstalledSkills()
    alert('Skill uninstalled successfully!')
  } catch (e) {
    alert(`Failed to uninstall: ${e}`)
  }
}
</script>

<template>
  <div class="marketplace-page">
    <header class="header">
      <div class="title-row">
        <h1>Marketplace</h1>
        <button
          class="icon-btn"
          @click="handleRefresh"
          :disabled="store.loading"
        >
          <RefreshCw :size="20" :class="{ spinning: store.loading }" />
        </button>
      </div>

      <div class="filter-row">
        <SearchInput
          v-model="store.searchQuery"
          :placeholder="
            isSkillsmpSelected
              ? store.searchMode === 'ai'
                ? 'Ask AI: e.g. skills for trading...'
                : 'Search 65,000+ skills...'
              : 'Search skills, descriptions...'
          "
        />

        <!-- Search Mode Toggle (only for SkillsMP) -->
        <button
          v-if="isSkillsmpSelected"
          class="search-mode-btn"
          :class="{ active: store.searchMode === 'ai' }"
          @click="toggleSearchMode"
          title="Toggle AI Semantic Search"
        >
          <Sparkles v-if="store.searchMode === 'ai'" :size="16" />
          <Search v-else :size="16" />
          <span>{{ store.searchMode === 'ai' ? 'AI Search' : 'Keyword' }}</span>
        </button>

        <div class="source-filter">
          <Filter :size="16" />
          <select v-model="store.selectedSource">
            <option :value="null">All Sources</option>
            <option
              v-for="source in store.sources"
              :key="source.id"
              :value="source.id"
            >
              {{ source.name }}
            </option>
          </select>
        </div>
      </div>

      <!-- API Key Warning -->
      <div v-if="showNoApiKeyWarning" class="api-warning">
        <AlertCircle :size="16" />
        <span>No API key configured.</span>
        <router-link to="/settings"
          >Add your SkillsMP API key in Settings</router-link
        >
      </div>
    </header>

    <div v-if="store.loading && skills.length === 0" class="empty-state">
      <div class="loader"></div>
      <p class="loading-status">
        <template v-if="store.fetchProgress.status === 'loading_sources'">
          Loading sources...
        </template>
        <template v-else-if="store.fetchProgress.status === 'fetching'">
          Fetching {{ store.fetchProgress.currentSource }}...
          <span class="progress-count"
            >({{ store.fetchProgress.current }}/{{
              store.fetchProgress.total
            }})</span
          >
        </template>
        <template v-else> Fetching skills from marketplace... </template>
      </p>
      <div v-if="store.fetchProgress.total > 0" class="progress-bar">
        <div
          class="progress-fill"
          :style="{
            width: `${
              (store.fetchProgress.current / store.fetchProgress.total) * 100
            }%`,
          }"
        ></div>
      </div>
    </div>

    <div v-else-if="store.error" class="error-state">
      <AlertCircle :size="32" />
      <p>{{ store.error }}</p>
      <button @click="handleRefresh">Retry</button>
    </div>

    <div v-else-if="skills.length === 0" class="empty-state">
      <p>No skills found matching your search.</p>
    </div>

    <div v-else class="skills-grid">
      <SkillCard
        v-for="skill in skills"
        :key="skill.name + skill.source_id"
        :skill="skill"
        :show-source="true"
        @install="openInstallModal"
        @update="openInstallModal"
        @uninstall="handleUninstall"
      />
    </div>

    <!-- Installation Modal -->
    <Modal
      :show="showInstallModal"
      :title="`Install ${selectedSkill?.name}`"
      @close="showInstallModal = false"
    >
      <div class="install-form">
        <p class="form-help">
          Select the agents you want to install this skill to.
        </p>

        <div class="agent-selection">
          <div
            v-for="agent in agentsStore.agents"
            :key="agent.agent_type"
            class="agent-option"
            :class="{
              selected: selectedAgents.includes(agent.agent_type),
              disabled: !agent.installed,
            }"
            @click="
              agent.installed &&
                (selectedAgents.includes(agent.agent_type)
                  ? (selectedAgents = selectedAgents.filter(
                      (a) => a !== agent.agent_type
                    ))
                  : selectedAgents.push(agent.agent_type))
            "
          >
            <span class="agent-icon">{{
              agent.icon === 'Sparkles'
                ? '✨'
                : agent.icon === 'Terminal'
                ? '💻'
                : agent.icon === 'Bot'
                ? '🤖'
                : agent.icon === 'Code'
                ? '📄'
                : '🖱️'
            }}</span>
            <div class="agent-name">{{ agent.display_name }}</div>
            <div
              class="check-wrap"
              v-if="selectedAgents.includes(agent.agent_type)"
            >
              <CheckCircle2 :size="16" />
            </div>
          </div>
        </div>

        <div class="scope-selection">
          <label>Installation Scope</label>
          <div class="scope-options">
            <button
              class="scope-btn"
              :class="{ active: installScope === 'project' }"
              @click="installScope = 'project'"
            >
              Project
            </button>
            <button
              class="scope-btn"
              :class="{ active: installScope === 'global' }"
              @click="installScope = 'global'"
            >
              Global
            </button>
          </div>
        </div>
      </div>

      <template #footer>
        <button class="footer-btn secondary" @click="showInstallModal = false">
          Cancel
        </button>
        <button
          class="footer-btn primary"
          :disabled="selectedAgents.length === 0 || installing"
          @click="handleInstall"
        >
          {{ installing ? 'Installing...' : 'Install Skill' }}
        </button>
      </template>
    </Modal>
  </div>
</template>

<style scoped>
.marketplace-page {
  display: flex;
  flex-direction: column;
  gap: 32px;
}

.title-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

h1 {
  font-size: 28px;
  font-weight: 700;
  margin: 0;
}

.filter-row {
  display: flex;
  gap: 16px;
  align-items: center;
}

.search-mode-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 16px;
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  font-size: 13px;
  color: var(--text-secondary);
  transition: all 0.2s;
}

.search-mode-btn:hover {
  border-color: var(--accent-primary);
  color: var(--accent-primary);
}

.search-mode-btn.active {
  background: linear-gradient(
    135deg,
    rgba(139, 92, 246, 0.1),
    rgba(168, 85, 247, 0.15)
  );
  border-color: var(--accent-primary);
  color: var(--accent-primary);
}

.source-filter {
  display: flex;
  align-items: center;
  gap: 10px;
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  padding: 0 12px;
  height: 42px;
}

select {
  background: none;
  border: none;
  outline: none;
  font-size: 14px;
  color: var(--text-primary);
  padding-right: 8px;
}

.api-warning {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  background-color: rgba(251, 191, 36, 0.1);
  border: 1px solid rgba(251, 191, 36, 0.3);
  border-radius: var(--border-radius);
  color: var(--text-secondary);
  font-size: 14px;
  margin-top: 16px;
}

.api-warning svg {
  color: #f59e0b;
  flex-shrink: 0;
}

.api-warning a {
  color: var(--accent-primary);
  margin-left: 4px;
}

.api-warning a:hover {
  text-decoration: underline;
}

.skills-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 20px;
}

.empty-state,
.error-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px 0;
  color: var(--text-secondary);
  text-align: center;
}

.error-state svg {
  color: var(--accent-error);
  margin-bottom: 16px;
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

.loading-status {
  margin: 0 0 16px;
}

.progress-count {
  color: var(--text-muted);
  font-size: 13px;
}

.progress-bar {
  width: 200px;
  height: 6px;
  background-color: var(--bg-tertiary);
  border-radius: 3px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(
    90deg,
    var(--accent-primary),
    var(--accent-secondary)
  );
  border-radius: 3px;
  transition: width 0.3s ease;
}

/* Previous styles... */
.install-form {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.form-help {
  font-size: 14px;
  color: var(--text-secondary);
  margin: 0;
}

.agent-selection {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.agent-option {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background-color: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  cursor: pointer;
  transition: all 0.2s;
}

.agent-option:hover:not(.disabled) {
  border-color: var(--accent-primary);
}

.agent-option.selected {
  border-color: var(--accent-primary);
  background-color: rgba(139, 92, 246, 0.05);
}

.agent-option.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.agent-icon {
  font-size: 18px;
}

.agent-name {
  font-size: 14px;
  font-weight: 500;
  flex: 1;
}

.check-wrap {
  color: var(--accent-primary);
}

.scope-selection {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.scope-selection label {
  font-size: 14px;
  font-weight: 500;
}

.scope-options {
  display: flex;
  gap: 8px;
  background-color: var(--bg-tertiary);
  padding: 4px;
  border-radius: 10px;
}

.scope-btn {
  flex: 1;
  padding: 8px;
  border-radius: 8px;
  font-size: 13px;
  color: var(--text-secondary);
}

.scope-btn.active {
  background-color: var(--bg-secondary);
  color: var(--accent-primary);
  box-shadow: var(--shadow-sm);
}

.footer-btn {
  padding: 8px 20px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
}

.footer-btn.primary {
  background-color: var(--accent-primary);
  color: white;
}

.footer-btn.primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.footer-btn.secondary {
  color: var(--text-secondary);
}

.footer-btn.secondary:hover {
  background-color: var(--bg-hover);
}
</style>
