// ── Taps State ───────────────────────────────────────────────────────────────
import { ref } from "vue"
import { invoke } from "@tauri-apps/api/core"
import { nextTick } from "vue"
import { t } from "../i18n"
import type { ApiResponse } from "../types"
import {
  showGlobalLoading, hideGlobalLoading, notify, makeRequestId,
  setDetailLoading, detailModalOpen, detailTitle, detailText, activeRequestId,
  showConfirm, resolveErrorMessage,
} from "./ui"
import { refreshAll } from "./packages"

export const taps = ref<string[]>([])
export const showAddTapModal = ref(false)
export const newTapName = ref("")

async function runTapStream(action: "tap" | "untap", name: string, title: string, loadingText: string): Promise<boolean> {
  showGlobalLoading(loadingText)
  await nextTick()
  await new Promise(r => setTimeout(r, 80))

  detailTitle.value = title
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
      requestId, action, name, kind: null,
    })) as ApiResponse<boolean>
    return res.ok
  } catch (e) {
    setDetailLoading(false)
    detailText.value += t.value.logException(String(e))
    return false
  }
}

export async function addTap() {
  const name = newTapName.value.trim()
  if (!name) return
  showAddTapModal.value = false

  const ok = await runTapStream("tap", name, t.value.titleAddTap(name), t.value.loadingAddTap(name))
  if (ok) {
    newTapName.value = ""
    await notify(t.value.notifyTapAdded, name)
    await refreshAll()
  } else {
    const { pushLog } = await import("./packages")
    pushLog(t.value.logTapAddFailed(resolveErrorMessage(null, "")))
  }
}

export async function removeTap(name: string) {
  const yes = await showConfirm(t.value.confirmRemoveTapTitle, t.value.confirmRemoveTap(name))
  if (!yes) return

  const ok = await runTapStream("untap", name, t.value.titleRemoveTap(name), t.value.loadingRemoveTap(name))
  if (ok) {
    await notify(t.value.notifyTapRemoved, name)
    await refreshAll()
  }
}
