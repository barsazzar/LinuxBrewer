<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue"
import { t } from "../i18n"
import {
  packages, filteredPackages, selectedPackages,
  search, filterKind, installName, installKind, loading,
  listSearchRef, sortKey, sortDir, pinnedNames,
  toggleSelection, toggleSort, installPkg, uninstallPkg, showInfo, togglePin,
} from "../store/brew"
import { Info, Plus, Pin, Trash2, CircleMinus } from "lucide-vue-next"

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
          <Info :size="14" />
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

    <!-- 排序栏 -->
    <div class="sort-bar">
      <button
        v-for="key in (['name', 'kind', 'version'] as const)"
        :key="key"
        class="sort-btn"
        :class="{ active: sortKey === key }"
        @click="toggleSort(key)"
      >
        {{ key === 'name' ? t.sortName : key === 'kind' ? t.sortKind : t.sortVersion }}
        <span v-if="sortKey === key" class="sort-arrow">{{ sortDir === 'asc' ? '↑' : '↓' }}</span>
      </button>
    </div>

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
        <Plus :size="14" />
        {{ t.btnInstall }}
      </button>
    </div>

    <!-- 包列表 -->
    <div class="package-list">
      <div
        v-for="pkg in filteredPackages"
        :key="`${pkg.kind}-${pkg.name}`"
        class="package-row"
        :class="{ selected: selectedPackages.has(pkg.name), pinned: pinnedNames.has(pkg.name) }"
      >
        <div class="pkg-check">
          <input type="checkbox" :checked="selectedPackages.has(pkg.name)" @change="toggleSelection(pkg.name)" />
        </div>
        <div class="pkg-info">
          <div class="pkg-name">{{ pkg.name }}</div>
          <div class="pkg-meta">
            <span class="pkg-kind" :class="pkg.kind">{{ pkg.kind }}</span>
            <span v-if="pkg.version" class="pkg-version">{{ pkg.version }}</span>
            <span v-if="pinnedNames.has(pkg.name)" class="pin-badge">📌</span>
          </div>
        </div>
        <div class="pkg-actions">
          <button class="icon-btn" @click="showInfo(pkg)" :title="t.ttInfo">
            <Info :size="14" />
          </button>
          <button
            class="icon-btn"
            :class="{ active: pinnedNames.has(pkg.name) }"
            @click="togglePin(pkg)"
            :title="pinnedNames.has(pkg.name) ? t.ttUnpin : t.ttPin"
          >
            <Pin :size="14" />
          </button>
          <button class="icon-btn danger" @click="uninstallPkg(pkg)" :title="t.ttUninstall">
            <Trash2 :size="14" />
          </button>
        </div>
      </div>

      <div v-if="filteredPackages.length === 0" class="list-empty">
        <CircleMinus :size="32" />
        <p>{{ t.noResults }}</p>
      </div>
    </div>
  </section>
</template>
