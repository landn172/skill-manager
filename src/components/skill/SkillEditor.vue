<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { Save, X, Eye, Code } from "lucide-vue-next";
import { invoke } from "@tauri-apps/api/core";

const props = defineProps<{
  skillPath: string;
  skillName: string;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "save"): void;
}>();

const content = ref("");
const originalContent = ref("");
const filename = ref("SKILL.md");
const loading = ref(true);
const saving = ref(false);
const viewMode = ref<"edit" | "preview">("edit");

onMounted(async () => {
  try {
    const result = await invoke<{ content: string; filename: string }>("get_skill_content", {
      skillPath: props.skillPath,
    });
    content.value = result.content;
    filename.value = result.filename;
    originalContent.value = content.value;
  } catch (e) {
    alert(`Failed to load skill content: ${e}`);
  } finally {
    loading.value = false;
  }
});

async function handleSave() {
  saving.value = true;
  try {
    await invoke("save_skill_content", {
      skillPath: props.skillPath,
      content: content.value,
      filename: filename.value,
    });
    originalContent.value = content.value;
    emit("save");
    alert("Skill saved successfully!");
  } catch (e) {
    alert(`Failed to save: ${e}`);
  } finally {
    saving.value = false;
  }
}

const hasChanges = computed(() => content.value !== originalContent.value);
</script>

<template>
  <div class="skill-editor">
    <header class="editor-header">
      <div class="editor-title">
        <span class="skill-name">{{ skillName }}</span>
        <span class="file-path">{{ filename }}</span>
        <span v-if="hasChanges" class="unsaved-dot"></span>
      </div>

      <div class="editor-actions">
        <div class="view-toggle">
          <button
            :class="{ active: viewMode === 'edit' }"
            @click="viewMode = 'edit'"
            title="Edit Mode"
          >
            <Code :size="16" />
          </button>
          <button
            :class="{ active: viewMode === 'preview' }"
            @click="viewMode = 'preview'"
            title="Preview Mode"
          >
            <Eye :size="16" />
          </button>
        </div>

        <button class="save-btn" :disabled="!hasChanges || saving" @click="handleSave">
          <Save :size="18" />
          <span>{{ saving ? "Saving..." : "Save" }}</span>
        </button>

        <button class="close-btn" @click="emit('close')">
          <X :size="20" />
        </button>
      </div>
    </header>

    <div class="editor-body">
      <div v-if="loading" class="loading-overlay">
        <div class="loader"></div>
      </div>

      <template v-else>
        <textarea
          v-if="viewMode === 'edit'"
          v-model="content"
          spellcheck="false"
          placeholder="Enter skill instructions here..."
        ></textarea>

        <div v-else class="preview-area">
          <!-- Simple markdown preview logic could go here, for now just pre -->
          <pre>{{ content }}</pre>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.skill-editor {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background-color: var(--bg-primary);
  display: flex;
  flex-direction: column;
  z-index: 200;
}

.editor-header {
  height: 60px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 24px;
  border-bottom: 1px solid var(--border-color);
  background-color: var(--bg-secondary);
}

.editor-title {
  display: flex;
  align-items: center;
  gap: 12px;
}

.skill-name {
  font-weight: 600;
}

.file-path {
  font-size: 13px;
  color: var(--text-muted);
}

.unsaved-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: var(--accent-warning);
}

.editor-actions {
  display: flex;
  align-items: center;
  gap: 16px;
}

.view-toggle {
  display: flex;
  background-color: var(--bg-tertiary);
  padding: 2px;
  border-radius: 6px;
}

.view-toggle button {
  padding: 6px 12px;
  border-radius: 4px;
  color: var(--text-muted);
}

.view-toggle button.active {
  background-color: var(--bg-secondary);
  color: var(--accent-primary);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

.save-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  background-color: var(--accent-primary);
  color: white;
  padding: 8px 16px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
}

.save-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  filter: grayscale(1);
}

.close-btn {
  padding: 8px;
  border-radius: 8px;
  color: var(--text-muted);
}

.close-btn:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

.editor-body {
  flex: 1;
  position: relative;
  overflow: hidden;
}

textarea {
  width: 100%;
  height: 100%;
  background: none;
  border: none;
  outline: none;
  padding: 32px;
  font-family: "Fira Code", "Courier New", Courier, monospace;
  font-size: 15px;
  line-height: 1.6;
  resize: none;
  color: var(--text-primary);
}

.preview-area {
  width: 100%;
  height: 100%;
  padding: 32px;
  overflow-y: auto;
}

pre {
  white-space: pre-wrap;
  word-wrap: break-word;
  font-size: 14px;
  line-height: 1.6;
  color: var(--text-secondary);
}

.loading-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--bg-primary);
}

.loader {
  width: 40px;
  height: 40px;
  border: 3px solid var(--border-color);
  border-radius: 50%;
  border-top-color: var(--accent-primary);
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
