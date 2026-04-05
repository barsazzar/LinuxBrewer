export type BrewKind = "formula" | "cask"

export interface ApiResponse<T> {
  ok: boolean
  data?: T
  errorCode?: string
  message: string
}

export interface BrewStatus {
  brewPath: string
  version: string
}

export interface BrewPackage {
  name: string
  version?: string
  newVersion?: string
  kind: BrewKind
  pinned?: boolean
  description?: string
}

export interface BrewLogEvent {
  requestId: string
  stage: "start" | "line" | "end"
  stream?: "stdout" | "stderr"
  line?: string
  success?: boolean
}

// ── NVM / FNM types ───────────────────────────────────────────────────────────

export interface NvmStatus {
  manager: "fnm" | "nvm" | "none"
  managerVersion?: string
  nodeDefault?: string
}

export interface NodeVersion {
  version: string
  major: number
  isCurrent: boolean
  isDefault: boolean
  ltsName?: string
}

export interface LtsSlot {
  major: number
  ltsName: string
  installed?: NodeVersion
  allInstalled: NodeVersion[]   // E1: 该 major 下全部已装版本
  latestAvailable?: string      // F3: 远程最新版（lazy）
  isLts: boolean                // E2: false = 非 LTS 额外 slot
}

export interface ProjectVersion {
  version: string
  file: string
}

export interface NvmRefreshResult {
  status: NvmStatus
  slots: LtsSlot[]
}

export interface RemoteLtsVersion {
  major: number
  latest: string
  ltsName?: string
}
