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
})
