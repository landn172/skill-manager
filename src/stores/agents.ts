import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { AgentConfig } from '@/types'

export const useAgentsStore = defineStore('agents', {
  state: () => ({
    agents: [] as AgentConfig[],
    loading: false,
    error: null as string | null,
  }),

  actions: {
    async fetchAgents() {
      this.loading = true
      try {
        this.agents = await invoke('detect_agents')
      } catch (e) {
        this.error = String(e)
      } finally {
        this.loading = false
      }
    },
  },

  getters: {
    getIcon: (state) => (agentType: string) => {
      const agent = state.agents.find(
        (a) => a.agent_type === agentType || a.name === agentType,
      )
      if (agent && agent.is_custom) {
        return agent.icon
      }
      return agentType // For known agents, the type itself is the icon key (e.g. 'vscode')
    },
  },
})
