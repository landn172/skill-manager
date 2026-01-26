<script setup lang="ts">
import { ref, computed } from "vue";
import Modal from "@/components/common/Modal.vue";
import { useAgentsStore } from "@/stores/agents";
import AgentIcon from "@/components/icons/AgentIcon.vue";
import { CheckCircle2, Rocket, ArrowRight } from "lucide-vue-next";
import BaseButton from "@/components/common/BaseButton.vue";

defineProps<{
  show: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

const agentsStore = useAgentsStore();
const step = ref(1);

const steps = [
  { id: 1, title: "Welcome" },
  { id: 2, title: "Detect Agents" },
  { id: 3, title: "Ready!" },
];

async function nextStep() {
  if (step.value === 1) {
    step.value = 2;
    await agentsStore.fetchAgents();
  } else if (step.value === 2) {
    step.value = 3;
  } else {
    emit("close");
  }
}

const installedAgentsCount = computed(() => agentsStore.agents.filter((a) => a.installed).length);
</script>

<template>
  <Modal :show="show" title="Welcome to Skill Manager" :maxWidth="'600px'" @close="() => {}">
    <div class="onboarding-content">
      <!-- Step 1: Welcome -->
      <div v-if="step === 1" class="step step-1 animate-slide-up">
        <div class="hero-icon glass">
          <Rocket :size="48" />
        </div>
        <h2>Supercharge your AI Agents</h2>
        <p>
          Discover, install, and manage skills for your favorite coding
          assistants like Cursor, Claude Code, and more.
        </p>
        <div class="features-list">
          <div class="feature-item glass-card">
            <CheckCircle2 class="feature-icon" />
            <span>Browse 65,000+ skills from SkillsMP</span>
          </div>
          <div class="feature-item glass-card">
            <CheckCircle2 class="feature-icon" />
            <span>One-click install to multiple agents</span>
          </div>
          <div class="feature-item glass-card">
            <CheckCircle2 class="feature-icon" />
            <span>Manage local and Git-based skills</span>
          </div>
        </div>
      </div>

      <!-- Step 2: Agents -->
      <div v-if="step === 2" class="step step-2 animate-slide-up">
        <div class="scan-header">
          <h2>Detecting Agents</h2>
          <p>Scanning your system for supported AI tools...</p>
        </div>

        <div v-if="agentsStore.loading" class="loader-wrap">
          <div class="loader"></div>
          <p>Searching globally...</p>
        </div>

        <div v-else class="results-grid">
          <div
            v-for="agent in agentsStore.agents"
            :key="agent.agent_type"
            class="res-card glass-card"
            :class="{ active: agent.installed }"
          >
            <div class="icon-wrap">
              <AgentIcon :type="agentsStore.getIcon(agent.agent_type)" :size="24" />
            </div>
            <div class="info">
              <span class="name">{{ agent.display_name }}</span>
              <span class="status">{{ agent.installed ? 'Detected' : 'Not Found' }}</span>
            </div>
            <div class="check" v-if="agent.installed">
              <CheckCircle2 :size="16" />
            </div>
          </div>
        </div>
      </div>

      <!-- Step 3: Finish -->
      <div v-if="step === 3" class="step step-3 animate-slide-up">
        <div class="hero-icon success glass">
          <CheckCircle2 :size="48" />
        </div>
        <h2>All Systems Go!</h2>
        <p>
          Found <strong>{{ installedAgentsCount }}</strong> agents. You're ready to explore the marketplace and boost your productivity.
        </p>
      </div>
    </div>

    <template #footer>
      <div class="footer-wrap">
        <div class="indicators">
          <div 
            v-for="s in steps" 
            :key="s.id" 
            class="ind" 
            :class="{ active: step === s.id }"
          ></div>
        </div>
        <BaseButton variant="primary" @click="nextStep">
          {{ step === 3 ? "Explore Now" : "Continue" }}
          <ArrowRight v-if="step < 3" :size="16" />
        </BaseButton>
      </div>
    </template>
  </Modal>
</template>

<style scoped>
.onboarding-content {
  text-align: center;
}

.step {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
}

.hero-icon {
  width: 96px;
  height: 96px;
  border-radius: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--accent-primary);
  margin-bottom: 8px;
}

.hero-icon.success {
  color: var(--accent-success);
}

h2 {
  font-size: 26px;
  font-weight: 800;
  margin: 0;
  background: var(--gradient-primary);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

p {
  color: var(--text-secondary);
  line-height: 1.6;
  max-width: 440px;
  margin: 0;
}

.features-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  width: 100%;
  margin-top: 8px;
}

.feature-item {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 14px 20px;
  text-align: left;
}

.feature-icon {
  color: var(--accent-primary);
  flex-shrink: 0;
}

/* Step 2 */
.scan-header {
  margin-bottom: 8px;
}

.loader-wrap {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding: 40px 0;
}

.loader {
  width: 40px;
  height: 40px;
  border: 3px solid var(--border-color);
  border-top-color: var(--accent-primary);
  border-radius: 50%;
  animation: spin 1s infinite linear;
}

.results-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  width: 100%;
}

.res-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  opacity: 0.5;
  transition: all 0.3s;
}

.res-card.active {
  opacity: 1;
  border-color: var(--accent-primary);
}

.res-card .icon-wrap {
  width: 36px;
  height: 36px;
  background: var(--bg-tertiary);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.res-card .info {
  flex: 1;
  text-align: left;
  display: flex;
  flex-direction: column;
}

.res-card .name {
  font-weight: 700;
  font-size: 14px;
}

.res-card .status {
  font-size: 11px;
  color: var(--text-muted);
}

.res-card .check {
  color: var(--accent-success);
}

/* Footer */
.footer-wrap {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.indicators {
  display: flex;
  gap: 8px;
}

.ind {
  width: 8px;
  height: 8px;
  border-radius: 4px;
  background: var(--border-color);
  transition: all 0.3s;
}

.ind.active {
  width: 24px;
  background: var(--accent-primary);
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
