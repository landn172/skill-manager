<script setup lang="ts">
import { onMounted, computed, ref } from 'vue'
import type { InstalledSkill } from '@/types'
import SkillList from '@/components/skill/SkillList.vue'
import SkillEditor from '@/components/skill/SkillEditor.vue'
import Modal from '@/components/common/Modal.vue'
import AgentIcon from '@/components/icons/AgentIcon.vue'
import { RefreshCw, Package, Plus, CheckCircle2 } from 'lucide-vue-next'
import { useSkillsStore } from '@/stores/skills'
import { useAgentsStore } from '@/stores/agents'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

const skillsStore = useSkillsStore()
const agentsStore = useAgentsStore()

// Install Modal State
const showInstallModal = ref(false)
const selectedSkill = ref<InstalledSkill | null>(null)
const selectedAgents = ref<string[]>([])
const installScope = ref<'project' | 'global'>('global')
const installing = ref(false)
const installLogs = ref<
  Array<{ time: string; message: string; type: 'info' | 'error' | 'success' }>
>([])

const scope = computed({
  get: () => skillsStore.scope,
  set: (val) => skillsStore.fetchInstalledSkills(val),
})

const skills = computed(() => skillsStore.installedSkills)
const loading = computed(() => skillsStore.loading)

const editingSkill = ref<InstalledSkill | null>(null)

onMounted(() => {
  skillsStore.fetchInstalledSkills()
  agentsStore.fetchAgents() // Ensure agents are loaded
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

// Open modal for supplemental install
function openInstallModal(skill: InstalledSkill) {
  installLogs.value = []
  selectedSkill.value = skill
  // Default to agents that are installed but DON'T have this skill yet
  const installedFor = skill.agents || []
  selectedAgents.value = agentsStore.agents
    .filter((a) => a.installed && !installedFor.includes(a.agent_type))
    .map((a) => a.agent_type)

  // If all possible agents already have it (shouldn't happen due to button logic),
  // maybe select none or all? Let's select none to force user choice.
  if (selectedAgents.value.length === 0) {
    selectedAgents.value = []
  }

  showInstallModal.value = true
}

function getLogTime() {
  return new Date().toLocaleTimeString('en-US', {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
}

async function handleInstall() {
  if (!selectedSkill.value || selectedAgents.value.length === 0) return

  installing.value = true
  installLogs.value = []
  installLogs.value.push({
    time: getLogTime(),
    message: 'Initializing installation...',
    type: 'info',
  })

  let unlisten: (() => void) | undefined

  try {
    unlisten = await listen<{
      skill: string
      status: string
      message: string
      agent?: string
    }>('install-progress', (event) => {
      const type =
        event.payload.status === 'error'
          ? 'error'
          : event.payload.status === 'finished'
            ? 'success'
            : 'info'
      installLogs.value.push({
        time: getLogTime(),
        message: event.payload.message,
        type,
      })

      const terminal = document.getElementById('install-terminal')
      if (terminal) {
        setTimeout(() => {
          terminal.scrollTop = terminal.scrollHeight
        }, 10)
      }
    })

    // We need to pass the inner `Skill` object from InstalledSkill
    // InstalledSkill extends Skill, so we can pass it directly.
    const skillToInstall = selectedSkill.value

    const results = await invoke<
      Array<{
        success: boolean
        path: string
        agent: string
        error?: string
      }>
    >('install_skill', {
      skill: skillToInstall,
      agents: selectedAgents.value,
      scope: installScope.value,
    })

    const successful = results.filter((r) => r.success)
    const failed = results.filter((r) => !r.success)

    if (failed.length > 0) {
      const errors = failed.map((f) => `${f.agent}: ${f.error}`).join('\n')
      installLogs.value.push({
        time: getLogTime(),
        message: `Some installations failed: ${errors}`,
        type: 'error',
      })
      if (successful.length === 0) {
        throw new Error(`All installations failed:\n${errors}`)
      }
    }

    if (successful.length > 0) {
      installLogs.value.push({
        time: getLogTime(),
        message: 'Installation completed successfully.',
        type: 'success',
      })

      await new Promise((resolve) => setTimeout(resolve, 1500))

      showInstallModal.value = false
      skillsStore.fetchInstalledSkills()
    }
  } catch (e) {
    installLogs.value.push({
      time: getLogTime(),
      message: `Installation failed: ${e}`,
      type: 'error',
    })
  } finally {
    if (unlisten) unlisten()
    installing.value = false
  }
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
        @install="openInstallModal"
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

    <!-- Installation Modal -->
    <Modal
      :show="showInstallModal"
      :title="`Install ${selectedSkill?.name}`"
      maxWidth="800px"
      @close="showInstallModal = false"
    >
      <div class="modal-content-grid">
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
                      (a) => a !== agent.agent_type,
                    ))
                  : selectedAgents.push(agent.agent_type))
              "
            >
              <AgentIcon
                :type="agentsStore.getIcon(agent.agent_type)"
                :size="18"
                class="agent-icon"
              />
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

        <div class="install-terminal-container">
          <div class="terminal-header">Installation Logs</div>
          <div id="install-terminal" class="install-terminal">
            <div v-if="installLogs.length === 0" class="terminal-placeholder">
              Ready to install...
            </div>
            <div
              v-else
              v-for="(log, idx) in installLogs"
              :key="idx"
              class="log-line"
              :class="log.type"
            >
              <span class="log-time">[{{ log.time }}]</span>
              <span class="log-msg">{{ log.message }}</span>
            </div>
          </div>
        </div>
      </div>

      <template #footer>
        <button
          class="footer-btn secondary"
          @click="showInstallModal = false"
          :disabled="installing"
        >
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

/* Modal Styles */
.modal-content-grid {
  display: flex;
  gap: 24px;
  height: 400px;
}

.install-form {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.install-terminal-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  background-color: #1e1e1e;
  border-radius: var(--border-radius);
  border: 1px solid #333;
  overflow: hidden;
  box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.3);
}

.terminal-header {
  padding: 8px 12px;
  background-color: #252526;
  border-bottom: 1px solid #333;
  font-size: 11px;
  font-weight: 600;
  color: #cccccc;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  display: flex;
  align-items: center;
  gap: 6px;
}

.terminal-header::before {
  content: '';
  display: block;
  width: 8px;
  height: 8px;
  background-color: #22c55e;
  border-radius: 50%;
}

.install-terminal {
  flex: 1;
  padding: 12px;
  overflow-y: auto;
  font-family:
    'JetBrains Mono', 'Fira Code', 'SF Mono', 'Roboto Mono', 'Menlo', monospace;
  font-size: 12px;
  line-height: 1.6;
  color: #d4d4d4;
}

/* Custom Scrollbar for terminal */
.install-terminal::-webkit-scrollbar {
  width: 8px;
}

.install-terminal::-webkit-scrollbar-track {
  background: #1e1e1e;
}

.install-terminal::-webkit-scrollbar-thumb {
  background-color: #424242;
  border-radius: 4px;
}

.terminal-placeholder {
  color: #6e7681;
  font-style: italic;
  padding: 4px 0;
}

.log-line {
  display: flex;
  gap: 12px;
  padding: 2px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.02);
}

.log-line:last-child {
  border-bottom: none;
}

.log-line.error {
  color: #f87171;
}

.log-line.success {
  color: #4ade80;
}

.log-line.info {
  color: #60a5fa;
}

.log-time {
  color: #6e7681;
  font-size: 11px;
  flex-shrink: 0;
  user-select: none;
}

.log-msg {
  word-break: break-word;
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
