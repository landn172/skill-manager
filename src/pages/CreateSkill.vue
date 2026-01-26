<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useRouter, useRoute } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { homeDir } from "@tauri-apps/api/path";
import { ChevronRight, Save, Folder, FileCode, ArrowLeft } from "lucide-vue-next";
import PageHeader from "@/components/common/PageHeader.vue";
import BaseButton from "@/components/common/BaseButton.vue";

const router = useRouter();
const route = useRoute();

const step = ref(1);
const creating = ref(false);
const form = ref({
  name: "",
  description: "",
  parentPath: "",
});

const isEditMode = computed(() => !!route.query.edit);
const editSkillPath = computed(() => (route.query.path as string) || "");

onMounted(() => {
  if (isEditMode.value) {
    form.value.name = (route.query.name as string) || "";
    form.value.description = (route.query.description as string) || "";
    form.value.parentPath = editSkillPath.value;
  }
});

async function selectParentPath() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: await homeDir(),
    });

    if (selected && typeof selected === "string") {
      form.value.parentPath = selected;
    }
  } catch (e) {
    console.error("Failed to select directory", e);
  }
}

async function handleSubmit() {
  creating.value = true;
  try {
    if (isEditMode.value) {
      await invoke("update_local_skill", {
        skillPath: editSkillPath.value,
        name: form.value.name || undefined,
        description: form.value.description || undefined,
      });
      router.push("/marketplace");
    } else {
      const result = await invoke<{
        success: boolean;
        path: string;
        message: string;
      }>("create_skill", {
        name: form.value.name,
        description: form.value.description,
        parentPath: form.value.parentPath,
      });

      if (result.success) {
        router.push("/installed");
      } else {
        throw new Error(result.message);
      }
    }
  } catch (e) {
    alert(`Failed to ${isEditMode.value ? "update" : "create"} skill: ${e}`);
  } finally {
    creating.value = false;
  }
}
</script>

<template>
  <div class="create-page animate-fade-in">
    <PageHeader 
      :title="isEditMode ? 'Edit Skill' : 'Create New Skill'" 
      :description="isEditMode ? 'Update the metadata for your local skill.' : 'Scaffold a new skill directory to start building.'"
    />

    <div class="stepper-wrap" v-if="!isEditMode">
      <div class="step" :class="{ active: step >= 1 }">
        <span class="num">1</span>
        <span class="label">Information</span>
      </div>
      <div class="line" :class="{ active: step >= 2 }"></div>
      <div class="step" :class="{ active: step >= 2 }">
        <span class="num">2</span>
        <span class="label">Location</span>
      </div>
    </div>

    <div class="form-area">
      <div v-if="!isEditMode && step === 1" class="promo-box glass">
        <div class="icon-box">
          <FileCode :size="20" />
        </div>
        <div class="text">
          <h3>Standardized Scaffolding</h3>
          <p>We'll create a <code>README.md</code>, <code>instructions.md</code>, and a <code>SKILL.md</code> for you.</p>
        </div>
      </div>

      <!-- Step 1 Form -->
      <div v-if="step === 1" class="form-glass glass-card animate-slide-up">
        <div class="form-group">
          <label>Skill Name</label>
          <input 
            v-model="form.name" 
            placeholder="e.g. react-expert" 
            class="styled-input"
            autofocus 
          />
          <p class="hint">Recommended: lowercase kebab-case.</p>
        </div>

        <div class="form-group">
          <label>Description</label>
          <textarea 
            v-model="form.description" 
            placeholder="Tell us what this skill helps with..." 
            class="styled-textarea"
          ></textarea>
        </div>
      </div>

      <!-- Step 2 Form -->
      <div v-if="step === 2 && !isEditMode" class="form-glass glass-card animate-slide-up">
        <div class="form-group">
          <label>Parent Directory</label>
          <div class="path-row">
            <input
              v-model="form.parentPath"
              placeholder="Select destination..."
              class="styled-input"
              readonly
              @click="selectParentPath"
            />
            <BaseButton variant="outline" @click="selectParentPath">
              <Folder :size="16" />
              Browse
            </BaseButton>
          </div>
          <p class="hint" v-if="form.name">
            Skill will be created at: <code>{{ form.parentPath }}/{{ form.name }}</code>
          </p>
        </div>
      </div>
    </div>

    <footer class="form-footer">
      <BaseButton v-if="step > 1 && !isEditMode" variant="ghost" @click="step--">
        <ArrowLeft :size="16" />
        Back
      </BaseButton>
      <div class="spacer"></div>
      
      <BaseButton
        v-if="step < 2 && !isEditMode"
        variant="primary"
        :disabled="!form.name"
        @click="step++"
      >
        Next Step
        <ChevronRight :size="18" />
      </BaseButton>
      
      <BaseButton
        v-else
        variant="primary"
        :disabled="(!isEditMode && !form.parentPath) || creating"
        :loading="creating"
        @click="handleSubmit"
      >
        <Save :size="18" />
        {{ isEditMode ? 'Save Changes' : 'Generate Skill' }}
      </BaseButton>
    </footer>
  </div>
</template>

<style scoped>
.create-page {
  padding: 20px;
  height: 100vh;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.stepper-wrap {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  margin-bottom: 24px;
}

.step {
  display: flex;
  align-items: center;
  gap: 10px;
  color: var(--text-muted);
  transition: all 0.3s;
}

.step.active {
  color: var(--accent-primary);
}

.step .num {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: var(--bg-tertiary);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 800;
  border: 1px solid var(--border-color);
}

.step.active .num {
  background: var(--accent-primary);
  color: white;
  border-color: var(--accent-primary);
}

.step .label {
  font-size: 14px;
  font-weight: 700;
}

.stepper-wrap .line {
  width: 60px;
  height: 2px;
  background: var(--border-color);
}

.stepper-wrap .line.active {
  background: var(--accent-primary);
}

.form-area {
  flex: 1;
  max-width: 640px;
  margin: 0 auto;
  width: 100%;
}

.promo-box {
  display: flex;
  gap: 16px;
  padding: 16px 20px;
  border-radius: 12px;
  margin-bottom: 24px;
  border: 1px solid var(--glass-border);
}

.promo-box .icon-box {
  width: 40px;
  height: 40px;
  background: rgba(139, 92, 246, 0.1);
  color: var(--accent-primary);
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.promo-box h3 {
  font-size: 15px;
  font-weight: 700;
  margin: 0 0 4px;
}

.promo-box p {
  font-size: 13px;
  color: var(--text-secondary);
  margin: 0;
}

.form-glass {
  padding: 32px;
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-group label {
  font-size: 13px;
  font-weight: 700;
  color: var(--text-secondary);
}

.styled-input {
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 12px 16px;
  color: var(--text-primary);
  font-size: 14px;
  transition: all 0.2s;
}

.styled-input:focus {
  outline: none;
  border-color: var(--accent-primary);
  background: var(--bg-primary);
}

.styled-textarea {
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 12px 16px;
  min-height: 120px;
  color: var(--text-primary);
  font-size: 14px;
  resize: none;
  transition: all 0.2s;
}

.styled-textarea:focus {
  outline: none;
  border-color: var(--accent-primary);
  background: var(--bg-primary);
}

.hint {
  font-size: 12px;
  color: var(--text-muted);
  margin: 0;
}

.path-row {
  display: flex;
  gap: 12px;
}

.form-footer {
  max-width: 640px;
  width: 100%;
  margin: 40px auto 0;
  display: flex;
  align-items: center;
  padding-top: 24px;
  border-top: 1px solid var(--glass-border);
}

.spacer {
  flex: 1;
}

code {
  background: rgba(0,0,0,0.2);
  padding: 2px 4px;
  border-radius: 4px;
  font-family: var(--font-mono);
  font-size: 0.9em;
}
</style>
