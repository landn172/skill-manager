import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import { useToastStore } from "./toast";
import type { MarketplaceSource, MarketplaceSkill, SearchMode } from "@/types";

interface FetchProgress {
  current: number;
  total: number;
  currentSource: string;
  status: "idle" | "loading_sources" | "fetching" | "done";
}

interface MarketplaceState {
  sources: MarketplaceSource[];
  skills: MarketplaceSkill[];
  loading: boolean;
  error: string | null;
  searchQuery: string;
  selectedSource: string | null;
  searchMode: SearchMode;
  hasApiKey: boolean;
  sortBy: "name" | "stars" | "updated";
  fetchProgress: FetchProgress;
  cachedSkills: import("@/types").CacheMetadata[];
  // Pagination
  page: number;
  pageSize: number;
  hasMore: boolean;
  total: number;
  searchHistory: string[];
}

export const useMarketplaceStore = defineStore("marketplace", {
  state: (): MarketplaceState => ({
    sources: [],
    skills: [],
    loading: false,
    error: null,
    searchQuery: "",
    selectedSource: null,
    searchMode: "keyword",
    hasApiKey: false,
    sortBy: "stars",
    fetchProgress: {
      current: 0,
      total: 0,
      currentSource: "",
      status: "idle",
    },
    cachedSkills: [],
    page: 1,
    pageSize: 50,
    hasMore: false,
    total: 0,
    searchHistory: JSON.parse(localStorage.getItem("marketplace_search_history") || "[]"),
  }),

  getters: {
    isSkillCached: (state: MarketplaceState) => (skillName: string) => {
      return state.cachedSkills.some((s) => s.skill_name === skillName);
    },
    getCachedAt: (state: MarketplaceState) => (skillName: string) => {
      return state.cachedSkills.find((s) => s.skill_name === skillName)?.downloaded_at;
    },

    filteredSkills(state: MarketplaceState) {
      let result = state.skills;

      // 1. Filter by Source
      if (state.selectedSource) {
        result = result.filter((s) => s.source_id === state.selectedSource);
      } else {
        // Deduplication priority...
        // ... (existing logic for deduplication) ...
        // I will copy the deduplication logic from the original file since I can't reference it inside the replacement block if I don't include it.
        // Wait, replace_file_content replaces a block. I need to be careful not to delete the deduplication logic if I'm replacing the whole filteredSkills.
        // The original filteredSkills is quite long. I should target smaller chunks if possible, or rewrite it carefully.
        // Let's rewrite the whole getter block to be safe and include the new logic.

        // Deduplication priority: Official > Local > Registry > Git > API
        const priorityMap: Record<string, number> = {
          local: 90,
          registry: 80,
          git: 70,
          api: 60,
        };

        const uniqueSkills = new Map<string, MarketplaceSkill>();

        for (const skill of result) {
          const source = state.sources.find((s) => s.id === skill.source_id);
          if (!source) continue;

          const existing = uniqueSkills.get(skill.name);
          let skillPriority = priorityMap[source.source_type] || 0;
          if (source.official) skillPriority = 100;

          if (!existing) {
            uniqueSkills.set(skill.name, skill);
          } else {
            const existingSource = state.sources.find((s) => s.id === existing.source_id);
            let existingPriority = 0;
            if (existingSource) {
              existingPriority = priorityMap[existingSource.source_type] || 0;
              if (existingSource.official) existingPriority = 100;
            }

            if (skillPriority > existingPriority) {
              uniqueSkills.set(skill.name, skill);
            }
          }
        }
        result = Array.from(uniqueSkills.values());
      }

      // 3. Filter by Search Query (Keyword mode)
      if (state.searchQuery && state.selectedSource !== "skillsmp") {
        const query = state.searchQuery.toLowerCase();
        result = result.filter(
          (s) =>
            s.name.toLowerCase().includes(query) || s.description.toLowerCase().includes(query),
        );
      }

      // 4. Sort
      result.sort((a: MarketplaceSkill, b: MarketplaceSkill) => {
        if (state.sortBy === "stars") {
          return (b.stars || 0) - (a.stars || 0);
        } else if (state.sortBy === "updated") {
          return 0;
        } else {
          return a.name.localeCompare(b.name);
        }
      });

      return result;
    },

    skillsmpSource(state: MarketplaceState) {
      return state.sources.find((s) => s.id === "skillsmp");
    },
  },

  actions: {
    async fetchSources() {
      try {
        this.sources = await invoke("get_marketplace_sources");
        await this.checkApiKey();
      } catch (e) {
        console.error("Failed to fetch marketplace sources", e);
      }
    },

    async checkApiKey() {
      try {
        const key = await invoke<string | null>("get_skillsmp_api_key_masked");
        this.hasApiKey = !!key;
      } catch (e) {
        this.hasApiKey = false;
      }
    },

    async fetchSkills(sourceId?: string, forceRefresh = false) {
      this.loading = true;
      this.error = null;
      this.fetchProgress = {
        current: 0,
        total: 0,
        currentSource: "",
        status: "loading_sources",
      };

      // Ensure sources are loaded first
      if (this.sources.length === 0) {
        await this.fetchSources();
      }

      // Clear if fetching all
      if (!sourceId) {
        this.skills = [];
      }

      try {
        const sourcesToFetch = sourceId
          ? this.sources.filter((s) => s.id === sourceId)
          : this.sources.filter((s) => s.enabled);

        this.fetchProgress.total = sourcesToFetch.length;
        this.fetchProgress.status = "fetching";

        const toastStore = useToastStore();

        // Fetch sources one by one to show progress
        for (let i = 0; i < sourcesToFetch.length; i++) {
          const source = sourcesToFetch[i];
          this.fetchProgress.current = i + 1;
          this.fetchProgress.currentSource = source.name;

          try {
            let newSkills: MarketplaceSkill[] = [];

            if (source.id === "skillsmp") {
              // Use Proxy for SkillsMP to bypass Cloudflare
              const { skills: skillsmpResults } = await this.fetchSkillsmpDirect("*", 1, this.pageSize);
              newSkills = skillsmpResults;
            } else {
              // Use standard backend fetch for others
              newSkills = await invoke<MarketplaceSkill[]>("fetch_marketplace_skills", {
                sourceId: source.id,
                forceRefresh,
              });
            }

            // Merge results, avoiding duplicates
            const existingIds = new Set(this.skills.map((s) => s.name + s.source_id));
            const uniqueSkills = newSkills.filter((s) => !existingIds.has(s.name + s.source_id));
            this.skills.push(...uniqueSkills);
          } catch (e) {
            console.error(`Failed to fetch from ${source.name}:`, e);
            toastStore.error(`Failed to load ${source.name}: ${e instanceof Error ? e.message : String(e)}`);
          }
        }

        this.fetchProgress.status = "done";
      } catch (e) {
        useToastStore().error(`Error during fetch: ${e instanceof Error ? e.message : String(e)}`);
      } finally {
        this.loading = false;
      }
    },

    async searchSkillsmp(query: string, append = false) {
      if (!query.trim()) {
        this.page = 1;
        this.hasMore = false;
        return this.fetchSkills("skillsmp");
      }

      this.addToHistory(query);

      if (!append) {
        this.page = 1;
        this.loading = true;
      }

      try {
        if (this.searchMode === "ai") {
          // AI semantic search
          const results = await invoke<MarketplaceSkill[]>("search_skillsmp_ai", {
            query,
          });
          this.skills = this.skills.filter((s) => s.source_id !== "skillsmp");
          this.skills.push(...results);
          this.hasMore = false;
        } else {
          const { skills: results, total } = await this.fetchSkillsmpDirect(query, this.page, this.pageSize);

          if (append) {
            this.skills.push(...results);
          } else {
            this.skills = this.skills.filter((s) => s.source_id !== "skillsmp");
            this.skills.push(...results);
          }
          
          this.total = total;
          this.hasMore = this.skills.filter(s => s.source_id === 'skillsmp').length < total;
        }
      } catch (e) {
        useToastStore().error(`Search failed: ${e instanceof Error ? e.message : String(e)}`);
      } finally {
        this.loading = false;
      }
    },

    addToHistory(query: string) {
      const q = query.trim();
      if (!q) return;

      // Deduplicate and move to front
      this.searchHistory = [q, ...this.searchHistory.filter((h) => h !== q)].slice(0, 10);
      localStorage.setItem("marketplace_search_history", JSON.stringify(this.searchHistory));
    },

    clearHistory() {
      this.searchHistory = [];
      localStorage.removeItem("marketplace_search_history");
    },

    async loadMoreSkillsmp() {
      if (this.loading || !this.hasMore || this.selectedSource !== "skillsmp") return;
      this.page++;
      await this.searchSkillsmp(this.searchQuery, true);
    },

    /**
     * Fetch SkillsMP using Tauri HTTP plugin (bypasses CORS and uses native HTTP client)
     */
    async fetchSkillsmpDirect(query: string, page = 1, limit = 50): Promise<{ skills: MarketplaceSkill[], total: number }> {
      const { fetch: tauriFetch } = await import("@tauri-apps/plugin-http");

      // Get API Key from Rust store
      const apiKey = await invoke<string | null>("get_skillsmp_api_key");
      if (!apiKey) {
        throw new Error(
          "SkillsMP API key not configured. Add SKILLSMP_API_KEY to .env or configure in Settings.",
        );
      }

      const url = `https://skillsmp.com/api/v1/skills/search?q=${encodeURIComponent(query)}&page=${page}&limit=${limit}&sortBy=stars`;

      console.log("[SkillsMP] Fetching via Tauri HTTP:", url);

      const response = await tauriFetch(url, {
        method: "GET",
        headers: {
          Authorization: `Bearer ${apiKey}`,
          "User-Agent":
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        },
      });

      if (!response.ok) {
        const text = await response.text();
        throw new Error(`SkillsMP API error (${response.status}): ${text}`);
      }

      const data = await response.json();
      
      if (!data.success) {
        throw new Error(data.error?.message || "SkillsMP request failed");
      }

      const total = data.pagination?.total || 0;

      // Handle different response structures
      let rawSkills = data.data;
      if (!Array.isArray(rawSkills)) {
        if (rawSkills && Array.isArray(rawSkills.skills)) {
          rawSkills = rawSkills.skills;
        } else if (rawSkills && Array.isArray(rawSkills.results)) {
          rawSkills = rawSkills.results;
        } else {
          rawSkills = [];
        }
      }

      const skills = rawSkills.map((s: any) => ({
        name: s.name,
        description: s.description || "",
        path: s.skillUrl || s.githubUrl || "", 
        version: undefined,
        metadata: {
          repo: s.githubUrl || "",
          repo_url: s.githubUrl || "",
          author: s.author || "",
          skillUrl: s.skillUrl || "",
        },
        source_id: "skillsmp",
        source_name: "SkillsMP",
        stars: s.stars || 0,
        repo: s.githubUrl, 
        repo_url: s.githubUrl,
        tags: [],
      }));

      return { skills, total };
    },

    async refreshAll() {
      return this.fetchSkills(undefined, true);
    },

    setSearchMode(mode: SearchMode) {
      this.searchMode = mode;
    },

    async discoverFromUrl(url: string): Promise<MarketplaceSkill[]> {
      this.loading = true;
      try {
        const skills = await invoke<MarketplaceSkill[]>("discover_skills_from_url", { url });
        return skills;
      } catch (e) {
        console.error("Failed to discover from URL:", e);
        useToastStore().error(`Discovery failed: ${e instanceof Error ? e.message : String(e)}`);
        throw e;
      } finally {
        this.loading = false;
      }
    },

    // Placeholder for adding custom sources (to be implemented)
    async addSource(url: string, name: string, type?: "registry" | "local" | "git") {
      try {
        this.sources = await invoke("add_marketplace_source", {
          url,
          name,
          sourceType: type,
        });
      } catch (e) {
        console.error("Failed to add source:", e);
        throw e;
      }
    },

    async removeSource(id: string) {
      try {
        this.sources = await invoke("remove_marketplace_source", { id });
      } catch (e) {
        console.error("Failed to remove source:", e);
        throw e;
      }
    },

    async toggleSource(id: string, enabled: boolean) {
      try {
        this.sources = await invoke("toggle_marketplace_source", {
          id,
          enabled,
        });
      } catch (e) {
        console.error("Failed to toggle source:", e);
        throw e;
      }
    },

    async fetchCachedSkills() {
      try {
        this.cachedSkills = await invoke("get_cached_skills");
      } catch (e) {
        console.error("Failed to fetch cached skills:", e);
      }
    },

    async clearCache(skillName: string) {
      try {
        await invoke("clear_skill_cache", { skillName });
        await this.fetchCachedSkills();
      } catch (e) {
        console.error("Failed to clear cache:", e);
        throw e;
      }
    },

    async clearAllCache() {
      try {
        await invoke("clear_all_cache");
        await this.fetchCachedSkills();
      } catch (e) {
        console.error("Failed to clear all cache:", e);
        throw e;
      }
    },
  },
});
