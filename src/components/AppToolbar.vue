<script setup lang="ts">
import { t } from "../i18n"
import { loading, outdated, showSearch, showLogs, showPathSettings, refreshAll, upgradeAll, runDoctor, runUpdate, runCleanup, status } from "../store/brew"
import { Beer, Search, RotateCw, Download, ArrowUp, Trash2, Stethoscope, ScrollText, Settings } from "lucide-vue-next"
</script>

<template>
  <header class="toolbar">
    <div class="toolbar-left">
      <div class="logo">
        <Beer :size="20" />
        <span>Brew Manager</span>
        <span v-if="status?.ok && status?.data?.version" class="version-badge">{{ status.data.version }}</span>
      </div>
    </div>

    <div class="toolbar-right">
      <button class="tool-btn" :class="{ active: showSearch }" @click="showSearch = !showSearch" :title="t.ttSearch">
        <Search :size="16" />
      </button>
      <div class="sep"></div>
      <button class="tool-btn" :disabled="loading" @click="refreshAll(true)" :title="t.ttRefresh">
        <RotateCw :size="16" />
      </button>
      <button class="tool-btn" :disabled="loading" @click="runUpdate" :title="t.ttUpdate">
        <Download :size="16" />
      </button>
      <button class="tool-btn" :disabled="loading || outdated.length === 0" @click="upgradeAll" :title="t.ttUpgradeAll">
        <ArrowUp :size="16" />
        <span v-if="outdated.length" class="badge">{{ outdated.length }}</span>
      </button>
      <button class="tool-btn" :disabled="loading" @click="runCleanup" :title="t.ttCleanup">
        <Trash2 :size="16" />
      </button>
      <button class="tool-btn" :disabled="loading" @click="runDoctor" title="brew doctor">
        <Stethoscope :size="16" />
      </button>
      <div class="sep"></div>
      <button class="tool-btn" @click="showLogs = !showLogs" :class="{ active: showLogs }" :title="t.ttLogs">
        <ScrollText :size="16" />
      </button>
      <button class="tool-btn" @click="showPathSettings = !showPathSettings" :title="t.ttSettings">
        <Settings :size="16" />
      </button>
    </div>
  </header>
</template>
