<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useMarketplaceStore } from "@/stores/marketplace";
import { Globe, Plus, Trash2, Folder } from "lucide-vue-next";
import BaseButton from "@/components/common/BaseButton.vue";
import Modal from "@/components/common/Modal.vue";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { homeDir } from "@tauri-apps/api/path";

const marketplaceStore = useMarketplaceStore();

onMounted(marketplaceStore.fetchSources);

const showAddRegistryModal = ref(false);
const registryUrl = ref("");
const registryName = ref("");
const addingRegistry = ref(false);

async function saveRegistrySource() {
  if (!registryUrl.value.trim() || !registryName.value.trim()) return;

  addingRegistry.value = true;
  try {
    await marketplaceStore.addSource(registryUrl.value.trim(), registryName.value.trim());
    showAddRegistryModal.value = false;
    registryUrl.value = "";
    registryName.value = "";
  } catch (e) {
    alert(`Failed to add registry source: ${e}`);
  } finally {
    addingRegistry.value = false;
  }
}

async function removeSource(id: string) {
  if (!confirm("Are you sure you want to remove this source?")) return;
  try {
    await marketplaceStore.removeSource(id);
  } catch (e) {
    alert(`Failed to remove source: ${e}`);
  }
}

async function toggleSource(id: string, event: Event) {
  const checkbox = event.target as HTMLInputElement;
  try {
    await marketplaceStore.toggleSource(id, checkbox.checked);
  } catch (e) {
    alert(`Failed to toggle source: ${e}`);
    checkbox.checked = !checkbox.checked; // Revert
  }
}

async function handleAddLocalSource() {
  try {
    const selected = await openDialog({
      directory: true,
      multiple: false,
      defaultPath: await homeDir(),
    });

    if (selected && typeof selected === "string") {
      const name = selected.split("/").pop() || "Local Skills";
      await marketplaceStore.addSource(selected, name, "local");
      await marketplaceStore.fetchSkills();
    }
  } catch (e) {
    alert(`Failed to add local source: ${e}`);
  }
}
</script>

<template>
  <section class="section">
    <div class="section-title">
      <Globe :size="20" class="icon" />
      <h2>Marketplace Sources</h2>
    </div>
    <p class="section-hint">Repositories where you find and update skills.</p>

    <div class="list-container">
      <div
        v-for="source in marketplaceStore.sources"
        :key="source.id"
        class="item-card glass-card"
        :class="{ disabled: !source.enabled }"
      >
        <div class="item-info">
          <div class="item-text">
            <div class="item-header-row">
              <span class="item-name">{{ source.name }}</span>
              <span v-if="source.official" class="badge official">Official</span>
              <span v-if="source.source_type === 'api'" class="badge api">API</span>
              <span v-if="source.source_type === 'registry'" class="badge registry">Registry</span>
            </div>
            <span class="item-subtext">{{ source.url }}</span>
          </div>
        </div>
        
        <div class="item-actions">
          <label class="switch">
            <input
              type="checkbox"
              :checked="source.enabled"
              @change="(e) => toggleSource(source.id, e)"
            />
            <span class="toggle-slider"></span>
          </label>

          <BaseButton
            v-if="!source.official && source.source_type !== 'api'"
            variant="ghost"
            size="icon"
            class="danger-ghost"
            @click="removeSource(source.id)"
          >
            <Trash2 :size="16" />
          </BaseButton>
        </div>
      </div>

      <div class="two-cols">
        <BaseButton variant="outline" class="dashed" @click="showAddRegistryModal = true">
          <Plus :size="18" />
          Add Registry
        </BaseButton>
        <BaseButton variant="outline" class="dashed" @click="handleAddLocalSource">
          <Folder :size="18" />
          Add Local Folder
        </BaseButton>
      </div>
    </div>

    <Modal
      :show="showAddRegistryModal"
      title="Add Registry Source"
      @close="showAddRegistryModal = false"
    >
      <div class="modal-form">
        <div class="form-item">
          <label>Registry Name</label>
          <input v-model="registryName" placeholder="e.g. Team Registry" class="styled-input" />
        </div>
        <div class="form-item">
          <label>Registry JSON URL</label>
          <input v-model="registryUrl" placeholder="https://example.com/registry.json" class="styled-input" />
        </div>
      </div>
      <template #footer>
        <BaseButton variant="ghost" @click="showAddRegistryModal = false">Cancel</BaseButton>
        <BaseButton variant="primary" :disabled="!registryName || !registryUrl" :loading="addingRegistry" @click="saveRegistrySource">Add Source</BaseButton>
      </template>
    </Modal>
  </section>
</template>

<style scoped>
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

.item-card.disabled {
  opacity: 0.6;
}

.item-info {
  display: flex;
  align-items: center;
  gap: 16px;
  min-width: 0;
}

.item-text {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.item-header-row {
  display: flex;
  align-items: center;
  gap: 8px;
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
  max-width: 500px;
}

.badge {
  font-size: 10px;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 4px;
  text-transform: uppercase;
}

.badge.official { background: rgba(139, 92, 246, 0.1); color: var(--accent-primary); }
.badge.api { background: rgba(99, 102, 241, 0.1); color: var(--accent-secondary); }
.badge.registry { background: rgba(34, 197, 94, 0.1); color: var(--accent-success); }

.item-actions {
  display: flex;
  align-items: center;
  gap: 16px;
}

.danger-ghost {
  color: var(--accent-error);
}

.switch {
  position: relative;
  display: inline-block;
  width: 36px;
  height: 20px;
}

.switch input { opacity: 0; width: 0; height: 0; }

.toggle-slider {
  position: absolute;
  cursor: pointer;
  top: 0; left: 0; right: 0; bottom: 0;
  background-color: var(--bg-tertiary);
  transition: .3s;
  border-radius: 20px;
  border: 1px solid var(--border-color);
}

.toggle-slider:before {
  position: absolute;
  content: "";
  height: 14px; width: 14px;
  left: 2px; bottom: 2px;
  background-color: var(--text-muted);
  transition: .3s;
  border-radius: 50%;
}

input:checked + .toggle-slider {
  background-color: var(--accent-primary);
  border-color: var(--accent-primary);
}

input:checked + .toggle-slider:before {
  transform: translateX(16px);
  background-color: white;
}

.two-cols {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.dashed {
  border-style: dashed;
  border-width: 2px;
}

.modal-form { display: flex; flex-direction: column; gap: 20px; }
.form-item { display: flex; flex-direction: column; gap: 8px; }
.form-item label { font-size: 13px; font-weight: 700; color: var(--text-secondary); }
.styled-input {
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  padding: 10px 16px;
  border-radius: 10px;
  color: var(--text-primary);
  outline: none;
}
</style>
