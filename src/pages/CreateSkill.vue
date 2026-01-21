<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { homeDir } from '@tauri-apps/api/path'
import { ChevronRight, Save, Folder, FileCode } from 'lucide-vue-next'

const router = useRouter()
const route = useRoute()

const step = ref(1)
const creating = ref(false)
const form = ref({
  name: '',
  description: '',
  parentPath: '',
})

// Edit mode: read from query params
const isEditMode = computed(() => !!route.query.edit)
const editSkillPath = computed(() => (route.query.path as string) || '')

onMounted(() => {
  // If editing, populate form from query params
  if (isEditMode.value) {
    form.value.name = (route.query.name as string) || ''
    form.value.description = (route.query.description as string) || ''
    form.value.parentPath = editSkillPath.value // Use skill path as parentPath for display
  }
})

async function selectParentPath() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: await homeDir(),
    })

    if (selected && typeof selected === 'string') {
      form.value.parentPath = selected
    }
  } catch (e) {
    console.error('Failed to select directory', e)
  }
}

async function handleSubmit() {
  creating.value = true
  try {
    if (isEditMode.value) {
      // Update existing skill
      await invoke('update_local_skill', {
        skillPath: editSkillPath.value,
        name: form.value.name || undefined,
        description: form.value.description || undefined,
      })
      alert('Skill updated successfully!')
      router.push('/marketplace')
    } else {
      // Create new skill
      const result = await invoke<{
        success: boolean
        path: string
        message: string
      }>('create_skill', {
        name: form.value.name,
        description: form.value.description,
        parentPath: form.value.parentPath,
      })

      if (result.success) {
        alert(`Skill created successfully at ${result.path}`)
        router.push('/installed')
      } else {
        throw new Error(result.message)
      }
    }
  } catch (e) {
    alert(`Failed to ${isEditMode.value ? 'update' : 'create'} skill: ${e}`)
  } finally {
    creating.value = false
  }
}
</script>

<template>
  <div class="create-skill-page">
    <header class="header">
      <h1>{{ isEditMode ? 'Edit Skill' : 'Create New Skill' }}</h1>
      <div v-if="!isEditMode" class="steps">
        <div class="step" :class="{ active: step >= 1 }">1</div>
        <div class="step-line"></div>
        <div class="step" :class="{ active: step >= 2 }">2</div>
      </div>
    </header>

    <div class="form-container">
      <div v-if="!isEditMode" class="intro-box">
        <FileCode :size="24" class="intro-icon" />
        <p>
          Scaffold a new skill directory with a standard structure (README,
          Instructions) to start building your own agent skill.
        </p>
      </div>

      <!-- Step 1: Basic Info -->
      <section v-if="step === 1" class="form-step">
        <h2>Basic Information</h2>
        <div class="input-group">
          <label>Skill Name</label>
          <input
            v-model="form.name"
            placeholder="e.g. react-hook-generator"
            autofocus
          />
          <span class="hint"
            >This will be used as the folder name (kebab-case
            recommended).</span
          >
        </div>
        <div class="input-group">
          <label>Description</label>
          <textarea
            v-model="form.description"
            placeholder="What does this skill do?"
          ></textarea>
        </div>
      </section>

      <!-- Step 2: Location (only for Create mode) -->
      <section v-if="step === 2 && !isEditMode" class="form-step">
        <h2>Location</h2>
        <div class="input-group">
          <label>Parent Directory</label>
          <div class="path-selector">
            <input
              v-model="form.parentPath"
              placeholder="Select where to create the skill folder..."
              readonly
            />
            <button class="btn secondary small" @click="selectParentPath">
              <Folder :size="16" />
              Browse
            </button>
          </div>
          <span class="hint"
            >A new folder named "{{ form.name || 'skill-name' }}" will be
            created here.</span
          >
        </div>
      </section>
    </div>

    <footer class="footer">
      <button
        v-if="step > 1 && !isEditMode"
        class="btn secondary"
        @click="step--"
      >
        <span>Back</span>
      </button>
      <div class="spacer"></div>
      <button
        v-if="step < 2 && !isEditMode"
        class="btn primary"
        @click="step++"
        :disabled="!form.name"
      >
        <span>Next</span>
        <ChevronRight :size="18" />
      </button>
      <button
        v-else
        class="btn primary"
        @click="handleSubmit"
        :disabled="(!isEditMode && !form.parentPath) || creating"
      >
        <Save :size="18" />
        <span>{{
          creating
            ? isEditMode
              ? 'Saving...'
              : 'Creating...'
            : isEditMode
              ? 'Save Changes'
              : 'Create Skill'
        }}</span>
      </button>
    </footer>
  </div>
</template>

<style scoped>
.create-skill-page {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 32px;
}

.intro-box {
  display: flex;
  align-items: center;
  gap: 16px;
  background-color: var(--bg-secondary);
  padding: 16px;
  border-radius: var(--border-radius);
  border: 1px solid var(--border-color);
  margin-bottom: 32px;
  color: var(--text-secondary);
}

.intro-icon {
  color: var(--accent-primary);
}

.steps {
  display: flex;
  align-items: center;
  gap: 8px;
}

.step {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background-color: var(--bg-tertiary);
  color: var(--text-muted);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  font-weight: 600;
  border: 1px solid var(--border-color);
}

.step.active {
  background-color: var(--accent-primary);
  color: white;
  border-color: var(--accent-primary);
}

.step-line {
  width: 32px;
  height: 2px;
  background-color: var(--border-color);
}

.form-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  max-width: 600px;
  margin: 0 auto;
  width: 100%;
}

.form-step {
  display: flex;
  flex-direction: column;
  gap: 24px;
  animation: fadeIn 0.3s ease;
}

.form-step.full-height {
  flex: 1;
}

h2 {
  font-size: 20px;
  font-weight: 600;
  margin-bottom: 8px;
}

.input-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-secondary);
}

input,
textarea {
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  padding: 12px 16px;
  outline: none;
  transition: border-color 0.2s;
  color: var(--text-primary);
  font-size: 14px;
}

input:focus,
textarea:focus {
  border-color: var(--accent-primary);
}

textarea {
  min-height: 120px;
  resize: vertical;
}

.content-editor {
  flex: 1;
  font-family: 'Fira Code', monospace;
  font-size: 14px;
  resize: none;
}

.agent-selection {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.agent-option {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  cursor: pointer;
}

.agent-option.selected {
  border-color: var(--accent-primary);
  background-color: rgba(139, 92, 246, 0.05);
}

.agent-option.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.scope-options {
  display: flex;
  gap: 12px;
}

.scope-btn {
  flex: 1;
  padding: 12px;
  border-radius: var(--border-radius);
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-color);
}

.scope-btn.active {
  border-color: var(--accent-primary);
  color: var(--accent-primary);
  background-color: rgba(139, 92, 246, 0.05);
}

.path-selector {
  display: flex;
  gap: 8px;
}

.hint {
  font-size: 12px;
  color: var(--text-muted);
}

.footer {
  margin-top: 40px;
  display: flex;
  padding-top: 24px;
  border-top: 1px solid var(--border-color);
}

.spacer {
  flex: 1;
}

.btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 24px;
  border-radius: 10px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  border: none;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn.primary {
  background-color: var(--accent-primary);
  color: white;
}

.btn.primary:hover:not(:disabled) {
  filter: brightness(1.1);
}

.btn.secondary {
  background-color: var(--bg-tertiary);
  color: var(--text-secondary);
}

.btn.secondary:hover {
  background-color: var(--bg-hover);
}

.btn.small {
  padding: 0 16px;
  white-space: nowrap;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(5px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
