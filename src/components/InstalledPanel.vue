<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue"
import { t } from "../i18n"
import {
  packages, filteredPackages, selectedPackages,
  search, filterKind, installName, installKind, loading,
  listSearchRef,
  toggleSelection, installPkg, uninstallPkg, showInfo,
} from "../store/brew"

const inputEl = ref<HTMLInputElement | null>(null)
onMounted(() => { listSearchRef.value = inputEl.value })
onUnmounted(() => { listSearchRef.value = null })

const showDesc = ref(false)
</script>

<template>
  <section class="panel flex-panel">
    <!-- 标题栏 -->
    <div class="panel-header">
      <div class="panel-title-group">
        <h3>{{ t.panelInstalled }} <span class="count">{{ filteredPackages.length }} / {{ packages.length }}</span></h3>
        <button class="icon-btn small" :class="{ active: showDesc }" @click="showDesc = !showDesc">
          <svg viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2"/>
            <path d="M12 16v-4M12 8h.01" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
          </svg>
        </button>
      </div>
      <div class="header-controls">
        <div class="filter-tabs">
          <button class="filter-tab" :class="{ active: filterKind === 'all' }" @click="filterKind = 'all'">{{ t.filterAll }}</button>
          <button class="filter-tab" :class="{ active: filterKind === 'formula' }" @click="filterKind = 'formula'">Formula</button>
          <button class="filter-tab" :class="{ active: filterKind === 'cask' }" @click="filterKind = 'cask'">Cask</button>
        </div>
        <input
          ref="inputEl"
          v-model="search"
          :placeholder="t.filterPlaceholder"
          class="list-search"
        />
      </div>
    </div>

    <!-- 面板描述 -->
    <div v-if="showDesc" class="panel-desc">{{ t.descInstalled }}</div>

    <!-- 快速安装行 -->
    <div class="install-row">
      <input
        v-model="installName"
        :placeholder="t.installPlaceholder"
        @keyup.enter="installPkg()"
      />
      <select v-model="installKind">
        <option value="formula">Formula</option>
        <option value="cask">Cask</option>
      </select>
      <button class="btn-primary" :disabled="loading" @click="installPkg()">
        <svg viewBox="0 0 24 24" fill="none">
          <path d="M12 5v14M5 12h14" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        </svg>
        {{ t.btnInstall }}
      </button>
    </div>

    <!-- 包列表 -->
    <div class="package-list">
      <div
        v-for="pkg in filteredPackages"
        :key="`${pkg.kind}-${pkg.name}`"
        class="package-row"
        :class="{ selected: selectedPackages.has(pkg.name) }"
      >
        <div class="pkg-check">
          <input type="checkbox" :checked="selectedPackages.has(pkg.name)" @change="toggleSelection(pkg.name)" />
        </div>
        <div class="pkg-info">
          <div class="pkg-name">{{ pkg.name }}</div>
          <div class="pkg-meta">
            <span class="pkg-kind" :class="pkg.kind">{{ pkg.kind }}</span>
            <span v-if="pkg.version" class="pkg-version">{{ pkg.version }}</span>
          </div>
        </div>
        <div class="pkg-actions">
          <button class="icon-btn" @click="showInfo(pkg)" :title="t.ttInfo">
            <svg viewBox="0 0 24 24" fill="none">
              <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2"/>
              <path d="M12 16v-4M12 8h.01" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
            </svg>
          </button>
          <button class="icon-btn danger" @click="uninstallPkg(pkg)" :title="t.ttUninstall">
            <svg viewBox="0 0 24 24" fill="none">
              <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6M8 6V4h8v2" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
              <line x1="10" y1="11" x2="10" y2="17" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
              <line x1="14" y1="11" x2="14" y2="17" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
            </svg>
          </button>
        </div>
      </div>

      <div v-if="filteredPackages.length === 0" class="list-empty">
        <svg viewBox="0 0 24 24" fill="none" width="32" height="32">
          <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2"/>
          <path d="M8 12h8" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        </svg>
        <p>{{ t.noResults }}</p>
      </div>
    </div>
  </section>
</template>
