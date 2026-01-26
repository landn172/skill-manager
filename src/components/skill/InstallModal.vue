<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { CheckCircle2 } from "lucide-vue-next";
import { useAgentsStore } from "@/stores/agents";
import Modal from "@/components/common/Modal.vue";
import AgentIcon from "../icons/AgentIcon.vue";
import BaseButton from "@/components/common/BaseButton.vue";
import type { Skill } from "@/types";

interface Props {
  show: boolean;
  skill: Skill | null;
}

const props = defineProps<Props>();
const emit = defineEmits(["close", "success"]);

const agentsStore = useAgentsStore();

const selectedAgents = ref<string[]>([]);
const installScope = ref<"project" | "global">("global");
const installing = ref(false);
const installLogs = ref<
  Array<{ time: string; message: string; type: "info" | "error" | "success" }>
>([]);

// Initialize selected agents when modal opens
watch(
  () => props.show,
  (show) => {
    if (show && props.skill) {
      installLogs.value = [];
      // Default to all installed agents for Marketplace, or uninstalled ones for Supplemental
      const installedFor = (props.skill as any).agents || [];
      selectedAgents.value = agentsStore.agents
        .filter((a) => a.installed && !installedFor.includes(a.agent_type))
        .map((a) => a.agent_type);
      
      // If already installed on all, or a fresh install from market
      if (selectedAgents.value.length === 0) {
        selectedAgents.value = agentsStore.agents
          .filter((a) => a.installed)
          .map((a) => a.agent_type);
      }
    }
  }
);

function getLogTime() {
  return new Date().toLocaleTimeString("en-US", {
    hour12: false,
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

async function handleInstall() {
  if (!props.skill || selectedAgents.value.length === 0) return;

  installing.value = true;
  installLogs.value = [];
  installLogs.value.push({
    time: getLogTime(),
    message: "Initializing installation...",
    type: "info",
  });

  let unlisten: (() => void) | undefined;

  try {
    unlisten = await listen<{
      skill: string;
      status: string;
      message: string;
      agent?: string;
    }>("install-progress", (event) => {
      const type =
        event.payload.status === "error"
          ? "error"
          : event.payload.status === "finished"
            ? "success"
            : "info";
      installLogs.value.push({
        time: getLogTime(),
        message: event.payload.message,
        type,
      });

      const terminal = document.getElementById("install-terminal");
      if (terminal) {
        setTimeout(() => {
          terminal.scrollTop = terminal.scrollHeight;
        }, 10);
      }
    });

    const results = await invoke<
      Array<{
        success: boolean;
        path: string;
        agent: string;
        error?: string;
      }>
    >("install_skill", {
      skill: props.skill,
      agents: selectedAgents.value,
      scope: installScope.value,
    });

    const successful = results.filter((r) => r.success);
    const failed = results.filter((r) => !r.success);

    if (failed.length > 0) {
      const errors = failed.map((f) => `${f.agent}: ${f.error}`).join("\n");
      installLogs.value.push({
        time: getLogTime(),
        message: `Some installations failed: ${errors}`,
        type: "error",
      });
    }

    if (successful.length > 0) {
      installLogs.value.push({
        time: getLogTime(),
        message: "Installation completed successfully.",
        type: "success",
      });

      await new Promise((resolve) => setTimeout(resolve, 1500));
      emit("success");
      emit("close");
    }
  } catch (e) {
    installLogs.value.push({
      time: getLogTime(),
      message: `Installation failed: ${e}`,
      type: "error",
    });
  } finally {
    if (unlisten) unlisten();
    installing.value = false;
  }
}

function toggleAgent(agentType: string) {
  if (selectedAgents.value.includes(agentType)) {
    selectedAgents.value = selectedAgents.value.filter((a) => a !== agentType);
  } else {
    selectedAgents.value.push(agentType);
  }
}
</script>

<template>
  <Modal
    :show="show"
    :title="`Install ${skill?.name}`"
    maxWidth="850px"
    @close="emit('close')"
  >
    <div class="install-container">
      <div class="setup-section">
        <section class="config-group">
          <label class="group-label">Select Agents</label>
          <div class="agents-grid">
            <div
              v-for="agent in agentsStore.agents"
              :key="agent.agent_type"
              class="agent-card"
              :class="{
                selected: selectedAgents.includes(agent.agent_type),
                disabled: !agent.installed,
              }"
              @click="agent.installed && toggleAgent(agent.agent_type)"
            >
              <div class="agent-info">
                <AgentIcon
                  :type="agentsStore.getIcon(agent.agent_type)"
                  :size="20"
                  class="icon"
                />
                <span class="name">{{ agent.display_name }}</span>
              </div>
              <div v-if="selectedAgents.includes(agent.agent_type)" class="check">
                <CheckCircle2 :size="16" />
              </div>
            </div>
          </div>
        </section>

        <section class="config-group">
          <label class="group-label">Installation Scope</label>
          <div class="scope-picker">
            <button
              class="scope-opt"
              :class="{ active: installScope === 'project' }"
              @click="installScope = 'project'"
            >
              Project
            </button>
            <button
              class="scope-opt"
              :class="{ active: installScope === 'global' }"
              @click="installScope = 'global'"
            >
              Global
            </button>
          </div>
          <p class="scope-hint">
            {{ installScope === 'project' ? 'Available only in current project.' : 'Available across all projects.' }}
          </p>
        </section>
      </div>

      <div class="terminal-section">
        <div class="terminal-head">
          <div class="dots">
            <span class="dot red"></span>
            <span class="dot yellow"></span>
            <span class="dot green"></span>
          </div>
          <span class="label">INSTALLATION LOGS</span>
        </div>
        <div id="install-terminal" class="terminal-body">
          <div v-if="installLogs.length === 0" class="placeholder">
            Waiting for installation to start...
          </div>
          <div
            v-for="(log, idx) in installLogs"
            :key="idx"
            class="log-line"
            :class="log.type"
          >
            <span class="time">[{{ log.time }}]</span>
            <span class="msg">{{ log.message }}</span>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="modal-footer">
        <BaseButton variant="ghost" @click="emit('close')" :disabled="installing">Cancel</BaseButton>
        <BaseButton
          variant="primary"
          :loading="installing"
          :disabled="selectedAgents.length === 0"
          @click="handleInstall"
        >
          {{ installing ? "Installing..." : "Install Skill" }}
        </BaseButton>
      </div>
    </template>
  </Modal>
</template>

<style scoped>
.install-container {
  display: grid;
  grid-template-columns: 1fr 1.2fr;
  gap: 32px;
  min-height: 440px;
}

.setup-section {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.config-group {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.group-label {
  font-size: 13px;
  font-weight: 700;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.agents-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 10px;
}

.agent-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.agent-card:hover:not(.disabled) {
  border-color: var(--accent-primary);
  background: var(--bg-hover);
}

.agent-card.selected {
  border-color: var(--accent-primary);
  background: rgba(139, 92, 246, 0.05);
}

.agent-card.disabled {
  opacity: 0.5;
  cursor: not-allowed;
  filter: grayscale(1);
}

.agent-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.agent-info .name {
  font-weight: 500;
  font-size: 14px;
}

.check {
  color: var(--accent-primary);
}

.scope-picker {
  display: flex;
  background: var(--bg-tertiary);
  padding: 4px;
  border-radius: 12px;
  border: 1px solid var(--border-color);
}

.scope-opt {
  flex: 1;
  padding: 10px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
  transition: all 0.2s;
}

.scope-opt.active {
  background: var(--bg-primary);
  color: var(--accent-primary);
  box-shadow: var(--shadow-sm);
}

.scope-hint {
  font-size: 12px;
  color: var(--text-muted);
  margin: 0;
}

/* Terminal Styles */
.terminal-section {
  display: flex;
  flex-direction: column;
  background: #0d0d0f;
  border-radius: 16px;
  border: 1px solid #27272a;
  overflow: hidden;
}

.terminal-head {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 16px;
  background: #18181b;
  border-bottom: 1px solid #27272a;
}

.dots {
  display: flex;
  gap: 6px;
}

.dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
}

.dot.red { background: #ff5f56; }
.dot.yellow { background: #ffbd2e; }
.dot.green { background: #27c93f; }

.terminal-head .label {
  font-family: inherit;
  font-size: 11px;
  font-weight: 700;
  color: #71717a;
  letter-spacing: 0.1em;
}

.terminal-body {
  flex: 1;
  padding: 16px;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 12px;
  line-height: 1.6;
  color: #e4e4e7;
  overflow-y: auto;
  max-height: 400px;
}

.placeholder {
  color: #52525b;
  font-style: italic;
}

.log-line {
  display: flex;
  gap: 12px;
  margin-bottom: 4px;
}

.log-line.error { color: #f87171; }
.log-line.success { color: #4ade80; }
.log-line.info { color: #60a5fa; }

.time {
  color: #52525b;
  flex-shrink: 0;
}

.msg {
  word-break: break-all;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  width: 100%;
}
</style>
