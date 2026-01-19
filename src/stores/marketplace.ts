import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { MarketplaceSource, MarketplaceSkill, SearchMode } from '@/types'

export const useMarketplaceStore = defineStore('marketplace', {
  state: () => ({
    sources: [] as MarketplaceSource[],
    skills: [] as MarketplaceSkill[],
    loading: false,
    error: null as string | null,
    searchQuery: '',
    selectedSource: null as string | null,
    searchMode: 'keyword' as SearchMode,
    hasApiKey: false,
    fetchProgress: {
      current: 0,
      total: 0,
      currentSource: '',
      status: '' as 'idle' | 'loading_sources' | 'fetching' | 'done',
    },
  }),

  getters: {
    filteredSkills(state) {
      let result = state.skills

      if (state.selectedSource) {
        result = result.filter((s) => s.source_id === state.selectedSource)
      }

      // Only filter locally if using keyword mode and not searching via API
      if (state.searchQuery && state.selectedSource !== 'skillsmp') {
        const query = state.searchQuery.toLowerCase()
        result = result.filter(
          (s) =>
            s.name.toLowerCase().includes(query) ||
            s.description.toLowerCase().includes(query)
        )
      }

      return result
    },

    skillsmpSource(state) {
      return state.sources.find((s) => s.id === 'skillsmp')
    },
  },

  actions: {
    async fetchSources() {
      try {
        this.sources = await invoke('get_marketplace_sources')
        await this.checkApiKey()
      } catch (e) {
        console.error('Failed to fetch marketplace sources', e)
      }
    },

    async checkApiKey() {
      try {
        const key = await invoke<string | null>('get_skillsmp_api_key_masked')
        this.hasApiKey = !!key
      } catch (e) {
        this.hasApiKey = false
      }
    },

    async fetchSkills(sourceId?: string, forceRefresh = false) {
      this.loading = true
      this.error = null
      this.fetchProgress = {
        current: 0,
        total: 0,
        currentSource: '',
        status: 'loading_sources',
      }

      // Ensure sources are loaded first
      if (this.sources.length === 0) {
        await this.fetchSources()
      }

      // Clear if fetching all
      if (!sourceId) {
        this.skills = []
      }

      try {
        const sourcesToFetch = sourceId
          ? this.sources.filter((s) => s.id === sourceId)
          : this.sources.filter((s) => s.enabled)

        this.fetchProgress.total = sourcesToFetch.length
        this.fetchProgress.status = 'fetching'

        // Fetch sources one by one to show progress
        for (let i = 0; i < sourcesToFetch.length; i++) {
          const source = sourcesToFetch[i]
          this.fetchProgress.current = i + 1
          this.fetchProgress.currentSource = source.name

          try {
            const newSkills = await invoke<MarketplaceSkill[]>(
              'fetch_marketplace_skills',
              {
                sourceId: source.id,
                forceRefresh,
              }
            )

            // Merge results, avoiding duplicates
            const existingIds = new Set(
              this.skills.map((s) => s.name + s.source_id)
            )
            const uniqueSkills = newSkills.filter(
              (s) => !existingIds.has(s.name + s.source_id)
            )
            this.skills.push(...uniqueSkills)
          } catch (e) {
            console.error(`Failed to fetch from ${source.name}:`, e)
            // Store error for SkillsMP specifically
            if (source.id === 'skillsmp') {
              this.error = String(e)
            }
          }
        }

        this.fetchProgress.status = 'done'
      } catch (e) {
        this.error = String(e)
      } finally {
        this.loading = false
      }
    },

    async searchSkillsmp(query: string) {
      if (!query.trim()) {
        return this.fetchSkills('skillsmp')
      }

      this.loading = true
      this.error = null

      try {
        if (this.searchMode === 'ai') {
          // AI semantic search
          const results = await invoke<MarketplaceSkill[]>(
            'search_skillsmp_ai',
            {
              query,
            }
          )
          // Replace skills with search results
          this.skills = this.skills.filter((s) => s.source_id !== 'skillsmp')
          this.skills.push(...results)
        } else {
          // Keyword search
          const results = await invoke<MarketplaceSkill[]>(
            'fetch_skillsmp_skills',
            {
              query,
              page: 1,
              limit: 50,
              sortBy: 'stars',
            }
          )
          // Replace skills with search results
          this.skills = this.skills.filter((s) => s.source_id !== 'skillsmp')
          this.skills.push(...results)
        }
      } catch (e) {
        this.error = String(e)
      } finally {
        this.loading = false
      }
    },

    async refreshAll() {
      return this.fetchSkills(undefined, true)
    },

    setSearchMode(mode: SearchMode) {
      this.searchMode = mode
    },

    // Placeholder for adding custom sources (to be implemented)
    async addSource() {
      // TODO: Open a modal to add custom Git sources
      alert('Custom source management coming soon!')
    },
  },
})
