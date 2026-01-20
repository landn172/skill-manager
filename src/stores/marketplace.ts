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
            let newSkills: MarketplaceSkill[] = []

            if (source.id === 'skillsmp') {
              // Use Proxy for SkillsMP to bypass Cloudflare
              newSkills = await this.fetchSkillsmpDirect('*')
            } else {
              // Use standard backend fetch for others
              newSkills = await invoke<MarketplaceSkill[]>(
                'fetch_marketplace_skills',
                {
                  sourceId: source.id,
                  forceRefresh,
                },
              )
            }

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
          const results = await this.fetchSkillsmpDirect(query)

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
     * Fetch SkillsMP using Tauri HTTP plugin (bypasses CORS and uses native HTTP client)
     *
     * Architecture Note:
     * - Browser fetch() is blocked by CORS in dev mode (localhost origin)
     * - Tauri HTTP plugin makes native requests, bypassing CORS entirely
     * - This also works around Cloudflare's browser detection
     */
    async fetchSkillsmpDirect(
      query: string,
      page = 1,
      limit = 50,
    ): Promise<MarketplaceSkill[]> {
      const { fetch: tauriFetch } = await import('@tauri-apps/plugin-http')

      // Get API Key from Rust store
      const apiKey = await invoke<string | null>('get_skillsmp_api_key')
      if (!apiKey) {
        throw new Error(
          'SkillsMP API key not configured. Add SKILLSMP_API_KEY to .env or configure in Settings.',
        )
      }

      const url = `https://skillsmp.com/api/v1/skills/search?q=${encodeURIComponent(query)}&page=${page}&limit=${limit}&sortBy=stars`

      console.log('[SkillsMP] Fetching via Tauri HTTP:', url)

      const response = await tauriFetch(url, {
        method: 'GET',
        headers: {
          Authorization: `Bearer ${apiKey}`,
          'User-Agent':
            'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
        },
      })

      console.log('[SkillsMP] Response status:', response.status)

      if (!response.ok) {
        const text = await response.text()
        throw new Error(`SkillsMP API error (${response.status}): ${text}`)
      }

      const data = await response.json()
      console.log('[SkillsMP] Raw response:', data)

      if (!data.success) {
        throw new Error(data.error?.message || 'SkillsMP request failed')
      }

      // Handle different response structures
      let skills = data.data
      if (!Array.isArray(skills)) {
        console.warn(
          '[SkillsMP] data.data is not an array:',
          typeof skills,
          skills,
        )
        // Try to extract from nested structure
        if (skills && Array.isArray(skills.skills)) {
          skills = skills.skills
        } else if (skills && Array.isArray(skills.results)) {
          skills = skills.results
        } else {
          skills = []
        }
      }
      console.log('[SkillsMP] Got', skills.length, 'skills')

      return skills.map((s: any) => ({
        name: s.name,
        description: s.description || '',
        path: s.skillUrl || s.githubUrl || '', // Use skillUrl or githubUrl as path
        version: undefined,
        metadata: {
          // Use githubUrl directly - it's already a full URL
          repo: s.githubUrl || '',
          repo_url: s.githubUrl || '',
          author: s.author || '',
          skillUrl: s.skillUrl || '',
        },
        source_id: 'skillsmp',
        source_name: 'SkillsMP',
        stars: s.stars || 0,
        repo: s.githubUrl, // For display purposes
        repo_url: s.githubUrl,
        tags: [],
      }))
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
