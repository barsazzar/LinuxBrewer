<script setup lang="ts">
import { t } from "../i18n"
import {
  searchQuery, searchResults, searchLoading, searchError,
  installedNames, loading,
  onSearchQueryChange, doSearch, clearSearch,
} from "../store/brew"
import { installPkg } from "../store/brew"
import { Search, Plus } from "lucide-vue-next"

// Type extension for desc field from enrichment
interface EnrichedPackage {
  name: string
  kind: string
  version?: string
  desc?: string
}
</script>

<template>
  <div class="search-panel">
    <div class="search-bar">
      <Search :size="16" class="search-icon" />
      <input
        v-model="searchQuery"
        @input="onSearchQueryChange"
        @keyup.enter="doSearch"
        :placeholder="t.searchPlaceholder"
        autofocus
      />
      <div v-if="searchLoading" class="spinner search-spinner"></div>
      <button v-if="searchQuery" class="clear-btn" @click="clearSearch">×</button>
    </div>

    <div v-if="searchError" class="search-empty">{{ searchError }}</div>
    <div v-else-if="searchResults.length" class="search-results">
      <div
        v-for="pkg in searchResults"
        :key="`sr-${pkg.kind}-${pkg.name}`"
        class="search-row"
      >
        <div class="sr-info">
          <div class="sr-name-row">
            <span class="sr-name">{{ pkg.name }}</span>
            <span class="sr-kind" :class="pkg.kind">{{ pkg.kind }}</span>
            <span v-if="(pkg as EnrichedPackage).version" class="sr-version">v{{ (pkg as EnrichedPackage).version }}</span>
          </div>
          <div v-if="(pkg as EnrichedPackage).desc" class="sr-desc">{{ (pkg as EnrichedPackage).desc }}</div>
        </div>
        <div class="sr-actions">
          <span v-if="installedNames.has(pkg.name)" class="installed-tag">{{ t.installedTag }}</span>
          <button
            v-else
            class="btn-install"
            :disabled="loading"
            @click="installPkg(pkg.name, pkg.kind)"
          >
            <Plus :size="14" />
            {{ t.btnInstall }}
          </button>
        </div>
      </div>
    </div>
    <div v-else-if="searchQuery && !searchLoading" class="search-empty">
      {{ t.searchHint }}
    </div>
  </div>
</template>
