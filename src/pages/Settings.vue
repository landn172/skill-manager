<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import { useAgentsStore } from '@/stores/agents'
import { useMarketplaceStore } from '@/stores/marketplace'
import {
  CheckCircle2,
  XCircle,
  Plus,
  Trash2,
  ShieldCheck,
  Globe,
  Folder,
  Key,
  ExternalLink,
} from 'lucide-vue-next'
import { useProjectStore } from '@/stores/project'
import { open } from '@tauri-apps/plugin-dialog'
import { homeDir } from '@tauri-apps/api/path'
import { invoke } from '@tauri-apps/api/core'

const agentsStore = useAgentsStore()
const marketplaceStore = useMarketplaceStore()
const projectStore = useProjectStore()

// API Key management
const apiKeyInput = ref('')
const maskedApiKey = ref<string | null>(null)
const apiKeySource = ref<string | null>(null) // 'env' or 'db'
const savingApiKey = ref(false)

const hasApiKey = computed(() => !!maskedApiKey.value)
const isFromEnv = computed(() => apiKeySource.value === 'env')

async function loadApiKey() {
  try {
    maskedApiKey.value = await invoke<string | null>(
      'get_skillsmp_api_key_masked'
    )
    apiKeySource.value = await invoke<string | null>(
      'get_skillsmp_api_key_source'
    )
  } catch (e) {
    console.error('Failed to load API key', e)
  }
}

async function saveApiKey() {
  if (!apiKeyInput.value.trim()) return

  savingApiKey.value = true
  try {
    await invoke('set_skillsmp_api_key', { key: apiKeyInput.value.trim() })
    apiKeyInput.value = ''
    await loadApiKey()
    alert('API key saved successfully!')
  } catch (e) {
    alert(`Failed to save API key: ${e}`)
  } finally {
    savingApiKey.value = false
  }
}

async function clearApiKey() {
  if (
    !confirm(
      'Are you sure you want to remove your SkillsMP API key from Settings?'
    )
  )
    return

  try {
    await invoke('clear_skillsmp_api_key')
    await loadApiKey() // Reload to check if still exists in .env
    if (!hasApiKey.value) {
      alert('API key removed.')
    } else {
      alert('Database key removed. Key from .env is still active.')
    }
  } catch (e) {
    alert(`Failed to clear API key: ${e}`)
  }
}

onMounted(() => {
  agentsStore.fetchAgents()
  marketplaceStore.fetchSources()
  projectStore.fetchProjects()
  loadApiKey()
})

const handleAddProject = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: await homeDir(),
    })

    if (selected && typeof selected === 'string') {
      const name = selected.split('/').pop() || 'New Project'
      await projectStore.addProject(name, selected)
    }
  } catch (e) {
    console.error('Failed to add project', e)
  }
}
</script>

<template>
  <div class="settings-page">
    <header class="header">
      <h1>Settings</h1>
    </header>

    <div class="sections">
      <!-- API Configuration Section -->
      <section class="settings-section">
        <div class="section-header">
          <Key :size="20" class="section-icon" />
          <h2>SkillsMP API</h2>
        </div>
        <p class="section-desc">
          Configure your SkillsMP API key to access 65,000+ skills from the
          marketplace.
        </p>

        <div class="api-config">
          <div v-if="hasApiKey" class="api-key-display">
            <div class="key-info">
              <span class="key-label">Current API Key:</span>
              <code class="key-value">{{ maskedApiKey }}</code>
              <span v-if="isFromEnv" class="env-badge">from .env</span>
              <span v-else class="db-badge">from Settings</span>
            </div>
            <button v-if="!isFromEnv" class="btn-danger" @click="clearApiKey">
              <Trash2 :size="16" />
              Remove Key
            </button>
          </div>

          <div v-else class="api-key-input">
            <input
              v-model="apiKeyInput"
              type="password"
              placeholder="Enter your SkillsMP API key (sk_live_...)"
              class="input-field"
            />
            <button
              class="btn-primary"
              :disabled="!apiKeyInput.trim() || savingApiKey"
              @click="saveApiKey"
            >
              {{ savingApiKey ? 'Saving...' : 'Save Key' }}
            </button>
          </div>

          <a href="https://skillsmp.com" target="_blank" class="get-key-link">
            <ExternalLink :size="14" />
            Get your API key from skillsmp.com
          </a>
        </div>
      </section>

      <section class="settings-section">
        <div class="section-header">
          <ShieldCheck :size="20" class="section-icon" />
          <h2>Detected Agents</h2>
        </div>
        <p class="section-desc">
          We automatically detect installed coding agents on your system.
        </p>

        <div class="agents-list">
          <div
            v-for="agent in agentsStore.agents"
            :key="agent.name"
            class="agent-card"
          >
            <div class="agent-info">
              <span class="agent-icon-wrap">{{
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
              <div class="agent-details">
                <span class="agent-name">{{ agent.display_name }}</span>
                <span class="agent-path">{{ agent.global_skills_dir }}</span>
              </div>
            </div>
            <div class="agent-status" :class="{ installed: agent.installed }">
              <template v-if="agent.installed">
                <CheckCircle2 :size="16" />
                <span>Detected</span>
              </template>
              <template v-else>
                <XCircle :size="16" />
                <span>Not Found</span>
              </template>
            </div>
          </div>
        </div>
      </section>

      <section class="settings-section">
        <div class="section-header">
          <Globe :size="20" class="section-icon" />
          <h2>Marketplace Sources</h2>
        </div>
        <p class="section-desc">
          Manage the repositories where you discover new skills.
        </p>

        <div class="sources-list">
          <div
            v-for="source in marketplaceStore.sources"
            :key="source.id"
            class="source-card"
          >
            <div class="source-info">
              <div class="source-name-row">
                <span class="source-name">{{ source.name }}</span>
                <span v-if="source.official" class="official-badge"
                  >Official</span
                >
                <span v-if="source.source_type === 'api'" class="api-badge"
                  >API</span
                >
              </div>
              <span class="source-url">{{ source.url }}</span>
            </div>
            <div class="source-actions">
              <button
                class="icon-btn delete"
                v-if="!source.official && source.source_type !== 'api'"
              >
                <Trash2 :size="18" />
              </button>
            </div>
          </div>

          <button class="add-btn" @click="marketplaceStore.addSource">
            <Plus :size="18" />
            <span>Add Custom Source</span>
          </button>
        </div>
      </section>

      <section class="settings-section">
        <div class="section-header">
          <Folder :size="20" class="section-icon" />
          <h2>Managed Projects</h2>
        </div>
        <p class="section-desc">
          Add the local directories where your projects are located to manage
          their skills.
        </p>

        <div class="sources-list">
          <div
            v-for="project in projectStore.projects"
            :key="project.id"
            class="source-card"
          >
            <div class="source-info">
              <div class="source-name-row">
                <span class="source-name">{{ project.name }}</span>
              </div>
              <span class="source-url">{{ project.path }}</span>
            </div>
            <div class="source-actions">
              <button
                class="icon-btn delete"
                @click="projectStore.removeProject(project.id!)"
              >
                <Trash2 :size="18" />
              </button>
            </div>
          </div>

          <button class="add-btn" @click="handleAddProject">
            <Plus :size="18" />
            <span>Add Project Directory</span>
          </button>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.settings-page {
  display: flex;
  flex-direction: column;
  gap: 32px;
}

h1 {
  font-size: 28px;
  font-weight: 700;
  margin: 0;
}

.sections {
  display: flex;
  flex-direction: column;
  gap: 40px;
  max-width: 800px;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}

.section-icon {
  color: var(--accent-primary);
}

h2 {
  font-size: 20px;
  font-weight: 600;
  margin: 0;
}

.section-desc {
  color: var(--text-secondary);
  font-size: 14px;
  margin: 0 0 20px;
}

/* API Configuration Styles */
.api-config {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.api-key-display {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  padding: 16px;
}

.key-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.key-label {
  font-size: 14px;
  color: var(--text-secondary);
}

.key-value {
  font-family: monospace;
  background-color: var(--bg-tertiary);
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 13px;
}

.env-badge {
  font-size: 11px;
  font-weight: 600;
  color: #22c55e;
  background-color: rgba(34, 197, 94, 0.1);
  padding: 2px 8px;
  border-radius: 4px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.db-badge {
  font-size: 11px;
  font-weight: 600;
  color: var(--accent-primary);
  background-color: rgba(139, 92, 246, 0.1);
  padding: 2px 8px;
  border-radius: 4px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.api-key-input {
  display: flex;
  gap: 12px;
}

.input-field {
  flex: 1;
  padding: 12px 16px;
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  font-size: 14px;
  color: var(--text-primary);
}

.input-field:focus {
  outline: none;
  border-color: var(--accent-primary);
}

.btn-primary {
  padding: 12px 24px;
  background-color: var(--accent-primary);
  color: white;
  border-radius: var(--border-radius);
  font-weight: 500;
  font-size: 14px;
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-danger {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  color: var(--accent-error);
  border: 1px solid var(--accent-error);
  border-radius: var(--border-radius);
  font-size: 13px;
}

.btn-danger:hover {
  background-color: rgba(239, 68, 68, 0.1);
}

.get-key-link {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--accent-primary);
}

.get-key-link:hover {
  text-decoration: underline;
}

.api-badge {
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  font-weight: 700;
  color: var(--accent-success);
  background-color: rgba(34, 197, 94, 0.1);
  padding: 2px 6px;
  border-radius: 4px;
}

/* Existing Styles */
.agents-list,
.sources-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.agent-card,
.source-card {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  padding: 16px;
}

.agent-info {
  display: flex;
  align-items: center;
  gap: 16px;
}

.agent-icon-wrap {
  font-size: 20px;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--bg-tertiary);
  border-radius: 8px;
}

.agent-details,
.source-info {
  display: flex;
  flex-direction: column;
}

.agent-name,
.source-name {
  font-weight: 600;
  font-size: 15px;
}

.agent-path,
.source-url {
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 2px;
}

.agent-status {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-muted);
  padding: 4px 10px;
  background-color: var(--bg-tertiary);
  border-radius: 20px;
}

.agent-status.installed {
  color: var(--accent-success);
  background-color: rgba(34, 197, 94, 0.1);
}

.source-name-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.official-badge {
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  font-weight: 700;
  color: var(--accent-primary);
  background-color: rgba(139, 92, 246, 0.1);
  padding: 2px 6px;
  border-radius: 4px;
}

.add-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 12px;
  border: 1px dashed var(--border-color);
  border-radius: var(--border-radius);
  color: var(--text-secondary);
  transition: all 0.2s;
  margin-top: 8px;
}

.add-btn:hover {
  border-color: var(--accent-primary);
  color: var(--accent-primary);
  background-color: rgba(139, 92, 246, 0.05);
}

.icon-btn.delete {
  color: var(--text-muted);
  padding: 8px;
  border-radius: 8px;
  transition: all 0.2s;
}

.icon-btn.delete:hover {
  color: var(--accent-error);
  background-color: rgba(239, 68, 68, 0.1);
}
</style>
