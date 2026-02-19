<script setup lang="ts">
import { t } from "../../i18n"
import { detailModalOpen, detailTitle, detailText, detailLoading, closeDetailModal } from "../../store/brew"
</script>

<template>
  <div v-if="detailModalOpen" class="modal-backdrop" @click.self="closeDetailModal">
    <div class="modal modal-large">
      <div class="modal-header">
        <h3>{{ detailTitle }}</h3>
        <button class="close-btn" @click="closeDetailModal">×</button>
      </div>
      <div v-if="detailLoading" class="modal-status">
        <div class="spinner"></div>
        <span>{{ t.running }}</span>
      </div>
      <pre class="modal-output">{{ detailText || t.waitingOutput }}</pre>
    </div>
  </div>
</template>
