export type AgentType =
  | 'opencode'
  | 'claude-code'
  | 'codex'
  | 'cursor'
  | 'gemini'
  | 'vscode'
  | string // Allow custom agent names

export interface AgentConfig {
  agent_type: AgentType
  name: string
  display_name: string
  skills_dir: string
  global_skills_dir: string
  icon: string
  installed: boolean
  is_custom?: boolean
}

export interface Skill {
  name: string
  description: string
  path: string
  version?: string
  metadata?: Record<string, string>
}

export interface InstalledSkill extends Skill {
  install_date: string
  source: string
  source_id: string
  scope: 'project' | 'global'
  agents: AgentType[]
  agent_paths: Record<string, string>
  installed_version?: string
}

// Source types for marketplace
export type SourceType = 'git' | 'api' | 'local' | 'registry'
export type SearchMode = 'keyword' | 'ai'

export interface MarketplaceSource {
  id: string
  name: string
  url: string
  description?: string
  official: boolean
  enabled: boolean
  last_fetched?: string
  source_type: SourceType
}

export interface MarketplaceSkill extends Skill {
  source_id: string
  source_name: string
  category?: string
  tags: string[]
  // SkillsMP API fields
  stars?: number
  repo?: string
  repo_url?: string
}

// SkillsMP API response types
export interface SkillsmpSkill {
  name: string
  file_name?: string
  description?: string
  stars: number
  repo?: string
  url?: string
  updated_at?: string
}

export interface SkillsmpPagination {
  page: number
  limit: number
  total: number
  total_pages: number
}

export interface AppConfig {
  theme: 'light' | 'dark' | 'system'
  default_scope: 'project' | 'global'
  project_path?: string
}

export interface Project {
  id?: number
  name: string
  path: string
  created_at?: string
}
