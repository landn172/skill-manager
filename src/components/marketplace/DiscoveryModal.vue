<script setup lang="ts">
import { ref } from "vue";
import { useMarketplaceStore } from "@/stores/marketplace";
import Modal from "@/components/common/Modal.vue";
import BaseButton from "@/components/common/BaseButton.vue";
import { AlertCircle } from "lucide-vue-next";
import type { Skill } from "@/types";

const props = defineProps<{
  show: boolean;
  initialUrl?: string;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "install", skill: Skill): void;
}>();

const store = useMarketplaceStore();

const discoveryUrl = ref(props.initialUrl || "");
const discoveredSkills = ref<any[]>([]);
const discovering = ref(false);
const discoveryError = ref<string | null>(null);

async function handleDiscovery() {
  const url = discoveryUrl.value.trim();
  if (!url) return;
  
  discovering.value = true;
  discoveryError.value = null;
  discoveredSkills.value = [];
  
  try {
    const results = await store.discoverFromUrl(url);
    discoveredSkills.value = results;
    if (results.length === 0) {
      discoveryError.value = "No skills found at this URL.";
    }
  } catch (e) {
    discoveryError.value = String(e);
  } finally {
    discovering.value = false;
  }
}

// Handle props change if needed (e.g. if opened from a suggestion)
import { watch } from "vue";
watch(() => props.initialUrl, (newUrl) => {
  if (newUrl) {
    discoveryUrl.value = newUrl;
    if (props.show) handleDiscovery();
  }
});
</script>

<template>
  <Modal
    :show="show"
    title="Install from GitHub"
    maxWidth="600px"
    @close="emit('close')"
  >
    <div class="discovery-box">
      <div class="input-row">
        <input
          v-model="discoveryUrl"
          placeholder="e.g. owner/repo or full URL"
          @keyup.enter="handleDiscovery"
          :disabled="discovering"
          class="styled-input"
        />
        <BaseButton 
          variant="primary" 
          @click="handleDiscovery" 
          :loading="discovering" 
          :disabled="!discoveryUrl"
        >
          Search
        </BaseButton>
      </div>

      <div v-if="discoveryError" class="discovery-err">
        <AlertCircle :size="16" />
        <span>{{ discoveryError }}</span>
      </div>

      <div v-if="discoveredSkills.length > 0" class="results-list">
        <h3>Available Skills</h3>
        <div class="results-scroll">
          <div v-for="skill in discoveredSkills" :key="skill.path" class="result-item glass-card">
            <div class="info">
              <span class="name">{{ skill.name }}</span>
              <span class="desc">{{ skill.description }}</span>
            </div>
            <BaseButton 
              size="sm" 
              variant="primary" 
              @click="emit('install', skill)"
            >
              Install
            </BaseButton>
          </div>
        </div>
      </div>
    </div>
  </Modal>
</template>

<style scoped>
.discovery-box {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.input-row {
  display: flex;
  gap: 12px;
}

.styled-input {
  flex: 1;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  padding: 12px 16px;
  border-radius: 10px;
  color: var(--text-primary);
  outline: none;
}

.styled-input:focus {
  border-color: var(--accent-primary);
}

.discovery-err {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  background-color: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: var(--border-radius);
  color: var(--accent-error);
  font-size: 13px;
}

.results-list h3 {
  font-size: 14px;
  margin-bottom: 12px;
  font-weight: 700;
}

.results-scroll {
  max-height: 400px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-right: 4px;
}

.result-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border-radius: 12px;
}

.result-item .info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.result-item .name { font-weight: 700; }
.result-item .desc { font-size: 12px; color: var(--text-secondary); }
</style>
