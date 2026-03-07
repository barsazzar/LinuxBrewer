<script setup lang="ts">
import { ref } from "vue"
import { t } from "../i18n"
import { taps, loading, showAddTapModal, removeTap } from "../store/brew"
import { Info, Plus, X } from "lucide-vue-next"

const showDesc = ref(false)
</script>

<template>
  <section class="panel sidebar-flex-panel">
    <div class="panel-header">
      <div class="panel-title-group">
        <h3>Taps <span class="count">{{ taps.length }}</span></h3>
        <button class="icon-btn small" :class="{ active: showDesc }" @click="showDesc = !showDesc">
          <Info :size="14" />
        </button>
      </div>
      <button class="tap-add-btn" @click="showAddTapModal = true">
        <Plus :size="12" />
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
          <X :size="14" />
        </button>
      </div>
    </div>
  </section>
</template>
