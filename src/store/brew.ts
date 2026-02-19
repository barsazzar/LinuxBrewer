import { ref, computed, nextTick } from "vue"
import { invoke } from "@tauri-apps/api/core"
import { listen, type UnlistenFn } from "@tauri-apps/api/event"
import { sendNotification } from "@tauri-apps/plugin-notification"
import { t } from "../i18n"
import type { ApiResponse, BrewStatus, BrewPackage, BrewKind, BrewLogEvent } from "../types"

// ── 私有工具 ──────────────────────────────────────────────────────────────────
let unlistenBrewLog: UnlistenFn | null = null
let searchTimer: ReturnType<typeof setTimeout> | null = null
let toastTimer: ReturnType<typeof setTimeout> | null = null
let confirmResolve: ((v: boolean) => void) | null = null

function makeRequestId() {
  return `req-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`
}

function showGlobalLoading(text: string) {
  globalLoadingText.value = text
  globalLoading.value = true
}

function hideGlobalLoading() {
  globalLoading.value = false
}

function setDetailLoading(v: boolean) {
  if (v) {
    detailLoadingStartedAt.value = Date.now()
    detailLoading.value = true
    return
  }
  const rest = Math.max(0, MIN_LOADING_VISIBLE_MS - (Date.now() - detailLoadingStartedAt.value))
  if (rest > 0) setTimeout(() => { detailLoading.value = false }, rest)
  else detailLoading.value = false
}

function pushLog(text: string) {
  const ts = new Date().toLocaleTimeString()
  logs.value.unshift(`[${ts}] ${text}`)
}

export function showToast(msg: string) {
  toastMsg.value = msg
  if (toastTimer) clearTimeout(toastTimer)
  toastTimer = setTimeout(() => { toastMsg.value = "" }, 3000)
}

async function notify(title: string, body: string) {
  showToast(`${title}: ${body}`)
  try { await sendNotification({ title, body }) } catch { /* silent */ }
}

function showConfirm(title: string, message: string): Promise<boolean> {
  confirmTitle.value = title
  confirmMessage.value = message
  confirmVisible.value = true
  return new Promise(resolve => { confirmResolve = resolve })
}

// ── 状态（模块级单例）────────────────────────────────────────────────────────
const MIN_LOADING_VISIBLE_MS = 500
const detailLoadingStartedAt = ref(0)

export const initialLoading = ref(true)
export const loading = ref(false)
export const status = ref<ApiResponse<BrewStatus> | null>(null)
export const packages = ref<BrewPackage[]>([])
export const outdated = ref<BrewPackage[]>([])
export const taps = ref<string[]>([])
export const selectedPackages = ref(new Set<string>())
export const isSelectionMode = computed(() => selectedPackages.value.size > 0)

export const search = ref("")
export const filterKind = ref<"all" | BrewKind>("all")
export const installName = ref("")
export const installKind = ref<BrewKind>("formula")
export const logs = ref<string[]>([])
export const showLogs = ref(false)
export const customBrewPath = ref("")
export const showPathSettings = ref(false)

export const searchQuery = ref("")
export const searchResults = ref<BrewPackage[]>([])
export const searchLoading = ref(false)
export const searchError = ref("")
export const showSearch = ref(false)

export const showAddTapModal = ref(false)
export const newTapName = ref("")

export const confirmVisible = ref(false)
export const confirmTitle = ref("")
export const confirmMessage = ref("")

export const detailTitle = ref("")
export const detailText = ref("")
export const detailModalOpen = ref(false)
export const detailLoading = ref(false)
export const activeRequestId = ref("")

export const globalLoading = ref(false)
export const globalLoadingText = ref("")
export const toastMsg = ref("")

export const listSearchRef = ref<HTMLInputElement | null>(null)

export const filteredPackages = computed(() => {
  let list = packages.value
  if (filterKind.value !== "all") list = list.filter(p => p.kind === filterKind.value)
  const q = search.value.trim().toLowerCase()
  if (q) list = list.filter(p => p.name.toLowerCase().includes(q))
  return list
})

export const installedNames = computed(() => new Set(packages.value.map(p => p.name)))

// ── 核心操作 ──────────────────────────────────────────────────────────────────
export async function refreshAll(showLoader = false) {
  loading.value = true
  if (showLoader) {
    showGlobalLoading(t.value.loadingRefresh)
    await nextTick()
    await new Promise(r => setTimeout(r, 50))
  }
  try {
    selectedPackages.value.clear()
    const s = (await invoke("brew_status")) as ApiResponse<BrewStatus>
    status.value = s

    const [listRes, outRes, tapRes] = await Promise.all([
      invoke("brew_list_installed") as Promise<ApiResponse<BrewPackage[]>>,
      invoke("brew_outdated") as Promise<ApiResponse<BrewPackage[]>>,
      invoke("brew_tap_list") as Promise<ApiResponse<string[]>>,
    ])

    if (listRes.ok && listRes.data) {
      packages.value = listRes.data
      pushLog(t.value.logLoaded(listRes.data.length))
    } else {
      pushLog(t.value.logLoadFailed(listRes.message))
    }

    if (outRes.ok && outRes.data) {
      outdated.value = outRes.data
      const count = outRes.data.length
      if (count) pushLog(t.value.logUpdatable(count))
      invoke("update_tray", {
        title: count > 0 ? `Brew Manager (${count})` : "Brew Manager",
        count,
      }).catch(e => console.error("Update tray failed:", e))
    }

    if (tapRes.ok && tapRes.data) taps.value = tapRes.data

  } catch (e) {
    pushLog(t.value.logRefreshFailed(String(e)))
  } finally {
    loading.value = false
    if (showLoader) hideGlobalLoading()
  }
}

export async function addTap() {
  const name = newTapName.value.trim()
  if (!name) return

  showAddTapModal.value = false
  showGlobalLoading(t.value.loadingAddTap(name))
  await nextTick()
  await new Promise(r => setTimeout(r, 80))

  detailTitle.value = t.value.titleAddTap(name)
  detailText.value = ""
  setDetailLoading(true)
  const requestId = makeRequestId()
  activeRequestId.value = requestId
  hideGlobalLoading()
  detailModalOpen.value = true
  await nextTick()
  await new Promise(r => setTimeout(r, 50))

  try {
    const res = (await invoke("brew_run_stream", {
      requestId, action: "tap", name, kind: null,
    })) as ApiResponse<boolean>
    if (res.ok) {
      newTapName.value = ""
      await notify(t.value.notifyTapAdded, name)
      await refreshAll()
    } else {
      pushLog(t.value.logTapAddFailed(res.message))
    }
  } catch (e) {
    setDetailLoading(false)
    detailText.value += t.value.logException(String(e))
    pushLog(t.value.logTapAddError(String(e)))
  }
}

export async function removeTap(name: string) {
  const yes = await showConfirm(t.value.confirmRemoveTapTitle, t.value.confirmRemoveTap(name))
  if (!yes) return

  showGlobalLoading(t.value.loadingRemoveTap(name))
  await nextTick()
  await new Promise(r => setTimeout(r, 80))

  detailTitle.value = t.value.titleRemoveTap(name)
  detailText.value = ""
  setDetailLoading(true)
  const requestId = makeRequestId()
  activeRequestId.value = requestId
  hideGlobalLoading()
  detailModalOpen.value = true
  await nextTick()
  await new Promise(r => setTimeout(r, 50))

  try {
    const res = (await invoke("brew_run_stream", {
      requestId, action: "untap", name, kind: null,
    })) as ApiResponse<boolean>
    if (res.ok) {
      await notify(t.value.notifyTapRemoved, name)
      await refreshAll()
    } else {
      pushLog(t.value.logTapRemoveFailed(res.message))
    }
  } catch (e) {
    setDetailLoading(false)
    detailText.value += t.value.logException(String(e))
    pushLog(t.value.logTapRemoveError(String(e)))
  }
}

export function toggleSelection(name: string) {
  if (selectedPackages.value.has(name)) selectedPackages.value.delete(name)
  else selectedPackages.value.add(name)
}

export async function batchUninstall() {
  const targets = Array.from(selectedPackages.value)
  if (targets.length === 0) return

  const yes = await showConfirm(t.value.confirmBatchUninstallTitle, t.value.confirmBatchUninstall(targets.length))
  if (!yes) return

  showGlobalLoading(t.value.loadingBatchUninstall)
  await nextTick()
  await new Promise(r => setTimeout(r, 100))

  detailTitle.value = t.value.titleBatchUninstall(targets.length)
  detailText.value = ""
  setDetailLoading(true)
  const requestId = makeRequestId()
  activeRequestId.value = requestId
  hideGlobalLoading()
  detailModalOpen.value = true
  await nextTick()

  let successCount = 0
  try {
    for (let i = 0; i < targets.length; i++) {
      const name = targets[i]
      const pkg = packages.value.find(p => p.name === name)
      if (!pkg) continue

      detailText.value += t.value.logBatchUninstalling(i + 1, targets.length, name)
      const el = document.querySelector(".modal-output")
      if (el) el.scrollTop = el.scrollHeight

      const res = (await invoke("brew_run_stream", {
        requestId, action: "uninstall", name, kind: pkg.kind,
      })) as ApiResponse<boolean>

      if (res.ok) { successCount++; detailText.value += t.value.logBatchUninstallOk(name) }
      else { detailText.value += t.value.logBatchUninstallFail(name, res.message) }
    }
    detailText.value += t.value.logBatchDone(successCount, targets.length - successCount)
    await notify(t.value.notifyBatchUninstallDone, t.value.notifyBatchResult(successCount, targets.length))
    await refreshAll()
  } catch (e) {
    setDetailLoading(false)
    detailText.value += t.value.logInterrupted(String(e))
  }
}

export async function batchUpgrade() {
  const targets = Array.from(selectedPackages.value)
  if (targets.length === 0) return

  const yes = await showConfirm(t.value.confirmBatchUpgradeTitle, t.value.confirmBatchUpgrade(targets.length))
  if (!yes) return

  showGlobalLoading(t.value.loadingBatchUpgrade)
  await nextTick()

  detailTitle.value = t.value.titleBatchUpgrade(targets.length)
  detailText.value = ""
  setDetailLoading(true)
  const requestId = makeRequestId()
  activeRequestId.value = requestId
  hideGlobalLoading()
  detailModalOpen.value = true
  await nextTick()

  let successCount = 0
  try {
    for (let i = 0; i < targets.length; i++) {
      const name = targets[i]
      const pkg = packages.value.find(p => p.name === name)
      if (!pkg) continue

      detailText.value += t.value.logBatchUpgrading(i + 1, targets.length, name)
      const res = (await invoke("brew_run_stream", {
        requestId, action: "upgrade", name, kind: pkg.kind,
      })) as ApiResponse<boolean>

      if (res.ok) { successCount++; detailText.value += t.value.logBatchUpgradeOk(name) }
      else { detailText.value += t.value.logBatchUpgradeFail(name, res.message) }
    }
    detailText.value += t.value.logBatchDone(successCount, targets.length - successCount)
    await notify(t.value.notifyBatchUpgradeDone, t.value.notifyBatchResult(successCount, targets.length))
    await refreshAll()
  } catch (e) {
    setDetailLoading(false)
    detailText.value += t.value.logInterrupted(String(e))
  }
}

export async function installPkg(name?: string, kind?: BrewKind) {
  const pkgName = (name ?? installName.value).trim()
  const pkgKind = kind ?? installKind.value
  if (!pkgName) { pushLog(t.value.logPleaseEnterName); return }

  showGlobalLoading(t.value.loadingInstall(pkgName))
  await nextTick()
  await new Promise(r => setTimeout(r, 80))

  detailTitle.value = t.value.titleInstall(pkgName, pkgKind)
  detailText.value = ""
  setDetailLoading(true)
  const requestId = makeRequestId()
  activeRequestId.value = requestId
  hideGlobalLoading()
  detailModalOpen.value = true
  await nextTick()
  await new Promise(r => setTimeout(r, 50))

  try {
    const res = (await invoke("brew_run_stream", {
      requestId, action: "install", name: pkgName, kind: pkgKind,
    })) as ApiResponse<boolean>
    const ok = res.ok
    pushLog(ok ? t.value.logInstallOk(pkgName) : t.value.logInstallFail(res.message))
    if (ok) {
      if (!name) installName.value = ""
      await notify(t.value.notifyInstallDone, pkgName)
      await refreshAll()
    }
  } catch (e) {
    setDetailLoading(false)
    detailText.value += t.value.logException(String(e))
    pushLog(t.value.logInstallError(String(e)))
  }
}

export async function uninstallPkg(pkg: BrewPackage) {
  const yes = await showConfirm(t.value.confirmUninstallTitle, t.value.confirmUninstall(pkg.name, pkg.kind))
  if (!yes) return

  showGlobalLoading(t.value.loadingUninstall(pkg.name))
  await nextTick()
  await new Promise(r => setTimeout(r, 80))

  detailTitle.value = t.value.titleUninstall(pkg.name, pkg.kind)
  detailText.value = ""
  setDetailLoading(true)
  const requestId = makeRequestId()
  activeRequestId.value = requestId
  hideGlobalLoading()
  detailModalOpen.value = true
  await nextTick()
  await new Promise(r => setTimeout(r, 50))

  try {
    const res = (await invoke("brew_run_stream", {
      requestId, action: "uninstall", name: pkg.name, kind: pkg.kind,
    })) as ApiResponse<boolean>
    const ok = res.ok
    pushLog(ok ? t.value.logUninstallOk(pkg.name) : t.value.logUninstallFail(res.message))
    if (ok) {
      await notify(t.value.notifyUninstallDone, pkg.name)
      await refreshAll()
    }
  } catch (e) {
    setDetailLoading(false)
    detailText.value += t.value.logException(String(e))
    pushLog(t.value.logUninstallError(String(e)))
  }
}

export async function upgradeAll() {
  const yes = await showConfirm(t.value.confirmUpgradeAllTitle, t.value.confirmUpgradeAll(outdated.value.length))
  if (!yes) return

  showGlobalLoading(t.value.loadingUpgradeAll)
  await nextTick()
  await new Promise(r => setTimeout(r, 80))

  detailTitle.value = t.value.titleUpgradeAll
  detailText.value = ""
  setDetailLoading(true)
  const requestId = makeRequestId()
  activeRequestId.value = requestId
  hideGlobalLoading()
  detailModalOpen.value = true
  await nextTick()
  await new Promise(r => setTimeout(r, 50))

  try {
    const res = (await invoke("brew_run_stream", {
      requestId, action: "upgrade_all", name: null, kind: null,
    })) as ApiResponse<boolean>
    pushLog(res.ok ? t.value.logUpgradeAllDone : t.value.logUpgradeAllFail(res.message))
    if (res.ok) {
      await notify(t.value.notifyUpgradeAllDone, t.value.notifyUpgradeAllBody)
      await refreshAll()
    }
  } catch (e) {
    setDetailLoading(false)
    detailText.value += t.value.logException(String(e))
    pushLog(t.value.logUpgradeAllError(String(e)))
  }
}

export async function upgradeSingle(pkg: BrewPackage) {
  showGlobalLoading(t.value.loadingUpgrade(pkg.name))
  await nextTick()
  await new Promise(r => setTimeout(r, 80))

  detailTitle.value = t.value.titleUpgrade(pkg.name)
  detailText.value = ""
  setDetailLoading(true)
  const requestId = makeRequestId()
  activeRequestId.value = requestId
  hideGlobalLoading()
  detailModalOpen.value = true
  await nextTick()
  await new Promise(r => setTimeout(r, 50))

  try {
    const res = (await invoke("brew_run_stream", {
      requestId, action: "upgrade", name: pkg.name, kind: pkg.kind,
    })) as ApiResponse<boolean>
    pushLog(res.ok ? t.value.logUpgradeOk(pkg.name) : t.value.logUpgradeFail(res.message))
    if (res.ok) {
      await notify(t.value.notifyUpgradeDone, pkg.name)
      await refreshAll()
    }
  } catch (e) {
    setDetailLoading(false)
    detailText.value += t.value.logException(String(e))
    pushLog(t.value.logUpgradeError(String(e)))
  }
}

// ── 搜索 ──────────────────────────────────────────────────────────────────────
export function onSearchQueryChange() {
  searchError.value = ""
  searchResults.value = []
  if (!searchQuery.value.trim()) return
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(doSearch, 600)
}

export async function doSearch() {
  if (!searchQuery.value.trim()) return
  searchLoading.value = true
  searchError.value = ""
  try {
    const res = (await invoke("brew_search", { query: searchQuery.value })) as ApiResponse<BrewPackage[]>
    if (res.ok && res.data) { searchResults.value = res.data }
    else { searchError.value = res.message; searchResults.value = [] }
  } catch (e) {
    searchError.value = t.value.searchFailed(String(e))
  } finally {
    searchLoading.value = false
  }
}

// ── 设置 ──────────────────────────────────────────────────────────────────────
export function saveBrewPath() {
  if (customBrewPath.value.trim()) {
    localStorage.setItem("customBrewPath", customBrewPath.value.trim())
    pushLog(t.value.logSavedPath(customBrewPath.value))
  } else {
    localStorage.removeItem("customBrewPath")
    pushLog(t.value.logClearedPath)
  }
  showPathSettings.value = false
  refreshAll(true)
}

// ── 详情 / Doctor ─────────────────────────────────────────────────────────────
export function closeDetailModal() {
  detailModalOpen.value = false
  setDetailLoading(false)
  activeRequestId.value = ""
}

export async function showInfo(pkg: BrewPackage) {
  showGlobalLoading(t.value.loadingInfo(pkg.name))
  await nextTick()
  await new Promise(r => setTimeout(r, 100))

  detailTitle.value = `${pkg.name} (${pkg.kind})`
  detailText.value = ""
  setDetailLoading(true)
  const requestId = makeRequestId()
  activeRequestId.value = requestId
  hideGlobalLoading()
  detailModalOpen.value = true
  await nextTick()
  await new Promise(r => setTimeout(r, 50))

  try {
    const res = (await invoke("brew_run_stream", {
      requestId, action: "info", name: pkg.name, kind: pkg.kind,
    })) as ApiResponse<boolean>
    if (!res.ok && !detailText.value.trim()) {
      setDetailLoading(false)
      detailText.value = t.value.infoFailed(res.message)
    }
  } catch (e) {
    setDetailLoading(false)
    detailText.value = t.value.infoError(String(e))
  }
}

export async function runDoctor() {
  showGlobalLoading(t.value.loadingDoctor)
  await nextTick()
  await new Promise(r => setTimeout(r, 100))

  detailTitle.value = "brew doctor"
  detailText.value = ""
  setDetailLoading(true)
  const requestId = makeRequestId()
  activeRequestId.value = requestId
  hideGlobalLoading()
  detailModalOpen.value = true
  await nextTick()
  await new Promise(r => setTimeout(r, 50))

  try {
    const res = (await invoke("brew_run_stream", {
      requestId, action: "doctor", name: null, kind: null,
    })) as ApiResponse<boolean>
    if (!res.ok && !detailText.value.trim()) {
      setDetailLoading(false)
      detailText.value = t.value.doctorDone(res.message)
    }
  } catch (e) {
    setDetailLoading(false)
    detailText.value = t.value.doctorError(String(e))
  }
}

export function handleConfirm(yes: boolean) {
  confirmVisible.value = false
  confirmResolve?.(yes)
  confirmResolve = null
}

// ── 键盘快捷键 ────────────────────────────────────────────────────────────────
export function onKeyDown(e: KeyboardEvent) {
  const mod = e.metaKey || e.ctrlKey
  const tag = (e.target as HTMLElement).tagName.toLowerCase()
  const inInput = tag === "input" || tag === "textarea" || tag === "select"

  if (e.key === "Escape") {
    if (confirmVisible.value) { handleConfirm(false); return }
    if (detailModalOpen.value) { closeDetailModal(); return }
    if (showPathSettings.value) { showPathSettings.value = false; return }
    if (showAddTapModal.value) { showAddTapModal.value = false; return }
    if (showSearch.value) { showSearch.value = false; return }
    if (showLogs.value) { showLogs.value = false; return }
    return
  }

  if (!mod) return

  if (e.key === "r" || e.key === "R") {
    if (inInput) return
    e.preventDefault()
    if (!loading.value) refreshAll(true)
    return
  }
  if (e.key === "k" || e.key === "K") {
    e.preventDefault()
    showSearch.value = !showSearch.value
    return
  }
  if (e.key === "f" || e.key === "F") {
    if (showSearch.value) return
    e.preventDefault()
    listSearchRef.value?.focus()
    return
  }
}

// ── 生命周期 ──────────────────────────────────────────────────────────────────
export async function initApp() {
  const savedPath = localStorage.getItem("customBrewPath")
  if (savedPath) customBrewPath.value = savedPath

  window.addEventListener("keydown", onKeyDown)

  await nextTick()
  await refreshAll()
  initialLoading.value = false

  unlistenBrewLog = await listen<BrewLogEvent>("brew-log", (event) => {
    const p = event.payload
    if (!p || p.requestId !== activeRequestId.value) return
    if (p.stage === "start") { setDetailLoading(true); detailModalOpen.value = true; return }
    if (p.stage === "line") {
      const prefix = p.stream === "stderr" ? "[err] " : ""
      detailText.value += `${prefix}${p.line ?? ""}\n`
      return
    }
    if (p.stage === "end") {
      setDetailLoading(false)
      if (!p.success && !detailText.value.trim()) detailText.value = t.value.cmdNoOutput
      pushLog(p.success ? t.value.cmdDone : t.value.cmdDoneWarning)
    }
  })
}

export function cleanupApp() {
  window.removeEventListener("keydown", onKeyDown)
  if (unlistenBrewLog) { unlistenBrewLog(); unlistenBrewLog = null }
  if (searchTimer) clearTimeout(searchTimer)
  if (toastTimer) clearTimeout(toastTimer)
}
