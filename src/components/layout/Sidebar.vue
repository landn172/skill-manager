<script setup lang="ts">
import { useRoute } from "vue-router";
import { ShoppingBag, Box, Settings, PlusCircle, ChevronDown } from "lucide-vue-next";
import { useProjectStore } from "@/stores/project";
import { useSkillsStore } from "@/stores/skills";
import { onMounted, ref } from "vue";
import { getVersion } from "@tauri-apps/api/app";

const route = useRoute();
const projectStore = useProjectStore();
const skillsStore = useSkillsStore();
const appVersion = ref("...");

onMounted(async () => {
  projectStore.fetchProjects();
  try {
    appVersion.value = await getVersion();
  } catch (e) {
    appVersion.value = "?.?.?";
  }
});

const handleProjectChange = (projectId: string) => {
  if (projectId === "global") {
    projectStore.setCurrentProject(null);
    skillsStore.fetchInstalledSkills("global");
  } else {
    const project = projectStore.projects.find((p) => p.id?.toString() === projectId);
    if (project) {
      projectStore.setCurrentProject(project);
      skillsStore.fetchInstalledSkills("project");
    }
  }
};

const navItems = [
  { name: "Market", path: "/marketplace", icon: ShoppingBag },
  { name: "Installed", path: "/installed", icon: Box },
  { name: "Create", path: "/create", icon: PlusCircle },
  { name: "Settings", path: "/settings", icon: Settings },
];
</script>

<template>
  <aside class="sidebar glass">
    <div class="logo">
      <img src="/logo.png" alt="Skill Manager Logo" class="logo-img" />
      <span class="logo-text">Skill Manager</span>
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
            <option v-for="project in projectStore.projects" :key="project.id" :value="project.id">
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
        <div class="icon-wrap">
          <component :is="item.icon" :size="20" />
        </div>
        <span>{{ item.name }}</span>
      </router-link>
    </nav>

    <div class="footer">
      <div class="version">Version {{ appVersion }}</div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: var(--sidebar-width);
  height: 100vh;
  display: flex;
  flex-direction: column;
  padding: 32px 16px;
  z-index: 10;
  border-right: 1px solid var(--glass-border);
}

.logo {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0 8px 40px;
}

.logo-img {
  width: 32px;
  height: 32px;
  object-fit: contain;
}

.logo-text {
  font-size: 20px;
  font-weight: 800;
  letter-spacing: -0.03em;
  background: var(--accent-gradient);
  background-clip: text;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.project-switcher {
  margin-bottom: 32px;
}

.switcher-label {
  font-size: 11px;
  font-weight: 800;
  color: var(--text-muted);
  letter-spacing: 0.1em;
  margin-bottom: 12px;
  padding-left: 8px;
}

.select-wrapper {
  position: relative;
}

select {
  width: 100%;
  appearance: none;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--glass-border);
  border-radius: 12px;
  padding: 10px 36px 10px 14px;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  cursor: pointer;
  transition: all 0.2s;
}

select:hover {
  background: rgba(255, 255, 255, 0.06);
  border-color: var(--glass-border);
}

.select-icon {
  position: absolute;
  right: 14px;
  top: 50%;
  transform: translateY(-50%);
  pointer-events: none;
  color: var(--text-muted);
}

.nav {
  display: flex;
  flex-direction: column;
  gap: 8px;
  flex: 1;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 12px 14px;
  border-radius: 12px;
  color: var(--text-secondary);
  font-weight: 500;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  border: 1px solid transparent;
}

.icon-wrap {
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.nav-item:hover {
  background: rgba(255, 255, 255, 0.05);
  color: var(--text-primary);
  transform: translateX(4px);
}

.nav-item:hover .icon-wrap {
  transform: scale(1.1) rotate(-5deg);
  color: var(--accent-primary);
}

.nav-item.active {
  background: rgba(139, 92, 246, 0.1);
  color: var(--accent-primary);
  border-color: rgba(139, 92, 246, 0.3);
  box-shadow: var(--accent-glow);
}

.nav-item.active .icon-wrap {
  color: var(--accent-primary);
  transform: scale(1.1);
  filter: drop-shadow(0 0 5px var(--accent-primary));
}

.footer {
  padding: 16px 8px 0;
  border-top: 1px solid var(--glass-border);
}

.version {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
</style>
