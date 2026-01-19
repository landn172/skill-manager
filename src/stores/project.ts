import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { Project } from '@/types'

export const useProjectStore = defineStore('projects', {
  state: () => ({
    projects: [] as Project[],
    currentProject: null as Project | null,
    loading: false,
    error: null as string | null,
  }),

  getters: {
    hasProjects: (state) => state.projects.length > 0,
  },

  actions: {
    async fetchProjects() {
      this.loading = true
      try {
        this.projects = await invoke('list_projects')
        // Restore current project from local storage if possible
        const storedId = localStorage.getItem('current_project_id')
        if (storedId) {
          const id = parseInt(storedId)
          this.currentProject = this.projects.find((p) => p.id === id) || null
        }
      } catch (e) {
        this.error = String(e)
      } finally {
        this.loading = false
      }
    },

    async addProject(name: string, path: string) {
      try {
        const newProject = await invoke<Project>('add_project', { name, path })
        this.projects.unshift(newProject)
        this.setCurrentProject(newProject)
        return newProject
      } catch (e) {
        this.error = String(e)
        throw e
      }
    },

    async removeProject(id: number) {
      try {
        await invoke('remove_project', { id })
        this.projects = this.projects.filter((p) => p.id !== id)
        if (this.currentProject?.id === id) {
          this.setCurrentProject(null)
        }
      } catch (e) {
        this.error = String(e)
        throw e
      }
    },

    setCurrentProject(project: Project | null) {
      this.currentProject = project
      if (project?.id) {
        localStorage.setItem('current_project_id', project.id.toString())
      } else {
        localStorage.removeItem('current_project_id')
      }
    },
  },
})
