<script setup lang="ts">
import { computed } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";

const props = defineProps<{
  type: string;
  size?: number | string;
}>();

const iconSize = computed(() => {
  const s = props.size || 24;
  return typeof s === "number" ? `${s}px` : s;
});

const processedSrc = computed(() => {
  if (
    props.type.startsWith("http") ||
    props.type.startsWith("data:image/")
  ) {
    return props.type;
  }
  // Assume local path if it contains separators but isn't http/data
  if (props.type.includes("/") || props.type.includes("\\")) {
    return convertFileSrc(props.type);
  }
  return props.type;
});

const isImage = computed(() => {
  const t = props.type.toLowerCase();
  return (
    t.startsWith("data:image/") ||
    t.startsWith("http") ||
    t.includes("/") ||
    t.includes("\\") ||
    // Also check for common image extensions even if no path separators
    t.endsWith(".png") ||
    t.endsWith(".jpg") ||
    t.endsWith(".jpeg") ||
    t.endsWith(".svg") ||
    t.endsWith(".webp") ||
    t.endsWith(".ico") ||
    t.endsWith(".gif")
  );
});

const isEmoji = computed(() => {
  if (isImage.value) return false;
  // Basic emoji check: single character or short string that doesn't match known types
  const knownTypes = [
    "vscode", 
    "cursor", 
    "gemini", 
    "claude-code", 
    "codex", 
    "opencode",
    "windsurf",
    "trae",
    "antigravity"
  ];
  return !knownTypes.includes(props.type) && props.type.length <= 8;
});
</script>

<template>
  <div v-if="isImage" class="agent-icon-image">
    <img :src="processedSrc" :style="{ width: iconSize, height: iconSize }" alt="" />
  </div>

  <div v-else-if="isEmoji" class="agent-icon-emoji" :style="{ fontSize: iconSize }">
    {{ type }}
  </div>

  <svg
    v-else-if="type === 'vscode'"
    :width="iconSize"
    :height="iconSize"
    viewBox="0 0 256 254"
    xmlns="http://www.w3.org/2000/svg"
  >
    <!-- VS Code paths -->
    <defs>
      <linearGradient id="vscode_grad" x1="50%" x2="50%" y1="0%" y2="100%">
        <stop offset="0%" stop-color="#FFF" />
        <stop offset="100%" stop-color="#FFF" stop-opacity="0" />
      </linearGradient>
    </defs>
    <path
      fill="#0065A9"
      d="M246.135 26.873L193.593 1.575a15.885 15.885 0 0 0-18.123 3.08L3.466 161.482c-4.626 4.219-4.62 11.502.012 15.714l14.05 12.772a10.625 10.625 0 0 0 13.569.604L238.229 33.436c6.949-5.271 16.93-.315 16.93 8.407v-.61a15.94 15.94 0 0 0-9.024-14.36"
    />
    <path
      fill="#007ACC"
      d="m246.135 226.816l-52.542 25.298a15.89 15.89 0 0 1-18.123-3.08L3.466 92.207c-4.626-4.218-4.62-11.502.012-15.713l14.05-12.773a10.625 10.625 0 0 1 13.569-.603l207.132 157.135c6.949 5.271 16.93.315 16.93-8.408v.611a15.94 15.94 0 0 1-9.024 14.36"
    />
    <path
      fill="#1F9CF0"
      d="M193.428 252.134a15.89 15.89 0 0 1-18.125-3.083c5.881 5.88 15.938 1.715 15.938-6.603V11.273c0-8.318-10.057-12.483-15.938-6.602a15.89 15.89 0 0 1 18.125-3.084l52.533 25.263a15.94 15.94 0 0 1 9.03 14.363V212.51c0 6.125-3.51 11.709-9.03 14.363z"
    />
  </svg>

  <svg
    v-else-if="type === 'cursor'"
    :width="iconSize"
    :height="iconSize"
    viewBox="0 0 24 24"
    fill="currentColor"
    xmlns="http://www.w3.org/2000/svg"
  >
    <!-- Cursor paths -->
    <path
      d="M11.503.131 1.891 5.678a.84.84 0 0 0-.42.726v11.188c0 .3.162.575.42.724l9.609 5.55a1 1 0 0 0 .998 0l9.61-5.55a.84.84 0 0 0 .42-.724V6.404a.84.84 0 0 0-.42-.726L12.497.131a1.01 1.01 0 0 0-.996 0M2.657 6.338h18.55c.263 0 .43.287.297.515L12.23 22.918c-.062.107-.229.064-.229-.06V12.335a.59.59 0 0 0-.295-.51l-9.11-5.257c-.109-.063-.064-.23.061-.23"
    />
  </svg>

  <svg
    v-else-if="type === 'gemini'"
    :width="iconSize"
    :height="iconSize"
    viewBox="0 0 24 24"
    fill="currentColor"
    xmlns="http://www.w3.org/2000/svg"
    style="color: #8e75b2"
  >
    <path
      d="M11.04 19.32Q12 21.51 12 24q0-2.49.93-4.68.96-2.19 2.58-3.81t3.81-2.55Q21.51 12 24 12q-2.49 0-4.68-.93a12.3 12.3 0 0 1-3.81-2.58 12.3 12.3 0 0 1-2.58-3.81Q12 2.49 12 0q0 2.49-.96 4.68-.93 2.19-2.55 3.81a12.3 12.3 0 0 1-3.81 2.58Q2.49 12 0 12q2.49 0 4.68.96 2.19.93 3.81 2.55t2.55 3.81"
    />
  </svg>

  <svg
    v-else-if="type === 'claude-code'"
    :width="iconSize"
    :height="iconSize"
    viewBox="0 0 24 24"
    fill="currentColor"
    xmlns="http://www.w3.org/2000/svg"
    style="color: #d97757"
  >
    <path
      d="M17.3041 3.541h-3.6718l6.696 16.918H24Zm-10.6082 0L0 20.459h3.7442l1.3693-3.5527h7.0052l1.3693 3.5528h3.7442L10.5363 3.5409Zm-.3712 10.2232 2.2914-5.9456 2.2914 5.9456Z"
    />
  </svg>

  <svg
    v-else-if="type === 'opencode'"
    :width="iconSize"
    :height="iconSize"
    viewBox="0 0 24 30"
    fill="currentColor"
    xmlns="http://www.w3.org/2000/svg"
  >
    <path d="M18 30H6V18H18V30Z" fill="currentColor" fill-opacity="0.5"/>
    <path fill-rule="evenodd" clip-rule="evenodd" d="M24 30H0V0H24V30ZM18 6H6V24H18V6Z" fill="currentColor"/>
  </svg>

  <svg
    v-else-if="type === 'codex'"
    :width="iconSize"
    :height="iconSize"
    viewBox="0 0 24 24"
    fill="currentColor"
    xmlns="http://www.w3.org/2000/svg"
  >
    <path
      d="M22.282 9.821a6 6 0 0 0-.516-4.91a6.05 6.05 0 0 0-6.51-2.9A6.065 6.065 0 0 0 4.981 4.18a6 6 0 0 0-3.998 2.9a6.05 6.05 0 0 0 .743 7.097a5.98 5.98 0 0 0 .51 4.911a6.05 6.05 0 0 0 6.515 2.9A6 6 0 0 0 13.26 24a6.06 6.06 0 0 0 5.772-4.206a6 6 0 0 0 3.997-2.9a6.06 6.06 0 0 0-.747-7.073M13.26 22.43a4.48 4.48 0 0 1-2.876-1.04l.141-.081l4.779-2.758a.8.8 0 0 0 .392-.681v-6.737l2.02 1.168a.07.07 0 0 1 .038.052v5.583a4.504 4.504 0 0 1-4.494 4.494M3.6 18.304a4.47 4.47 0 0 1-.535-3.014l.142.085l4.783 2.759a.77.77 0 0 0 .78 0l5.843-3.369v2.332a.08.08 0 0 1-.033.062L9.74 19.95a4.5 4.5 0 0 1-6.14-1.646M2.34 7.896a4.5 4.5 0 0 1 2.366-1.973V11.6a.77.77 0 0 0 .388.677l5.815 3.354l-2.02 1.168a.08.08 0 0 1-.071 0l-4.83-2.786A4.504 4.504 0 0 1 2.34 7.872zm16.597 3.855l-5.833-3.387L15.119 7.2a.08.08 0 0 1 .071 0l4.83 2.791a4.494 4.494 0 0 1-.676 8.105v-5.678a.79.79 0 0 0-.407-.667m2.01-3.023l-.141-.085l-4.774-2.782a.78.78 0 0 0-.785 0L9.409 9.23V6.897a.07.07 0 0 1 .028-.061l4.83-2.787a4.5 4.5 0 0 1 6.68 4.66zm-12.64 4.135l-2.02-1.164a.08.08 0 0 1-.038-.057V6.075a4.5 4.5 0 0 1 7.375-3.453l-.142.08L8.704 5.46a.8.8 0 0 0-.393.681zm1.097-2.365l2.602-1.5l2.607 1.5v2.999l-2.597 1.5l-2.607-1.5Z"
    />
  </svg>

  <svg
    v-else-if="type === 'windsurf'"
    :width="iconSize"
    :height="iconSize"
    viewBox="0 0 24 24"
    fill="currentColor"
    xmlns="http://www.w3.org/2000/svg"
  >
    <title>Windsurf</title>
    <path clip-rule="evenodd" d="M23.78 5.004h-.228a2.187 2.187 0 00-2.18 2.196v4.912c0 .98-.804 1.775-1.76 1.775a1.818 1.818 0 01-1.472-.773L13.168 5.95a2.197 2.197 0 00-1.81-.95c-1.134 0-2.154.972-2.154 2.173v4.94c0 .98-.797 1.775-1.76 1.775-.57 0-1.136-.289-1.472-.773L.408 5.098C.282 4.918 0 5.007 0 5.228v4.284c0 .216.066.426.188.604l5.475 7.889c.324.466.8.812 1.351.938 1.377.316 2.645-.754 2.645-2.117V11.89c0-.98.787-1.775 1.76-1.775h.002c.586 0 1.135.288 1.472.773l4.972 7.163a2.15 2.15 0 001.81.95c1.158 0 2.151-.973 2.151-2.173v-4.939c0-.98.787-1.775 1.76-1.775h.194c.122 0 .22-.1.22-.222V5.225a.221.221 0 00-.22-.222z"></path>
  </svg>

  <svg
    v-else-if="type === 'trae'"
    :width="iconSize"
    :height="iconSize"
    viewBox="0 0 28 21"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    style="color: #32f08c"
  >
    <g clip-path="url(#trae_clip)">
      <path fill="currentColor" d="M28.002 20.846H4v-3.998H0V.846h28.002zM4 16.848h20.002V4.845H4zm10.002-6.062-2.829 2.828-2.828-2.828 2.828-2.829zm8-.002-2.828 2.828-2.829-2.828 2.829-2.829z"></path>
    </g>
    <defs>
      <clipPath id="trae_clip">
        <path fill="#fff" d="M0 .846h28.002v20H0z"></path>
      </clipPath>
    </defs>
  </svg>
  
  <svg
    v-else-if="type === 'antigravity'"
    :width="iconSize"
    :height="iconSize"
    viewBox="16 18 80 80"
    fill="currentColor"
    xmlns="http://www.w3.org/2000/svg"
    style="color: #3186FF"
  >
    <path d="M89.6992 93.695C94.3659 97.195 101.366 94.8617 94.9492 88.445C75.6992 69.7783 79.7825 18.445 55.8659 18.445C31.9492 18.445 36.0325 69.7783 16.7825 88.445C9.78251 95.445 17.3658 97.195 22.0325 93.695C40.1159 81.445 38.9492 59.8617 55.8659 59.8617C72.7825 59.8617 71.6159 81.445 89.6992 93.695Z" />
  </svg>

  <svg
    v-else
    :width="iconSize"
    :height="iconSize"
    viewBox="0 0 24 24"
    fill="currentColor"
    xmlns="http://www.w3.org/2000/svg"
  >
    <path
      d="M20 19V7H4v12zm0-16a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2zm-7 14v-2h5v2zm-3.42-4L5.57 9H8.4l3.3 3.3c.39.39.39 1.03 0 1.42L8.42 17H5.59z"
    />
  </svg>
</template>

<style scoped>
.agent-icon-image {
  display: flex;
  align-items: center;
  justify-content: center;
}
.agent-icon-image img {
  object-fit: contain;
  border-radius: 4px;
}
.agent-icon-emoji {
  display: flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
}
</style>
