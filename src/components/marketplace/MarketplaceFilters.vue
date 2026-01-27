<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { useMarketplaceStore } from "@/stores/marketplace";
import SearchInput from "@/components/common/SearchInput.vue";
import BaseButton from "@/components/common/BaseButton.vue";
import {
  Filter,
  Sparkles,
  Search,
  AlertCircle,
  ArrowUpDown,
  Github,
  History,
} from "lucide-vue-next";
import type { SearchMode } from "@/types";

const emit = defineEmits<{
  (e: "discover", url: string): void;
}>();

const store = useMarketplaceStore();
const showHistory = ref(false);
const filtersRef = ref<HTMLElement | null>(null);

const isSkillsmpSelected = computed(() => store.selectedSource === "skillsmp");
const showNoApiKeyWarning = computed(() => isSkillsmpSelected.value && !store.hasApiKey);

// GitHub URL Detection
const isGithubUrl = computed(() => {
  const query = store.searchQuery.trim();
  // Match owner/repo or full github URL
  return /^[a-zA-Z0-9-]+\/[a-zA-Z0-9._-]+$/.test(query) || 
         /^https?:\/\/github\.com\/[a-zA-Z0-9-]+\/[a-zA-Z0-9._-]+/.test(query);
});

function toggleSearchMode() {
  const newMode: SearchMode = store.searchMode === "keyword" ? "ai" : "keyword";
  store.setSearchMode(newMode);
  if (store.searchQuery && isSkillsmpSelected.value) {
    store.searchSkillsmp(store.searchQuery);
  }
}

function handleDiscovery() {
  if (isGithubUrl.value) {
    emit("discover", store.searchQuery.trim());
  }
}

function selectHistory(item: string) {
  store.searchQuery = item;
  showHistory.value = false;
  if (isSkillsmpSelected.value) {
    store.searchSkillsmp(item);
  } else {
    // If not SkillsMP, we just update the query (which triggers local filter)
  }
}

function handleClickOutside(event: MouseEvent) {
  if (filtersRef.value && !filtersRef.value.contains(event.target as Node)) {
    showHistory.value = false;
  }
}

onMounted(() => {
  document.addEventListener("mousedown", handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener("mousedown", handleClickOutside);
});
</script>

<template>
  <div ref="filtersRef" class="filters-container glass">
    <div class="search-row">
      <div class="search-wrap">
        <SearchInput
          v-model="store.searchQuery"
          @focus="showHistory = true"
          :placeholder="
            isSkillsmpSelected
              ? store.searchMode === 'ai'
                ? 'Ask AI: e.g. skills for trading...'
                : 'Search 65,000+ skills...'
              : 'Search skills, descriptions...'
          "
        />
        
        <!-- Search History Dropdown -->
        <transition name="fade">
          <div v-if="showHistory && store.searchHistory.length > 0" class="history-dropdown glass">
            <div class="history-header">
              <span>Recent Searches</span>
              <button class="clear-btn" @click.stop="store.clearHistory()">Clear All</button>
            </div>
            <div class="history-list">
              <div
                v-for="item in store.searchHistory"
                :key="item"
                class="history-item"
                @click="selectHistory(item)"
              >
                <History :size="14" />
                <span>{{ item }}</span>
              </div>
            </div>
          </div>
        </transition>

        <!-- Quick Install from URL suggestion -->
        <div v-if="isGithubUrl" class="discovery-suggestion animate-fade-in">
          <Github :size="14" />
          <span>Looks like a GitHub repo. </span>
          <button @click="handleDiscovery">Directly Discover Skills</button>
        </div>
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
</template>

<style scoped>
.filters-container {
  padding: 20px;
  border-radius: 12px;
  margin-bottom: 24px;
  display: flex;
  flex-direction: column;
  gap: 20px;
  position: relative;
  z-index: 100;
}

.search-row {
  display: flex;
  gap: 12px;
  position: relative;
  z-index: 101;
}

.search-wrap {
  flex: 1;
  position: relative;
  z-index: 102;
}

.history-dropdown {
  position: absolute;
  top: calc(100% + 8px);
  left: 0;
  right: 0;
  z-index: 50;
  border-radius: 12px;
  max-height: 320px;
  overflow: hidden;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
  display: flex;
  flex-direction: column;
}

.history-header {
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 11px;
  font-weight: 700;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.1em;
}

.clear-btn {
  color: var(--accent-primary);
  cursor: pointer;
  text-transform: none;
  letter-spacing: 0;
  font-weight: 500;
}

.history-list {
  overflow-y: auto;
  padding: 8px 0;
}

.history-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 16px;
  cursor: pointer;
  transition: all 0.2s;
  color: var(--text-secondary);
  font-size: 14px;
}

.history-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
  padding-left: 20px;
}

.discovery-suggestion {
  position: absolute;
  top: calc(100% + 8px);
  left: 0;
  right: 0;
  z-index: 10;
  padding: 8px 16px;
  background: var(--bg-secondary);
  border: 1px solid var(--accent-primary);
  border-radius: 8px;
  font-size: 12px;
  display: flex;
  align-items: center;
  gap: 8px;
  box-shadow: var(--shadow-md);
}

/* Transitions */
.fade-enter-active,
.fade-leave-active {
  transition: all 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}

.discovery-suggestion button {
  color: var(--accent-primary);
  font-weight: 600;
  text-decoration: underline;
  padding: 0;
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
  outline: none;
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

/* Animation */
@keyframes fadeIn {
  from { opacity: 0; transform: translateY(-4px); }
  to { opacity: 1; transform: translateY(0); }
}

.animate-fade-in {
  animation: fadeIn 0.2s ease-out forwards;
}
</style>
