<script setup lang="ts">
import { t } from "../i18n"
import { logs, showLogs } from "../store/brew"
import { showToast } from "../store/brew"
import { Copy, Trash2 } from "lucide-vue-next"

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
          <Copy :size="14" />
        </button>
        <button class="icon-btn small" @click="clearLogs" :title="t.btnClearLogs">
          <Trash2 :size="14" />
        </button>
        <button class="close-btn" @click="showLogs = false">×</button>
      </div>
    </div>
    <pre class="drawer-body">{{ logs.join("\n") || t.noLogs }}</pre>
  </div>
</template>
