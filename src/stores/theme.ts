import { defineStore } from 'pinia'
import { ref, watch, computed } from 'vue'

export type Theme = 'light' | 'dark' | 'system'

export const useThemeStore = defineStore('theme', () => {
  // Load from local storage or default to system
  const savedTheme = localStorage.getItem('theme') as Theme | null
  const theme = ref<Theme>(savedTheme || 'system')

  const systemIsDark = ref(
    window.matchMedia('(prefers-color-scheme: dark)').matches,
  )

  // Listen for system changes
  window
    .matchMedia('(prefers-color-scheme: dark)')
    .addEventListener('change', (e) => {
      systemIsDark.value = e.matches
    })

  const currentTheme = computed(() => {
    if (theme.value === 'system') {
      return systemIsDark.value ? 'dark' : 'light'
    }
    return theme.value
  })

  function setTheme(newTheme: Theme) {
    theme.value = newTheme
    localStorage.setItem('theme', newTheme)
    applyTheme()
  }

  function toggleTheme() {
    const next: Theme = currentTheme.value === 'dark' ? 'light' : 'dark'
    setTheme(next)
  }

  function applyTheme() {
    const root = document.documentElement
    const isDark = currentTheme.value === 'dark'

    if (isDark) {
      root.setAttribute('data-theme', 'dark')
      root.classList.add('dark')
    } else {
      root.setAttribute('data-theme', 'light')
      root.classList.remove('dark')
    }
  }

  // Initial apply
  applyTheme()

  // Watch for changes (e.g. if system preference changes while in system mode)
  watch([theme, systemIsDark], () => {
    applyTheme()
  })

  return {
    theme,
    currentTheme,
    setTheme,
    toggleTheme,
    applyTheme,
  }
})
