<script setup lang="ts">
import { t } from "../i18n"
import { loading, outdated, showSearch, showLogs, showPathSettings, refreshAll, upgradeAll, runDoctor } from "../store/brew"
</script>

<template>
  <header class="toolbar">
    <div class="toolbar-left">
      <div class="logo">
        <svg viewBox="0 0 24 24" fill="none">
          <rect x="4" y="6" width="16" height="14" rx="2" stroke="currentColor" stroke-width="2"/>
          <path d="M8 6V4M12 6V4M16 6V4M8 12h8M8 16h5" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        </svg>
        <span>Brew Manager</span>
      </div>
    </div>

    <div class="toolbar-right">
      <button class="tool-btn" :class="{ active: showSearch }" @click="showSearch = !showSearch" :title="t.ttSearch">
        <svg viewBox="0 0 24 24" fill="none">
          <circle cx="11" cy="11" r="8" stroke="currentColor" stroke-width="2"/>
          <path d="m21 21-4.35-4.35" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        </svg>
      </button>
      <div class="sep"></div>
      <button class="tool-btn" :disabled="loading" @click="refreshAll(true)" :title="t.ttRefresh">
        <svg viewBox="0 0 24 24" fill="none">
          <path d="M21 10h-4M21 10a8 8 0 1 1-1.4-4.5" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
          <path d="M21 6v4" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        </svg>
      </button>
      <button class="tool-btn" :disabled="loading || outdated.length === 0" @click="upgradeAll" :title="t.ttUpgradeAll">
        <svg viewBox="0 0 24 24" fill="none">
          <path d="M12 19V5m0 0-7 7m7-7 7 7" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        </svg>
        <span v-if="outdated.length" class="badge">{{ outdated.length }}</span>
      </button>
      <button class="tool-btn" :disabled="loading" @click="runDoctor" title="brew doctor">
        <svg viewBox="0 0 24 24" fill="none">
          <circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="2"/>
          <path d="M12 8v4M12 16h.01" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        </svg>
      </button>
      <div class="sep"></div>
      <button class="tool-btn" @click="showLogs = !showLogs" :class="{ active: showLogs }" :title="t.ttLogs">
        <svg viewBox="0 0 24 24" fill="none">
          <rect x="3" y="3" width="18" height="18" rx="2" stroke="currentColor" stroke-width="2"/>
          <path d="M8 8h8M8 12h8M8 16h5" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        </svg>
      </button>
      <button class="tool-btn" @click="showPathSettings = !showPathSettings" :title="t.ttSettings">
        <svg viewBox="0 0 24 24" fill="none">
          <circle cx="12" cy="12" r="3" stroke="currentColor" stroke-width="2"/>
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" stroke="currentColor" stroke-width="2"/>
        </svg>
      </button>
    </div>
  </header>
</template>
