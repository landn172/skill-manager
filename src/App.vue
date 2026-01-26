<script setup lang="ts">
import { ref, onMounted } from "vue";
import MainLayout from "@/components/layout/MainLayout.vue";
import { useThemeStore } from "@/stores/theme";
import OnboardingModal from "@/components/onboarding/OnboardingModal.vue";
import UpdateNotification from "@/components/common/UpdateNotification.vue";
import Toast from "@/components/common/Toast.vue";

// Initialize theme
useThemeStore();

const showOnboarding = ref(false);

onMounted(() => {
  const completed = localStorage.getItem("onboarding_completed");
  if (!completed) {
    showOnboarding.value = true;
  }
});

function closeOnboarding() {
  localStorage.setItem("onboarding_completed", "true");
  showOnboarding.value = false;
}
</script>

<template>
  <MainLayout>
    <router-view />
    <OnboardingModal :show="showOnboarding" @close="closeOnboarding" />
  </MainLayout>
  <UpdateNotification />
  <Toast />
</template>

<style>
@import "@/assets/styles/main.css";
</style>
