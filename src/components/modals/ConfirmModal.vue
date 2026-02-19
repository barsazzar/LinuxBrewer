<script setup lang="ts">
import { t } from "../../i18n"
import { confirmVisible, confirmTitle, confirmMessage, handleConfirm } from "../../store/brew"
</script>

<template>
  <div v-if="confirmVisible" class="modal-backdrop" @click.self="handleConfirm(false)">
    <div class="modal confirm-modal">
      <div class="modal-header">
        <h3>{{ confirmTitle }}</h3>
      </div>
      <div class="modal-body">
        <p class="confirm-msg">{{ confirmMessage }}</p>
      </div>
      <div class="modal-footer">
        <button class="btn-secondary" @click="handleConfirm(false)">{{ t.btnCancel }}</button>
        <button class="btn-danger" @click="handleConfirm(true)">{{ t.btnConfirmOk }}</button>
      </div>
    </div>
  </div>
</template>
