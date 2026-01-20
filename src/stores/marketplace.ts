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
      } else {
        // Deduplication priority: Official > Local > Registry > Git > API
        const priorityMap: Record<string, number> = {
          local: 90,
          registry: 80,
          git: 70,
          api: 60,
        }

        const uniqueSkills = new Map<string, MarketplaceSkill>()

        for (const skill of result) {
          const source = state.sources.find((s) => s.id === skill.source_id)
          // If source not found (shouldn't happen), skip or keep
          if (!source) continue

          const existing = uniqueSkills.get(skill.name)
          let skillPriority = priorityMap[source.source_type] || 0
          if (source.official) skillPriority = 100

          if (!existing) {
            uniqueSkills.set(skill.name, skill)
          } else {
            const existingSource = state.sources.find(
              (s) => s.id === existing.source_id,
            )
            let existingPriority = 0
            if (existingSource) {
              existingPriority = priorityMap[existingSource.source_type] || 0
              if (existingSource.official) existingPriority = 100
            }

            if (skillPriority > existingPriority) {
              uniqueSkills.set(skill.name, skill)
            }
          }
        }
        result = Array.from(uniqueSkills.values())
      }

      // Only filter locally if using keyword mode and not searching via API
      if (state.searchQuery && state.selectedSource !== 'skillsmp') {
        const query = state.searchQuery.toLowerCase()
        result = result.filter(
          (s) =>
            s.name.toLowerCase().includes(query) ||
            s.description.toLowerCase().includes(query),
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
              },
            )

            // Merge results, avoiding duplicates
            const existingIds = new Set(
              this.skills.map((s) => s.name + s.source_id),
            )
            const uniqueSkills = newSkills.filter(
              (s) => !existingIds.has(s.name + s.source_id),
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
            },
          )
          // Replace skills with search results
          this.skills = this.skills.filter((s) => s.source_id !== 'skillsmp')
          this.skills.push(...results)
        } else {
          // Keyword search using WebView Proxy to bypass Cloudflare
          const results = await this.fetchSkillsmpViaProxy(query)

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

    /**
     * Fetch SkillsMP via a separate WebView to bypass Cloudflare JA3/Challenge
     */
    async fetchSkillsmpViaProxy(query: string): Promise<MarketplaceSkill[]> {
      const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow')
      const { emit, listen } = await import('@tauri-apps/api/event')

      // Get API Key from Rust store
      const apiKey = await invoke<string | null>('get_skillsmp_api_key')
      if (!apiKey) {
        throw new Error('SkillsMP API key not configured.')
      }

      const label = 'skillsmp-proxy'
      let webview = await WebviewWindow.getByLabel(label)

      if (!webview) {
        // Create the proxy window if it doesn't exist
        webview = new WebviewWindow(label, {
          url: '/proxy.html', // Local asset
          visible: false,
          title: 'SkillsMP Cloudflare Proxy',
          width: 500,
          height: 600,
        })
      }

      const reqId = Math.random().toString(36).substring(7)
      const url = `https://skillsmp.com/api/v1/skills/search?q=${encodeURIComponent(query)}&page=1&limit=50&sortBy=stars`

      return new Promise((resolve, reject) => {
        const timeout = setTimeout(() => {
          reject(new Error('Proxy fetch timed out'))
        }, 45000)

        const unlistens: any[] = []

        // Listen for response from proxy
        listen('proxy-response', (event: any) => {
          const payload = event.payload
          if (payload.reqId !== reqId) return

          clearTimeout(timeout)
          unlistens.forEach((f) => f())

          if (payload.success) {
            const skillsmpResults = payload.data.data || []
            const formatted = skillsmpResults.map((s: any) => ({
              name: s.name,
              description: s.description || '',
              path: s.url || '',
              source_id: 'skillsmp',
              source_name: 'SkillsMP',
              stars: s.stars || 0,
              repo: s.repo,
              repo_url: s.repo ? `https://github.com/${s.repo}` : undefined,
              tags: [],
            }))
            resolve(formatted)
          } else {
            reject(new Error(payload.error))
          }
        }).then((u) => unlistens.push(u))

        listen('proxy-challenge', (event: any) => {
          if (event.payload.reqId !== reqId) return

          // Window is already shown by proxy.html logica
          alert(
            'Cloudflare challenge detected. Please solve it in the popup and click Search again.',
          )
          clearTimeout(timeout)
          unlistens.forEach((f) => f())
          reject(new Error('CHALLENGE_REQUIRED'))
        }).then((u) => unlistens.push(u))

        // Give the proxy a moment to load
        setTimeout(() => {
          emit('proxy-request', { url, apiKey, reqId })
        }, 1000)
      })
    },

    async refreshAll() {
      return this.fetchSkills(undefined, true)
    },

    setSearchMode(mode: SearchMode) {
      this.searchMode = mode
    },

    // Placeholder for adding custom sources (to be implemented)
    async addSource(url: string, name: string) {
      try {
        this.sources = await invoke('add_marketplace_source', { url, name })
      } catch (e) {
        console.error('Failed to add source:', e)
        throw e
      }
    },

    async removeSource(id: string) {
      try {
        this.sources = await invoke('remove_marketplace_source', { id })
      } catch (e) {
        console.error('Failed to remove source:', e)
        throw e
      }
    },

    async toggleSource(id: string, enabled: boolean) {
      try {
        this.sources = await invoke('toggle_marketplace_source', {
          id,
          enabled,
        })
      } catch (e) {
        console.error('Failed to toggle source:', e)
        throw e
      }
    },
  },
})
