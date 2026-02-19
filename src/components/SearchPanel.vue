<script setup lang="ts">
import { t } from "../i18n"
import {
  searchQuery, searchResults, searchLoading, searchError,
  installedNames, loading,
  onSearchQueryChange, doSearch, installPkg,
} from "../store/brew"
</script>

<template>
  <div class="search-panel">
    <div class="search-bar">
      <svg viewBox="0 0 24 24" fill="none" class="search-icon">
        <circle cx="11" cy="11" r="8" stroke="currentColor" stroke-width="2"/>
        <path d="m21 21-4.35-4.35" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      </svg>
      <input
        v-model="searchQuery"
        @input="onSearchQueryChange"
        @keyup.enter="doSearch"
        :placeholder="t.searchPlaceholder"
        autofocus
      />
      <div v-if="searchLoading" class="spinner search-spinner"></div>
      <button v-if="searchQuery" class="clear-btn" @click="searchQuery = ''; searchResults = []; searchError = ''">×</button>
    </div>

    <div v-if="searchError" class="search-empty">{{ searchError }}</div>
    <div v-else-if="searchResults.length" class="search-results">
      <div
        v-for="pkg in searchResults"
        :key="`sr-${pkg.kind}-${pkg.name}`"
        class="search-row"
      >
        <div class="sr-info">
          <span class="sr-name">{{ pkg.name }}</span>
          <span class="sr-kind" :class="pkg.kind">{{ pkg.kind }}</span>
        </div>
        <div class="sr-actions">
          <span v-if="installedNames.has(pkg.name)" class="installed-tag">{{ t.installedTag }}</span>
          <button
            v-else
            class="btn-install"
            :disabled="loading"
            @click="installPkg(pkg.name, pkg.kind)"
          >
            <svg viewBox="0 0 24 24" fill="none">
              <path d="M12 5v14M5 12h14" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"/>
            </svg>
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
