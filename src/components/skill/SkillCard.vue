<script setup lang="ts">
import { computed } from 'vue'
import {
  Download,
  Trash2,
  Folder,
  PlusCircle,
  ExternalLink,
  Edit3,
} from 'lucide-vue-next'
import { invoke } from '@tauri-apps/api/core'
import { openUrl } from '@tauri-apps/plugin-opener'
import type { MarketplaceSkill, Skill } from '@/types'
import { useSkillsStore } from '@/stores/skills'

const props = defineProps<{
  skill: MarketplaceSkill | Skill
  showSource?: boolean
}>()

const emit = defineEmits<{
  (e: 'install', skill: Skill): void
  (e: 'uninstall', skillName: string): void
  (e: 'update', skill: Skill): void
  (e: 'delete', skill: Skill): void
  (e: 'edit', skill: Skill): void
}>()

const skillsStore = useSkillsStore()
const isInstalled = computed(() => skillsStore.isInstalled(props.skill.name))
const installedSkill = computed(() =>
  skillsStore.getSkillByName(props.skill.name),
)
const installedVersion = computed(() => installedSkill.value?.installed_version)

const hasUpdate = computed(() => {
  if (!isInstalled.value || !props.skill.version || !installedVersion.value)
    return false
  return props.skill.version !== installedVersion.value
})

// Get the external URL (GitHub or SkillsMP page)
const externalUrl = computed(() => {
  const skill = props.skill as MarketplaceSkill
  // Priority: repo_url (GitHub) > skillUrl (SkillsMP page) > metadata.repo_url
  return (
    skill.repo_url ||
    skill.metadata?.skillUrl ||
    skill.metadata?.repo_url ||
    null
  )
})

// Check if this is a local skill (from a Local source)
const isLocalSkill = computed(() => {
  const skill = props.skill as MarketplaceSkill
  const result =
    skill.source_id?.startsWith('custom_') ||
    skill.source_name?.toLowerCase() === 'local'
  console.log('isLocalSkill check:', {
    name: skill.name,
    source_id: skill.source_id,
    source_name: skill.source_name,
    result,
  })
  return result
})

// Open the installed skill's directory
const handleOpenFolder = async () => {
  try {
    // Use the installed skill's path, not the marketplace skill's path (which is a URL)
    const pathToOpen = installedSkill.value?.path
    if (pathToOpen) {
      await invoke('open_in_explorer', { path: pathToOpen })
    } else {
      alert('Could not find installed skill path')
    }
  } catch (e) {
    alert(`Failed to open folder: ${e}`)
  }
}

// Open external link (GitHub or SkillsMP page)
const handleOpenExternal = async () => {
  if (externalUrl.value) {
    try {
      await openUrl(externalUrl.value)
    } catch (e) {
      // Fallback to window.open
      window.open(externalUrl.value, '_blank')
    }
  }
}
</script>

<template>
  <div class="skill-card">
    <div class="card-header">
      <div class="title-row">
        <h3 class="name">{{ skill.name }}</h3>
        <span v-if="skill.version" class="version-badge"
          >v{{ skill.version }}</span
        >
      </div>
      <div v-if="showSource" class="source">
        {{ (skill as MarketplaceSkill).source_name }}
      </div>
    </div>

    <p class="description">{{ skill.description }}</p>

    <div class="card-footer">
      <div class="tags">
        <span
          v-for="tag in (skill as MarketplaceSkill).tags"
          :key="tag"
          class="tag"
        >
          {{ tag }}
        </span>
      </div>

      <div class="action-btns">
        <!-- Edit button for local skills -->
        <button
          v-if="isLocalSkill"
          class="icon-btn secondary"
          title="Edit skill"
          @click="emit('edit', skill)"
        >
          <Edit3 :size="16" />
        </button>

        <!-- Delete button for local skills -->
        <button
          v-if="isLocalSkill"
          class="icon-btn danger"
          title="Delete skill"
          @click="emit('delete', skill)"
        >
          <Trash2 :size="16" />
        </button>

        <!-- External link button -->
        <button
          v-if="externalUrl"
          class="icon-btn secondary"
          title="Open in browser"
          @click="handleOpenExternal"
        >
          <ExternalLink :size="16" />
        </button>

        <!-- Open installed folder button -->
        <button
          v-if="isInstalled"
          class="icon-btn secondary"
          title="Open install folder"
          @click="handleOpenFolder"
        >
          <Folder :size="16" />
        </button>

        <button
          v-if="isInstalled && hasUpdate"
          class="update-btn"
          @click="emit('update', skill)"
        >
          <PlusCircle :size="16" />
          <span>Update</span>
        </button>
        <button
          v-if="isInstalled"
          class="uninstall-btn"
          @click="emit('uninstall', skill.name)"
        >
          <Trash2 :size="16" />
          <span>Uninstall</span>
        </button>
        <button v-else class="install-btn" @click="emit('install', skill)">
          <Download :size="16" />
          <span>Install</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.skill-card {
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  padding: 20px;
  display: flex;
  flex-direction: column;
  transition: all 0.2s;
  height: 200px;
}

.skill-card:hover {
  border-color: var(--accent-primary);
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 12px;
}

.title-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.version-badge {
  font-size: 11px;
  background-color: var(--bg-tertiary);
  color: var(--text-muted);
  padding: 1px 6px;
  border-radius: 4px;
  border: 1px solid var(--border-color);
}

.name {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.source {
  font-size: 12px;
  color: var(--text-muted);
  background-color: var(--bg-tertiary);
  padding: 2px 8px;
  border-radius: 4px;
}

.description {
  font-size: 14px;
  color: var(--text-secondary);
  line-height: 1.5;
  margin: 0 0 16px;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
  flex: 1;
}

.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.tags {
  display: flex;
  gap: 8px;
}

.tag {
  font-size: 11px;
  color: var(--accent-secondary);
  background-color: rgba(99, 102, 241, 0.1);
  padding: 2px 6px;
  border-radius: 4px;
}

.install-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 16px;
  background-color: var(--accent-primary);
  color: white;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  transition: background-color 0.2s;
}

.install-btn:hover {
  background-color: var(--accent-secondary);
}

.uninstall-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 16px;
  background-color: var(--error, #ef4444);
  color: white;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  transition: background-color 0.2s;
}

.uninstall-btn:hover {
  background-color: #dc2626;
}

.update-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 16px;
  background-color: var(--warning, #f59e0b);
  color: white;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  transition: background-color 0.2s;
}

.update-btn:hover {
  background-color: #d97706;
}

.action-btns {
  display: flex;
  gap: 8px;
}

.icon-btn.secondary {
  padding: 6px 10px;
  background-color: var(--bg-tertiary);
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.icon-btn.secondary:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
  border-color: var(--accent-primary);
}

.icon-btn.danger {
  padding: 6px 10px;
  background-color: transparent;
  color: var(--accent-error, #ef4444);
  border: 1px solid var(--accent-error, #ef4444);
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.icon-btn.danger:hover {
  background-color: rgba(239, 68, 68, 0.1);
  color: #dc2626;
}
</style>
