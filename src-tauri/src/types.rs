use serde::Serialize;
use std::sync::Mutex;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    pub ok: bool,
    pub data: Option<T>,
    pub error_code: Option<String>,
    pub message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrewStatus {
    pub brew_path: String,
    pub version: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrewPackage {
    pub name: String,
    pub version: Option<String>,
    pub new_version: Option<String>,
    pub kind: String,
    pub description: Option<String>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BrewLogEvent {
    pub request_id: String,
    pub stage: String,
    pub stream: Option<String>,
    pub line: Option<String>,
    pub success: Option<bool>,
}

pub fn ok<T: Serialize>(data: T, message: &str) -> ApiResponse<T> {
    ApiResponse {
        ok: true,
        data: Some(data),
        error_code: None,
        message: message.to_string(),
    }
}

pub fn err<T>(code: &str, message: &str) -> ApiResponse<T> {
    ApiResponse {
        ok: false,
        data: None,
        error_code: Some(code.to_string()),
        message: message.to_string(),
    }
}

// ── NVM / FNM types ───────────────────────────────────────────────────────────

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NvmManagerInfo {
    pub kind: String, // "fnm" | "nvm"
    pub path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NvmStatus {
    pub manager: String, // "fnm" | "nvm" | "none"
    pub manager_version: Option<String>,
    pub node_default: Option<String>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NodeVersion {
    pub version: String,
    pub major: u32,
    pub is_current: bool,
    pub is_default: bool,
    pub lts_name: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtsSlot {
    pub major: u32,
    pub lts_name: String,
    pub installed: Option<NodeVersion>,
    pub all_installed: Vec<NodeVersion>,    // E1: 该 major 下所有已装版本
    pub latest_available: Option<String>,  // F3: nodejs.org 最新版（lazy fetch）
    pub is_lts: bool,                      // E2: false = 非 LTS 的额外 slot
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectVersion {
    pub version: String,
    pub file: String,
}

pub struct NvmState {
    pub cached: Mutex<Option<NvmManagerInfo>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NvmRefreshResult {
    pub status: NvmStatus,
    pub slots: Vec<LtsSlot>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteLtsVersion {
    pub major: u32,
    pub latest: String,
    pub lts_name: Option<String>,
}

impl NvmState {
    pub fn new() -> Self {
        NvmState { cached: Mutex::new(None) }
    }
}

// ── Brew state ────────────────────────────────────────────────────────────────

/// App-wide state: caches the detected brew path and custom path override.
pub struct BrewState {
    /// User-configured custom path (set from settings UI)
    pub custom_path: Mutex<Option<String>>,
    /// Cached resolved brew binary path (invalidated on set_brew_path)
    pub cached_path: Mutex<Option<String>>,
    /// Cached brew version string — avoids re-running `brew --version` on every refresh
    pub cached_version: Mutex<Option<String>>,
}

impl BrewState {
    pub fn new() -> Self {
        BrewState {
            custom_path: Mutex::new(None),
            cached_path: Mutex::new(None),
            cached_version: Mutex::new(None),
        }
    }
}
