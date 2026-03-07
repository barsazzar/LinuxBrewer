// ── Settings State: brew path, keyboard shortcuts ────────────────────────────
import { ref } from "vue"
import { invoke } from "@tauri-apps/api/core"
import { t } from "../i18n"
import type { ApiResponse, BrewStatus } from "../types"
import { showToast, resolveErrorMessage } from "./ui"
import { pushLog } from "./packages"

export const status = ref<ApiResponse<BrewStatus> | null>(null)
export const customBrewPath = ref("")
export const showPathSettings = ref(false)

export async function saveBrewPath() {
  const path = customBrewPath.value.trim() || null
  try {
    const res = (await invoke("set_brew_path", { path })) as ApiResponse<boolean>
    if (!res.ok) {
      showToast(resolveErrorMessage(res.errorCode, res.message))
      return
    }
    if (path) localStorage.setItem("customBrewPath", path)
    else localStorage.removeItem("customBrewPath")
    pushLog(path ? t.value.logSavedPath(path) : t.value.logClearedPath)
    showPathSettings.value = false
    const { refreshAll } = await import("./packages")
    await refreshAll(true)
  } catch (e) {
    showToast(String(e))
  }
}
