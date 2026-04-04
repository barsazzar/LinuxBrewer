// ── Search State: query, results ──────────────────────────────────────────────
import { ref } from "vue"
import { invoke } from "@tauri-apps/api/core"
import { t } from "../i18n"
import type { ApiResponse, BrewPackage } from "../types"

export const searchQuery = ref("")
export const searchResults = ref<BrewPackage[]>([])
export const searchLoading = ref(false)
export const searchError = ref("")
export const showSearch = ref(false)

// Max results to display — brew search can return hundreds of matches
const MAX_DISPLAY = 50

let searchTimer: ReturnType<typeof setTimeout> | null = null

// Monotonically increasing token: when a response arrives for an older token,
// it means a newer search is already in flight — discard the stale result.
let searchToken = 0

export function onSearchQueryChange() {
  searchError.value = ""
  searchResults.value = []
  if (!searchQuery.value.trim()) return
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(doSearch, 350)
}

export async function doSearch() {
  if (!searchQuery.value.trim()) return
  const token = ++searchToken
  searchLoading.value = true
  searchError.value = ""
  try {
    const res = (await invoke("brew_search", { query: searchQuery.value })) as ApiResponse<BrewPackage[]>
    // Discard if a newer search has already started
    if (token !== searchToken) return
    if (res.ok && res.data) {
      searchResults.value = res.data.slice(0, MAX_DISPLAY)
    } else {
      searchError.value = t.value.searchFailed(res.message)
      searchResults.value = []
    }
  } catch (e) {
    if (token !== searchToken) return
    searchError.value = t.value.searchFailed(String(e))
  } finally {
    if (token === searchToken) searchLoading.value = false
  }
}

export function clearSearch() {
  searchQuery.value = ""
  searchResults.value = []
  searchError.value = ""
  if (searchTimer) clearTimeout(searchTimer)
  searchToken++ // invalidate any in-flight request
}

export function cleanupSearch() {
  if (searchTimer) clearTimeout(searchTimer)
  searchToken++
}
