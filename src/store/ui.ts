// ── UI State: Loading, Toast, Confirm, Detail Modal ──────────────────────────
import { ref } from "vue"
import { t } from "../i18n"

export const globalLoading = ref(false)
export const globalLoadingText = ref("")
export const toastMsg = ref("")

export const confirmVisible = ref(false)
export const confirmTitle = ref("")
export const confirmMessage = ref("")
let confirmResolve: ((v: boolean) => void) | null = null

export const detailTitle = ref("")
export const detailText = ref("")
export const detailModalOpen = ref(false)
export const detailLoading = ref(false)
export const activeRequestId = ref("")

const MIN_LOADING_VISIBLE_MS = 500
const detailLoadingStartedAt = ref(0)

let toastTimer: ReturnType<typeof setTimeout> | null = null

export function showGlobalLoading(text: string) {
  globalLoadingText.value = text
  globalLoading.value = true
}

export function hideGlobalLoading() {
  globalLoading.value = false
}

export function setDetailLoading(v: boolean) {
  if (v) {
    detailLoadingStartedAt.value = Date.now()
    detailLoading.value = true
    return
  }
  const rest = Math.max(0, MIN_LOADING_VISIBLE_MS - (Date.now() - detailLoadingStartedAt.value))
  if (rest > 0) setTimeout(() => { detailLoading.value = false }, rest)
  else detailLoading.value = false
}

export function showToast(msg: string) {
  toastMsg.value = msg
  if (toastTimer) clearTimeout(toastTimer)
  toastTimer = setTimeout(() => { toastMsg.value = "" }, 3000)
}

export function showConfirm(title: string, message: string): Promise<boolean> {
  confirmTitle.value = title
  confirmMessage.value = message
  confirmVisible.value = true
  return new Promise(resolve => { confirmResolve = resolve })
}

export function handleConfirm(yes: boolean) {
  confirmVisible.value = false
  confirmResolve?.(yes)
  confirmResolve = null
}

export function closeDetailModal() {
  detailModalOpen.value = false
  setDetailLoading(false)
  activeRequestId.value = ""
}

export function makeRequestId() {
  return `req-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`
}

export async function notify(title: string, body: string) {
  const { sendNotification } = await import("@tauri-apps/plugin-notification")
  showToast(`${title}: ${body}`)
  try { await sendNotification({ title, body }) } catch { /* silent */ }
}

export function cleanupUI() {
  if (toastTimer) clearTimeout(toastTimer)
}

// i18n error code → message
const ERROR_CODE_KEYS: Record<string, (t: any) => string> = {
  BREW_NOT_FOUND: (t) => t.errBrewNotFound,
  INVALID_NAME: (t) => t.errInvalidName,
  INVALID_KIND: (t) => t.errInvalidKind,
  INVALID_BREW_PATH: (t) => t.errInvalidBrewPath,
  INVALID_ACTION: (t) => t.errInvalidAction,
  BREW_EXEC_FAILED: (t) => t.errBrewExecFailed,
  COMMAND_FAILED: (t) => t.cmdDoneWarning,
  CANCELLED: (t) => t.opCancelled,
  NO_RESULTS: (t) => t.searchNoResults,
}

export function resolveErrorMessage(code: string | null | undefined, fallback: string): string {
  if (!code) return fallback
  const fn = ERROR_CODE_KEYS[code]
  return fn ? fn(t.value) : fallback
}
