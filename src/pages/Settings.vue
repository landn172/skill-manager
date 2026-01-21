<script setup lang="ts">
import { onMounted, ref, computed } from "vue";
import { useAgentsStore } from "@/stores/agents";
import { useMarketplaceStore } from "@/stores/marketplace";
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
  Sun,
  Moon,
  Monitor,
  Palette,
  Download,
  Upload,
  Edit2,
} from "lucide-vue-next";
import { useThemeStore } from "@/stores/theme";
import AgentIcon from "@/components/icons/AgentIcon.vue";
import { useProjectStore } from "@/stores/project";
import { open, save } from "@tauri-apps/plugin-dialog";
import { homeDir } from "@tauri-apps/api/path";
import { invoke } from "@tauri-apps/api/core";
import { writeTextFile, readTextFile, readFile } from "@tauri-apps/plugin-fs";

const agentsStore = useAgentsStore();
const marketplaceStore = useMarketplaceStore();
const projectStore = useProjectStore();
const themeStore = useThemeStore();

// API Key management
const apiKeyInput = ref("");
const maskedApiKey = ref<string | null>(null);
const apiKeySource = ref<string | null>(null); // 'env' or 'db'
const savingApiKey = ref(false);

const hasApiKey = computed(() => !!maskedApiKey.value);
const isFromEnv = computed(() => apiKeySource.value === "env");

async function loadApiKey() {
  try {
    maskedApiKey.value = await invoke<string | null>("get_skillsmp_api_key_masked");
    apiKeySource.value = await invoke<string | null>("get_skillsmp_api_key_source");
  } catch (e) {
    console.error("Failed to load API key", e);
  }
}

async function saveApiKey() {
  if (!apiKeyInput.value.trim()) return;

  savingApiKey.value = true;
  try {
    await invoke("set_skillsmp_api_key", { key: apiKeyInput.value.trim() });
    apiKeyInput.value = "";
    await loadApiKey();
    alert("API key saved successfully!");
  } catch (e) {
    alert(`Failed to save API key: ${e}`);
  } finally {
    savingApiKey.value = false;
  }
}

const showAddRegistryModal = ref(false);
const registryUrl = ref("");
const registryName = ref("");
const addingRegistry = ref(false);

async function saveRegistrySource() {
  if (!registryUrl.value.trim() || !registryName.value.trim()) return;

  addingRegistry.value = true;
  try {
    await marketplaceStore.addSource(registryUrl.value.trim(), registryName.value.trim());
    showAddRegistryModal.value = false;
    registryUrl.value = "";
    registryName.value = "";
    alert("Registry source added successfully!");
  } catch (e) {
    alert(`Failed to add registry source: ${e}`);
  } finally {
    addingRegistry.value = false;
  }
}

async function removeSource(id: string) {
  if (!confirm("Are you sure you want to remove this source?")) return;
  try {
    await marketplaceStore.removeSource(id);
  } catch (e) {
    alert(`Failed to remove source: ${e}`);
  }
}

async function toggleSource(id: string, event: Event) {
  const checkbox = event.target as HTMLInputElement;
  try {
    await marketplaceStore.toggleSource(id, checkbox.checked);
  } catch (e) {
    alert(`Failed to toggle source: ${e}`);
    checkbox.checked = !checkbox.checked; // Revert
  }
}
async function clearApiKey() {
  if (!confirm("Are you sure you want to remove your SkillsMP API key from Settings?")) return;

  try {
    await invoke("clear_skillsmp_api_key");
    await loadApiKey(); // Reload to check if still exists in .env
    if (!hasApiKey.value) {
      alert("API key removed.");
    } else {
      alert("Database key removed. Key from .env is still active.");
    }
  } catch (e) {
    alert(`Failed to clear API key: ${e}`);
  }
}

onMounted(() => {
  agentsStore.fetchAgents();
  marketplaceStore.fetchSources();
  projectStore.fetchProjects();
  loadApiKey();
});

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
    const message = await invoke<string>("clear_cache");
    alert(message);
    // Optionally refresh marketplace store if needed
    await marketplaceStore.fetchSources();
  } catch (e) {
    alert(`Failed to clear cache: ${e}`);
  } finally {
    clearingCache.value = false;
  }
}

const handleAddProject = async () => {
  try {
    const selected = await open({
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

async function handleAddLocalSource() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: await homeDir(),
    });

    if (selected && typeof selected === "string") {
      const name = selected.split("/").pop() || "Local Skills";
      // Prompt for name (optional, could use a modal but prompt is simple for now)
      // Since window.prompt might be blocked or ugly, let's just use directory name for now
      // or repurpose the registry modal?
      // Let's just use directory name + "(Local)" to start, can be improved.
      // Actually, let's use a browser prompt if possible or just auto-add.
      // Auto-add is smoothest.

      await marketplaceStore.addSource(selected, name, "local");
      alert(`Added local source: ${name}`);
      await marketplaceStore.fetchSkills(); // Refresh skills to show new ones
    }
  } catch (e) {
    alert(`Failed to add local source: ${e}`);
  }
}

async function handleExportConfig() {
  try {
    const json = await invoke<string>("export_config");
    const filePath = await save({
      filters: [{ name: "JSON", extensions: ["json"] }],
      defaultPath: "skill-manager-config.json",
    });

    if (filePath) {
      await writeTextFile(filePath, json);
      alert("Configuration exported successfully!");
    }
  } catch (e) {
    alert(`Failed to export config: ${e}`);
  }
}

async function handleImportConfig() {
  try {
    const filePath = await open({
      filters: [{ name: "JSON", extensions: ["json"] }],
      multiple: false,
    });

    if (filePath && typeof filePath === "string") {
      const json = await readTextFile(filePath);
      await invoke("import_config", { json });
      alert("Configuration imported successfully! refreshing...");
      marketplaceStore.fetchSources();
    }
  } catch (e) {
    alert(`Failed to import config: ${e}`);
  }
}

// Agent Path Management
const editingAgent = ref<any>(null);
const editAgentPath = ref("");
const savingAgentPath = ref(false);

function openEditAgent(agent: any) {
  editingAgent.value = agent;
  editAgentPath.value = agent.global_skills_dir;
}

async function saveAgentPath() {
  if (!editingAgent.value) return;

  savingAgentPath.value = true;
  try {
    await invoke("update_agent_path", {
      agentType: editingAgent.value.agent_type,
      path: editAgentPath.value,
    });

    // Refresh agents
    await agentsStore.fetchAgents();
    editingAgent.value = null;
    editAgentPath.value = "";
    alert("Agent path updated!");
  } catch (e) {
    alert(`Failed to update agent path: ${e}`);
  } finally {
    savingAgentPath.value = false;
  }
}

async function browseForAgentPath() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: await homeDir(),
    });

    if (selected && typeof selected === "string") {
      editAgentPath.value = selected;
    }
  } catch (e) {
    console.error(e);
  }
}

// Custom Agent Management
const showAddAgentModal = ref(false);
const newAgentName = ref("");
const newAgentPath = ref("");
const newAgentIconType = ref<"emoji" | "image">("emoji");
const newAgentIcon = ref("🚀");
const addingAgent = ref(false);

async function pickIconImage() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "svg", "webp"] }],
    });
    if (selected && typeof selected === "string") {
      const data = await readFile(selected);
      const blob = new Blob([data]);
      const reader = new FileReader();
      reader.onload = () => {
        newAgentIcon.value = reader.result as string;
      };
      reader.readAsDataURL(blob);
    }
  } catch (e) {
    console.error("Failed to pick icon image", e);
  }
}

async function addCustomAgent() {
  if (!newAgentName.value || !newAgentPath.value) return;

  addingAgent.value = true;
  try {
    await invoke("add_custom_agent", {
      name: newAgentName.value,
      path: newAgentPath.value,
      icon: newAgentIcon.value,
    });

    await agentsStore.fetchAgents();
    showAddAgentModal.value = false;
    newAgentName.value = "";
    newAgentPath.value = "";
    newAgentIcon.value = "🚀";
    newAgentIconType.value = "emoji";
    alert("Custom agent added!");
  } catch (e) {
    alert(`Failed to add agent: ${e}`);
  } finally {
    addingAgent.value = false;
  }
}

async function removeCustomAgent(agent: any) {
  if (!confirm(`Remove custom agent "${agent.display_name}"?`)) return;

  try {
    await invoke("remove_custom_agent", {
      agentType: agent.agent_type,
    });
    await agentsStore.fetchAgents();
  } catch (e) {
    alert(`Failed to remove agent: ${e}`);
  }
}

async function browseForNewAgentPath() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: await homeDir(),
    });

    if (selected && typeof selected === "string") {
      newAgentPath.value = selected;
    }
  } catch (e) {
    console.error(e);
  }
}
</script>

<template>
  <div class="settings-page">
    <header class="header">
      <h1>Settings</h1>
    </header>

    <div class="sections">
      <!-- Appearance Section -->
      <section class="settings-section">
        <div class="section-header">
          <Palette :size="20" class="section-icon" />
          <h2>App Appearance</h2>
        </div>
        <p class="section-desc">Customize the look and feel of the application.</p>

        <div class="theme-selector">
          <button
            class="theme-btn"
            :class="{ active: themeStore.theme === 'light' }"
            @click="themeStore.setTheme('light')"
          >
            <Sun :size="18" />
            <span>Light</span>
          </button>
          <button
            class="theme-btn"
            :class="{ active: themeStore.theme === 'dark' }"
            @click="themeStore.setTheme('dark')"
          >
            <Moon :size="18" />
            <span>Dark</span>
          </button>
          <button
            class="theme-btn"
            :class="{ active: themeStore.theme === 'system' }"
            @click="themeStore.setTheme('system')"
          >
            <Monitor :size="18" />
            <span>System</span>
          </button>
        </div>
      </section>

      <!-- API Configuration Section -->
      <section class="settings-section">
        <div class="section-header">
          <Key :size="20" class="section-icon" />
          <h2>SkillsMP API</h2>
        </div>
        <p class="section-desc">
          Configure your SkillsMP API key to access 65,000+ skills from the marketplace.
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
              {{ savingApiKey ? "Saving..." : "Save Key" }}
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
        <p class="section-desc">We automatically detect installed coding agents on your system.</p>

        <div class="agents-list">
          <div v-for="agent in agentsStore.agents" :key="agent.name" class="agent-card">
            <div class="agent-info">
              <div class="agent-icon-wrap">
                <AgentIcon :type="agentsStore.getIcon(agent.agent_type)" :size="24" />
              </div>
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
            <!-- Add Edit Button -->
            <button class="icon-btn edit-agent" @click="openEditAgent(agent)" title="Edit Path">
              <Edit2 :size="16" />
            </button>
            <button
              v-if="agent.is_custom"
              class="icon-btn delete-agent"
              @click="removeCustomAgent(agent)"
              title="Remove Agent"
            >
              <Trash2 :size="16" />
            </button>
          </div>
        </div>

        <div class="agents-actions">
          <button class="add-btn" @click="showAddAgentModal = true">
            <Plus :size="18" />
            <span>Add Custom Agent</span>
          </button>
        </div>
      </section>

      <!-- Add Agent Modal -->
      <div v-if="showAddAgentModal" class="modal-overlay">
        <div class="modal-content">
          <h3>Add Custom Agent</h3>
          <p>Add a new IDE or tool to the list.</p>

          <div class="form-group">
            <label>Agent Name</label>
            <input v-model="newAgentName" placeholder="e.g. My IDE" class="input-field" />
          </div>

          <div class="form-group">
            <label>Icon</label>
            <div class="icon-selector-wrap">
              <div class="icon-type-toggle">
                <button
                  class="toggle-btn"
                  :class="{ active: newAgentIconType === 'emoji' }"
                  @click="newAgentIconType = 'emoji'"
                >
                  Emoji
                </button>
                <button
                  class="toggle-btn"
                  :class="{ active: newAgentIconType === 'image' }"
                  @click="newAgentIconType = 'image'"
                >
                  Image
                </button>
              </div>

              <div class="icon-input-row">
                <div class="icon-preview">
                  <AgentIcon :type="newAgentIcon" :size="32" />
                </div>
                <template v-if="newAgentIconType === 'emoji'">
                  <input
                    v-model="newAgentIcon"
                    placeholder="Emoji (e.g. 🚀)"
                    class="input-field icon-input"
                    maxlength="8"
                  />
                </template>
                <template v-else>
                  <button class="btn-secondary" @click="pickIconImage">
                    {{ newAgentIcon.startsWith("data:") ? "Change Image" : "Select Image" }}
                  </button>
                </template>
              </div>
            </div>
          </div>

          <div class="form-group">
            <label>Skills Directory Path</label>
            <div class="input-row">
              <input v-model="newAgentPath" class="input-field" />
              <button class="btn-secondary" @click="browseForNewAgentPath">Browse</button>
            </div>
          </div>

          <div class="modal-actions">
            <button class="btn-secondary" @click="showAddAgentModal = false">Cancel</button>
            <button
              class="btn-primary"
              :disabled="!newAgentName || !newAgentPath || addingAgent"
              @click="addCustomAgent"
            >
              {{ addingAgent ? "Adding..." : "Add Agent" }}
            </button>
          </div>
        </div>
      </div>

      <!-- Edit Agent Modal -->
      <div v-if="editingAgent" class="modal-overlay">
        <div class="modal-content">
          <h3>Edit {{ editingAgent.display_name }} Path</h3>
          <p>Manually set the skills directory for this agent.</p>

          <div class="form-group">
            <label>Skills Directory Path</label>
            <div class="input-row">
              <input v-model="editAgentPath" class="input-field" />
              <button class="btn-secondary" @click="browseForAgentPath">Browse</button>
            </div>
          </div>

          <div class="modal-actions">
            <button class="btn-secondary" @click="editingAgent = null">Cancel</button>
            <button class="btn-primary" :disabled="savingAgentPath" @click="saveAgentPath">
              {{ savingAgentPath ? "Saving..." : "Save Path" }}
            </button>
          </div>
        </div>
      </div>

      <section class="settings-section">
        <div class="section-header">
          <Globe :size="20" class="section-icon" />
          <h2>Marketplace Sources</h2>
        </div>
        <p class="section-desc">Manage the repositories where you discover new skills.</p>

        <div class="sources-list">
          <div
            v-for="source in marketplaceStore.sources"
            :key="source.id"
            class="source-card"
            :class="{ disabled: !source.enabled }"
          >
            <div class="source-info">
              <div class="source-name-row">
                <span class="source-name">{{ source.name }}</span>
                <span v-if="source.official" class="official-badge">Official</span>
                <span v-if="source.source_type === 'api'" class="api-badge">API</span>
                <span v-if="source.source_type === 'registry'" class="registry-badge"
                  >Registry</span
                >
              </div>
              <span class="source-url">{{ source.url }}</span>
            </div>
            <div class="source-actions">
              <!-- Enabled Toggle -->
              <label class="toggle-switch">
                <input
                  type="checkbox"
                  :checked="source.enabled"
                  @change="(e) => toggleSource(source.id, e)"
                />
                <span class="slider round"></span>
              </label>

              <button
                class="icon-btn delete"
                v-if="!source.official && source.source_type !== 'api'"
                @click="removeSource(source.id)"
              >
                <Trash2 :size="18" />
              </button>
            </div>
          </div>

          <div class="add-buttons">
            <button class="add-btn" @click="showAddRegistryModal = true">
              <Plus :size="18" />
              <span>Add Registry Source</span>
            </button>
            <button class="add-btn" @click="handleAddLocalSource">
              <Folder :size="18" />
              <span>Add Local Skill Folder</span>
            </button>
          </div>
        </div>
      </section>

      <!-- Add Registry Modal -->
      <div v-if="showAddRegistryModal" class="modal-overlay">
        <div class="modal-content">
          <h3>Add Registry Source</h3>
          <p>Add a remote JSON registry to discover skills.</p>

          <div class="form-group">
            <label>Registry Name</label>
            <input v-model="registryName" placeholder="e.g. My Team Skills" class="input-field" />
          </div>

          <div class="form-group">
            <label>Registry JSON URL</label>
            <input v-model="registryUrl" placeholder="https://..." class="input-field" />
          </div>

          <div class="modal-actions">
            <button class="btn-secondary" @click="showAddRegistryModal = false">Cancel</button>
            <button
              class="btn-primary"
              :disabled="!registryName || !registryUrl || addingRegistry"
              @click="saveRegistrySource"
            >
              {{ addingRegistry ? "Adding..." : "Add Source" }}
            </button>
          </div>
        </div>
      </div>

      <section class="settings-section">
        <div class="section-header">
          <Folder :size="20" class="section-icon" />
          <h2>Managed Projects</h2>
        </div>
        <p class="section-desc">
          Add the local directories where your projects are located to manage their skills.
        </p>

        <div class="sources-list">
          <div v-for="project in projectStore.projects" :key="project.id" class="source-card">
            <div class="source-info">
              <div class="source-name-row">
                <span class="source-name">{{ project.name }}</span>
              </div>
              <span class="source-url">{{ project.path }}</span>
            </div>
            <div class="source-actions">
              <button class="icon-btn delete" @click="projectStore.removeProject(project.id!)">
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

      <!-- Cache Management Section -->
      <section class="settings-section">
        <div class="section-header">
          <Trash2 :size="20" class="section-icon" />
          <h2>Cache Management</h2>
        </div>
        <p class="section-desc">
          Clear cached marketplace data to force a fresh fetch of all skills.
        </p>

        <div class="cache-actions">
          <button class="btn-warning" @click="handleClearCache" :disabled="clearingCache">
            <Trash2 :size="16" />
            {{ clearingCache ? "Clearing..." : "Clear Skills Cache" }}
          </button>
          <span class="cache-hint"
            >This will clear all cached skill data from marketplace sources.</span
          >
        </div>
      </section>
    </div>
    <!-- Data Management -->
    <section class="settings-section">
      <div class="section-header">
        <Download :size="20" class="section-icon" />
        <h2>Data & Storage</h2>
      </div>
      <p class="section-desc">Manage your application data and configuration.</p>

      <div class="data-actions">
        <button class="action-btn" @click="handleExportConfig">
          <Download :size="16" />
          <span>Export Configuration</span>
        </button>
        <button class="action-btn" @click="handleImportConfig">
          <Upload :size="16" />
          <span>Import Configuration</span>
        </button>
      </div>
    </section>
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

/* Theme Selector */
.theme-selector {
  display: flex;
  gap: 12px;
  background-color: var(--bg-secondary);
  padding: 6px;
  border-radius: var(--border-radius);
  border: 1px solid var(--border-color);
  width: fit-content;
}

.theme-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-secondary);
  transition: all 0.2s;
}

.theme-btn:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

.data-actions {
  display: flex;
  gap: 12px;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  color: var(--text-primary);
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s;
}

.action-btn:hover {
  background-color: var(--bg-tertiary);
  border-color: var(--accent-primary);
}

.theme-btn.active {
  background-color: var(--bg-tertiary);
  color: var(--accent-primary);
  box-shadow: var(--shadow-sm);
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

.registry-badge {
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  font-weight: 700;
  color: var(--accent-info);
  background-color: rgba(59, 130, 246, 0.1);
  padding: 2px 6px;
  border-radius: 4px;
}

/* Toggle Switch */
.toggle-switch {
  position: relative;
  display: inline-block;
  width: 40px;
  height: 20px;
  margin-right: 8px;
}

.toggle-switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--bg-tertiary);
  transition: 0.4s;
  border-radius: 20px;
  border: 1px solid var(--border-color);
}

.slider:before {
  position: absolute;
  content: "";
  height: 14px;
  width: 14px;
  left: 3px;
  bottom: 2px;
  background-color: var(--text-muted);
  transition: 0.4s;
  border-radius: 50%;
}

input:checked + .slider {
  background-color: var(--accent-primary);
  border-color: var(--accent-primary);
}

input:checked + .slider:before {
  transform: translateX(20px);
  background-color: white;
}

/* Modal Styles */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.modal-content {
  background-color: var(--bg-primary);
  padding: 24px;
  border-radius: var(--border-radius);
  width: 400px;
  box-shadow: var(--shadow-lg);
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.modal-content h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-group label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-secondary);
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 8px;
}

.btn-secondary {
  padding: 8px 16px;
  background-color: var(--bg-tertiary);
  color: var(--text-primary);
  border-radius: var(--border-radius);
  font-size: 14px;
}

.btn-secondary:hover {
  background-color: var(--bg-hover);
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

.source-card.disabled {
  opacity: 0.7;
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

.add-buttons {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
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
  flex: 1;
  min-width: 200px;
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

.cache-actions {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.btn-warning {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 10px 20px;
  background-color: transparent;
  color: #f59e0b;
  border: 1px solid #f59e0b;
  border-radius: var(--border-radius);
  font-weight: 500;
  width: fit-content;
  transition: all 0.2s;
}

.btn-warning:hover:not(:disabled) {
  background-color: rgba(245, 158, 11, 0.1);
}

.btn-warning:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.cache-hint {
  font-size: 12px;
  color: var(--text-muted);
}

.edit-agent {
  opacity: 0;
  transition: opacity 0.2s;
  margin-left: 12px;
}

.agent-card:hover .edit-agent {
  opacity: 1;
}

.input-row {
  display: flex;
  gap: 8px;
}

.delete-agent {
  opacity: 0;
  transition: opacity 0.2s;
  color: var(--text-muted);
}

.delete-agent:hover {
  color: var(--accent-error);
  background-color: rgba(239, 68, 68, 0.1);
}

.agent-card:hover .delete-agent {
  opacity: 1;
}

.agents-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: 12px;
}

/* Icon Selector Styles */
.icon-selector-wrap {
  display: flex;
  flex-direction: column;
  gap: 12px;
  background-color: var(--bg-tertiary);
  padding: 12px;
  border-radius: 8px;
  border: 1px solid var(--border-color);
}

.icon-type-toggle {
  display: flex;
  gap: 4px;
  background-color: var(--bg-secondary);
  padding: 4px;
  border-radius: 6px;
  width: fit-content;
}

.toggle-btn {
  padding: 4px 12px;
  border-radius: 4px;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-secondary);
  transition: all 0.2s;
}

.toggle-btn:hover {
  color: var(--text-primary);
}

.toggle-btn.active {
  background-color: var(--bg-tertiary);
  color: var(--accent-primary);
  box-shadow: var(--shadow-sm);
}

.icon-input-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.icon-preview {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  overflow: hidden;
}

.icon-input {
  flex: 1;
}
</style>
