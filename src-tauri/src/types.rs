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
