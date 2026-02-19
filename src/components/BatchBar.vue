<script setup lang="ts">
import { t } from "../i18n"
import { selectedPackages, isSelectionMode, batchUninstall, batchUpgrade } from "../store/brew"
</script>

<template>
  <div class="batch-bar" :class="{ show: isSelectionMode }">
    <div class="batch-info">{{ t.batchSelected(selectedPackages.size) }}</div>
    <div class="batch-actions">
      <button class="btn-text" @click="selectedPackages.clear()">{{ t.btnCancel }}</button>
      <button class="btn-primary btn-danger" @click="batchUninstall">{{ t.btnBatchUninstall }}</button>
      <button class="btn-primary" @click="batchUpgrade">{{ t.btnBatchUpgrade }}</button>
    </div>
  </div>
</template>
