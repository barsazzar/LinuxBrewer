<script setup lang="ts">
import { t } from "../i18n"
import { logs, showLogs } from "../store/brew"
import { showToast } from "../store/brew"

function clearLogs() {
  logs.value = []
}

function copyLogs() {
  const text = logs.value.join("\n")
  navigator.clipboard.writeText(text).then(() => showToast("Copied!"))
}
</script>

<template>
  <div class="drawer">
    <div class="drawer-header">
      <h4>{{ t.logsTitle }}</h4>
      <div class="drawer-actions">
        <button class="icon-btn small" @click="copyLogs" :title="t.btnCopyLogs">
          <svg viewBox="0 0 24 24" fill="none">
            <rect x="9" y="9" width="13" height="13" rx="2" stroke="currentColor" stroke-width="2"/>
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" stroke="currentColor" stroke-width="2"/>
          </svg>
        </button>
        <button class="icon-btn small" @click="clearLogs" :title="t.btnClearLogs">
          <svg viewBox="0 0 24 24" fill="none">
            <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
          </svg>
        </button>
        <button class="close-btn" @click="showLogs = false">×</button>
      </div>
    </div>
    <pre class="drawer-body">{{ logs.join("\n") || t.noLogs }}</pre>
  </div>
</template>
