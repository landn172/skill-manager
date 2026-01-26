<script setup lang="ts">
import { onMounted, computed, ref, watch } from "vue";
import { useMarketplaceStore } from "@/stores/marketplace";
import SkillCard from "@/components/skill/SkillCard.vue";
import SearchInput from "@/components/common/SearchInput.vue";
import Modal from "@/components/common/Modal.vue";
import PageHeader from "@/components/common/PageHeader.vue";
import InstallModal from "@/components/skill/InstallModal.vue";
import BaseButton from "@/components/common/BaseButton.vue";
import {
  RefreshCw,
  Filter,
  Sparkles,
  Search,
  AlertCircle,
  ArrowUpDown,
} from "lucide-vue-next";
import { useSkillsStore } from "@/stores/skills";
import type { Skill, SearchMode } from "@/types";

const store = useMarketplaceStore();
const skillsStore = useSkillsStore();

const showInstallModal = ref(false);
const selectedSkill = ref<Skill | null>(null);
const searchDebounceTimer = ref<ReturnType<typeof setTimeout> | null>(null);

// URL Discovery state
const showUrlDiscoveryModal = ref(false);
const discoveryUrl = ref("");
const discoveredSkills = ref<import("@/types").MarketplaceSkill[]>([]);
const discovering = ref(false);
const discoveryError = ref<string | null>(null);

onMounted(() => {
  store.fetchSources();
  store.fetchSkills();
  store.fetchCachedSkills();
  skillsStore.fetchInstalledSkills();
});

const skills = computed(() => store.filteredSkills);
const isSkillsmpSelected = computed(() => store.selectedSource === "skillsmp");
const showNoApiKeyWarning = computed(() => isSkillsmpSelected.value && !store.hasApiKey);

// Debounced search for API
watch(
  () => store.searchQuery,
  (query) => {
    const shouldSearchSkillsmp = isSkillsmpSelected.value || (store.selectedSource === null && query.trim().length > 0);
    if (!shouldSearchSkillsmp) return;

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
  if (store.searchQuery && isSkillsmpSelected.value) {
    store.searchSkillsmp(store.searchQuery);
  }
}

function openInstallModal(skill: Skill) {
  selectedSkill.value = skill;
  showInstallModal.value = true;
}

async function handleUninstall(skillName: string) {
  if (!confirm(`Are you sure you want to uninstall ${skillName}?`)) return;
  try {
    const installedSkill = skillsStore.getSkillByName(skillName);
    if (installedSkill) {
      for (const agent of installedSkill.agents) {
        await skillsStore.uninstallSkill(skillName, agent);
      }
    }
    skillsStore.fetchInstalledSkills();
  } catch (e) {
    alert(`Failed to uninstall: ${e}`);
  }
}

async function handleDiscoveryFromUrl() {
  if (!discoveryUrl.value.trim()) return;
  discovering.value = true;
  discoveryError.value = null;
  discoveredSkills.value = [];
  try {
    const results = await store.discoverFromUrl(discoveryUrl.value.trim());
    discoveredSkills.value = results;
    if (results.length === 0) {
      discoveryError.value = "No skills found at this URL.";
    }
  } catch (e) {
    discoveryError.value = String(e);
  } finally {
    discovering.value = false;
  }
}

function openDiscoveryModal() {
  discoveryUrl.value = "";
  discoveredSkills.value = [];
  discoveryError.value = null;
  showUrlDiscoveryModal.value = true;
}

function onInstallSuccess() {
  skillsStore.fetchInstalledSkills();
}
</script>

<template>
  <div class="marketplace-page animate-fade-in">
    <PageHeader title="Marketplace" description="Discover and install new skills for your agents.">
      <template #actions>
        <BaseButton variant="outline" @click="openDiscoveryModal">
          <Filter :size="16" />
          <span>Install from URL</span>
        </BaseButton>
        <BaseButton variant="ghost" size="icon" @click="handleRefresh" :disabled="store.loading">
          <RefreshCw :size="20" :class="{ spinning: store.loading }" />
        </BaseButton>
      </template>
    </PageHeader>

    <div class="filters-container glass">
      <div class="search-row">
        <div class="search-wrap">
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
        </div>

        <BaseButton
          v-if="isSkillsmpSelected"
          :variant="store.searchMode === 'ai' ? 'primary' : 'secondary'"
          size="md"
          @click="toggleSearchMode"
        >
          <Sparkles v-if="store.searchMode === 'ai'" :size="16" />
          <Search v-else :size="16" />
          <span>{{ store.searchMode === "ai" ? "AI Search" : "Keyword" }}</span>
        </BaseButton>
      </div>

      <div class="controls-row">
        <div class="filter-group">
          <div class="filter-item">
            <ArrowUpDown :size="16" class="icon" />
            <select v-model="store.sortBy">
              <option value="name">Sort by Name</option>
              <option value="stars">Sort by Stars</option>
            </select>
          </div>

          <div class="filter-item">
            <Filter :size="16" class="icon" />
            <select v-model="store.selectedSource">
              <option :value="null">All Sources</option>
              <option v-for="source in store.sources" :key="source.id" :value="source.id">
                {{ source.name }}
              </option>
            </select>
          </div>
        </div>

        <div v-if="showNoApiKeyWarning" class="api-warning">
          <AlertCircle :size="14" />
          <span>No API key configured. <router-link to="/settings">Add in Settings</router-link></span>
        </div>
      </div>
    </div>

    <!-- Main Content -->
    <div class="main-content">
      <div v-if="store.loading && skills.length === 0" class="loading-state">
        <div class="loader"></div>
        <div class="progress-info">
          <p v-if="store.fetchProgress.status === 'loading_sources'">Loading sources...</p>
          <p v-else-if="store.fetchProgress.status === 'fetching'">
            Fetching {{ store.fetchProgress.currentSource }}...
          </p>
          <div v-if="store.fetchProgress.total > 0" class="progress-bar-wrap">
            <div class="bar" :style="{ width: `${(store.fetchProgress.current / store.fetchProgress.total) * 100}%` }"></div>
          </div>
        </div>
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
    </div>

    <!-- Shared Installation Modal -->
    <InstallModal
      :show="showInstallModal"
      :skill="selectedSkill"
      @close="showInstallModal = false"
      @success="onInstallSuccess"
    />

    <!-- URL Discovery Modal -->
    <Modal
      :show="showUrlDiscoveryModal"
      title="Install from GitHub"
      maxWidth="600px"
      @close="showUrlDiscoveryModal = false"
    >
      <div class="discovery-box">
        <div class="input-row">
          <input
            v-model="discoveryUrl"
            placeholder="e.g. owner/repo or full URL"
            @keyup.enter="handleDiscoveryFromUrl"
            :disabled="discovering"
          />
          <BaseButton variant="primary" @click="handleDiscoveryFromUrl" :loading="discovering" :disabled="!discoveryUrl">
            Search
          </BaseButton>
        </div>

        <div v-if="discoveryError" class="discovery-err">
          <AlertCircle :size="16" />
          <span>{{ discoveryError }}</span>
        </div>

        <div v-if="discoveredSkills.length > 0" class="results-list">
          <h3>Available Skills</h3>
          <div class="results-scroll">
            <div v-for="skill in discoveredSkills" :key="skill.path" class="result-item glass-card">
              <div class="info">
                <span class="name">{{ skill.name }}</span>
                <span class="desc">{{ skill.description }}</span>
              </div>
              <BaseButton size="sm" variant="primary" @click="() => { showUrlDiscoveryModal = false; openInstallModal(skill); }">Install</BaseButton>
            </div>
          </div>
        </div>
      </div>
    </Modal>
  </div>
</template>

<style scoped>
.marketplace-page {
  padding: 20px;
  height: 100vh;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.filters-container {
  padding: 20px;
  border-radius: 12px;
  margin-bottom: 24px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.search-row {
  display: flex;
  gap: 12px;
}

.search-wrap {
  flex: 1;
}

.controls-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.filter-group {
  display: flex;
  gap: 12px;
}

.filter-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 12px;
  height: 40px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  min-width: 160px;
}

.filter-item .icon {
  color: var(--text-muted);
}

.filter-item select {
  background: none;
  border: none;
  font-size: 13px;
  width: 100%;
}

.api-warning {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--accent-warning);
  background: rgba(245, 158, 11, 0.1);
  padding: 8px 16px;
  border-radius: 8px;
  border: 1px solid rgba(245, 158, 11, 0.2);
}

.api-warning a {
  text-decoration: underline;
  font-weight: 600;
}

.main-content {
  flex: 1;
}

.skills-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 20px;
  padding-bottom: 32px;
}

/* Loading/Empty States */
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 24px;
  padding: 80px 0;
}

.loader {
  width: 40px;
  height: 40px;
  border: 3px solid var(--border-color);
  border-top-color: var(--accent-primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

.progress-info {
  text-align: center;
  color: var(--text-secondary);
  font-size: 14px;
}

.progress-bar-wrap {
  width: 200px;
  height: 4px;
  background: var(--bg-tertiary);
  border-radius: 2px;
  margin-top: 12px;
  overflow: hidden;
}

.progress-bar-wrap .bar {
  height: 100%;
  background: var(--accent-gradient);
  transition: width 0.3s ease;
}

.error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
  padding: 40px;
  text-align: center;
  max-width: 400px;
  margin: 40px auto;
}

/* Modal Discovery */
.discovery-box {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.input-row {
  display: flex;
  gap: 12px;
}

.input-row input {
  flex: 1;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  padding: 12px 16px;
  border-radius: 10px;
}

.results-list h3 {
  font-size: 14px;
  margin-bottom: 12px;
}

.result-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  margin-bottom: 8px;
}

.result-item .info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.result-item .name { font-weight: 700; }
.result-item .desc { font-size: 12px; color: var(--text-secondary); }


.title-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.btn-secondary {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  font-size: 14px;
  color: var(--text-primary);
  cursor: pointer;
  transition: all 0.2s;
}

.btn-secondary:hover:not(:disabled) {
  background-color: var(--bg-hover);
  border-color: var(--accent-primary);
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

.discovery-form {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.url-input-row {
  display: flex;
  gap: 12px;
}

.url-input-row input {
  flex: 1;
  padding: 10px 16px;
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  color: var(--text-primary);
  outline: none;
}

.url-input-row input:focus {
  border-color: var(--accent-primary);
}

.url-input-row button {
  padding: 0 24px;
  background-color: var(--accent-primary);
  color: white;
  border: none;
  border-radius: var(--border-radius);
  font-weight: 600;
  cursor: pointer;
}

.url-input-row button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.discovery-error {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  background-color: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: var(--border-radius);
  color: var(--accent-error);
  font-size: 13px;
}

.discovered-results h3 {
  font-size: 16px;
  margin-bottom: 12px;
}

.discovery-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  max-height: 400px;
  overflow-y: auto;
}

.discovered-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
}

.item-info {
  flex: 1;
}

.item-name {
  font-weight: 600;
  font-size: 14px;
  margin-bottom: 2px;
}

.item-desc {
  font-size: 12px;
  color: var(--text-muted);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.btn-sm {
  padding: 6px 12px;
  background-color: var(--accent-primary);
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
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
