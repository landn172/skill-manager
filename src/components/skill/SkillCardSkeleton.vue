<template>
  <div class="skill-card-skeleton glass-card shimmer">
    <div class="card-header">
      <div class="skeleton-title"></div>
      <div class="skeleton-version"></div>
    </div>
    
    <div class="skeleton-agents">
      <div v-for="i in 3" :key="i" class="skeleton-agent-pill"></div>
    </div>

    <div class="skeleton-meta">
      <div class="skeleton-tag-small"></div>
    </div>

    <div class="skeleton-description">
      <div class="skeleton-line"></div>
      <div class="skeleton-line short"></div>
    </div>

    <div class="skeleton-tags">
      <div v-for="i in 2" :key="i" class="skeleton-tag"></div>
    </div>

    <div class="card-footer">
      <div class="skeleton-icon-btn"></div>
      <div class="skeleton-btn"></div>
    </div>
  </div>
</template>

<style scoped>
.skill-card-skeleton {
  padding: 20px;
  min-height: 240px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  background: rgba(255, 255, 255, 0.03);
  pointer-events: none;
}

.skeleton-title {
  width: 60%;
  height: 20px;
  background: var(--bg-tertiary);
  border-radius: 4px;
}

.skeleton-version {
  width: 40px;
  height: 16px;
  background: var(--bg-tertiary);
  border-radius: 4px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.skeleton-agents {
  display: flex;
  gap: 8px;
}

.skeleton-agent-pill {
  width: 26px;
  height: 26px;
  background: var(--bg-tertiary);
  border-radius: 6px;
}

.skeleton-meta {
  height: 14px;
  width: 80px;
  background: var(--bg-tertiary);
  border-radius: 4px;
}

.skeleton-description {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 8px;
}

.skeleton-line {
  width: 100%;
  height: 12px;
  background: var(--bg-tertiary);
  border-radius: 4px;
}

.skeleton-line.short {
  width: 70%;
}

.skeleton-tags {
  display: flex;
  gap: 8px;
  margin-top: auto;
}

.skeleton-tag {
  width: 50px;
  height: 14px;
  background: var(--bg-tertiary);
  border-radius: 4px;
}

.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--glass-border);
}

.skeleton-icon-btn {
  width: 32px;
  height: 32px;
  background: var(--bg-tertiary);
  border-radius: 8px;
}

.skeleton-btn {
  width: 80px;
  height: 32px;
  background: var(--bg-tertiary);
  border-radius: 8px;
}
</style>
