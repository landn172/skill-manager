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
  Edit2,
} from "lucide-vue-next";
import { useThemeStore } from "@/stores/theme";
import AgentIcon from "@/components/icons/AgentIcon.vue";
import { useProjectStore } from "@/stores/project";
import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import { homeDir } from "@tauri-apps/api/path";
import { invoke } from "@tauri-apps/api/core";
import { writeTextFile, readTextFile, readFile } from "@tauri-apps/plugin-fs";
import PageHeader from "@/components/common/PageHeader.vue";
import BaseButton from "@/components/common/BaseButton.vue";
import Modal from "@/components/common/Modal.vue";

const agentsStore = useAgentsStore();
const marketplaceStore = useMarketplaceStore();
const projectStore = useProjectStore();
const themeStore = useThemeStore();

// API Key management
const apiKeyInput = ref("");
const maskedApiKey = ref<string | null>(null);
const apiKeySource = ref<string | null>(null); // 'env' or 'db'
const savingApiKey = ref(false);
const isEditingApiKey = ref(false);

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

async function startEditingApiKey() {
  try {
    const fullKey = await invoke<string | null>("get_skillsmp_api_key");
    apiKeyInput.value = fullKey || "";
    isEditingApiKey.value = true;
  } catch (e) {
    console.error("Failed to fetch full API key", e);
    isEditingApiKey.value = true;
  }
}

async function saveApiKey() {
  if (!apiKeyInput.value.trim()) return;

  savingApiKey.value = true;
  try {
    await invoke("set_skillsmp_api_key", { key: apiKeyInput.value.trim() });
    apiKeyInput.value = "";
    isEditingApiKey.value = false;
    await loadApiKey();
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
    await loadApiKey();
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
    await invoke<string>("clear_cache");
    await marketplaceStore.fetchSources();
  } catch (e) {
    alert(`Failed to clear cache: ${e}`);
  } finally {
    clearingCache.value = false;
  }
}

async function handleOpenUrl(url: string) {
  try {
    await openUrl(url);
  } catch (e) {
    console.error("Failed to open URL", e);
    window.open(url, "_blank");
  }
}

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

async function handleAddLocalSource() {
  try {
    const selected = await openDialog({
      directory: true,
      multiple: false,
      defaultPath: await homeDir(),
    });

    if (selected && typeof selected === "string") {
      const name = selected.split("/").pop() || "Local Skills";
      await marketplaceStore.addSource(selected, name, "local");
      await marketplaceStore.fetchSkills();
    }
  } catch (e) {
    alert(`Failed to add local source: ${e}`);
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

    await agentsStore.fetchAgents();
    editingAgent.value = null;
    editAgentPath.value = "";
  } catch (e) {
    alert(`Failed to update agent path: ${e}`);
  } finally {
    savingAgentPath.value = false;
  }
}

async function browseForAgentPath() {
  try {
    const selected = await openDialog({
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
    const selected = await openDialog({
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
    const selected = await openDialog({
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
  <div class="settings-page animate-fade-in">
    <PageHeader title="Settings" description="Customize application behavior and manage configurations." />

    <div class="sections">
      <!-- Appearance Section -->
      <section class="section">
        <div class="section-title">
          <Palette :size="20" class="icon" />
          <h2>App Appearance</h2>
        </div>
        <p class="section-hint">Choose between light, dark, or system-preferred theme.</p>

        <div class="theme-picker glass">
          <button
            class="theme-opt"
            :class="{ active: themeStore.theme === 'light' }"
            @click="themeStore.setTheme('light')"
          >
            <Sun :size="16" />
            <span>Light</span>
          </button>
          <button
            class="theme-opt"
            :class="{ active: themeStore.theme === 'dark' }"
            @click="themeStore.setTheme('dark')"
          >
            <Moon :size="16" />
            <span>Dark</span>
          </button>
          <button
            class="theme-opt"
            :class="{ active: themeStore.theme === 'system' }"
            @click="themeStore.setTheme('system')"
          >
            <Monitor :size="16" />
            <span>System</span>
          </button>
        </div>
      </section>

      <!-- API Configuration Section -->
      <section class="section">
        <div class="section-title">
          <Key :size="20" class="icon" />
          <h2>SkillsMP API</h2>
        </div>
        <p class="section-hint">
          Configure your SkillsMP API key to discover over 65,000 skills.
        </p>

        <div class="glass-card api-box">
          <div v-if="hasApiKey && !isEditingApiKey" class="api-status">
            <div class="key-container">
              <span class="label">Current API Key</span>
              <div class="key-wrap">
                <code class="key-value">{{ maskedApiKey }}</code>
                <span :class="isFromEnv ? 'env-badge' : 'db-badge'">
                  {{ isFromEnv ? "via .env" : "via Database" }}
                </span>
              </div>
            </div>
            
            <div class="api-actions">
              <BaseButton v-if="!isFromEnv" variant="outline" size="sm" @click="startEditingApiKey">
                <Edit2 :size="14" />
                Change Key
              </BaseButton>
              <BaseButton v-if="!isFromEnv" variant="danger" size="sm" @click="clearApiKey">
                <Trash2 :size="14" />
                Remove
              </BaseButton>
            </div>
          </div>

          <div v-else class="api-input-wrap">
            <div class="input-header" v-if="isEditingApiKey">
              <span class="label">Edit API Key</span>
              <button class="cancel-link" @click="isEditingApiKey = false">Cancel</button>
            </div>
            <div class="input-row">
              <input
                v-model="apiKeyInput"
                type="password"
                placeholder="Paste your sk_live_... key here"
                class="styled-input"
              />
              <BaseButton
                variant="primary"
                :disabled="!apiKeyInput.trim() || savingApiKey"
                :loading="savingApiKey"
                @click="saveApiKey"
              >
                {{ isEditingApiKey ? 'Update Key' : 'Save Key' }}
              </BaseButton>
            </div>
          </div>

          <a href="#" class="external-link" @click.prevent="handleOpenUrl('https://skillsmp.com/docs/api')">
            <ExternalLink :size="12" />
            Get your API key at skillsmp.com
          </a>
        </div>
      </section>

      <!-- Agents Section -->
      <section class="section">
        <div class="section-title">
          <ShieldCheck :size="20" class="icon" />
          <h2>Detected Agents</h2>
        </div>
        <p class="section-hint">Manage the IDEs and tools where skills can be installed.</p>

        <div class="list-container">
          <div v-for="agent in agentsStore.agents" :key="agent.name" class="item-card glass-card">
            <div class="item-info">
              <div class="item-icon-box">
                <AgentIcon :type="agentsStore.getIcon(agent.agent_type)" :size="20" />
              </div>
              <div class="item-text">
                <span class="item-name">{{ agent.display_name }}</span>
                <span class="item-subtext">{{ agent.global_skills_dir }}</span>
              </div>
            </div>
            
            <div class="item-actions">
              <div class="status-indicator" :class="{ detected: agent.installed }">
                <CheckCircle2 v-if="agent.installed" :size="14" />
                <XCircle v-else :size="14" />
                <span>{{ agent.installed ? 'Detected' : 'Missing' }}</span>
              </div>
              
              <BaseButton variant="ghost" size="icon" @click="openEditAgent(agent)" title="Edit Directory">
                <Edit2 :size="16" />
              </BaseButton>
              
              <BaseButton
                v-if="agent.is_custom"
                variant="ghost"
                size="icon"
                class="danger-ghost"
                @click="removeCustomAgent(agent)"
                title="Remove Custom Agent"
              >
                <Trash2 :size="16" />
              </BaseButton>
            </div>
          </div>
          
          <BaseButton variant="outline" class="w-full dashed" @click="showAddAgentModal = true">
            <Plus :size="18" />
            Add Custom Agent
          </BaseButton>
        </div>
      </section>

      <!-- Marketplace Sources Section -->
      <section class="section">
        <div class="section-title">
          <Globe :size="20" class="icon" />
          <h2>Marketplace Sources</h2>
        </div>
        <p class="section-hint">Repositories where you find and update skills.</p>

        <div class="list-container">
          <div
            v-for="source in marketplaceStore.sources"
            :key="source.id"
            class="item-card glass-card"
            :class="{ disabled: !source.enabled }"
          >
            <div class="item-info">
              <div class="item-text">
                <div class="item-header-row">
                  <span class="item-name">{{ source.name }}</span>
                  <span v-if="source.official" class="badge official">Official</span>
                  <span v-if="source.source_type === 'api'" class="badge api">API</span>
                  <span v-if="source.source_type === 'registry'" class="badge registry">Registry</span>
                </div>
                <span class="item-subtext">{{ source.url }}</span>
              </div>
            </div>
            
            <div class="item-actions">
              <label class="switch">
                <input
                  type="checkbox"
                  :checked="source.enabled"
                  @change="(e) => toggleSource(source.id, e)"
                />
                <span class="toggle-slider"></span>
              </label>

              <BaseButton
                v-if="!source.official && source.source_type !== 'api'"
                variant="ghost"
                size="icon"
                class="danger-ghost"
                @click="removeSource(source.id)"
              >
                <Trash2 :size="16" />
              </BaseButton>
            </div>
          </div>

          <div class="two-cols">
            <BaseButton variant="outline" class="dashed" @click="showAddRegistryModal = true">
              <Plus :size="18" />
              Add Registry
            </BaseButton>
            <BaseButton variant="outline" class="dashed" @click="handleAddLocalSource">
              <Folder :size="18" />
              Add Local Folder
            </BaseButton>
          </div>
        </div>
      </section>

      <!-- Managed Projects Section -->
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

      <!-- Cache & Storage Section -->
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
    </div>

    <!-- Modals -->
    <Modal
      :show="showAddAgentModal"
      title="Add Custom Agent"
      @close="showAddAgentModal = false"
    >
      <div class="modal-form">
        <div class="form-item">
          <label>Agent Name</label>
          <input v-model="newAgentName" placeholder="e.g. Cursor IDE" class="styled-input" />
        </div>

        <div class="form-item">
          <label>Icon</label>
          <div class="icon-picker-box">
            <div class="picker-tabs">
              <button :class="{ active: newAgentIconType === 'emoji' }" @click="newAgentIconType = 'emoji'">Emoji</button>
              <button :class="{ active: newAgentIconType === 'image' }" @click="newAgentIconType = 'image'">Image</button>
            </div>
            <div class="picker-field">
              <div class="preview-box">
                <AgentIcon :type="newAgentIcon" :size="24" />
              </div>
              <input v-if="newAgentIconType === 'emoji'" v-model="newAgentIcon" class="styled-input" maxlength="2" />
              <BaseButton v-else variant="outline" size="sm" @click="pickIconImage">
                {{ newAgentIcon.startsWith('data:') ? 'Change' : 'Select' }} Image
              </BaseButton>
            </div>
          </div>
        </div>

        <div class="form-item">
          <label>Skills Path</label>
          <div class="row">
            <input v-model="newAgentPath" class="styled-input" placeholder="/path/to/global/skills" />
            <BaseButton variant="outline" size="md" @click="browseForNewAgentPath">Browse</BaseButton>
          </div>
        </div>
      </div>
      <template #footer>
        <BaseButton variant="ghost" @click="showAddAgentModal = false">Cancel</BaseButton>
        <BaseButton variant="primary" :disabled="!newAgentName || !newAgentPath" :loading="addingAgent" @click="addCustomAgent">Add Agent</BaseButton>
      </template>
    </Modal>

    <Modal
      :show="!!editingAgent"
      :title="`Edit ${editingAgent?.display_name} Path`"
      @close="editingAgent = null"
    >
      <div class="modal-form">
        <div class="form-item">
          <label>Skills Directory Path</label>
          <div class="row">
            <input v-model="editAgentPath" class="styled-input" />
            <BaseButton variant="outline" size="md" @click="browseForAgentPath">Browse</BaseButton>
          </div>
        </div>
      </div>
      <template #footer>
        <BaseButton variant="ghost" @click="editingAgent = null">Cancel</BaseButton>
        <BaseButton variant="primary" :loading="savingAgentPath" @click="saveAgentPath">Save Path</BaseButton>
      </template>
    </Modal>

    <Modal
      :show="showAddRegistryModal"
      title="Add Registry Source"
      @close="showAddRegistryModal = false"
    >
      <div class="modal-form">
        <div class="form-item">
          <label>Registry Name</label>
          <input v-model="registryName" placeholder="e.g. Team Registry" class="styled-input" />
        </div>
        <div class="form-item">
          <label>Registry JSON URL</label>
          <input v-model="registryUrl" placeholder="https://example.com/registry.json" class="styled-input" />
        </div>
      </div>
      <template #footer>
        <BaseButton variant="ghost" @click="showAddRegistryModal = false">Cancel</BaseButton>
        <BaseButton variant="primary" :disabled="!registryName || !registryUrl" :loading="addingRegistry" @click="saveRegistrySource">Add Source</BaseButton>
      </template>
    </Modal>
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
  gap: 32px;
  max-width: 800px;
  padding-bottom: 40px;
}

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

/* Theme Picker */
.theme-picker {
  display: flex;
  padding: 4px;
  border-radius: 12px;
  width: fit-content;
}

.theme-opt {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 20px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  transition: all 0.2s;
}

.theme-opt:hover {
  color: var(--text-primary);
  background: var(--bg-hover);
}

.theme-opt.active {
  background: var(--bg-primary);
  color: var(--accent-primary);
  box-shadow: var(--shadow-sm);
}

/* API Box */
.api-box {
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.api-status {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
}

.api-actions {
  display: flex;
  gap: 8px;
}

.key-container {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.key-container .label {
  font-size: 12px;
  font-weight: 700;
  text-transform: uppercase;
  color: var(--text-muted);
}

.key-wrap {
  display: flex;
  align-items: center;
  gap: 12px;
}

.key-value {
  font-family: var(--font-mono);
  font-size: 14px;
  background: var(--bg-tertiary);
  padding: 4px 10px;
  border-radius: 6px;
  color: var(--text-primary);
}

.env-badge, .db-badge {
  font-size: 10px;
  font-weight: 700;
  padding: 2px 8px;
  border-radius: 4px;
}

.env-badge {
  background: rgba(34, 197, 94, 0.1);
  color: var(--accent-success);
}

.db-badge {
  background: rgba(139, 92, 246, 0.1);
  color: var(--accent-primary);
}

.api-input-wrap {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.input-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.input-header .label {
  font-size: 12px;
  font-weight: 700;
  text-transform: uppercase;
  color: var(--text-muted);
}

.cancel-link {
  font-size: 12px;
  color: var(--text-muted);
  cursor: pointer;
  background: none;
  border: none;
  padding: 0;
  text-decoration: underline;
}

.cancel-link:hover {
  color: var(--text-primary);
}

.input-row {
  display: flex;
  gap: 12px;
}

.external-link {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--accent-primary);
  font-weight: 600;
  width: fit-content;
}

.external-link:hover {
  text-decoration: underline;
}

/* List Items */
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
  transition: all 0.2s;
}

.item-card:hover {
  border-color: var(--accent-primary);
}

.item-card.disabled {
  opacity: 0.6;
}

.item-info {
  display: flex;
  align-items: center;
  gap: 16px;
}

.item-icon-box {
  width: 40px;
  height: 40px;
  background: var(--bg-tertiary);
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.item-text {
  display: flex;
  flex-direction: column;
}

.item-header-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.item-name {
  font-weight: 700;
  font-size: 15px;
}

.item-subtext {
  font-size: 12px;
  color: var(--text-muted);
}

.item-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  background: var(--bg-tertiary);
  padding: 4px 10px;
  border-radius: 20px;
  color: var(--text-muted);
}

.status-indicator.detected {
  background: rgba(34, 197, 94, 0.1);
  color: var(--accent-success);
}

.badge {
  font-size: 10px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: 4px;
}

.badge.official { background: rgba(139, 92, 246, 0.1); color: var(--accent-primary); }
.badge.api { background: rgba(34, 197, 94, 0.1); color: var(--accent-success); }
.badge.registry { background: rgba(59, 130, 246, 0.1); color: var(--accent-info); }

.two-cols {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.dashed {
  border-style: dashed;
}

/* Maintenance */
.maintenance-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.maintenance-card {
  padding: 20px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  gap: 16px;
}

.maintenance-card .title {
  display: block;
  font-weight: 700;
  font-size: 15px;
  margin-bottom: 4px;
}

.maintenance-card .hint {
  font-size: 12px;
  color: var(--text-muted);
}

.maintenance-card .btns {
  display: flex;
  gap: 8px;
}

/* Form Elements */
.styled-input {
  flex: 1;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 10px 16px;
  color: var(--text-primary);
  font-size: 14px;
  transition: all 0.2s;
}

.styled-input:focus {
  outline: none;
  border-color: var(--accent-primary);
  background: var(--bg-primary);
}

.modal-form {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.form-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-item label {
  font-size: 13px;
  font-weight: 700;
  color: var(--text-secondary);
}

.form-item .row {
  display: flex;
  gap: 8px;
}

.icon-picker-box {
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 12px;
}

.picker-tabs {
  display: flex;
  gap: 4px;
  margin-bottom: 12px;
}

.picker-tabs button {
  padding: 4px 12px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
}

.picker-tabs button.active {
  background: var(--bg-primary);
  color: var(--accent-primary);
}

.picker-field {
  display: flex;
  align-items: center;
  gap: 12px;
}

.preview-box {
  width: 44px;
  height: 44px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* Toggle Switch */
.switch {
  position: relative;
  display: inline-block;
  width: 40px;
  height: 20px;
}

.switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--bg-tertiary);
  transition: 0.3s;
  border-radius: 20px;
  border: 1px solid var(--border-color);
}

.toggle-slider:before {
  position: absolute;
  content: "";
  height: 14px;
  width: 14px;
  left: 2px;
  bottom: 2px;
  background-color: var(--text-muted);
  transition: 0.3s;
  border-radius: 50%;
}

input:checked + .toggle-slider {
  background-color: var(--accent-primary);
  border-color: var(--accent-primary);
}

input:checked + .toggle-slider:before {
  transform: translateX(20px);
  background-color: white;
}

.danger-ghost:hover {
  color: var(--accent-error) !important;
  background: rgba(239, 68, 68, 0.1) !important;
}

.w-full {
  width: 100%;
}
</style>
