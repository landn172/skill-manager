<script setup lang="ts">
import { onMounted, onUnmounted, computed, ref, watch } from "vue";
import { useMarketplaceStore } from "@/stores/marketplace";
import { useSkillsStore } from "@/stores/skills";
import { useAgentsStore } from "@/stores/agents";
import SkillCard from "@/components/skill/SkillCard.vue";
import SkillCardSkeleton from "@/components/skill/SkillCardSkeleton.vue";
import PageHeader from "@/components/common/PageHeader.vue";
import InstallModal from "@/components/skill/InstallModal.vue";
import BaseButton from "@/components/common/BaseButton.vue";
import MarketplaceFilters from "@/components/marketplace/MarketplaceFilters.vue";
import DiscoveryModal from "@/components/marketplace/DiscoveryModal.vue";
import { RefreshCw, Filter } from "lucide-vue-next";
import type { Skill } from "@/types";

const store = useMarketplaceStore();
const skillsStore = useSkillsStore();
const agentsStore = useAgentsStore();

const showInstallModal = ref(false);
const showUrlDiscoveryModal = ref(false);
const selectedSkill = ref<Skill | null>(null);
const initialDiscoveryUrl = ref("");
const searchDebounceTimer = ref<ReturnType<typeof setTimeout> | null>(null);

const sentinel = ref<HTMLElement | null>(null);
let observer: IntersectionObserver | null = null;

onMounted(() => {
  store.fetchSources();
  store.fetchSkills();
  store.fetchCachedSkills();
  skillsStore.fetchInstalledSkills();
  agentsStore.fetchAgents();

  // Setup intersection observer for infinite scroll
  observer = new IntersectionObserver((entries) => {
    if (entries[0].isIntersecting && store.hasMore && !store.loading) {
      store.loadMoreSkillsmp();
    }
  }, { threshold: 0.1 });

  if (sentinel.value) {
    observer.observe(sentinel.value);
  }
});

watch(sentinel, (newVal) => {
  if (newVal && observer) {
    observer.observe(newVal);
  }
});

onUnmounted(() => {
  if (observer) {
    observer.disconnect();
  }
});

const skills = computed(() => store.filteredSkills);
const isSkillsmpSelected = computed(() => store.selectedSource === "skillsmp");

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
      if (shouldSearchSkillsmp) {
        store.searchSkillsmp(query);
      } else {
        store.addToHistory(query);
      }
    }, 500);
  },
);

async function handleRefresh() {
  await store.fetchSkills(store.selectedSource || undefined, true);
}

function openInstallModal(skill: Skill) {
  selectedSkill.value = skill;
  showInstallModal.value = true;
}

function handleDiscover(url: string) {
  initialDiscoveryUrl.value = url;
  showUrlDiscoveryModal.value = true;
}

function openDiscoveryModal() {
  initialDiscoveryUrl.value = "";
  showUrlDiscoveryModal.value = true;
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

    <MarketplaceFilters @discover="handleDiscover" />

    <!-- Main Content -->
    <div class="main-content">
      <div v-if="store.loading && skills.length === 0" class="skills-grid no-margin">
        <SkillCardSkeleton v-for="i in 8" :key="i" />
      </div>

      <div v-else-if="skills.length === 0" class="empty-state">
        <p>No skills found matching your search.</p>
      </div>

      <div v-else>
        <div class="skills-grid">
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

        <!-- Infinite Scroll Sentinel -->
        <div v-if="store.hasMore" ref="sentinel" class="scroll-sentinel">
          <div v-if="store.loading" class="skills-grid full-width">
            <SkillCardSkeleton v-for="i in 4" :key="i" />
          </div>
          <p v-else class="sentinel-text">Scroll for more...</p>
        </div>
      </div>
    </div>

    <!-- Installation Modal -->
    <InstallModal
      :show="showInstallModal"
      :skill="selectedSkill"
      @close="showInstallModal = false"
      @success="onInstallSuccess"
    />

    <!-- URL Discovery Modal -->
    <DiscoveryModal
      :show="showUrlDiscoveryModal"
      :initialUrl="initialDiscoveryUrl"
      @close="showUrlDiscoveryModal = false"
      @install="(skill) => { showUrlDiscoveryModal = false; openInstallModal(skill); }"
    />
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

.main-content {
  flex: 1;
}

.skills-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 24px;
  padding-bottom: 40px;
}

.load-more {
  display: flex;
  justify-content: center;
  padding: 20px 0 40px;
}

/* Loading/Empty States */
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 24px;
  padding: 80px 0;
}

.skills-grid.no-margin {
  padding-bottom: 0;
}

.skills-grid.full-width {
  width: 100%;
}

.scroll-sentinel {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 0 0 80px;
  min-height: 100px;
}

.sentinel-text {
  font-size: 13px;
  color: var(--text-muted);
  font-weight: 500;
  letter-spacing: 0.02em;
  padding: 40px 0;
}
</style>
