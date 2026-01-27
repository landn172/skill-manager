<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useAgentsStore } from "@/stores/agents";
import { ShieldCheck, CheckCircle2, XCircle, Edit2, Trash2, Plus } from "lucide-vue-next";
import AgentIcon from "@/components/icons/AgentIcon.vue";
import BaseButton from "@/components/common/BaseButton.vue";
import Modal from "@/components/common/Modal.vue";
import { invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { homeDir } from "@tauri-apps/api/path";
import { readFile } from "@tauri-apps/plugin-fs";

const agentsStore = useAgentsStore();

onMounted(agentsStore.fetchAgents);

// Agent Path Management
const editingAgent = ref<any>(null);
const editAgentPath = ref("");
const savingAgentPath = ref(false);

function openEditAgent(agent: any) {
  editingAgent.value = agent;
  editAgentPath.value = agent.global_skills_dir;
}

async function saveAgentPath() {
  if (!editingAgent.value) return;

  savingAgentPath.value = true;
  try {
    await invoke("update_agent_path", {
      agentType: editingAgent.value.agent_type,
      path: editAgentPath.value,
    });

    await agentsStore.fetchAgents();
    editingAgent.value = null;
    editAgentPath.value = "";
  } catch (e) {
    alert(`Failed to update agent path: ${e}`);
  } finally {
    savingAgentPath.value = false;
  }
}

async function browseForAgentPath() {
  try {
    const selected = await openDialog({
      directory: true,
      multiple: false,
      defaultPath: await homeDir(),
    });

    if (selected && typeof selected === "string") {
      editAgentPath.value = selected;
    }
  } catch (e) {
    console.error(e);
  }
}

// Custom Agent Management
const showAddAgentModal = ref(false);
const newAgentName = ref("");
const newAgentPath = ref("");
const newAgentIconType = ref<"emoji" | "image">("emoji");
const newAgentIcon = ref("🚀");
const addingAgent = ref(false);

async function pickIconImage() {
  try {
    const selected = await openDialog({
      multiple: false,
      filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "svg", "webp"] }],
    });
    if (selected && typeof selected === "string") {
      const data = await readFile(selected);
      const blob = new Blob([data]);
      const reader = new FileReader();
      reader.onload = () => {
        newAgentIcon.value = reader.result as string;
      };
      reader.readAsDataURL(blob);
    }
  } catch (e) {
    console.error("Failed to pick icon image", e);
  }
}

async function addCustomAgent() {
  if (!newAgentName.value || !newAgentPath.value) return;

  addingAgent.value = true;
  try {
    await invoke("add_custom_agent", {
      name: newAgentName.value,
      path: newAgentPath.value,
      icon: newAgentIcon.value,
    });

    await agentsStore.fetchAgents();
    showAddAgentModal.value = false;
    newAgentName.value = "";
    newAgentPath.value = "";
    newAgentIcon.value = "🚀";
    newAgentIconType.value = "emoji";
  } catch (e) {
    alert(`Failed to add agent: ${e}`);
  } finally {
    addingAgent.value = false;
  }
}

const showDeleteModal = ref(false);
const agentToDelete = ref<any>(null);
const deletingAgent = ref(false);

function confirmRemoveAgent(agent: any) {
  agentToDelete.value = agent;
  showDeleteModal.value = true;
}

async function handleDeleteAgent() {
  if (!agentToDelete.value) return;
  
  deletingAgent.value = true;
  try {
    await invoke("remove_custom_agent", {
      agentType: agentToDelete.value.agent_type,
    });
    await agentsStore.fetchAgents();
    showDeleteModal.value = false;
    agentToDelete.value = null;
  } catch (e) {
    alert(`Failed to remove agent: ${e}`);
  } finally {
    deletingAgent.value = false;
  }
}

async function browseForNewAgentPath() {
  try {
    const selected = await openDialog({
      directory: true,
      multiple: false,
      defaultPath: await homeDir(),
    });

    if (selected && typeof selected === "string") {
      newAgentPath.value = selected;
    }
  } catch (e) {
    console.error(e);
  }
}
</script>

<template>
  <section class="section">
    <div class="section-title">
      <ShieldCheck :size="20" class="icon" />
      <h2>Detected Agents</h2>
    </div>
    <p class="section-hint">Manage the IDEs and tools where skills can be installed.</p>

    <div class="list-container">
      <div v-for="agent in agentsStore.agents" :key="agent.name" class="item-card glass-card">
        <div class="item-info">
          <div class="item-icon-box">
            <AgentIcon :type="agentsStore.getIcon(agent.agent_type)" :size="20" />
          </div>
          <div class="item-text">
            <span class="item-name">{{ agent.display_name }}</span>
            <span class="item-subtext">{{ agent.global_skills_dir }}</span>
          </div>
        </div>
        
        <div class="item-actions">
          <div class="status-indicator" :class="{ detected: agent.installed }">
            <CheckCircle2 v-if="agent.installed" :size="14" />
            <XCircle v-else :size="14" />
            <span>{{ agent.installed ? 'Detected' : 'Missing' }}</span>
          </div>
          
          <BaseButton variant="ghost" size="icon" @click="openEditAgent(agent)" title="Edit Directory">
            <Edit2 :size="16" />
          </BaseButton>
          
          <BaseButton
            v-if="agent.is_custom"
            variant="ghost"
            size="icon"
            class="danger-ghost"
            title="Remove Custom Agent"
            @click="confirmRemoveAgent(agent)"
          >
            <Trash2 :size="16" />
          </BaseButton>
        </div>
      </div>
      
      <BaseButton variant="outline" class="w-full dashed" @click="showAddAgentModal = true">
        <Plus :size="18" />
        Add Custom Agent
      </BaseButton>
    </div>

    <!-- Modals -->
    <Modal
      :show="showAddAgentModal"
      title="Add Custom Agent"
      @close="showAddAgentModal = false"
    >
      <div class="modal-form">
        <div class="form-item">
          <label>Agent Name</label>
          <input v-model="newAgentName" placeholder="e.g. Cursor IDE" class="styled-input" />
        </div>

        <div class="form-item">
          <label>Icon</label>
          <div class="icon-picker-box">
            <div class="picker-tabs">
              <button :class="{ active: newAgentIconType === 'emoji' }" @click="newAgentIconType = 'emoji'">Emoji</button>
              <button :class="{ active: newAgentIconType === 'image' }" @click="newAgentIconType = 'image'">Image</button>
            </div>
            <div class="picker-field">
              <div class="preview-box">
                <AgentIcon :type="newAgentIcon" :size="24" />
              </div>
              <input v-if="newAgentIconType === 'emoji'" v-model="newAgentIcon" class="styled-input" maxlength="2" />
              <BaseButton v-else variant="outline" size="sm" @click="pickIconImage">
                {{ newAgentIcon.startsWith('data:') ? 'Change' : 'Select' }} Image
              </BaseButton>
            </div>
          </div>
        </div>

        <div class="form-item">
          <label>Skills Path</label>
          <div class="row">
            <input v-model="newAgentPath" class="styled-input" placeholder="/path/to/global/skills" />
            <BaseButton variant="outline" size="md" @click="browseForNewAgentPath">Browse</BaseButton>
          </div>
        </div>
      </div>
      <template #footer>
        <BaseButton variant="ghost" @click="showAddAgentModal = false">Cancel</BaseButton>
        <BaseButton variant="primary" :disabled="!newAgentName || !newAgentPath" :loading="addingAgent" @click="addCustomAgent">Add Agent</BaseButton>
      </template>
    </Modal>

    <Modal
      :show="!!editingAgent"
      :title="`Edit ${editingAgent?.display_name} Path`"
      @close="editingAgent = null"
    >
      <div class="modal-form">
        <div class="form-item">
          <label>Skills Directory Path</label>
          <div class="row">
            <input v-model="editAgentPath" class="styled-input" />
            <BaseButton variant="outline" size="md" @click="browseForAgentPath">Browse</BaseButton>
          </div>
        </div>
      </div>
      <template #footer>
        <BaseButton variant="ghost" @click="editingAgent = null">Cancel</BaseButton>
        <BaseButton variant="primary" :loading="savingAgentPath" @click="saveAgentPath">Save Path</BaseButton>
      </template>
    </Modal>

    <!-- Delete Confirmation Modal -->
    <Modal
      :show="showDeleteModal"
      title="Remove Agent"
      @close="showDeleteModal = false"
    >
      <div class="confirm-content">
        <p>Are you sure you want to remove <strong>{{ agentToDelete?.display_name }}</strong>?</p>
        <p class="sub-text">This will only remove it from Skill Manager. Your local files and configuration for the agent will remain untouched.</p>
      </div>
      <template #footer>
        <BaseButton variant="ghost" @click="showDeleteModal = false">Cancel</BaseButton>
        <BaseButton variant="primary" class="danger-btn" :loading="deletingAgent" @click="handleDeleteAgent">Remove</BaseButton>
      </template>
    </Modal>
  </section>
</template>

<style scoped>
.confirm-content {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.confirm-content strong {
  color: var(--text-primary);
}

.sub-text {
  font-size: 13px;
  color: var(--text-muted);
  line-height: 1.5;
}

.danger-btn {
  background-color: var(--accent-error) !important;
  color: white !important;
  border: none !important;
}

.danger-btn:hover {
  background-color: #dc2626 !important; /* Red-600 */
}

.section {
  display: flex;
  flex-direction: column;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}

.section-title h2 {
  font-size: 20px;
  font-weight: 700;
  margin: 0;
}

.section-title .icon {
  color: var(--accent-primary);
}

.section-hint {
  font-size: 14px;
  color: var(--text-secondary);
  margin: 0 0 24px;
}

.list-container {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.item-card {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
}

.item-info {
  display: flex;
  align-items: center;
  gap: 16px;
  min-width: 0; 
}

.item-icon-box {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-tertiary);
  border-radius: 10px;
}

.item-text {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.item-name {
  font-weight: 700;
  font-size: 15px;
  color: var(--text-primary);
}

.item-subtext {
  font-size: 12px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 400px;
}

.item-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  font-weight: 600;
  padding: 4px 10px;
  border-radius: 20px;
  background: rgba(239, 68, 68, 0.1);
  color: var(--accent-error);
}

.status-indicator.detected {
  background: rgba(34, 197, 94, 0.1);
  color: var(--accent-success);
}

.danger-ghost {
  color: var(--accent-error);
}

.danger-ghost:hover {
  background: rgba(239, 68, 68, 0.1);
}

.dashed {
  border-style: dashed;
  border-width: 2px;
}

.w-full { width: 100%; }

/* Modal Form Styles */
.modal-form {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.form-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-item label {
  font-size: 13px;
  font-weight: 700;
  color: var(--text-secondary);
}

.row {
  display: flex;
  gap: 12px;
}

.styled-input {
  flex: 1;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  padding: 10px 16px;
  border-radius: 10px;
  color: var(--text-primary);
  outline: none;
}

.icon-picker-box {
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  overflow: hidden;
}

.picker-tabs {
  display: flex;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
}

.picker-tabs button {
  flex: 1;
  padding: 8px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
}

.picker-tabs button.active {
  color: var(--accent-primary);
  background: var(--bg-tertiary);
}

.picker-field {
  padding: 16px;
  display: flex;
  align-items: center;
  gap: 12px;
}

.preview-box {
  width: 44px;
  height: 44px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
