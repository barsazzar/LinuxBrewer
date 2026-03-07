<script setup lang="ts">
import { ref } from "vue"
import { t } from "../i18n"
import { outdated, loading, selectedPackages, toggleSelection, upgradeSingle } from "../store/brew"
import { Info, CircleCheck, ArrowUp } from "lucide-vue-next"

const showDesc = ref(false)
</script>

<template>
  <section class="panel sidebar-flex-panel">
    <div class="panel-header">
      <div class="panel-title-group">
        <h3>{{ t.panelUpgradable }} <span class="count">{{ outdated.length }}</span></h3>
        <button class="icon-btn small" :class="{ active: showDesc }" @click="showDesc = !showDesc">
          <Info :size="14" />
        </button>
      </div>
    </div>
    <div v-if="showDesc" class="panel-desc">{{ t.descUpgradable }}</div>
    <div class="upgrade-list">
      <div v-if="outdated.length === 0" class="list-empty small">
        <CircleCheck :size="22" />
        <p>{{ t.upToDate }}</p>
      </div>
      <div v-for="pkg in outdated" :key="`up-${pkg.kind}-${pkg.name}`" class="upgrade-row" :class="{ selected: selectedPackages.has(pkg.name) }">
        <div class="pkg-check-small">
          <input type="checkbox" :checked="selectedPackages.has(pkg.name)" @change="toggleSelection(pkg.name)" />
        </div>
        <div class="up-info">
          <span class="up-name" :title="pkg.name">{{ pkg.name }}</span>
          <span class="up-kind" :class="pkg.kind">{{ pkg.kind }}</span>
          <!-- Version diff display -->
          <span v-if="pkg.version || pkg.newVersion" class="up-version-diff">
            <span class="ver-old">{{ pkg.version ?? '?' }}</span>
            <span class="ver-arrow">→</span>
            <span class="ver-new">{{ pkg.newVersion ?? '?' }}</span>
          </span>
        </div>
        <button class="upgrade-btn" :disabled="loading" @click="upgradeSingle(pkg)" :title="t.ttUpgrade">
          <ArrowUp :size="14" />
        </button>
      </div>
    </div>
  </section>
</template>
