<script setup lang="ts">
import { onMounted, computed, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { useMarketplaceStore } from "@/stores/marketplace";
import SkillCard from "@/components/skill/SkillCard.vue";
import SearchInput from "@/components/common/SearchInput.vue";
import Modal from "@/components/common/Modal.vue";
import AgentIcon from "@/components/icons/AgentIcon.vue";
import {
  RefreshCw,
  Filter,
  CheckCircle2,
  Sparkles,
  Search,
  AlertCircle,
  ArrowUpDown,
  Tag,
  X,
} from "lucide-vue-next";
import { useAgentsStore } from "@/stores/agents";
import { useSkillsStore } from "@/stores/skills";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Skill, SearchMode } from "@/types";

const store = useMarketplaceStore();
const agentsStore = useAgentsStore();
const skillsStore = useSkillsStore();
const router = useRouter();

const showInstallModal = ref(false);
const selectedSkill = ref<Skill | null>(null);
const selectedAgents = ref<string[]>([]);

const installScope = ref<"project" | "global">("global");
const installing = ref(false);
const installLogs = ref<
  Array<{ time: string; message: string; type: "info" | "error" | "success" }>
>([]);
const searchDebounceTimer = ref<ReturnType<typeof setTimeout> | null>(null);

// Edit modal state
const showEditModal = ref(false);
const editingSkill = ref<Skill | null>(null);
const editForm = ref({ name: "", description: "" });

onMounted(() => {
  store.fetchSources();
  store.fetchSkills();
  store.fetchCachedSkills();
  agentsStore.fetchAgents();
  skillsStore.fetchInstalledSkills();
});

const skills = computed(() => store.filteredSkills);
const isSkillsmpSelected = computed(() => store.selectedSource === "skillsmp");
const showNoApiKeyWarning = computed(() => isSkillsmpSelected.value && !store.hasApiKey);

// Debounced search for API
watch(
  () => store.searchQuery,
  (query) => {
    if (!isSkillsmpSelected.value) return;

    if (searchDebounceTimer.value) {
      clearTimeout(searchDebounceTimer.value);
    }

    searchDebounceTimer.value = setTimeout(() => {
      store.searchSkillsmp(query);
    }, 500);
  },
);

async function handleRefresh() {
  await store.fetchSkills(store.selectedSource || undefined, true);
}

function toggleSearchMode() {
  const newMode: SearchMode = store.searchMode === "keyword" ? "ai" : "keyword";
  store.setSearchMode(newMode);
  // Re-search if there's an existing query
  if (store.searchQuery && isSkillsmpSelected.value) {
    store.searchSkillsmp(store.searchQuery);
  }
}

function openInstallModal(skill: Skill) {
  installLogs.value = [];
  selectedSkill.value = skill;
  selectedAgents.value = agentsStore.agents.filter((a) => a.installed).map((a) => a.agent_type);
  showInstallModal.value = true;
  console.log("Opening install modal for skill:", skill, "Available agents:", agentsStore.agents);
}

function getLogTime() {
  return new Date().toLocaleTimeString("en-US", {
    hour12: false,
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

async function handleInstall() {
  if (!selectedSkill.value || selectedAgents.value.length === 0) return;

  installing.value = true;
  installLogs.value = [];
  installLogs.value.push({
    time: getLogTime(),
    message: "Initializing installation...",
    type: "info",
  });

  // Set up event listener
  let unlisten: (() => void) | undefined;

  try {
    unlisten = await listen<{
      skill: string;
      status: string;
      message: string;
      agent?: string;
    }>("install-progress", (event) => {
      console.log("Install progress:", event.payload);
      const type =
        event.payload.status === "error"
          ? "error"
          : event.payload.status === "finished"
            ? "success"
            : "info";
      installLogs.value.push({
        time: getLogTime(),
        message: event.payload.message,
        type,
      });

      // Auto-scroll to bottom of terminal
      const terminal = document.getElementById("install-terminal");
      if (terminal) {
        setTimeout(() => {
          terminal.scrollTop = terminal.scrollHeight;
        }, 10);
      }
    });

    // Filter out any agents that are not installed (detected) to prevent
    // installing to disabled/greyed-out agents
    const installedAgentTypes = agentsStore.agents
      .filter((a) => a.installed)
      .map((a) => a.agent_type);
    const validAgents = selectedAgents.value.filter((agent) => installedAgentTypes.includes(agent));

    if (validAgents.length === 0) {
      installLogs.value.push({
        time: getLogTime(),
        message: "No valid agents selected for installation.",
        type: "error",
      });
      installing.value = false;
      return;
    }

    console.log(
      "Installing skill:",
      selectedSkill.value,
      "to agents:",
      validAgents,
      "scope:",
      installScope.value,
    );

    const results = await invoke<
      Array<{
        success: boolean;
        path: string;
        agent: string;
        error?: string;
      }>
    >("install_skill", {
      skill: selectedSkill.value,
      agents: validAgents,
      scope: installScope.value,
    });

    console.log("Install results:", results);

    const successful = results.filter((r) => r.success);
    const failed = results.filter((r) => !r.success);

    if (failed.length > 0) {
      const errors = failed.map((f) => `${f.agent}: ${f.error}`).join("\n");
      installLogs.value.push({
        time: getLogTime(),
        message: `Some installations failed: ${errors}`,
        type: "error",
      });
      if (successful.length === 0) {
        throw new Error(`All installations failed:\n${errors}`);
      }
    }

    if (successful.length > 0) {
      installLogs.value.push({
        time: getLogTime(),
        message: "Installation completed successfully.",
        type: "success",
      });

      // Short delay to let user see success before closing
      await new Promise((resolve) => setTimeout(resolve, 1500));

      showInstallModal.value = false;
      skillsStore.fetchInstalledSkills();
    }
  } catch (e) {
    console.error("Installation failed:", e);
    installLogs.value.push({
      time: getLogTime(),
      message: `Installation failed: ${e}`,
      type: "error",
    });
    // Don't alert here, let the logs show it
  } finally {
    if (unlisten) unlisten();
    installing.value = false;
  }
}

async function handleUninstall(skillName: string) {
  if (!confirm(`Are you sure you want to uninstall ${skillName}?`)) return;

  try {
    // Uninstall from all agents that have it installed
    const installedSkill = skillsStore.getSkillByName(skillName);
    if (installedSkill) {
      for (const agent of installedSkill.agents) {
        await skillsStore.uninstallSkill(skillName, agent);
      }
    } else {
      // Fallback: try to uninstall from all agents with global scope
      for (const agent of agentsStore.agents.filter((a) => a.installed)) {
        try {
          await invoke("uninstall_skill", {
            skillName,
            agent: agent.agent_type,
            scope: "global",
          });
        } catch (e) {
          // Ignore errors for agents that don't have this skill
        }
      }
    }
    skillsStore.fetchInstalledSkills();
    alert("Skill uninstalled successfully!");
  } catch (e) {
    alert(`Failed to uninstall: ${e}`);
  }
}

async function handleDeleteLocalSkill(skill: Skill) {
  if (
    !confirm(
      `Are you sure you want to DELETE "${skill.name}"? This will permanently remove the skill files from disk.`,
    )
  )
    return;

  try {
    await invoke("delete_local_skill", { skillPath: skill.path });
    alert("Skill deleted successfully!");
    // Refresh marketplace to remove deleted skill
    await store.fetchSkills(undefined, true);
  } catch (e) {
    alert(`Failed to delete skill: ${e}`);
  }
}

async function handleEditLocalSkill(skill: Skill) {
  console.log("handleEditLocalSkill called with:", skill);
  // Navigate to Create page in edit mode
  router.push({
    path: "/create",
    query: {
      edit: "true",
      name: skill.name,
      description: skill.description || "",
      path: skill.path,
    },
  });
}

async function submitEditSkill() {
  if (!editingSkill.value) return;

  try {
    await invoke("update_local_skill", {
      skillPath: editingSkill.value.path,
      name: editForm.value.name || undefined,
      description: editForm.value.description || undefined,
    });
    showEditModal.value = false;
    editingSkill.value = null;
    alert("Skill updated successfully!");
    // Refresh marketplace to show updated metadata
    await store.fetchSkills(undefined, true);
  } catch (e) {
    alert(`Failed to update skill: ${e}`);
  }
}
</script>

<template>
  <div class="marketplace-page">
    <header class="header">
      <div class="title-row">
        <h1>Marketplace</h1>
        <button class="icon-btn" @click="handleRefresh" :disabled="store.loading">
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
          <span>{{ store.searchMode === "ai" ? "AI Search" : "Keyword" }}</span>
        </button>

        <div class="control-group">
          <!-- Sort -->
          <div class="filter-dropdown">
            <ArrowUpDown :size="16" />
            <select v-model="store.sortBy">
              <option value="name">Name</option>
              <option value="stars">Stars</option>
            </select>
          </div>

          <!-- Tag Filter -->
          <div class="filter-dropdown">
            <Tag :size="16" />
            <select
              :value="''"
              @change="
                (e) => {
                  const target = e.target as HTMLSelectElement;
                  if (target.value && !store.selectedTags.includes(target.value)) {
                    store.selectedTags.push(target.value);
                  }
                  target.value = '';
                }
              "
            >
              <option value="" disabled selected>Filter by Tag</option>
              <option v-for="tag in store.availableTags" :key="tag" :value="tag">
                {{ tag }}
              </option>
            </select>
          </div>

          <!-- Source Filter -->
          <div class="filter-dropdown">
            <Filter :size="16" />
            <select v-model="store.selectedSource">
              <option :value="null">All Sources</option>
              <option v-for="source in store.sources" :key="source.id" :value="source.id">
                {{ source.name }}
              </option>
            </select>
          </div>
        </div>
      </div>

      <!-- Active Tags -->
      <div v-if="store.selectedTags.length > 0" class="active-tags">
        <div
          v-for="tag in store.selectedTags"
          :key="tag"
          class="tag-chip"
          @click="store.selectedTags = store.selectedTags.filter((t) => t !== tag)"
        >
          <span>{{ tag }}</span>
          <X :size="12" />
        </div>
        <button class="clear-tags" @click="store.selectedTags = []">Clear filters</button>
      </div>

      <!-- API Key Warning -->
      <div v-if="showNoApiKeyWarning" class="api-warning">
        <AlertCircle :size="16" />
        <span>No API key configured.</span>
        <router-link to="/settings">Add your SkillsMP API key in Settings</router-link>
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
            >({{ store.fetchProgress.current }}/{{ store.fetchProgress.total }})</span
          >
        </template>
        <template v-else> Fetching skills from marketplace... </template>
      </p>
      <div v-if="store.fetchProgress.total > 0" class="progress-bar">
        <div
          class="progress-fill"
          :style="{
            width: `${(store.fetchProgress.current / store.fetchProgress.total) * 100}%`,
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
        @delete="handleDeleteLocalSkill"
        @edit="handleEditLocalSkill"
      />
    </div>

    <!-- Installation Modal -->
    <Modal
      :show="showInstallModal"
      :title="`Install ${selectedSkill?.name}`"
      maxWidth="800px"
      @close="showInstallModal = false"
    >
      <div class="modal-content-grid">
        <div class="install-form">
          <p class="form-help">Select the agents you want to install this skill to.</p>

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
                  ? (selectedAgents = selectedAgents.filter((a) => a !== agent.agent_type))
                  : selectedAgents.push(agent.agent_type))
              "
            >
              <AgentIcon
                :type="agentsStore.getIcon(agent.agent_type)"
                :size="18"
                class="agent-icon"
              />
              <div class="agent-name">{{ agent.display_name }}</div>
              <div class="check-wrap" v-if="selectedAgents.includes(agent.agent_type)">
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
          {{ installing ? "Installing..." : "Install Skill" }}
        </button>
      </template>
    </Modal>

    <!-- Edit Skill Modal -->
    <Modal
      :show="showEditModal"
      :title="`Edit ${editingSkill?.name}`"
      @close="showEditModal = false"
    >
      <div class="edit-form">
        <div class="input-group">
          <label>Skill Name</label>
          <input v-model="editForm.name" placeholder="Skill name" />
        </div>
        <div class="input-group">
          <label>Description</label>
          <textarea
            v-model="editForm.description"
            placeholder="Skill description"
            rows="4"
          ></textarea>
        </div>
      </div>

      <template #footer>
        <button class="footer-btn secondary" @click="showEditModal = false">Cancel</button>
        <button class="footer-btn primary" @click="submitEditSkill">Save Changes</button>
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
  background: linear-gradient(135deg, rgba(139, 92, 246, 0.1), rgba(168, 85, 247, 0.15));
  border-color: var(--accent-primary);
  color: var(--accent-primary);
}

.control-group {
  display: flex;
  gap: 12px;
  align-items: center;
}

.filter-dropdown {
  display: flex;
  align-items: center;
  gap: 8px;
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  padding: 0 12px;
  height: 40px;
  min-width: 140px;
}

select {
  background: none;
  border: none;
  outline: none;
  font-size: 13px;
  color: var(--text-primary);
  width: 100%;
  cursor: pointer;
}

.active-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: -16px;
  /* Pull closer to header */
  margin-bottom: 8px;
}

.tag-chip {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  background-color: rgba(139, 92, 246, 0.1);
  border: 1px solid rgba(139, 92, 246, 0.2);
  color: var(--accent-primary);
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.tag-chip:hover {
  background-color: rgba(139, 92, 246, 0.2);
}

.clear-tags {
  font-size: 12px;
  color: var(--text-muted);
  background: none;
  border: none;
  cursor: pointer;
  padding: 4px 8px;
}

.clear-tags:hover {
  text-decoration: underline;
  color: var(--text-secondary);
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
  background: linear-gradient(90deg, var(--accent-primary), var(--accent-secondary));
  border-radius: 3px;
  transition: width 0.3s ease;
}

/* Previous styles... */
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
  content: "";
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
  font-family: "JetBrains Mono", "Fira Code", "SF Mono", "Roboto Mono", "Menlo", monospace;
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
