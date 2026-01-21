<script setup lang="ts">
import { ref, computed } from 'vue'
import Modal from '@/components/common/Modal.vue'
import { useAgentsStore } from '@/stores/agents'
import AgentIcon from '@/components/icons/AgentIcon.vue'
import { CheckCircle2, XCircle, Rocket, ArrowRight } from 'lucide-vue-next'

defineProps<{
  show: boolean
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

const agentsStore = useAgentsStore()
const step = ref(1)

const steps = [
  { id: 1, title: 'Welcome' },
  { id: 2, title: 'Detect Agents' },
  { id: 3, title: 'Ready!' },
]

async function nextStep() {
  if (step.value === 1) {
    step.value = 2
    await agentsStore.fetchAgents()
  } else if (step.value === 2) {
    step.value = 3
  } else {
    emit('close')
  }
}

const installedAgentsCount = computed(
  () => agentsStore.agents.filter((a) => a.installed).length,
)
</script>

<template>
  <Modal
    :show="show"
    title="Welcome to Skill Manager"
    :maxWidth="'600px'"
    @close="() => {}"
  >
    <div class="onboarding-content">
      <!-- Step 1: Welcome -->
      <div v-if="step === 1" class="step step-1">
        <div class="hero-icon">
          <Rocket :size="64" />
        </div>
        <h2>Supercharge your AI Agents</h2>
        <p>
          Skill Manager helps you discover, install, and manage skills for your
          favorite coding assistants like Cursor, Claude Code, and more.
        </p>
        <div class="features-list">
          <div class="feature-item">
            <CheckCircle2 class="feature-icon" />
            <span>Browse 65,000+ skills from SkillsMP</span>
          </div>
          <div class="feature-item">
            <CheckCircle2 class="feature-icon" />
            <span>One-click install to multiple agents</span>
          </div>
          <div class="feature-item">
            <CheckCircle2 class="feature-icon" />
            <span>Manage local and Git-based skills</span>
          </div>
        </div>
      </div>

      <!-- Step 2: Agents -->
      <div v-if="step === 2" class="step step-2">
        <h2>Detecting Agents</h2>
        <p>We're scanning your system for supported AI agents...</p>

        <div v-if="agentsStore.loading" class="loader-container">
          <div class="loader"></div>
          <span>Scanning...</span>
        </div>

        <div v-else class="agents-results">
          <div
            v-for="agent in agentsStore.agents"
            :key="agent.agent_type"
            class="agent-card"
            :class="{ installed: agent.installed }"
          >
            <AgentIcon
              :type="agentsStore.getIcon(agent.agent_type)"
              :size="32"
            />
            <div class="agent-info">
              <span class="agent-name">{{ agent.display_name }}</span>
              <span class="agent-status">
                {{ agent.installed ? 'Detected' : 'Not Found' }}
              </span>
            </div>
            <CheckCircle2 v-if="agent.installed" class="status-icon success" />
            <XCircle v-else class="status-icon info" />
          </div>
        </div>
      </div>

      <!-- Step 3: Finish -->
      <div v-if="step === 3" class="step step-3">
        <div class="hero-icon success">
          <CheckCircle2 :size="64" />
        </div>
        <h2>All Set!</h2>
        <p>
          We detected <strong>{{ installedAgentsCount }}</strong> agents on your
          system. You're ready to start exploring the marketplace.
        </p>
      </div>
    </div>

    <template #footer>
      <div class="footer-actions">
        <div class="step-dots">
          <span
            v-for="s in steps"
            :key="s.id"
            class="dot"
            :class="{ active: step === s.id }"
          ></span>
        </div>
        <button class="btn-primary" @click="nextStep">
          <span>{{ step === 3 ? 'Get Started' : 'Continue' }}</span>
          <ArrowRight v-if="step < 3" :size="16" />
        </button>
      </div>
    </template>
  </Modal>
</template>

<style scoped>
.onboarding-content {
  text-align: center;
  padding: 10px 0;
}

.step {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  animation: fadeIn 0.3s ease;
}

.hero-icon {
  width: 96px;
  height: 96px;
  background-color: rgba(139, 92, 246, 0.1);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--accent-primary);
  margin-bottom: 8px;
}

.hero-icon.success {
  background-color: rgba(34, 197, 94, 0.1);
  color: var(--accent-success);
}

h2 {
  font-size: 24px;
  font-weight: 700;
  margin: 0;
}

p {
  color: var(--text-secondary);
  line-height: 1.6;
  max-width: 400px;
  margin: 0;
}

.features-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-top: 16px;
  text-align: left;
}

.feature-item {
  display: flex;
  align-items: center;
  gap: 12px;
  color: var(--text-primary);
}

.feature-icon {
  color: var(--accent-primary);
  width: 20px;
  height: 20px;
}

/* Step 2 Styles */
.loader-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  margin: 32px 0;
  color: var(--text-muted);
}

.loader {
  width: 32px;
  height: 32px;
  border: 3px solid var(--border-color);
  border-radius: 50%;
  border-top-color: var(--accent-primary);
  animation: spin 1s ease-in-out infinite;
}

.agents-results {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
  width: 100%;
  margin-top: 16px;
}

.agent-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background-color: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  opacity: 0.6;
}

.agent-card.installed {
  opacity: 1;
  border-color: var(--accent-primary);
  background-color: rgba(139, 92, 246, 0.05);
}

.agent-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
}

.agent-name {
  font-weight: 600;
  font-size: 14px;
}

.agent-status {
  font-size: 12px;
  color: var(--text-muted);
}

.status-icon {
  width: 18px;
  height: 18px;
}

.status-icon.success {
  color: var(--accent-success);
}

.status-icon.info {
  color: var(--text-muted);
}

/* Footer */
.footer-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.step-dots {
  display: flex;
  gap: 8px;
}

.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: var(--border-color);
  transition: all 0.3s;
}

.dot.active {
  background-color: var(--accent-primary);
  width: 16px;
  border-radius: 4px;
}

.btn-primary {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 24px;
  background-color: var(--accent-primary);
  color: white;
  border-radius: var(--border-radius);
  font-weight: 600;
  font-size: 14px;
  transition: all 0.2s;
}

.btn-primary:hover {
  filter: brightness(1.1);
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
