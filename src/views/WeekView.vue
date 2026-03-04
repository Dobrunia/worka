<script setup lang="ts">
import { DbrCard } from "dobruniaui-vue";
</script>

<template>
  <div class="week-view">
    <h2 class="view-title dbru-text-lg dbru-text-main">Неделя</h2>
    <DbrCard class="placeholder-card">
      <p class="dbru-text-sm dbru-text-muted">Статистика за неделю появится здесь</p>
    </DbrCard>
  </div>
</template>

<style scoped>
.week-view {
  display: flex;
  flex-direction: column;
  gap: var(--dbru-space-4);
  max-width: 960px;
}

.view-title {
  margin: 0;
  font-weight: var(--dbru-font-weight-semibold);
}

.placeholder-card {
  padding: var(--dbru-space-6);
  text-align: center;
}

.placeholder-card p {
  margin: 0;
}
</style>
