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
    if (res.ok && res.data) {
      // Rust already provides version + description for the first 30 results.
      // Slice to MAX_DISPLAY to avoid rendering hundreds of DOM nodes at once.
      searchResults.value = res.data.slice(0, MAX_DISPLAY)
    } else {
      searchError.value = t.value.searchFailed(res.message)
      searchResults.value = []
    }
  } catch (e) {
    searchError.value = t.value.searchFailed(String(e))
  } finally {
    searchLoading.value = false
  }
}

export function clearSearch() {
  searchQuery.value = ""
  searchResults.value = []
  searchError.value = ""
  if (searchTimer) clearTimeout(searchTimer)
}

export function cleanupSearch() {
  if (searchTimer) clearTimeout(searchTimer)
}
