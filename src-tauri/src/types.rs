use serde::Serialize;

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
    pub kind: String,
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
