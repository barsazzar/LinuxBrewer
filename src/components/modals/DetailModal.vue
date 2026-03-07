<script setup lang="ts">
import { computed } from "vue"
import { t } from "../../i18n"
import { detailModalOpen, detailTitle, detailText, detailLoading, activeRequestId, closeDetailModal } from "../../store/brew"
import { invoke } from "@tauri-apps/api/core"

async function cancelOperation() {
  const rid = activeRequestId.value
  if (!rid) return
  await invoke("cancel_brew_stream", { requestId: rid }).catch(() => {})
}

// Syntax-highlight: colorize Error/Warning/success lines in the output
const highlightedText = computed(() => {
  if (!detailText.value) return ""
  return detailText.value
    .split("\n")
    .map(line => {
      if (/^(\[err\]|error:|Error:|fatal:)/i.test(line)) {
        return `<span class="out-error">${escapeHtml(line)}</span>`
      }
      if (/^(warning:|Warning:|==>)/i.test(line)) {
        return `<span class="out-warn">${escapeHtml(line)}</span>`
      }
      if (/^(✓|✔|success|Successfully)/i.test(line)) {
        return `<span class="out-ok">${escapeHtml(line)}</span>`
      }
      if (/^(✗|✘|fail|Failed)/i.test(line)) {
        return `<span class="out-error">${escapeHtml(line)}</span>`
      }
      return `<span>${escapeHtml(line)}</span>`
    })
    .join("\n")
})

function escapeHtml(s: string) {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
}
</script>

<template>
  <div v-if="detailModalOpen" class="modal-backdrop" @click.self="closeDetailModal">
    <div class="modal modal-large">
      <div class="modal-header">
        <h3>{{ detailTitle }}</h3>
        <div class="modal-header-actions">
          <button v-if="detailLoading" class="btn-secondary btn-sm" @click="cancelOperation">
            {{ t.btnCancelOp }}
          </button>
          <button class="close-btn" @click="closeDetailModal">×</button>
        </div>
      </div>
      <div v-if="detailLoading" class="modal-status">
        <div class="spinner"></div>
        <span>{{ t.running }}</span>
      </div>
      <pre
        class="modal-output"
        v-html="highlightedText || `<span>${t.waitingOutput}</span>`"
      ></pre>
    </div>
  </div>
</template>
