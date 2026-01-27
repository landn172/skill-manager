<script setup lang="ts">
import { computed } from "vue";
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
} from "lucide-vue-next";
import type { SearchMode } from "@/types";

const emit = defineEmits<{
  (e: "discover", url: string): void;
}>();

const store = useMarketplaceStore();

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
</script>

<template>
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
}

.search-row {
  display: flex;
  gap: 12px;
}

.search-wrap {
  flex: 1;
  position: relative;
}

.discovery-suggestion {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  z-index: 10;
  margin-top: 8px;
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
