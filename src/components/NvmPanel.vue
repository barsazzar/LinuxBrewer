<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue"
import { open as openDialog } from "@tauri-apps/plugin-dialog"
import { t } from "../i18n"
import {
  nvmStatus, nvmSlots, nvmLoading, nvmLoadingMajor,
  nvmProjectVersion, nvmProjectDir,
  nvmInstallAsDefault,
  nvmRemoteVersions, nvmLoadingVersions,
  refreshNvm, nvmInstall, nvmUninstall, nvmSetDefault, nvmRedetect,
  nvmReadProject, nvmWriteProject, nvmCopyBinPath,
  fetchVersionsForMajor,
} from "../store/nvm"
import { RefreshCw, Download, Trash2, Star, FolderOpen, Save, ScanSearch, Copy, Loader2, ChevronDown, ChevronUp, PlusCircle } from "lucide-vue-next"

const EOL_MAJORS = new Set([16])

const projectVersionInput = ref("")
const expandedMajors = ref<Set<number>>(new Set())
// F1: 各卡片选中的版本（select v-model），默认空 = 装最新
const selectedVersions = ref<Record<number, string>>({})
// F1: 已安装卡片是否展开安装其他版本区域
const showInstallOther = ref<Set<number>>(new Set())

// slots 加载后为每个 major 初始化 "" — 保证 select 第一项（placeholder）能正常选中
watch(nvmSlots, (slots) => {
  slots.forEach(s => {
    if (!(s.major in selectedVersions.value)) {
      selectedVersions.value[s.major] = ""
    }
  })
}, { immediate: true })

onMounted(refreshNvm)

const managerLabel = computed(() => {
  if (!nvmStatus.value || nvmStatus.value.manager === "none") return null
  const ver = nvmStatus.value.managerVersion ?? ""
  return t.value.nvmManager(nvmStatus.value.manager, ver)
})

const defaultLabel = computed(() => {
  if (!nvmStatus.value) return ""
  return nvmStatus.value.nodeDefault
    ? t.value.nvmDefault(nvmStatus.value.nodeDefault)
    : t.value.nvmNoDefault
})

const installedCount = computed(() =>
  nvmSlots.value.filter(s => s.installed).length
)

function toggleExpand(major: number) {
  if (expandedMajors.value.has(major)) expandedMajors.value.delete(major)
  else expandedMajors.value.add(major)
}

function toggleInstallOther(major: number) {
  if (showInstallOther.value.has(major)) showInstallOther.value.delete(major)
  else { showInstallOther.value.add(major); fetchVersionsForMajor(major) }
}

// 下拉展开时：预加载已完成则什么都不做，否则触发单 major 加载作为 fallback
function onSelectOpen(major: number) {
  if (!nvmRemoteVersions.value[major]) fetchVersionsForMajor(major)
}

function installVersion(major: number) {
  const selected = selectedVersions.value[major]?.trim()
  nvmInstall(selected || String(major))
}

function hasUpdate(slot: { installed?: { version: string }, latestAvailable?: string }): boolean {
  if (!slot.installed || !slot.latestAvailable) return false
  return slot.installed.version !== slot.latestAvailable
}

async function loadProject() {
  await nvmReadProject(nvmProjectDir.value)
  projectVersionInput.value = nvmProjectVersion.value?.version ?? ""
}

async function saveProject() {
  if (!nvmProjectDir.value.trim() || !projectVersionInput.value.trim()) return
  await nvmWriteProject(nvmProjectDir.value, projectVersionInput.value)
}

async function pickFolder() {
  const dir = await openDialog({ directory: true, multiple: false, title: "Select project directory" })
  if (typeof dir === "string" && dir) {
    nvmProjectDir.value = dir
    await loadProject()
  }
}
</script>

<template>
  <div class="nvm-panel">

    <!-- ── Status bar ──────────────────────────────────────────────────── -->
    <div class="nvm-status-bar">
      <template v-if="managerLabel">
        <span class="nvm-manager-badge">{{ nvmStatus!.manager }}</span>
        <span class="nvm-manager-ver">{{ nvmStatus!.managerVersion }}</span>
        <span class="nvm-sep">·</span>
        <span class="nvm-default-label">{{ defaultLabel }}</span>
        <span v-if="installedCount > 0" class="nvm-sep">·</span>
        <span v-if="installedCount > 0" class="nvm-installed-count">{{ t.nvmInstalledCount(installedCount) }}</span>
      </template>
      <template v-else>
        <span class="nvm-not-found">{{ t.nvmNotFound }}</span>
      </template>

      <div class="nvm-status-actions">
        <!-- 重新检测 fnm/nvm 安装路径 -->
        <button
          class="icon-btn small"
          :disabled="nvmLoading"
          :title="t.nvmRedetect"
          @click="nvmRedetect"
        >
          <ScanSearch :size="14" />
        </button>
        <!-- 刷新数据并检查远端版本更新 -->
        <button
          class="icon-btn small"
          :disabled="nvmLoading"
          :title="t.nvmRefreshAndCheck"
          @click="refreshNvm({ checkUpdates: true })"
        >
          <RefreshCw :size="13" :class="{ spinning: nvmLoading }" />
        </button>
      </div>
    </div>

    <!-- Not-found hint -->
    <div v-if="!managerLabel && !nvmLoading" class="nvm-hint-box">
      <p>{{ t.nvmNotFoundHint }}</p>
      <code>curl -fsSL https://fnm.vercel.app/install | bash</code>
    </div>

    <!-- S1: 安装后设为默认 -->
    <label v-if="managerLabel" class="nvm-default-toggle">
      <input type="checkbox" v-model="nvmInstallAsDefault" />
      <span>{{ t.nvmSetAsDefault }}</span>
    </label>

    <!-- ── LTS version grid ────────────────────────────────────────────── -->
    <div class="nvm-grid">
      <div
        v-for="slot in nvmSlots"
        :key="slot.major"
        class="nvm-card"
        :class="{
          'nvm-card--installed': slot.installed && !slot.installed.isCurrent,
          'nvm-card--current':   slot.installed?.isCurrent,
          'nvm-card--loading':   nvmLoadingMajor === slot.major,
        }"
      >
        <div v-if="nvmLoadingMajor === slot.major" class="nvm-card-loading-overlay">
          <Loader2 :size="20" class="spinning" />
        </div>

        <!-- 卡片头 -->
        <div class="nvm-card-header">
          <div class="nvm-major-row">
            <span class="nvm-major">v{{ slot.major }}</span>
            <span v-if="!slot.isLts" class="nvm-badge nvm-badge--extra">custom</span>
            <span v-else-if="EOL_MAJORS.has(slot.major)" class="nvm-badge nvm-badge--eol">{{ t.nvmEol }}</span>
          </div>
          <span class="nvm-lts-name">{{ slot.ltsName }}</span>
        </div>

        <!-- ── 已安装 ── -->
        <template v-if="slot.installed">
          <div class="nvm-version-row">
            <span class="nvm-version">{{ slot.installed.version }}</span>
            <span v-if="slot.installed.isCurrent" class="nvm-badge nvm-badge--current">{{ t.nvmCurrent }}</span>
            <span v-if="slot.installed.isDefault" class="nvm-badge nvm-badge--default">default</span>
          </div>

          <!-- F3: 更新提示 -->
          <div v-if="hasUpdate(slot)" class="nvm-update-row">
            <span class="nvm-update-badge">{{ t.nvmUpdateAvailable(slot.latestAvailable!) }}</span>
          </div>

          <!-- E1: 多版本展开 -->
          <template v-if="slot.allInstalled.length > 1">
            <button class="nvm-expand-btn" @click="toggleExpand(slot.major)">
              <template v-if="!expandedMajors.has(slot.major)">
                <ChevronDown :size="11" />{{ t.nvmMoreVersions(slot.allInstalled.length - 1) }}
              </template>
              <template v-else>
                <ChevronUp :size="11" />{{ t.nvmCollapseVersions }}
              </template>
            </button>
            <div v-if="expandedMajors.has(slot.major)" class="nvm-extra-versions">
              <div
                v-for="v in slot.allInstalled.filter(v => v.version !== slot.installed!.version)"
                :key="v.version"
                class="nvm-extra-version-row"
              >
                <span class="nvm-version">{{ v.version }}</span>
                <span v-if="v.isCurrent" class="nvm-badge nvm-badge--current">{{ t.nvmCurrent }}</span>
                <span v-if="v.isDefault" class="nvm-badge nvm-badge--default">default</span>
                <button v-if="!v.isDefault" class="nvm-btn nvm-btn--ghost nvm-btn--icon" :disabled="nvmLoadingMajor !== null" @click="nvmSetDefault(v.version)"><Star :size="10" /></button>
                <button class="nvm-btn nvm-btn--danger nvm-btn--icon" :disabled="nvmLoadingMajor !== null" @click="nvmUninstall(v.version)"><Trash2 :size="10" /></button>
              </div>
            </div>
          </template>

          <div class="nvm-card-actions">
            <button v-if="!slot.installed.isDefault" class="nvm-btn nvm-btn--ghost" :disabled="nvmLoadingMajor !== null" @click="nvmSetDefault(slot.installed!.version)">
              <Star :size="12" />{{ t.nvmSetDefault }}
            </button>
            <button class="nvm-btn nvm-btn--ghost nvm-btn--icon" :title="t.nvmCopyPath" :disabled="nvmLoadingMajor !== null" @click="nvmCopyBinPath(slot.installed!.version)"><Copy :size="12" /></button>
            <button class="nvm-btn nvm-btn--danger" :disabled="nvmLoadingMajor !== null" @click="nvmUninstall(slot.installed!.version)"><Trash2 :size="12" /></button>
          </div>

          <!-- F1: 已安装卡片也可装其他 patch 版本 -->
          <button class="nvm-install-other-btn" @click="toggleInstallOther(slot.major)">
            <PlusCircle :size="11" />
            {{ showInstallOther.has(slot.major) ? t.nvmCollapseVersions : t.nvmInstallOther }}
          </button>
          <div v-if="showInstallOther.has(slot.major)" class="nvm-install-other-area">
            <select
              v-model="selectedVersions[slot.major]"
              class="nvm-version-select"
              :disabled="nvmLoadingVersions[slot.major]"
              @focus="onSelectOpen(slot.major)"
            >
              <option value="">{{ nvmLoadingVersions[slot.major] ? t.nvmLoadingVersions : t.nvmSelectVersion }}</option>
              <option
                v-for="ver in nvmRemoteVersions[slot.major]"
                :key="ver"
                :value="ver"
              >{{ ver }}</option>
            </select>
            <button class="nvm-btn nvm-btn--install nvm-btn--compact" :disabled="nvmLoadingMajor !== null" @click="installVersion(slot.major)">
              <Download :size="11" />
            </button>
          </div>
        </template>

        <!-- ── 未安装 ── -->
        <template v-else>
          <div class="nvm-not-installed">{{ t.nvmNotInstalled }}</div>
          <!-- F1: 版本选择下拉菜单，懒加载 -->
          <select
            v-model="selectedVersions[slot.major]"
            class="nvm-version-select"
            :disabled="nvmLoadingVersions[slot.major]"
            @focus="onSelectOpen(slot.major)"
          >
            <option value="">{{ nvmLoadingVersions[slot.major] ? t.nvmLoadingVersions : t.nvmSelectVersion }}</option>
            <option
              v-for="ver in nvmRemoteVersions[slot.major]"
              :key="ver"
              :value="ver"
            >{{ ver }}</option>
          </select>
          <div class="nvm-card-actions">
            <button class="nvm-btn nvm-btn--install" :disabled="!managerLabel || nvmLoadingMajor !== null" @click="installVersion(slot.major)">
              <Download :size="12" />Install
            </button>
          </div>
        </template>
      </div>
    </div>

    <!-- ── Project .nvmrc section ─────────────────────────────────────── -->
    <div class="nvm-project-section">
      <div class="nvm-project-title">{{ t.nvmProjectTitle }}</div>
      <div class="nvm-project-row">
        <FolderOpen :size="14" class="nvm-project-icon" />
        <input v-model="nvmProjectDir" class="nvm-input" :placeholder="t.nvmProjectDirPlaceholder" @keydown.enter="loadProject" />
        <button class="nvm-btn nvm-btn--ghost" @click="pickFolder">{{ t.nvmPickFolder }}</button>
        <button class="nvm-btn nvm-btn--ghost" @click="loadProject">{{ t.nvmProjectLoad }}</button>
      </div>
      <div v-if="nvmProjectVersion" class="nvm-project-row">
        <span class="nvm-project-current">{{ t.nvmProjectCurrent(nvmProjectVersion.version) }}</span>
        <span class="nvm-project-file">{{ t.nvmFileLabel(nvmProjectVersion.file.split('/').pop()!) }}</span>
        <input v-model="projectVersionInput" class="nvm-input nvm-input--short" :placeholder="t.nvmProjectVersionPlaceholder" @keydown.enter="saveProject" />
        <button class="nvm-btn nvm-btn--primary" @click="saveProject">
          <Save :size="12" />{{ t.nvmProjectSave }}
        </button>
      </div>
      <div v-else-if="nvmProjectDir" class="nvm-project-none">{{ t.nvmProjectNone }}</div>
    </div>
  </div>
</template>
