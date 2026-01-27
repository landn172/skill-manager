<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { Save, Eye, Code } from "lucide-vue-next";
import { invoke } from "@tauri-apps/api/core";
import { marked } from "marked";
import Modal from "@/components/common/Modal.vue";
import BaseButton from "@/components/common/BaseButton.vue";

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
  } catch (e) {
    alert(`Failed to save: ${e}`);
  } finally {
    saving.value = false;
  }
}

const hasChanges = computed(() => content.value !== originalContent.value);

const renderedMarkdown = computed(() => {
  return marked.parse(content.value);
});
</script>

<template>
  <Modal
    :show="true"
    :title="`Edit ${skillName}`"
    maxWidth="950px"
    @close="emit('close')"
  >
    <div class="editor-container">
      <div class="editor-toolbar">
        <div class="file-info">
          <Code :size="14" />
          <span>{{ filename }}</span>
          <span v-if="hasChanges" class="unsaved-badge">Modified</span>
        </div>

        <div class="view-toggle">
          <button
            :class="{ active: viewMode === 'edit' }"
            @click="viewMode = 'edit'"
          >
            <Code :size="16" />
            <span>Edit</span>
          </button>
          <button
            :class="{ active: viewMode === 'preview' }"
            @click="viewMode = 'preview'"
          >
            <Eye :size="16" />
            <span>Preview</span>
          </button>
        </div>
      </div>

      <div class="editor-body">
        <div v-if="loading" class="loading-state">
          <div class="loader"></div>
          <p>Loading content...</p>
        </div>

        <template v-else>
          <textarea
            v-if="viewMode === 'edit'"
            v-model="content"
            spellcheck="false"
            placeholder="Enter skill instructions here..."
            class="styled-textarea"
          ></textarea>

          <div
            v-else
            class="preview-area markdown-body"
            v-html="renderedMarkdown"
          ></div>
        </template>
      </div>
    </div>

    <template #footer>
      <div class="editor-footer">
        <BaseButton variant="ghost" @click="emit('close')" :disabled="saving">Cancel</BaseButton>
        <BaseButton
          variant="primary"
          :loading="saving"
          :disabled="!hasChanges || loading"
          @click="handleSave"
        >
          <Save :size="18" />
          <span>Save Changes</span>
        </BaseButton>
      </div>
    </template>
  </Modal>
</template>

<style scoped>
.editor-container {
  display: flex;
  flex-direction: column;
  gap: 16px;
  height: 60vh;
  min-height: 500px;
}

.editor-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 4px;
}

.file-info {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--text-secondary);
  font-family: 'JetBrains Mono', monospace;
}

.unsaved-badge {
  font-size: 10px;
  background: var(--accent-warning);
  color: #000;
  padding: 2px 6px;
  border-radius: 4px;
  font-weight: 700;
  text-transform: uppercase;
}

.view-toggle {
  display: flex;
  background: var(--bg-tertiary);
  padding: 3px;
  border-radius: 10px;
  border: 1px solid var(--border-color);
}

.view-toggle button {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 16px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-muted);
  transition: all 0.2s;
}

.view-toggle button:hover {
  color: var(--text-primary);
}

.view-toggle button.active {
  background: var(--bg-primary);
  color: var(--accent-primary);
  box-shadow: var(--shadow-sm);
}

.editor-body {
  flex: 1;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  overflow: hidden;
  position: relative;
  min-height: 400px;
  display: flex;
  flex-direction: column;
}

.styled-textarea {
  width: 100%;
  flex: 1;
  background: transparent;
  border: none;
  border-radius: 12px;
  outline: none;
  padding: 24px;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 14px;
  line-height: 1.62;
  color: var(--text-primary);
  resize: none;
}

.preview-area {
  padding: 32px;
  height: 100%;
  overflow-y: auto;
  background: var(--bg-primary);
}

/* Markdown Styles */
.markdown-body {
  font-family: var(--font-family);
  line-height: 1.6;
  color: var(--text-primary);
}

.markdown-body :deep(h1),
.markdown-body :deep(h2),
.markdown-body :deep(h3) {
  margin-top: 24px;
  margin-bottom: 16px;
  font-weight: 600;
  line-height: 1.25;
}

.markdown-body :deep(h1) { font-size: 1.5em; border-bottom: 1px solid var(--border-color); padding-bottom: 0.3em; }
.markdown-body :deep(h2) { font-size: 1.25em; border-bottom: 1px solid var(--border-color); padding-bottom: 0.3em; }
.markdown-body :deep(h3) { font-size: 1.1em; }

.markdown-body :deep(p) { margin-bottom: 16px; }

.markdown-body :deep(code) {
  padding: 0.2em 0.4em;
  margin: 0;
  font-size: 85%;
  background-color: var(--bg-hover);
  border-radius: 6px;
  font-family: 'JetBrains Mono', monospace;
}

.markdown-body :deep(pre) {
  padding: 16px;
  overflow: auto;
  font-size: 85%;
  line-height: 1.45;
  background-color: var(--bg-secondary);
  border-radius: 12px;
  margin-bottom: 16px;
  border: 1px solid var(--border-color);
}

.markdown-body :deep(pre code) {
  padding: 0;
  background: none;
  font-size: 100%;
}

.markdown-body :deep(ul),
.markdown-body :deep(ol) {
  padding-left: 2em;
  margin-bottom: 16px;
}

.markdown-body :deep(blockquote) {
  padding: 0 1em;
  color: var(--text-muted);
  border-left: 0.25em solid var(--accent-primary);
  margin: 0 0 16px 0;
}

.markdown-body :deep(hr) {
  height: 1px;
  padding: 0;
  margin: 24px 0;
  background-color: var(--border-color);
  border: 0;
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 16px;
  color: var(--text-muted);
}

.loader {
  width: 32px;
  height: 32px;
  border: 3px solid var(--border-color);
  border-top-color: var(--accent-primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

.editor-footer {
  display: flex;
  gap: 12px;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
