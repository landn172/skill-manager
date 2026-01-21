<script setup lang="ts">
import { Pencil, Trash2 } from 'lucide-vue-next'
import { invoke } from '@tauri-apps/api/core'
import type { InstalledSkill } from '@/types'
import AgentIcon from '@/components/icons/AgentIcon.vue'
import { useAgentsStore } from '@/stores/agents'

const agentsStore = useAgentsStore()

defineProps<{
  skills: InstalledSkill[]
}>()

const emit = defineEmits<{
  (e: 'edit', skill: InstalledSkill): void
  (e: 'uninstall', skill: InstalledSkill): void
}>()

const handleOpenInAgent = async (skill: InstalledSkill, agent: string) => {
  const path = skill.agent_paths?.[agent] || skill.path
  if (!path) return
  try {
    await invoke('open_in_agent', { path, agent })
  } catch (e) {
    alert(`Failed to open in agent: ${e}`)
  }
}
</script>

<template>
  <div class="skill-list">
    <div v-for="skill in skills" :key="skill.name" class="skill-row">
      <div class="skill-main">
        <h3 class="skill-name">{{ skill.name }}</h3>
        <p class="skill-desc">{{ skill.description }}</p>

        <div class="installed-agents">
          <div
            v-for="agent in skill.agents"
            :key="agent"
            class="agent-badge clickable"
            :title="`Open in ${agent}`"
            @click.stop="handleOpenInAgent(skill, agent)"
          >
            <AgentIcon :type="agentsStore.getIcon(agent)" :size="16" />
          </div>
        </div>
      </div>

      <div class="skill-actions">
        <button
          class="action-btn edit"
          @click="emit('edit', skill)"
          title="Edit SKILL.md"
        >
          <Pencil :size="18" />
        </button>
        <button
          class="action-btn delete"
          @click="emit('uninstall', skill)"
          title="Uninstall"
        >
          <Trash2 :size="18" />
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.skill-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.skill-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  padding: 16px 20px;
  transition: border-color 0.2s;
}

.skill-row:hover {
  border-color: var(--accent-primary);
}

.skill-main {
  flex: 1;
}

.skill-name {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}

.skill-desc {
  margin: 4px 0 12px;
  font-size: 14px;
  color: var(--text-secondary);
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.installed-agents {
  display: flex;
  gap: 6px;
}

.agent-badge {
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--bg-tertiary);
  border-radius: 4px;
  border: 1px solid var(--border-color);
  padding: 4px;
  color: var(--text-secondary);
}

.agent-badge.clickable {
  cursor: pointer;
  transition: all 0.2s;
}

.agent-badge.clickable:hover {
  background-color: var(--bg-hover);
  border-color: var(--accent-primary);
  color: var(--accent-primary);
}

.skill-actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  padding: 8px;
  border-radius: 8px;
  color: var(--text-muted);
  transition: all 0.2s;
}

.action-btn:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

.action-btn.delete:hover {
  color: var(--accent-error);
  background-color: rgba(239, 68, 68, 0.1);
}

.action-btn.edit:hover {
  color: var(--accent-primary);
  background-color: rgba(139, 92, 246, 0.1);
}
</style>
