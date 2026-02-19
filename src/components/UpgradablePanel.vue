<script setup lang="ts">
import { t } from "../i18n"
import { outdated, loading, upgradeSingle } from "../store/brew"
</script>

<template>
  <section class="panel sidebar-flex-panel">
    <div class="panel-header">
      <h3>{{ t.panelUpgradable }} <span class="count">{{ outdated.length }}</span></h3>
    </div>
    <div class="upgrade-list">
      <div v-if="outdated.length === 0" class="list-empty small">
        <svg viewBox="0 0 24 24" fill="none" width="22" height="22">
          <path d="M9 12l2 2 4-4" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
          <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2"/>
        </svg>
        <p>{{ t.upToDate }}</p>
      </div>
      <div v-for="pkg in outdated" :key="`up-${pkg.kind}-${pkg.name}`" class="upgrade-row">
        <div class="up-info">
          <span class="up-name" :title="pkg.name">{{ pkg.name }}</span>
          <span class="up-kind" :class="pkg.kind">{{ pkg.kind }}</span>
        </div>
        <button class="upgrade-btn" :disabled="loading" @click="upgradeSingle(pkg)" :title="t.ttUpgrade">
          <svg viewBox="0 0 24 24" fill="none">
            <path d="M12 19V5m0 0-7 7m7-7 7 7" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
          </svg>
        </button>
      </div>
    </div>
  </section>
</template>
