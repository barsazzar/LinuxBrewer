<script setup lang="ts">
import { t } from "../i18n"
import { logs, showLogs } from "../store/brew"
</script>

<template>
  <div class="drawer">
    <div class="drawer-header">
      <h4>{{ t.logsTitle }}</h4>
      <button class="close-btn" @click="showLogs = false">×</button>
    </div>
    <pre class="drawer-body">{{ logs.join("\n") || t.noLogs }}</pre>
  </div>
</template>
