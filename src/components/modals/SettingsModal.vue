<script setup lang="ts">
import { ref, onMounted } from "vue"
import { getVersion } from "@tauri-apps/api/app"
import { t, lang, setLang } from "../../i18n"
import { showPathSettings, customBrewPath, status, saveBrewPath } from "../../store/brew"

const activeTab = ref<"settings" | "about" | "license">("settings")
const appVersion = ref("0.1.0")

onMounted(async () => {
  try { appVersion.value = await getVersion() } catch { /* keep default */ }
})
</script>

<template>
  <div v-if="showPathSettings" class="modal-backdrop" @click.self="showPathSettings = false">
    <div class="modal">
      <div class="modal-header">
        <h3>Brew Manager</h3>
        <button class="close-btn" @click="showPathSettings = false">×</button>
      </div>

      <!-- Tab bar -->
      <div class="settings-tabs">
        <button class="settings-tab-btn" :class="{ active: activeTab === 'settings' }" @click="activeTab = 'settings'">{{ t.settingsTabSettings }}</button>
        <button class="settings-tab-btn" :class="{ active: activeTab === 'about' }" @click="activeTab = 'about'">{{ t.settingsTabAbout }}</button>
        <button class="settings-tab-btn" :class="{ active: activeTab === 'license' }" @click="activeTab = 'license'">{{ t.settingsTabLicense }}</button>
      </div>

      <!-- Settings tab -->
      <template v-if="activeTab === 'settings'">
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
      </template>

      <!-- About tab -->
      <template v-else-if="activeTab === 'about'">
        <div class="modal-body about-body">
          <div class="about-icon">
            <svg viewBox="0 0 24 24" fill="none">
              <rect x="4" y="6" width="16" height="14" rx="2" stroke="currentColor" stroke-width="2"/>
              <path d="M8 6V4M12 6V4M16 6V4M8 12h8M8 16h5" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
            </svg>
          </div>
          <div class="about-name">Brew Manager</div>
          <div class="about-version">{{ t.aboutVersion }} {{ appVersion }}</div>
          <p class="about-desc">{{ t.aboutDesc }}</p>
          <div class="about-meta">
            <div>{{ t.aboutAuthor }}: Ding Li</div>
            <div>{{ t.aboutBuiltWith }}</div>
            <div>{{ t.aboutLicense }}</div>
          </div>
          <span class="open-source-badge">{{ t.aboutOpenSource }}</span>
        </div>
      </template>

      <!-- License tab -->
      <template v-else-if="activeTab === 'license'">
        <div class="modal-body">
          <pre class="license-text">MIT License

Copyright (c) 2026 Brew Manager Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.</pre>
        </div>
      </template>
    </div>
  </div>
</template>
