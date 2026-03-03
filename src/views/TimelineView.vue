<script setup lang="ts">
import { DbrCard } from "dobruniaui-vue";
</script>

<template>
  <DbrCard class="timeline-view">
    <header class="view-header">
      <h2 class="dbru-text-lg dbru-text-main">Таймлайн</h2>
    </header>

    <section class="placeholder-section">
      <p class="placeholder-text dbru-text-sm dbru-text-muted">
        Детальный просмотр дня появится здесь
      </p>
    </section>
  </DbrCard>
</template>

<style scoped>
.timeline-view {
  padding: var(--dbru-space-6);
  height: 100%;
}

.view-header {
  margin-bottom: var(--dbru-space-6);
}

.view-header h2 {
  margin: 0;
}

.placeholder-section {
  text-align: center;
  padding: var(--dbru-space-6);
}

.placeholder-text {
  margin: 0;
  font-style: italic;
}
</style>
