// ── Central re-export hub ─────────────────────────────────────────────────────
// This file re-exports from all sub-stores so components can continue to import
// from a single location. New imports can also come directly from sub-stores.
export * from "./ui"
export * from "./packages"
export * from "./taps"
export * from "./search"
export * from "./settings"

// ── App lifecycle ─────────────────────────────────────────────────────────────
import { nextTick } from "vue"
import { customBrewPath } from "./settings"
import { showSearch } from "./search"
import { showLogs, listSearchRef, loading, refreshAll, initialLoading, initBrewLog, cleanupBrewLog } from "./packages"
import { showPathSettings } from "./settings"
import { showAddTapModal } from "./taps"
import { confirmVisible, handleConfirm, detailModalOpen, closeDetailModal } from "./ui"
import { invoke } from "@tauri-apps/api/core"
import type { ApiResponse } from "../types"

export async function initApp() {
  // Restore saved brew path and pass to backend
  const savedPath = localStorage.getItem("customBrewPath")
  if (savedPath) {
    customBrewPath.value = savedPath
    try {
      await invoke("set_brew_path", { path: savedPath }) as ApiResponse<boolean>
    } catch { /* silent */ }
  }

  window.addEventListener("keydown", onKeyDown)

  await nextTick()
  await refreshAll()
  initialLoading.value = false

  await initBrewLog()
}

export function cleanupApp() {
  window.removeEventListener("keydown", onKeyDown)
  cleanupBrewLog()
}

// ── Keyboard shortcuts ────────────────────────────────────────────────────────
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
  if (e.key === "," ) {
    e.preventDefault()
    showPathSettings.value = true
    return
  }
}
