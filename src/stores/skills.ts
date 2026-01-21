import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import type { InstalledSkill } from "@/types";
import { useProjectStore } from "./project";

export const useSkillsStore = defineStore("skills", {
  state: () => ({
    installedSkills: [] as InstalledSkill[],
    loading: false,
    error: null as string | null,
    scope: "global" as "project" | "global",
  }),

  getters: {
    isInstalled: (state) => (skillName: string) => {
      return state.installedSkills.some((s) => s.name === skillName);
    },

    getSkillByName: (state) => (skillName: string) => {
      return state.installedSkills.find((s) => s.name === skillName);
    },
  },

  actions: {
    async fetchInstalledSkills(scope?: "project" | "global") {
      if (scope) this.scope = scope;

      this.loading = true;
      this.error = null;
      try {
        let project_path = null;
        if (this.scope === "project") {
          const projectStore = useProjectStore();
          project_path = projectStore.currentProject?.path;
        }

        this.installedSkills = await invoke("list_installed_skills", {
          scope: this.scope,
          projectPath: project_path,
        });
      } catch (e) {
        this.error = String(e);
        console.error("Failed to fetch installed skills", e);
      } finally {
        this.loading = false;
      }
    },

    async uninstallSkill(skillName: string, agentType: string) {
      try {
        await invoke("uninstall_skill", {
          skillName,
          agent: agentType,
          scope: this.scope,
        });
        await this.fetchInstalledSkills();
      } catch (e) {
        throw e;
      }
    },
  },
});
