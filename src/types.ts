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
  kind: BrewKind
}

export interface BrewLogEvent {
  requestId: string
  stage: "start" | "line" | "end"
  stream?: "stdout" | "stderr"
  line?: string
  success?: boolean
}
