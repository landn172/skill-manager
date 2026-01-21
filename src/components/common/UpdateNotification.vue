<script setup lang="ts">
import { ref, onMounted } from "vue";
import { check, Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { Download, RefreshCw, X, CheckCircle, AlertCircle } from "lucide-vue-next";

const updateAvailable = ref(false);
const updateInfo = ref<Update | null>(null);
const isDownloading = ref(false);
const downloadProgress = ref(0);
const downloadTotal = ref(0);
const updateError = ref<string | null>(null);
const showNotification = ref(true);
const updateComplete = ref(false);

onMounted(async () => {
  await checkForUpdates();
});

async function checkForUpdates() {
  try {
    updateError.value = null;
    const update = await check();
    if (update) {
      updateInfo.value = update;
      updateAvailable.value = true;
    }
  } catch (error) {
    console.error("Failed to check for updates:", error);
    // Silently fail - don't show error for update check failures
  }
}

async function downloadAndInstall() {
  if (!updateInfo.value) return;

  try {
    isDownloading.value = true;
    updateError.value = null;
    downloadProgress.value = 0;

    await updateInfo.value.downloadAndInstall((event) => {
      switch (event.event) {
        case "Started":
          downloadTotal.value = event.data.contentLength ?? 0;
          break;
        case "Progress":
          downloadProgress.value += event.data.chunkLength;
          break;
        case "Finished":
          updateComplete.value = true;
          break;
      }
    });

    // Wait a moment then relaunch
    setTimeout(async () => {
      await relaunch();
    }, 1500);
  } catch (error) {
    console.error("Failed to install update:", error);
    updateError.value = error instanceof Error ? error.message : "Update failed";
    isDownloading.value = false;
  }
}

function dismiss() {
  showNotification.value = false;
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

const progressPercent = computed(() => {
  if (downloadTotal.value === 0) return 0;
  return Math.round((downloadProgress.value / downloadTotal.value) * 100);
});
</script>

<template>
  <Transition name="slide">
    <div v-if="updateAvailable && showNotification" class="update-notification">
      <!-- Close button -->
      <button class="close-btn" @click="dismiss" :disabled="isDownloading">
        <X :size="16" />
      </button>

      <!-- Update complete state -->
      <template v-if="updateComplete">
        <div class="notification-content">
          <CheckCircle class="icon success" :size="20" />
          <div class="text">
            <span class="title">Update installed!</span>
            <span class="subtitle">Restarting app...</span>
          </div>
        </div>
      </template>

      <!-- Error state -->
      <template v-else-if="updateError">
        <div class="notification-content">
          <AlertCircle class="icon error" :size="20" />
          <div class="text">
            <span class="title">Update failed</span>
            <span class="subtitle">{{ updateError }}</span>
          </div>
          <button class="action-btn retry" @click="downloadAndInstall">
            <RefreshCw :size="14" />
            Retry
          </button>
        </div>
      </template>

      <!-- Downloading state -->
      <template v-else-if="isDownloading">
        <div class="notification-content">
          <Download class="icon downloading" :size="20" />
          <div class="text">
            <span class="title">Downloading update...</span>
            <span class="subtitle">
              {{ formatBytes(downloadProgress) }} / {{ formatBytes(downloadTotal) }} ({{
                progressPercent
              }}%)
            </span>
          </div>
        </div>
        <div class="progress-bar">
          <div class="progress-fill" :style="{ width: `${progressPercent}%` }"></div>
        </div>
      </template>

      <!-- Update available state -->
      <template v-else>
        <div class="notification-content">
          <Download class="icon" :size="20" />
          <div class="text">
            <span class="title">Update available: v{{ updateInfo?.version }}</span>
            <span v-if="updateInfo?.body" class="subtitle"
              >{{ updateInfo.body.slice(0, 100)
              }}{{ updateInfo.body.length > 100 ? "..." : "" }}</span
            >
          </div>
          <button class="action-btn" @click="downloadAndInstall">Update Now</button>
        </div>
      </template>
    </div>
  </Transition>
</template>

<script lang="ts">
import { computed } from "vue";
export default {
  name: "UpdateNotification",
};
</script>

<style scoped>
.update-notification {
  position: fixed;
  bottom: 20px;
  right: 20px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 16px 20px;
  min-width: 320px;
  max-width: 400px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  z-index: 1000;
}

.close-btn {
  position: absolute;
  top: 8px;
  right: 8px;
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.close-btn:hover:not(:disabled) {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.close-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.notification-content {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.icon {
  color: var(--accent-primary);
  flex-shrink: 0;
  margin-top: 2px;
}

.icon.success {
  color: #10b981;
}

.icon.error {
  color: #ef4444;
}

.icon.downloading {
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

.text {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.title {
  font-weight: 600;
  color: var(--text-primary);
  font-size: 14px;
}

.subtitle {
  color: var(--text-secondary);
  font-size: 12px;
  line-height: 1.4;
}

.action-btn {
  background: linear-gradient(135deg, var(--accent-primary), var(--accent-secondary));
  color: white;
  border: none;
  border-radius: 8px;
  padding: 8px 16px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  gap: 6px;
  white-space: nowrap;
  flex-shrink: 0;
}

.action-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(99, 102, 241, 0.4);
}

.action-btn.retry {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.action-btn.retry:hover {
  background: var(--bg-hover);
  box-shadow: none;
}

.progress-bar {
  margin-top: 12px;
  height: 4px;
  background: var(--bg-tertiary);
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--accent-primary), var(--accent-secondary));
  border-radius: 2px;
  transition: width 0.3s ease;
}

/* Transition animations */
.slide-enter-active,
.slide-leave-active {
  transition: all 0.3s ease;
}

.slide-enter-from,
.slide-leave-to {
  opacity: 0;
  transform: translateX(100px);
}
</style>
