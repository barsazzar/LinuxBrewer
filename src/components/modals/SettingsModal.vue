<script setup lang="ts">
import { t, lang, setLang } from "../../i18n"
import { showPathSettings, customBrewPath, status, saveBrewPath } from "../../store/brew"
</script>

<template>
  <div v-if="showPathSettings" class="modal-backdrop" @click.self="showPathSettings = false">
    <div class="modal">
      <div class="modal-header">
        <h3>{{ t.settingsTitle }}</h3>
        <button class="close-btn" @click="showPathSettings = false">×</button>
      </div>
      <div class="modal-body">
        <p class="hint">{{ t.settingsHint }}</p>
        <input v-model="customBrewPath" placeholder="/opt/homebrew/bin/brew" class="full-input" />
        <div class="info-box">
          <div class="label">{{ t.settingsCurrentPath }}</div>
          <code class="val">{{ status?.data?.brewPath || t.settingsNotDetected }}</code>
          <div class="hint" style="margin-top: 8px">{{ t.settingsStdPaths }}</div>
        </div>
      </div>

      <div class="settings-divider"></div>
      <div class="settings-section">
        <div class="settings-label">{{ t.settingsLang }}</div>
        <div class="lang-btns">
          <button class="lang-btn" :class="{ active: lang === 'en' }" @click="setLang('en')">English</button>
          <button class="lang-btn" :class="{ active: lang === 'zh' }" @click="setLang('zh')">中文</button>
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn-secondary" @click="customBrewPath = ''; showPathSettings = false">{{ t.btnClear }}</button>
        <button class="btn-primary" @click="saveBrewPath">{{ t.btnSave }}</button>
      </div>
    </div>
  </div>
</template>
