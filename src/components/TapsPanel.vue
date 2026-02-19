<script setup lang="ts">
import { ref } from "vue"
import { t } from "../i18n"
import { taps, loading, showAddTapModal, removeTap } from "../store/brew"

const showDesc = ref(false)
</script>

<template>
  <section class="panel sidebar-flex-panel">
    <div class="panel-header">
      <div class="panel-title-group">
        <h3>Taps <span class="count">{{ taps.length }}</span></h3>
        <button class="icon-btn small" :class="{ active: showDesc }" @click="showDesc = !showDesc">
          <svg viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2"/>
            <path d="M12 16v-4M12 8h.01" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
          </svg>
        </button>
      </div>
      <button class="tap-add-btn" @click="showAddTapModal = true">
        <svg viewBox="0 0 24 24" fill="none" width="12" height="12">
          <path d="M12 5v14M5 12h14" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"/>
        </svg>
        {{ t.btnAdd }}
      </button>
    </div>
    <div v-if="showDesc" class="panel-desc">{{ t.descTaps }}</div>
    <div class="upgrade-list">
      <div v-if="taps.length === 0" class="list-empty small">
        <p>{{ t.noTaps }}</p>
      </div>
      <div v-for="tap in taps" :key="tap" class="upgrade-row tap-row">
        <div class="up-info">
          <span class="up-name tap-name" :title="tap">{{ tap }}</span>
        </div>
        <button class="icon-btn danger small tap-remove" :disabled="loading" @click="removeTap(tap)" :title="t.ttRemoveTap">
          <svg viewBox="0 0 24 24" fill="none">
            <path d="M18 6L6 18M6 6l12 12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
          </svg>
        </button>
      </div>
    </div>
  </section>
</template>
