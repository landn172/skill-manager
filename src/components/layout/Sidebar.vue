<script setup lang="ts">
import { useRoute } from 'vue-router'
import {
  ShoppingBag,
  Box,
  Settings,
  PlusCircle,
  ChevronDown,
} from 'lucide-vue-next'
import { useProjectStore } from '@/stores/project'
import { useSkillsStore } from '@/stores/skills'
import { onMounted } from 'vue'

const route = useRoute()
const projectStore = useProjectStore()
const skillsStore = useSkillsStore()

onMounted(() => {
  projectStore.fetchProjects()
})

const handleProjectChange = (projectId: string) => {
  if (projectId === 'global') {
    projectStore.setCurrentProject(null)
    skillsStore.fetchInstalledSkills('global')
  } else {
    const project = projectStore.projects.find(
      (p) => p.id?.toString() === projectId
    )
    if (project) {
      projectStore.setCurrentProject(project)
      skillsStore.fetchInstalledSkills('project')
    }
  }
}

const navItems = [
  { name: 'Market', path: '/marketplace', icon: ShoppingBag },
  { name: 'Installed', path: '/installed', icon: Box },
  { name: 'Create', path: '/create', icon: PlusCircle },
  { name: 'Settings', path: '/settings', icon: Settings },
]
</script>

<template>
  <aside class="sidebar">
    <div class="logo">
      <img src="/logo.png" alt="Skill Manager Logo" class="logo-img" />
    </div>

    <div class="project-switcher">
      <div class="switcher-label">SCOPE</div>
      <div class="select-wrapper">
        <select
          :value="projectStore.currentProject?.id || 'global'"
          @change="(e) => handleProjectChange((e.target as HTMLSelectElement).value)"
        >
          <option value="global">🌐 Global Scope</option>
          <optgroup v-if="projectStore.projects.length" label="Projects">
            <option
              v-for="project in projectStore.projects"
              :key="project.id"
              :value="project.id"
            >
              📁 {{ project.name }}
            </option>
          </optgroup>
        </select>
        <ChevronDown :size="14" class="select-icon" />
      </div>
    </div>

    <nav class="nav">
      <router-link
        v-for="item in navItems"
        :key="item.path"
        :to="item.path"
        class="nav-item"
        :class="{ active: route.path === item.path }"
      >
        <component :is="item.icon" :size="20" />
        <span>{{ item.name }}</span>
      </router-link>
    </nav>

    <div class="footer">
      <div class="version">v1.0.0</div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: var(--sidebar-width);
  background-color: var(--bg-secondary);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  padding: 24px 12px;
}

.logo {
  display: flex;
  justify-content: center;
  padding: 0 12px 32px;
}

.logo-img {
  width: 48px;
  height: 48px;
  object-fit: contain;
}

.logo .text {
  font-weight: 700;
  font-size: 18px;
  background: linear-gradient(
    135deg,
    var(--accent-primary),
    var(--accent-secondary)
  );
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.project-switcher {
  margin: 0 12px 24px;
}

.switcher-label {
  font-size: 10px;
  font-weight: 700;
  color: var(--text-muted);
  letter-spacing: 0.05em;
  margin-bottom: 8px;
  padding-left: 4px;
}

.select-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

select {
  width: 100%;
  appearance: none;
  background-color: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 8px 32px 8px 12px;
  font-size: 13px;
  color: var(--text-primary);
  cursor: pointer;
  transition: all 0.2s;
}

select:hover {
  border-color: var(--accent-primary);
}

select:focus {
  outline: none;
  border-color: var(--accent-primary);
  box-shadow: 0 0 0 2px rgba(139, 92, 246, 0.2);
}

.select-icon {
  position: absolute;
  right: 12px;
  pointer-events: none;
  color: var(--text-muted);
}

.nav {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: var(--border-radius);
  color: var(--text-secondary);
  transition: all 0.2s;
}

.nav-item:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

.nav-item.active {
  background-color: var(--bg-tertiary);
  color: var(--accent-primary);
  font-weight: 500;
}

.footer {
  padding: 12px;
  border-top: 1px solid var(--border-color);
}

.version {
  font-size: 12px;
  color: var(--text-muted);
}
</style>
