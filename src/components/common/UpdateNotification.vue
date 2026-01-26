<script setup lang="ts">
import { ref, shallowRef, toRaw, onMounted, computed } from "vue";
import { check, Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { Download, RefreshCw, X, CheckCircle, AlertCircle } from "lucide-vue-next";
import BaseButton from "@/components/common/BaseButton.vue";

const updateAvailable = ref(false);
// 使用 shallowRef 避免 Vue 代理破坏 Update 类的私有字段
const updateInfo = shallowRef<Update | null>(null);
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

    // 使用 toRaw() 获取原始 Update 对象，避免 Vue 代理干扰私有字段
    const rawUpdate = toRaw(updateInfo.value);
    await rawUpdate.downloadAndInstall((event) => {
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
    <div v-if="updateAvailable && showNotification" class="update-notification glass-card">
      <button class="close-btn" @click="dismiss" :disabled="isDownloading">
        <X :size="16" />
      </button>

      <template v-if="updateComplete">
        <div class="notification-content">
          <CheckCircle class="icon success" :size="20" />
          <div class="text">
            <span class="title">Update Success</span>
            <span class="subtitle">Restarting to apply changes...</span>
          </div>
        </div>
      </template>

      <template v-else-if="updateError">
        <div class="notification-content">
          <AlertCircle class="icon error" :size="20" />
          <div class="text">
            <span class="title">Update Failed</span>
            <span class="subtitle">{{ updateError }}</span>
          </div>
          <button class="retry-btn" @click="downloadAndInstall">
            <RefreshCw :size="14" />
          </button>
        </div>
      </template>

      <template v-else-if="isDownloading">
        <div class="notification-content">
          <Download class="icon downloading" :size="20" />
          <div class="text">
            <span class="title">Updating...</span>
            <span class="subtitle">
              {{ formatBytes(downloadProgress) }} / {{ formatBytes(downloadTotal) }}
            </span>
          </div>
          <span class="percent">{{ progressPercent }}%</span>
        </div>
        <div class="progress-track">
          <div class="progress-fill" :style="{ width: `${progressPercent}%` }"></div>
        </div>
      </template>

      <template v-else>
        <div class="notification-content">
          <Download class="icon" :size="20" />
          <div class="text">
            <span class="title">New Version: v{{ updateInfo?.version }}</span>
            <span v-if="updateInfo?.body" class="subtitle"
              >{{ updateInfo.body.slice(0, 80) }}{{ updateInfo.body.length > 80 ? "..." : "" }}</span
            >
          </div>
          <BaseButton variant="primary" size="sm" @click="downloadAndInstall">Update</BaseButton>
        </div>
      </template>
    </div>
  </Transition>
</template>


<style scoped>
.update-notification {
  position: fixed;
  bottom: 24px;
  right: 24px;
  padding: 20px;
  min-width: 340px;
  max-width: 420px;
  z-index: 1000;
  box-shadow: var(--shadow-xl);
}

.close-btn {
  position: absolute;
  top: 10px;
  right: 10px;
  color: var(--text-muted);
  transition: all 0.2s;
}

.close-btn:hover:not(:disabled) {
  color: var(--text-primary);
  background: var(--bg-hover);
}

.notification-content {
  display: flex;
  align-items: center;
  gap: 16px;
}

.icon {
  color: var(--accent-primary);
  flex-shrink: 0;
}

.icon.success { color: var(--accent-success); }
.icon.error { color: var(--accent-error); }
.icon.downloading { animation: pulse 1.5s infinite; }

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.text {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.title {
  font-weight: 700;
  font-size: 14px;
  color: var(--text-primary);
}

.subtitle {
  color: var(--text-secondary);
  font-size: 12px;
  line-height: 1.4;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.percent {
  font-family: var(--font-mono);
  font-size: 12px;
  font-weight: 700;
  color: var(--accent-primary);
}

.retry-btn {
  padding: 6px;
  color: var(--text-muted);
  transition: all 0.2s;
}

.retry-btn:hover {
  color: var(--accent-primary);
  background: var(--bg-hover);
}

.progress-track {
  margin-top: 16px;
  height: 4px;
  background: var(--bg-tertiary);
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--accent-primary);
  transition: width 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.slide-enter-active, .slide-leave-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.slide-enter-from, .slide-leave-to { opacity: 0; transform: translateX(40px) scale(0.95); }
</style>
