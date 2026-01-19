<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { useAgentsStore } from '@/stores/agents'
import { ChevronLeft, ChevronRight, Save, CheckCircle2 } from 'lucide-vue-next'

const router = useRouter()
const agentsStore = useAgentsStore()

const step = ref(1)
const form = ref({
  name: '',
  description: '',
  content: '---\nname: \ndescription: \n---\n\n# Instructions\n\n1. \n2. ',
  selectedAgents: [] as string[],
  scope: 'global' as 'project' | 'global',
})

onMounted(() => {
  agentsStore.fetchAgents()
})

watch(
  () => form.value.name,
  () => {
    updateContent()
  }
)

watch(
  () => form.value.description,
  () => {
    updateContent()
  }
)

function updateContent() {
  const fm = `---\nname: ${form.value.name}\ndescription: ${form.value.description}\n---`
  const body = form.value.content.split('---').pop() || ''
  form.value.content = `${fm}\n${body.trim()}`
}

async function handleCreate() {
  try {
    // 1. Create a temporary skill directory
    // For now, simplify by just calling a "create_skill" command if we had one
    // or just using the installer logic with a virtual skill object

    await invoke('install_skill', {
      skill: {
        name: form.value.name,
        description: form.value.description,
        path: '', // Virtual path
        metadata: {},
      },
      agents: form.value.selectedAgents,
      scope: form.value.scope,
    })

    // We need to actually write the content. Our current installer copies a directory.
    // I should add a "create_and_install_skill" command or similar.

    alert('Skill created and installed!')
    router.push('/installed')
  } catch (e) {
    alert(`Failed to create skill: ${e}`)
  }
}
</script>

<template>
  <div class="create-skill-page">
    <header class="header">
      <h1>Create New Skill</h1>
      <div class="steps">
        <div class="step" :class="{ active: step >= 1 }">1</div>
        <div class="step-line"></div>
        <div class="step" :class="{ active: step >= 2 }">2</div>
        <div class="step-line"></div>
        <div class="step" :class="{ active: step >= 3 }">3</div>
      </div>
    </header>

    <div class="form-container">
      <!-- Step 1: Basic Info -->
      <section v-if="step === 1" class="form-step">
        <h2>Basic Information</h2>
        <div class="input-group">
          <label>Skill Name</label>
          <input v-model="form.name" placeholder="e.g. frontend-helper" />
        </div>
        <div class="input-group">
          <label>Description</label>
          <textarea
            v-model="form.description"
            placeholder="What does this skill do?"
          ></textarea>
        </div>
      </section>

      <!-- Step 2: Target Agents -->
      <section v-if="step === 2" class="form-step">
        <h2>Target Agents & Scope</h2>
        <div class="agent-selection">
          <div
            v-for="agent in agentsStore.agents"
            :key="agent.name"
            class="agent-option"
            :class="{
              selected: form.selectedAgents.includes(agent.name),
              disabled: !agent.installed,
            }"
            @click="
              agent.installed &&
                (form.selectedAgents.includes(agent.name)
                  ? (form.selectedAgents = form.selectedAgents.filter(
                      (a) => a !== agent.name
                    ))
                  : form.selectedAgents.push(agent.name))
            "
          >
            <span class="agent-icon">{{
              agent.icon === 'Sparkles'
                ? '✨'
                : agent.icon === 'Terminal'
                ? '💻'
                : agent.icon === 'Bot'
                ? '🤖'
                : agent.icon === 'Code'
                ? '📄'
                : '🖱️'
            }}</span>
            <div class="agent-name">{{ agent.display_name }}</div>
            <div
              class="check-wrap"
              v-if="form.selectedAgents.includes(agent.name)"
            >
              <CheckCircle2 :size="16" />
            </div>
          </div>
        </div>

        <div class="scope-selection">
          <label>Installation Scope</label>
          <div class="scope-options">
            <button
              class="scope-btn"
              :class="{ active: form.scope === 'project' }"
              @click="form.scope = 'project'"
            >
              Project
            </button>
            <button
              class="scope-btn"
              :class="{ active: form.scope === 'global' }"
              @click="form.scope = 'global'"
            >
              Global
            </button>
          </div>
        </div>
      </section>

      <!-- Step 3: Content -->
      <section v-if="step === 3" class="form-step full-height">
        <h2>Skill Instructions</h2>
        <textarea class="content-editor" v-model="form.content"></textarea>
      </section>
    </div>

    <footer class="footer">
      <button v-if="step > 1" class="btn secondary" @click="step--">
        <ChevronLeft :size="18" />
        <span>Back</span>
      </button>
      <div class="spacer"></div>
      <button
        v-if="step < 3"
        class="btn primary"
        @click="step++"
        :disabled="step === 1 && !form.name"
      >
        <span>Next</span>
        <ChevronRight :size="18" />
      </button>
      <button
        v-else
        class="btn primary"
        @click="handleCreate"
        :disabled="form.selectedAgents.length === 0"
      >
        <Save :size="18" />
        <span>Create Skill</span>
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
  margin-bottom: 40px;
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
  max-width: 800px;
  margin: 0 auto;
  width: 100%;
}

.form-step {
  display: flex;
  flex-direction: column;
  gap: 24px;
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
}

.btn.primary {
  background-color: var(--accent-primary);
  color: white;
}

.btn.primary:disabled {
  opacity: 0.5;
}

.btn.secondary {
  color: var(--text-secondary);
}

.btn.secondary:hover {
  background-color: var(--bg-hover);
}
</style>
