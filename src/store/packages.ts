// ── Packages State: installed list, sorting, filtering, outdated ──────────────
import { ref, computed } from "vue"
import { invoke } from "@tauri-apps/api/core"
import { listen, type UnlistenFn } from "@tauri-apps/api/event"
import { nextTick } from "vue"
import { t } from "../i18n"
import type { ApiResponse, BrewPackage, BrewKind, BrewLogEvent } from "../types"
import {
  showGlobalLoading, hideGlobalLoading, notify, makeRequestId,
  setDetailLoading, detailModalOpen, detailTitle, detailText, activeRequestId,
  showConfirm, resolveErrorMessage,
} from "./ui"

export type SortKey = "name" | "kind" | "version"
export type SortDir = "asc" | "desc"

export const initialLoading = ref(true)
export const loading = ref(false)
export const packages = ref<BrewPackage[]>([])
export const outdated = ref<BrewPackage[]>([])
export const selectedPackages = ref(new Set<string>())
export const isSelectionMode = computed(() => selectedPackages.value.size > 0)
export const pinnedNames = ref(new Set<string>())

export const search = ref("")
export const filterKind = ref<"all" | BrewKind>("all")
export const sortKey = ref<SortKey>("name")
export const sortDir = ref<SortDir>("asc")

export const installName = ref("")
export const installKind = ref<BrewKind>("formula")
export const showLogs = ref(false)
export const logs = ref<string[]>([])

export const listSearchRef = ref<HTMLInputElement | null>(null)

export const installedNames = computed(() => new Set(packages.value.map(p => p.name)))

export const filteredPackages = computed(() => {
  let list = packages.value
  if (filterKind.value !== "all") list = list.filter(p => p.kind === filterKind.value)
  const q = search.value.trim().toLowerCase()
  if (q) list = list.filter(p => p.name.toLowerCase().includes(q))

  // Sort
  list = [...list].sort((a, b) => {
    let cmp = 0
    if (sortKey.value === "name") cmp = a.name.localeCompare(b.name)
    else if (sortKey.value === "kind") cmp = a.kind.localeCompare(b.kind)
    else if (sortKey.value === "version") cmp = (a.version ?? "").localeCompare(b.version ?? "")
    return sortDir.value === "asc" ? cmp : -cmp
  })
  return list
})

export function pushLog(text: string) {
  const ts = new Date().toLocaleTimeString()
  logs.value.unshift(`[${ts}] ${text}`)
}

export function toggleSort(key: SortKey) {
  if (sortKey.value === key) {
    sortDir.value = sortDir.value === "asc" ? "desc" : "asc"
  } else {
    sortKey.value = key
    sortDir.value = "asc"
  }
}

export function toggleSelection(name: string) {
  if (selectedPackages.value.has(name)) selectedPackages.value.delete(name)
  else selectedPackages.value.add(name)
}

// Wait for two animation frames — ensures the browser has painted at least once.
// This is needed on WebKitGTK where CSS animations require a compositor frame
// to start before JS blocks the thread with an IPC call.
function waitFrames(n = 2): Promise<void> {
  return new Promise(resolve => {
    let count = 0
    const tick = () => { if (++count >= n) resolve(); else requestAnimationFrame(tick) }
    requestAnimationFrame(tick)
  })
}

// ── Stream operations helper ──────────────────────────────────────────────────
async function runStream(
  action: string,
  name: string | null,
  kind: string | null,
  title: string,
  loadingText: string,
): Promise<boolean> {
  showGlobalLoading(loadingText)
  await nextTick()
  await waitFrames(3)  // let WebKitGTK compositor start the spinner animation

  detailTitle.value = title
  detailText.value = ""
  setDetailLoading(true)
  const requestId = makeRequestId()
  activeRequestId.value = requestId
  hideGlobalLoading()
  detailModalOpen.value = true
  await nextTick()
  await waitFrames(2)  // let modal spinner render before invoke blocks

  try {
    const res = (await invoke("brew_run_stream", {
      requestId, action, name, kind,
    })) as ApiResponse<boolean>
    return res.ok
  } catch (e) {
    setDetailLoading(false)
    detailText.value += t.value.logException(String(e))
    return false
  }
}

// ── Core actions ──────────────────────────────────────────────────────────────
export async function installPkg(name?: string, kind?: BrewKind) {
  const pkgName = (name ?? installName.value).trim()
  const pkgKind = kind ?? installKind.value
  if (!pkgName) { pushLog(t.value.logPleaseEnterName); return }

  const ok = await runStream(
    "install", pkgName, pkgKind,
    t.value.titleInstall(pkgName, pkgKind),
    t.value.loadingInstall(pkgName),
  )
  pushLog(ok ? t.value.logInstallOk(pkgName) : t.value.logInstallFail(""))
  if (ok) {
    if (!name) installName.value = ""
    await notify(t.value.notifyInstallDone, pkgName)
    await refreshAll()
  }
}

export async function uninstallPkg(pkg: BrewPackage) {
  const yes = await showConfirm(t.value.confirmUninstallTitle, t.value.confirmUninstall(pkg.name, pkg.kind))
  if (!yes) return

  const ok = await runStream(
    "uninstall", pkg.name, pkg.kind,
    t.value.titleUninstall(pkg.name, pkg.kind),
    t.value.loadingUninstall(pkg.name),
  )
  pushLog(ok ? t.value.logUninstallOk(pkg.name) : t.value.logUninstallFail(""))
  if (ok) {
    await notify(t.value.notifyUninstallDone, pkg.name)
    await refreshAll()
  }
}

export async function upgradeSingle(pkg: BrewPackage) {
  const ok = await runStream(
    "upgrade", pkg.name, pkg.kind,
    t.value.titleUpgrade(pkg.name),
    t.value.loadingUpgrade(pkg.name),
  )
  pushLog(ok ? t.value.logUpgradeOk(pkg.name) : t.value.logUpgradeFail(""))
  if (ok) {
    await notify(t.value.notifyUpgradeDone, pkg.name)
    await refreshAll()
  }
}

export async function upgradeAll() {
  const yes = await showConfirm(t.value.confirmUpgradeAllTitle, t.value.confirmUpgradeAll(outdated.value.length))
  if (!yes) return

  const ok = await runStream(
    "upgrade_all", null, null,
    t.value.titleUpgradeAll,
    t.value.loadingUpgradeAll,
  )
  pushLog(ok ? t.value.logUpgradeAllDone : t.value.logUpgradeAllFail(""))
  if (ok) {
    await notify(t.value.notifyUpgradeAllDone, t.value.notifyUpgradeAllBody)
    await refreshAll()
  }
}

export async function runUpdate() {
  const ok = await runStream("update", null, null, "brew update", t.value.loadingUpdate)
  pushLog(ok ? t.value.logUpdateDone : t.value.logUpdateFail(""))
  if (ok) await refreshAll()
}

export async function runDoctor() {
  await runStream("doctor", null, null, "brew doctor", t.value.loadingDoctor)
}

export async function runCleanup() {
  const ok = await runStream("cleanup", null, null, "brew cleanup", t.value.loadingCleanup)
  if (ok) await notify(t.value.notifyCleanupDone, t.value.notifyCleanupBody)
}

export async function showInfo(pkg: BrewPackage) {
  showGlobalLoading(t.value.loadingInfo(pkg.name))
  await nextTick()
  await waitFrames(3)

  detailTitle.value = `${pkg.name} (${pkg.kind})`
  detailText.value = ""
  setDetailLoading(true)
  const requestId = makeRequestId()
  activeRequestId.value = requestId
  hideGlobalLoading()
  detailModalOpen.value = true
  await nextTick()
  await waitFrames(2)

  try {
    const res = (await invoke("brew_run_stream", {
      requestId, action: "info", name: pkg.name, kind: pkg.kind,
    })) as ApiResponse<boolean>
    if (!res.ok && !detailText.value.trim()) {
      setDetailLoading(false)
      detailText.value = resolveErrorMessage(res.errorCode, res.message)
    }
  } catch (e) {
    setDetailLoading(false)
    detailText.value = t.value.infoError(String(e))
  }
}

export async function togglePin(pkg: BrewPackage) {
  const isPinned = pinnedNames.value.has(pkg.name)
  const action = isPinned ? "unpin" : "pin"
  const loadingKey = isPinned ? t.value.loadingUnpin(pkg.name) : t.value.loadingPin(pkg.name)
  await runStream(action, pkg.name, pkg.kind, `brew ${action} ${pkg.name}`, loadingKey)
  if (isPinned) pinnedNames.value.delete(pkg.name)
  else pinnedNames.value.add(pkg.name)
}

export async function batchUninstall() {
  const targets = Array.from(selectedPackages.value)
  if (targets.length === 0) return

  const yes = await showConfirm(t.value.confirmBatchUninstallTitle, t.value.confirmBatchUninstall(targets.length))
  if (!yes) return

  showGlobalLoading(t.value.loadingBatchUninstall)
  await nextTick()
  await waitFrames(3)

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
      else { detailText.value += t.value.logBatchUninstallFail(name, resolveErrorMessage(res.errorCode, res.message)) }
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
      else { detailText.value += t.value.logBatchUpgradeFail(name, resolveErrorMessage(res.errorCode, res.message)) }
    }
    detailText.value += t.value.logBatchDone(successCount, targets.length - successCount)
    await notify(t.value.notifyBatchUpgradeDone, t.value.notifyBatchResult(successCount, targets.length))
    await refreshAll()
  } catch (e) {
    setDetailLoading(false)
    detailText.value += t.value.logInterrupted(String(e))
  }
}

// ── Refresh all data ──────────────────────────────────────────────────────────
export async function refreshAll(showLoader = false) {
  loading.value = true
  if (showLoader) {
    showGlobalLoading(t.value.loadingRefresh)
    await nextTick()
    await waitFrames(3)
  }
  try {
    selectedPackages.value.clear()
    const { status } = await import("./settings")

    const [s, listRes, outRes, tapRes] = await Promise.all([
      invoke("brew_status") as Promise<ApiResponse<{ brewPath: string; version: string }>>,
      invoke("brew_list_installed") as Promise<ApiResponse<BrewPackage[]>>,
      invoke("brew_outdated") as Promise<ApiResponse<BrewPackage[]>>,
      invoke("brew_tap_list") as Promise<ApiResponse<string[]>>,
    ])
    status.value = s

    if (listRes.ok && listRes.data) {
      packages.value = listRes.data
      pushLog(t.value.logLoaded(listRes.data.length))
    } else {
      pushLog(t.value.logLoadFailed(resolveErrorMessage(listRes.errorCode, listRes.message)))
    }

    if (outRes.ok && outRes.data) {
      outdated.value = outRes.data
      const count = outRes.data.length
      if (count) pushLog(t.value.logUpdatable(count))
      invoke("update_tray", {
        title: count > 0 ? `Brew Manager (${count})` : "Brew Manager",
        count,
      }).catch(() => {})
    }

    const { taps } = await import("./taps")
    if (tapRes.ok && tapRes.data) taps.value = tapRes.data

  } catch (e) {
    pushLog(t.value.logRefreshFailed(String(e)))
  } finally {
    loading.value = false
    if (showLoader) hideGlobalLoading()
  }
}

// ── Lifecycle listeners ───────────────────────────────────────────────────────
let unlistenBrewLog: UnlistenFn | null = null

export async function initBrewLog() {
  unlistenBrewLog = await listen<BrewLogEvent>("brew-log", (event) => {
    const p = event.payload
    if (!p || p.requestId !== activeRequestId.value) return
    if (p.stage === "start") { setDetailLoading(true); detailModalOpen.value = true; return }
    if (p.stage === "line") {
      const prefix = p.stream === "stderr" ? "[err] " : ""
      detailText.value += `${prefix}${p.line ?? ""}\n`
      // Auto-scroll modal output
      const el = document.querySelector(".modal-output")
      if (el) { el.scrollTop = el.scrollHeight }
      return
    }
    if (p.stage === "end") {
      setDetailLoading(false)
      if (!p.success && !detailText.value.trim()) detailText.value = t.value.cmdNoOutput
      pushLog(p.success ? t.value.cmdDone : t.value.cmdDoneWarning)
    }
  })
}

export function cleanupBrewLog() {
  if (unlistenBrewLog) { unlistenBrewLog(); unlistenBrewLog = null }
}
