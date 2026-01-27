<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Key, Edit2, Trash2, ExternalLink } from "lucide-vue-next";
import BaseButton from "@/components/common/BaseButton.vue";
import { openUrl } from "@tauri-apps/plugin-opener";

const apiKeyInput = ref("");
const maskedApiKey = ref<string | null>(null);
const apiKeySource = ref<string | null>(null); // 'env' or 'db'
const savingApiKey = ref(false);
const isEditingApiKey = ref(false);

const hasApiKey = computed(() => !!maskedApiKey.value);
const isFromEnv = computed(() => apiKeySource.value === "env");

async function loadApiKey() {
  try {
    maskedApiKey.value = await invoke<string | null>("get_skillsmp_api_key_masked");
    apiKeySource.value = await invoke<string | null>("get_skillsmp_api_key_source");
  } catch (e) {
    console.error("Failed to load API key", e);
  }
}

async function startEditingApiKey() {
  try {
    const fullKey = await invoke<string | null>("get_skillsmp_api_key");
    apiKeyInput.value = fullKey || "";
    isEditingApiKey.value = true;
  } catch (e) {
    console.error("Failed to fetch full API key", e);
    isEditingApiKey.value = true;
  }
}

async function saveApiKey() {
  if (!apiKeyInput.value.trim()) return;

  savingApiKey.value = true;
  try {
    await invoke("set_skillsmp_api_key", { key: apiKeyInput.value.trim() });
    apiKeyInput.value = "";
    isEditingApiKey.value = false;
    await loadApiKey();
  } catch (e) {
    alert(`Failed to save API key: ${e}`);
  } finally {
    savingApiKey.value = false;
  }
}

async function clearApiKey() {
  if (!confirm("Are you sure you want to remove your SkillsMP API key from Settings?")) return;

  try {
    await invoke("clear_skillsmp_api_key");
    await loadApiKey();
  } catch (e) {
    alert(`Failed to clear API key: ${e}`);
  }
}

async function handleOpenUrl(url: string) {
  try {
    await openUrl(url);
  } catch (e) {
    console.error("Failed to open URL", e);
    window.open(url, "_blank");
  }
}

onMounted(loadApiKey);
</script>

<template>
  <section class="section">
    <div class="section-title">
      <Key :size="20" class="icon" />
      <h2>SkillsMP API</h2>
    </div>
    <p class="section-hint">
      Configure your SkillsMP API key to discover over 65,000 skills.
    </p>

    <div class="glass-card api-box">
      <div v-if="hasApiKey && !isEditingApiKey" class="api-status">
        <div class="key-container">
          <span class="label">Current API Key</span>
          <div class="key-wrap">
            <code class="key-value">{{ maskedApiKey }}</code>
            <span :class="isFromEnv ? 'env-badge' : 'db-badge'">
              {{ isFromEnv ? "via .env" : "via Database" }}
            </span>
          </div>
        </div>
        
        <div class="api-actions">
          <BaseButton v-if="!isFromEnv" variant="outline" size="sm" @click="startEditingApiKey">
            <Edit2 :size="14" />
            Change Key
          </BaseButton>
          <BaseButton v-if="!isFromEnv" variant="danger" size="sm" @click="clearApiKey">
            <Trash2 :size="14" />
            Remove
          </BaseButton>
        </div>
      </div>

      <div v-else class="api-input-wrap">
        <div class="input-header" v-if="isEditingApiKey">
          <span class="label">Edit API Key</span>
          <button class="cancel-link" @click="isEditingApiKey = false">Cancel</button>
        </div>
        <div class="input-row">
          <input
            v-model="apiKeyInput"
            type="password"
            placeholder="Paste your sk_live_... key here"
            class="styled-input"
          />
          <BaseButton
            variant="primary"
            :disabled="!apiKeyInput.trim() || savingApiKey"
            :loading="savingApiKey"
            @click="saveApiKey"
          >
            {{ isEditingApiKey ? 'Update Key' : 'Save Key' }}
          </BaseButton>
        </div>
      </div>

      <a href="#" class="external-link" @click.prevent="handleOpenUrl('https://skillsmp.com/docs/api')">
        <ExternalLink :size="12" />
        Get your API key at skillsmp.com
      </a>
    </div>
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

.api-box {
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.api-status {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.key-container {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.label {
  font-size: 12px;
  font-weight: 700;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.key-wrap {
  display: flex;
  align-items: center;
  gap: 12px;
}

.key-value {
  font-family: var(--font-mono);
  background: var(--bg-tertiary);
  padding: 4px 10px;
  border-radius: 6px;
  border: 1px solid var(--border-color);
  font-size: 13px;
}

.env-badge, .db-badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 12px;
  font-weight: 700;
}

.env-badge {
  background: rgba(139, 92, 246, 0.1);
  color: var(--accent-primary);
  border: 1px solid rgba(139, 92, 246, 0.2);
}

.db-badge {
  background: rgba(34, 197, 94, 0.1);
  color: var(--accent-success);
  border: 1px solid rgba(34, 197, 94, 0.2);
}

.api-actions {
  display: flex;
  gap: 10px;
}

.api-input-wrap {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.input-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.cancel-link {
  font-size: 12px;
  color: var(--text-muted);
  text-decoration: underline;
}

.input-row {
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

.styled-input:focus {
  border-color: var(--accent-primary);
}

.external-link {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--accent-primary);
  font-weight: 500;
  width: fit-content;
}

.external-link:hover {
  text-decoration: underline;
}
</style>
