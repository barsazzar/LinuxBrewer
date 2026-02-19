<script setup lang="ts">
import { t } from "../../i18n"
import { showAddTapModal, newTapName, addTap } from "../../store/brew"
</script>

<template>
  <div v-if="showAddTapModal" class="modal-backdrop" @click.self="showAddTapModal = false">
    <div class="modal">
      <div class="modal-header">
        <h3>{{ t.modalAddTap }}</h3>
        <button class="close-btn" @click="showAddTapModal = false">×</button>
      </div>
      <div class="modal-body">
        <p class="hint">{{ t.tapHint }}</p>
        <input
          v-model="newTapName"
          placeholder="user/repo"
          class="full-input"
          @keyup.enter="addTap"
          autofocus
        />
      </div>
      <div class="modal-footer">
        <button class="btn-secondary" @click="showAddTapModal = false">{{ t.btnCancel }}</button>
        <button class="btn-primary" @click="addTap" :disabled="!newTapName">{{ t.btnAdd }}</button>
      </div>
    </div>
  </div>
</template>
