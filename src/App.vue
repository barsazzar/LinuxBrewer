<script setup lang="ts">
import { onMounted, onUnmounted } from "vue"
import { t } from "./i18n"
import {
  initialLoading, status, showSearch, showLogs,
  globalLoading, globalLoadingText, toastMsg,
  initApp, cleanupApp,
} from "./store/brew"
import AppToolbar from "./components/AppToolbar.vue"
import SearchPanel from "./components/SearchPanel.vue"
import InstalledPanel from "./components/InstalledPanel.vue"
import UpgradablePanel from "./components/UpgradablePanel.vue"
import TapsPanel from "./components/TapsPanel.vue"
import LogsDrawer from "./components/LogsDrawer.vue"
import BatchBar from "./components/BatchBar.vue"
import ConfirmModal from "./components/modals/ConfirmModal.vue"
import DetailModal from "./components/modals/DetailModal.vue"
import AddTapModal from "./components/modals/AddTapModal.vue"
import SettingsModal from "./components/modals/SettingsModal.vue"

onMounted(initApp)
onUnmounted(cleanupApp)
</script>

<template>
  <!-- 初始加载 -->
  <div v-if="initialLoading" class="initial-loader">
    <div class="loader-content">
      <div class="spinner-large"></div>
      <h2>Brew Manager</h2>
      <p>{{ t.loading }}</p>
    </div>
  </div>

  <!-- 主应用 -->
  <main v-else class="app">
    <AppToolbar />

    <SearchPanel v-if="showSearch" />

    <div class="workspace">
      <div class="main-area">
        <InstalledPanel />
      </div>
      <aside class="sidebar">
        <UpgradablePanel />
        <TapsPanel />
        <div class="env-bar">
          <span class="env-label">brew</span>
          <code v-if="status?.ok && status.data" class="env-path" :title="status.data.brewPath">
            {{ status.data.brewPath }}
          </code>
          <span v-else class="err-text">{{ status?.message ?? t.settingsNotDetected }}</span>
        </div>
      </aside>
    </div>

    <LogsDrawer v-if="showLogs" />
    <BatchBar />

    <Teleport to="body">
      <ConfirmModal />
      <AddTapModal />
      <DetailModal />
      <SettingsModal />

      <div v-if="globalLoading" class="loading-overlay">
        <div class="loading-card">
          <div class="spinner"></div>
          <p>{{ globalLoadingText }}</p>
        </div>
      </div>

      <div v-if="toastMsg" class="toast">
        <div class="toast-content">{{ toastMsg }}</div>
      </div>
    </Teleport>
  </main>
</template>
