<script setup lang="ts">
import { computed } from "vue";
import {
  Download,
  Trash2,
  Folder,
  ExternalLink,
  Edit3,
  Database,
  RefreshCw,
} from "lucide-vue-next";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { MarketplaceSkill, Skill } from "@/types";
import AgentIcon from "@/components/icons/AgentIcon.vue";
import { useSkillsStore } from "@/stores/skills";
import { useAgentsStore } from "@/stores/agents";
import { useMarketplaceStore } from "@/stores/marketplace";
import BaseButton from "@/components/common/BaseButton.vue";

const agentsStore = useAgentsStore();

const props = defineProps<{
  skill: MarketplaceSkill | Skill;
  showSource?: boolean;
}>();

const emit = defineEmits<{
  (e: "install", skill: Skill): void;
  (e: "uninstall", skillName: string): void;
  (e: "update", skill: Skill): void;
  (e: "delete", skill: Skill): void;
  (e: "edit", skill: Skill): void;
}>();

const skillsStore = useSkillsStore();
const isInstalled = computed(() => skillsStore.isInstalled(props.skill.name));
const installedSkill = computed(() => skillsStore.getSkillByName(props.skill.name));
const installedVersion = computed(() => installedSkill.value?.installed_version);

const installedAgents = computed(() => installedSkill.value?.agents || []);

const hasUpdate = computed(() => {
  if (!isInstalled.value || !props.skill.version || !installedVersion.value) return false;
  return props.skill.version !== installedVersion.value;
});

const externalUrl = computed(() => {
  const skill = props.skill as MarketplaceSkill;
  return skill.repo_url || skill.metadata?.skillUrl || skill.metadata?.repo_url || null;
});

const isLocalSkill = computed(() => {
  const skill = props.skill as MarketplaceSkill;
  return skill.source_id?.startsWith("custom_") || skill.source_name?.toLowerCase() === "local";
});

const canInstallMore = computed(() => {
  if (!isInstalled.value) return true;
  const allInstalledAgents = agentsStore.agents.filter((a) => a.installed);
  const skillAgents = installedAgents.value;
  return allInstalledAgents.length > skillAgents.length;
});

const handleOpenFolder = async () => {
  try {
    const pathToOpen = installedSkill.value?.path;
    if (pathToOpen) {
      await invoke("open_in_explorer", { path: pathToOpen });
    }
  } catch (e) {
    console.error(e);
  }
};

const handleOpenExternal = async () => {
  if (externalUrl.value) {
    try {
      await openUrl(externalUrl.value);
    } catch (e) {
      window.open(externalUrl.value, "_blank");
    }
  }
};

const handleOpenInAgent = async (agent: string) => {
  const path = installedSkill.value?.agent_paths?.[agent] || installedSkill.value?.path;
  if (!path) return;
  try {
    await invoke("open_in_agent", { path, agent });
  } catch (e) {
    console.error(e);
  }
};

const marketplaceStore = useMarketplaceStore();
const isCached = computed(() => marketplaceStore.isSkillCached(props.skill.name));
const cachedAt = computed(() => {
  const dateStr = marketplaceStore.getCachedAt(props.skill.name);
  if (!dateStr) return null;
  return new Date(dateStr).toLocaleDateString();
});

const handleClearCache = async () => {
  if (confirm(`Clear cache for ${props.skill.name}?`)) {
    await marketplaceStore.clearCache(props.skill.name);
  }
};
</script>

<template>
  <div class="skill-card glass-card">
    <div class="card-main">
      <div class="header">
        <div class="title-area">
          <h3 class="name">{{ skill.name }}</h3>
          <span v-if="skill.version" class="version">v{{ skill.version }}</span>
        </div>
      </div>

      <div v-if="isInstalled && installedAgents.length > 0" class="agents-row">
        <div
          v-for="agent in installedAgents"
          :key="agent"
          class="agent-pill"
          :title="`Open in ${agent}`"
          @click.stop="handleOpenInAgent(agent)"
        >
          <AgentIcon :type="agentsStore.getIcon(agent)" :size="10" />
          <span class="agent-name-tip">{{ agent }}</span>
        </div>
      </div>

      <div class="meta-row">
        <span v-if="showSource" class="source-tag">
          {{ (skill as MarketplaceSkill).source_name }}
        </span>
        <div v-if="isCached" class="cached-tag" :title="`Cached on ${cachedAt}`">
          <Database :size="10" />
          <span>Cached</span>
        </div>
      </div>

      <p class="description">{{ skill.description }}</p>

      <div class="tag-cloud">
        <span v-for="tag in (skill as MarketplaceSkill).tags" :key="tag" class="skill-tag">
          #{{ tag }}
        </span>
      </div>
    </div>

    <div class="card-actions">
      <div class="secondary-actions">
        <BaseButton v-if="isCached" variant="ghost" size="icon" title="Clear Cache" @click="handleClearCache">
          <RefreshCw :size="16" />
        </BaseButton>
        <BaseButton v-if="isLocalSkill" variant="ghost" size="icon" title="Edit" @click="emit('edit', skill)">
          <Edit3 :size="16" />
        </BaseButton>
        <BaseButton v-if="isLocalSkill" variant="danger" size="icon" title="Delete" @click="emit('delete', skill)">
          <Trash2 :size="16" />
        </BaseButton>
        <BaseButton v-if="externalUrl" variant="ghost" size="icon" title="View Source" @click="handleOpenExternal">
          <ExternalLink :size="16" />
        </BaseButton>
        <BaseButton v-if="isInstalled" variant="ghost" size="icon" title="Open Folder" @click="handleOpenFolder">
          <Folder :size="16" />
        </BaseButton>
      </div>

      <div class="primary-actions">
        <BaseButton v-if="isInstalled && hasUpdate" variant="primary" size="sm" @click="emit('update', skill)">
          Update
        </BaseButton>
        <BaseButton v-if="isInstalled && canInstallMore" variant="outline" size="sm" @click="emit('install', skill)">
          Add Agent
        </BaseButton>
        <BaseButton v-if="isInstalled" variant="ghost" size="icon" class="delete-btn" @click="emit('uninstall', skill.name)">
          <Trash2 :size="16" />
        </BaseButton>
        <BaseButton v-if="!isInstalled" variant="primary" size="sm" @click="emit('install', skill)">
          <Download :size="16" />
          Install
        </BaseButton>
      </div>
    </div>
  </div>
</template>

<style scoped>
.skill-card {
  display: flex;
  flex-direction: column;
  padding: 20px;
  min-height: 240px;
}

.card-main {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 8px;
}

.title-area {
  display: flex;
  align-items: baseline;
  gap: 8px;
}

.name {
  margin: 0;
  font-size: 16px;
  font-weight: 700;
  color: var(--text-primary);
}

.version {
  font-size: 11px;
  font-weight: 700;
  color: var(--text-muted);
  background: var(--bg-tertiary);
  padding: 1px 6px;
  border-radius: 6px;
}

.agents-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 8px;
}

.agent-pill {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
  flex-shrink: 0;
}

.agent-pill:hover {
  border-color: var(--accent-primary);
  background: var(--bg-hover);
}

.agent-name-tip {
  font-size: 10px;
  font-weight: 600;
  color: var(--text-secondary);
}

.meta-row {
  display: flex;
  gap: 12px;
  margin-bottom: 8px;
}

.source-tag {
  font-size: 11px;
  font-weight: 700;
  color: var(--accent-primary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.cached-tag {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 10px;
  color: var(--accent-success);
}

.description {
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.5;
  margin: 0 0 12px;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.tag-cloud {
  display: flex;
  gap: 8px;
  overflow: hidden;
  margin-top: auto;
}

.skill-tag {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-muted);
}

.card-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--glass-border);
}

.secondary-actions {
  display: flex;
  gap: 4px;
}

.primary-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.delete-btn {
  color: var(--text-muted);
}

.delete-btn:hover {
  color: var(--accent-error);
  background: rgba(239, 68, 68, 0.1);
}
</style>
