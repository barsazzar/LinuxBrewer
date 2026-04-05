// ── NVM/FNM State ──────────────────────────────────────────────────────────────
import { ref } from "vue"
import { invoke } from "@tauri-apps/api/core"
import { nextTick } from "vue"
import { t } from "../i18n"
import type { ApiResponse, NvmStatus, LtsSlot, ProjectVersion, NvmRefreshResult, RemoteLtsVersion } from "../types"
import {
  showGlobalLoading, hideGlobalLoading,
  detailTitle, detailText, setDetailLoading, activeRequestId, detailModalOpen,
  makeRequestId, showToast, showConfirm,
} from "./ui"

export const nvmStatus = ref<NvmStatus | null>(null)
export const nvmSlots = ref<LtsSlot[]>([])
export const nvmLoading = ref(false)
export const nvmProjectVersion = ref<ProjectVersion | null>(null)
export const nvmProjectDir = ref("")
export const nvmLoadingMajor = ref<number | null>(null)
// F3: 远程 latest 版本映射 major -> latestVersion
export const nvmLatestVersions = ref<Record<number, string>>({})
export const nvmCheckingUpdates = ref(false)
// F1: major -> 该 major 所有可用版本列表（懒加载缓存）
export const nvmRemoteVersions = ref<Record<number, string[]>>({})
export const nvmLoadingVersions = ref<Record<number, boolean>>({})
// S1: 安装后是否设为默认
export const nvmInstallAsDefault = ref(false)

function waitFrames(n = 2): Promise<void> {
  return new Promise(resolve => {
    let count = 0
    const tick = () => { if (++count >= n) resolve(); else requestAnimationFrame(tick) }
    requestAnimationFrame(tick)
  })
}

export async function refreshNvm(opts?: { redetect?: boolean; checkUpdates?: boolean }) {
  if (opts?.redetect) await invoke("nvm_reset_cache")
  nvmLoading.value = true
  try {
    const res = await invoke("nvm_refresh") as ApiResponse<NvmRefreshResult>
    if (res.ok && res.data) {
      nvmStatus.value = res.data.status
      nvmSlots.value = res.data.slots
    }
  } finally {
    nvmLoading.value = false
  }
  // 后台并行：预加载版本列表 + 可选检查远端更新
  preloadAllVersions()
  if (opts?.checkUpdates) fetchRemoteLts()
}

export async function nvmRedetect() {
  await refreshNvm({ redetect: true })
}

// F1: 一次性预加载所有 major 的版本列表
let _allVersionsFetched = false
export async function preloadAllVersions() {
  if (_allVersionsFetched) return
  // 标记全部 major 为 loading
  const majors = nvmSlots.value.map(s => s.major)
  const loadingMap: Record<number, boolean> = {}
  majors.forEach(m => { loadingMap[m] = true })
  nvmLoadingVersions.value = loadingMap

  try {
    const res = await invoke("nvm_fetch_all_versions") as ApiResponse<Record<number, string[]>>
    if (res.ok && res.data) {
      nvmRemoteVersions.value = res.data
      _allVersionsFetched = true
    }
  } finally {
    nvmLoadingVersions.value = {}
  }
}

// F1: 单 major 按需加载（后备，通常不走这里）
export async function fetchVersionsForMajor(major: number) {
  if (nvmRemoteVersions.value[major] || nvmLoadingVersions.value[major]) return
  nvmLoadingVersions.value = { ...nvmLoadingVersions.value, [major]: true }
  try {
    const res = await invoke("nvm_fetch_versions_for_major", { major }) as ApiResponse<string[]>
    if (res.ok && res.data) {
      nvmRemoteVersions.value = { ...nvmRemoteVersions.value, [major]: res.data }
    }
  } finally {
    const copy = { ...nvmLoadingVersions.value }
    delete copy[major]
    nvmLoadingVersions.value = copy
  }
}

// F3: 拉取远程 LTS 最新版并填充到 nvmLatestVersions
export async function fetchRemoteLts() {
  nvmCheckingUpdates.value = true
  try {
    const res = await invoke("nvm_fetch_remote_lts") as ApiResponse<RemoteLtsVersion[]>
    if (res.ok && res.data) {
      const map: Record<number, string> = {}
      for (const item of res.data) {
        map[item.major] = item.latest
      }
      nvmLatestVersions.value = map
      // 把 latestAvailable 回填到 slots（响应式更新）
      nvmSlots.value = nvmSlots.value.map(slot => ({
        ...slot,
        latestAvailable: map[slot.major],
      }))
    }
  } finally {
    nvmCheckingUpdates.value = false
  }
}

async function runNvmStream(action: string, version: string, title: string, loadingText: string): Promise<boolean> {
  showGlobalLoading(loadingText)
  await nextTick()
  await waitFrames(3)

  detailTitle.value = title
  detailText.value = ""
  setDetailLoading(true)
  const requestId = makeRequestId()
  activeRequestId.value = requestId
  hideGlobalLoading()
  detailModalOpen.value = true
  await nextTick()
  await waitFrames(2)

  try {
    const res = await invoke("nvm_run_stream", { requestId, action, version }) as ApiResponse<boolean>
    return res.ok
  } catch (e) {
    setDetailLoading(false)
    detailText.value += t.value.logException(String(e))
    return false
  }
}

export async function nvmInstall(versionOrMajor: string) {
  const major = parseInt(versionOrMajor.replace(/^v/, "").split(".")[0])
  nvmLoadingMajor.value = major
  try {
    const ok = await runNvmStream(
      "install", versionOrMajor,
      t.value.nvmInstallTitle(versionOrMajor),
      t.value.nvmInstalling(versionOrMajor),
    )
    if (ok) {
      showToast(t.value.nvmInstallDone(versionOrMajor))
      // S1: 安装后设为默认
      if (nvmInstallAsDefault.value) {
        const installed = await getInstalledVersion(major)
        if (installed) await nvmSetDefault(installed)
      }
      await refreshNvm()
    }
  } finally {
    nvmLoadingMajor.value = null
  }
}

// 安装完成后找到刚装的版本号（用于 S1）
async function getInstalledVersion(major: number): Promise<string | null> {
  try {
    const res = await invoke("nvm_refresh") as ApiResponse<NvmRefreshResult>
    if (!res.ok || !res.data) return null
    const slot = res.data.slots.find(s => s.major === major)
    return slot?.installed?.version ?? null
  } catch { return null }
}

export async function nvmUninstall(version: string) {
  const major = parseInt(version.replace(/^v/, "").split(".")[0])
  const confirmed = await showConfirm(
    t.value.nvmConfirmUninstallTitle,
    t.value.nvmConfirmUninstall(version),
  )
  if (!confirmed) return

  nvmLoadingMajor.value = major
  try {
    const ok = await runNvmStream(
      "uninstall", version,
      t.value.nvmUninstallTitle(version),
      t.value.nvmUninstalling(version),
    )
    if (ok) {
      showToast(t.value.nvmUninstallDone(version))
      await refreshNvm()
    }
  } finally {
    nvmLoadingMajor.value = null
  }
}

export async function nvmSetDefault(version: string) {
  const res = await invoke("nvm_set_default", { version }) as ApiResponse<boolean>
  if (res.ok) {
    showToast(t.value.nvmSetDefaultDone(version))
    await refreshNvm()
  } else {
    showToast(res.message)
  }
}

export async function nvmReadProject(dir: string) {
  nvmProjectVersion.value = null
  if (!dir.trim()) return
  const res = await invoke("nvm_read_project", { dir }) as ApiResponse<ProjectVersion>
  if (res.ok && res.data) {
    nvmProjectVersion.value = res.data
  }
}

export async function nvmWriteProject(dir: string, version: string) {
  const res = await invoke("nvm_write_project", { dir, version }) as ApiResponse<boolean>
  if (res.ok) {
    showToast(t.value.nvmProjectSaved)
    await nvmReadProject(dir)
  } else {
    showToast(res.message)
  }
}

export async function nvmCopyBinPath(version: string) {
  const res = await invoke("nvm_which", { version }) as ApiResponse<string>
  if (res.ok && res.data) {
    navigator.clipboard.writeText(res.data)
    showToast(t.value.nvmPathCopied(version))
  } else {
    showToast(t.value.nvmWhichFailed)
  }
}
