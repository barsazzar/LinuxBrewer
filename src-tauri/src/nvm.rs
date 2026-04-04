use std::collections::{HashMap, HashSet};
use std::process::Command;
use std::process::Stdio;
use std::sync::{Arc, Mutex};

use crate::brew::emit_brew_log;
use crate::types::{
    ok, err, ApiResponse, BrewLogEvent,
    NvmManagerInfo, NvmState, NodeVersion, NvmStatus, LtsSlot, ProjectVersion,
    NvmRefreshResult, RemoteLtsVersion,
};
use crate::stream::CancelRegistry;

// 展示的 LTS slot（静态已知列表）
const LTS_SLOTS: &[(u32, &str)] = &[
    (16, "Gallium"),
    (18, "Hydrogen"),
    (20, "Iron"),
    (22, "Jod"),
    (24, "Noble"),
];

fn lts_name_for_major(major: u32) -> Option<&'static str> {
    LTS_SLOTS.iter().find(|(m, _)| *m == major).map(|(_, n)| *n)
}

fn parse_major(version: &str) -> u32 {
    version
        .trim_start_matches('v')
        .split('.')
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

fn detect_manager() -> Option<NvmManagerInfo> {
    let home = std::env::var("HOME").unwrap_or_default();

    let static_candidates: &[&str] = &[
        "fnm",
        "/usr/local/bin/fnm",
        "/opt/homebrew/bin/fnm",
        "/home/linuxbrew/.linuxbrew/bin/fnm",
    ];
    let home_candidates = [
        format!("{}/.fnm/fnm", home),
        format!("{}/.local/share/fnm/fnm", home),
        format!("{}/.cargo/bin/fnm", home),
    ];

    for candidate in static_candidates
        .iter()
        .map(|s| s.to_string())
        .chain(home_candidates)
    {
        if let Ok(out) = Command::new(&candidate).arg("--version").output() {
            if out.status.success() {
                return Some(NvmManagerInfo { kind: "fnm".to_string(), path: candidate });
            }
        }
    }

    // nvm 是 shell 函数——检查脚本文件是否存在
    let nvm_sh = format!("{}/.nvm/nvm.sh", home);
    if std::path::Path::new(&nvm_sh).exists() {
        return Some(NvmManagerInfo { kind: "nvm".to_string(), path: nvm_sh });
    }

    None
}

pub fn resolve_manager(state: &NvmState) -> Option<NvmManagerInfo> {
    {
        let cached = state.cached.lock().unwrap();
        if let Some(ref info) = *cached {
            return Some(info.clone());
        }
    }
    let detected = detect_manager();
    {
        let mut cached = state.cached.lock().unwrap();
        *cached = detected.clone();
    }
    detected
}

/// 运行 nvm/fnm 命令并返回 stdout
fn exec_manager(info: &NvmManagerInfo, args: &[&str]) -> Result<String, String> {
    let out = if info.kind == "fnm" {
        Command::new(&info.path)
            .args(args)
            .env("NO_COLOR", "1")
            .output()
            .map_err(|e| format!("fnm exec failed: {e}"))?
    } else {
        let cmd = format!(
            "source '{}' --no-use && nvm {}",
            info.path,
            args.join(" ")
        );
        Command::new("bash")
            .arg("-c")
            .arg(&cmd)
            .output()
            .map_err(|e| format!("nvm bash exec failed: {e}"))?
    };

    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();

    if out.status.success() {
        Ok(stdout)
    } else {
        Err(if !stderr.is_empty() { stderr } else { stdout })
    }
}

/// 去除 ANSI 转义码
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for nc in chars.by_ref() {
                    if nc.is_ascii_alphabetic() { break; }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn parse_fnm_list(output: &str) -> Vec<NodeVersion> {
    let mut versions = Vec::new();
    for line in output.lines() {
        let trimmed = strip_ansi(line.trim());
        if trimmed.is_empty() { continue; }
        let is_current = trimmed.starts_with('*');
        let rest = trimmed.trim_start_matches('*').trim();
        let version = match rest.split_whitespace().next() {
            Some(v) if v.starts_with('v') => v.to_string(),
            _ => continue,
        };
        let is_default = rest.contains("aliases/default") || rest.contains("alias/default");
        let major = parse_major(&version);
        versions.push(NodeVersion {
            version,
            major,
            is_current,
            is_default,
            lts_name: lts_name_for_major(major).map(|s| s.to_string()),
        });
    }
    versions
}

fn parse_nvm_list(output: &str) -> Vec<NodeVersion> {
    let mut current_version: Option<String> = None;
    let mut default_version: Option<String> = None;

    for line in output.lines() {
        let s = strip_ansi(line.trim());
        if s.starts_with("->") && !s.starts_with("->  N/A") {
            if let Some(v) = s.splitn(2, "->").nth(1).and_then(|r| r.split_whitespace().next()) {
                if v.starts_with('v') { current_version = Some(v.to_string()); }
            }
        }
        if s.starts_with("default") {
            if let Some(v) = s.split("->").nth(1).and_then(|r| r.split_whitespace().next()) {
                if v.starts_with('v') { default_version = Some(v.to_string()); }
            }
        }
    }

    let mut versions = Vec::new();
    for line in output.lines() {
        let s = strip_ansi(line.trim());
        if s.is_empty() { continue; }
        if s.starts_with("default") || s.starts_with("lts") || s.starts_with("node")
            || s.starts_with("stable") || s.contains("system") { continue; }

        for token in s.split_whitespace() {
            if token.starts_with('v') {
                let major = parse_major(token);
                if major == 0 { break; }
                let is_current = current_version.as_deref() == Some(token);
                let is_default = default_version.as_deref() == Some(token);
                versions.push(NodeVersion {
                    version: token.to_string(),
                    major,
                    is_current,
                    is_default,
                    lts_name: lts_name_for_major(major).map(|s| s.to_string()),
                });
                break;
            }
        }
    }
    versions
}

fn list_installed(info: &NvmManagerInfo) -> Vec<NodeVersion> {
    let args: &[&str] = if info.kind == "fnm" {
        &["list"]
    } else {
        &["ls", "--no-colors"]
    };
    let output = exec_manager(info, args).unwrap_or_default();
    if info.kind == "fnm" { parse_fnm_list(&output) } else { parse_nvm_list(&output) }
}


/// 把 installed 列表 + LTS_SLOTS 组装成 Vec<LtsSlot>，包含 allInstalled（E1/E2）
fn build_slots(installed: &[NodeVersion]) -> Vec<LtsSlot> {
    let lts_majors: HashSet<u32> = LTS_SLOTS.iter().map(|(m, _)| *m).collect();

    // 构建 LTS slots
    let mut slots: Vec<LtsSlot> = LTS_SLOTS.iter().map(|(major, name)| {
        let for_major: Vec<NodeVersion> = installed.iter()
            .filter(|v| v.major == *major)
            .cloned()
            .collect();
        let best = for_major.iter()
            .find(|v| v.is_current)
            .or_else(|| for_major.iter().find(|v| v.is_default))
            .or_else(|| for_major.first())
            .cloned();
        LtsSlot {
            major: *major,
            lts_name: name.to_string(),
            installed: best,
            all_installed: for_major,
            latest_available: None,
            is_lts: true,
        }
    }).collect();

    // E2: 追加不在静态列表里的已装版本（如 v26+）
    let mut extra: HashMap<u32, Vec<NodeVersion>> = HashMap::new();
    for v in installed {
        if !lts_majors.contains(&v.major) {
            extra.entry(v.major).or_default().push(v.clone());
        }
    }
    let mut extra_slots: Vec<LtsSlot> = extra.into_iter().map(|(major, versions)| {
        let lts_name = versions.first()
            .and_then(|v| v.lts_name.clone())
            .unwrap_or_default();
        let best = versions.iter()
            .find(|v| v.is_current)
            .or_else(|| versions.iter().find(|v| v.is_default))
            .or_else(|| versions.first())
            .cloned();
        LtsSlot { major, lts_name, installed: best, all_installed: versions, latest_available: None, is_lts: false }
    }).collect();
    extra_slots.sort_by_key(|s| s.major);
    slots.extend(extra_slots);

    slots
}

// ── Tauri commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn nvm_status(state: tauri::State<NvmState>) -> ApiResponse<NvmStatus> {
    let Some(info) = resolve_manager(&state) else {
        return ok(NvmStatus { manager: "none".to_string(), manager_version: None, node_default: None }, "OK");
    };
    let manager_version = exec_manager(&info, &["--version"]).ok()
        .map(|s| s.trim().to_string());
    let node_default = list_installed(&info).into_iter()
        .find(|v| v.is_default)
        .map(|v| v.version);
    ok(NvmStatus { manager: info.kind, manager_version, node_default }, "OK")
}

#[tauri::command]
pub fn nvm_list_installed(state: tauri::State<NvmState>) -> ApiResponse<Vec<NodeVersion>> {
    let Some(info) = resolve_manager(&state) else {
        return err("NVM_NOT_FOUND", "NVM_NOT_FOUND");
    };
    ok(list_installed(&info), "OK")
}

#[tauri::command]
pub fn nvm_lts_slots(state: tauri::State<NvmState>) -> ApiResponse<Vec<LtsSlot>> {
    let installed = match resolve_manager(&state) {
        Some(ref info) => list_installed(info),
        None => vec![],
    };
    ok(build_slots(&installed), "OK")
}

/// P0-1: 单次 list_installed 同时返回 status + slots
#[tauri::command]
pub fn nvm_refresh(state: tauri::State<NvmState>) -> ApiResponse<NvmRefreshResult> {
    let Some(info) = resolve_manager(&state) else {
        return ok(NvmRefreshResult {
            status: NvmStatus { manager: "none".to_string(), manager_version: None, node_default: None },
            slots: build_slots(&[]),
        }, "OK");
    };

    let manager_version = exec_manager(&info, &["--version"]).ok()
        .map(|s| s.trim().to_string());
    let installed = list_installed(&info);
    let node_default = installed.iter().find(|v| v.is_default).map(|v| v.version.clone());

    ok(NvmRefreshResult {
        status: NvmStatus { manager: info.kind, manager_version, node_default },
        slots: build_slots(&installed),
    }, "OK")
}

/// P1-6: 清除缓存，下次调用重新探测管理器
#[tauri::command]
pub fn nvm_reset_cache(state: tauri::State<NvmState>) -> ApiResponse<bool> {
    let mut cached = state.cached.lock().unwrap();
    *cached = None;
    ok(true, "OK")
}

#[tauri::command]
pub fn nvm_set_default(state: tauri::State<NvmState>, version: String) -> ApiResponse<bool> {
    let Some(info) = resolve_manager(&state) else {
        return err("NVM_NOT_FOUND", "NVM_NOT_FOUND");
    };
    let result = if info.kind == "fnm" {
        exec_manager(&info, &["default", &version])
    } else {
        exec_manager(&info, &["alias", "default", &version])
    };
    match result {
        Ok(_) => ok(true, "OK"),
        Err(e) => err("NVM_SET_DEFAULT_FAILED", &e),
    }
}

/// E3: 同时检测 .nvmrc 和 .node-version
#[tauri::command]
pub fn nvm_read_project(dir: String) -> ApiResponse<ProjectVersion> {
    for filename in &[".nvmrc", ".node-version"] {
        let path = std::path::Path::new(&dir).join(filename);
        if let Ok(content) = std::fs::read_to_string(&path) {
            return ok(ProjectVersion {
                version: content.trim().to_string(),
                file: path.to_string_lossy().to_string(),
            }, "OK");
        }
    }
    err("NVMRC_NOT_FOUND", "NVMRC_NOT_FOUND")
}

/// E3: 写入已存在的文件，否则默认写 .nvmrc
#[tauri::command]
pub fn nvm_write_project(dir: String, version: String) -> ApiResponse<bool> {
    let base = std::path::Path::new(&dir);
    let target = if base.join(".node-version").exists() {
        base.join(".node-version")
    } else {
        base.join(".nvmrc")
    };
    match std::fs::write(&target, format!("{}\n", version.trim())) {
        Ok(_) => ok(true, "OK"),
        Err(e) => err("NVMRC_WRITE_FAILED", &format!("{e}")),
    }
}

/// P2-9: 返回指定版本的 node 二进制路径
#[tauri::command]
pub fn nvm_which(state: tauri::State<NvmState>, version: String) -> ApiResponse<String> {
    let Some(info) = resolve_manager(&state) else {
        return err("NVM_NOT_FOUND", "NVM_NOT_FOUND");
    };
    let result = if info.kind == "fnm" {
        exec_manager(&info, &["exec", &format!("--using={}", version), "--", "node", "-e", "process.stdout.write(process.execPath)"])
    } else {
        exec_manager(&info, &["which", &version])
    };
    match result {
        Ok(out) => ok(out.trim().to_string(), "OK"),
        Err(e) => err("NVM_WHICH_FAILED", &e),
    }
}

/// F1: 一次 curl，返回所有 major 的版本列表（major -> [versions]）
/// 前端预加载时调用，避免多次请求
#[tauri::command]
pub async fn nvm_fetch_all_versions() -> ApiResponse<std::collections::HashMap<u32, Vec<String>>> {
    let out = match tokio::process::Command::new("curl")
        .args(&["-s", "--max-time", "20", "--compressed",
                "https://nodejs.org/dist/index.json"])
        .output()
        .await
    {
        Ok(o) => o,
        Err(e) => return err("CURL_FAILED", &format!("{e}")),
    };

    if !out.status.success() {
        return err("CURL_FAILED", "curl request failed");
    }

    let json: Vec<serde_json::Value> = match serde_json::from_slice(&out.stdout) {
        Ok(j) => j,
        Err(e) => return err("PARSE_FAILED", &format!("{e}")),
    };

    let mut map: std::collections::HashMap<u32, Vec<String>> = std::collections::HashMap::new();
    for entry in &json {
        if let Some(v) = entry["version"].as_str() {
            let major = parse_major(v);
            if major > 0 {
                map.entry(major).or_default().push(v.to_string());
            }
        }
    }

    ok(map, "OK")
}

/// F1: 获取指定 major 的版本列表（保留，供按需单独调用）
#[tauri::command]
pub async fn nvm_fetch_versions_for_major(major: u32) -> ApiResponse<Vec<String>> {
    let out = match tokio::process::Command::new("curl")
        .args(&["-s", "--max-time", "15", "--compressed",
                "https://nodejs.org/dist/index.json"])
        .output()
        .await
    {
        Ok(o) => o,
        Err(e) => return err("CURL_FAILED", &format!("{e}")),
    };

    if !out.status.success() {
        return err("CURL_FAILED", "curl request failed");
    }

    let json: Vec<serde_json::Value> = match serde_json::from_slice(&out.stdout) {
        Ok(j) => j,
        Err(e) => return err("PARSE_FAILED", &format!("{e}")),
    };

    let versions: Vec<String> = json.iter()
        .filter_map(|entry| entry["version"].as_str())
        .filter(|v| parse_major(v) == major)
        .map(|v| v.to_string())
        .collect();

    ok(versions, "OK")
}

/// F3: 从 nodejs.org 获取各 major 的最新版本（异步，curl）
#[tauri::command]
pub async fn nvm_fetch_remote_lts() -> ApiResponse<Vec<RemoteLtsVersion>> {
    let out = match tokio::process::Command::new("curl")
        .args(&["-s", "--max-time", "15", "--compressed",
                "https://nodejs.org/dist/index.json"])
        .output()
        .await
    {
        Ok(o) => o,
        Err(e) => return err("CURL_FAILED", &format!("{e}")),
    };

    if !out.status.success() {
        return err("CURL_FAILED", "curl request failed");
    }

    let json: Vec<serde_json::Value> = match serde_json::from_slice(&out.stdout) {
        Ok(j) => j,
        Err(e) => return err("PARSE_FAILED", &format!("{e}")),
    };

    // index.json 按版本从新到旧排列，每个 major 第一次出现即为最新
    let mut seen: HashSet<u32> = HashSet::new();
    let mut results: Vec<RemoteLtsVersion> = Vec::new();

    for entry in &json {
        let version = match entry["version"].as_str() { Some(v) => v, None => continue };
        let major = parse_major(version);
        if major == 0 || seen.contains(&major) { continue; }
        let lts_name = match &entry["lts"] {
            serde_json::Value::String(s) => Some(s.clone()),
            _ => None,
        };
        seen.insert(major);
        results.push(RemoteLtsVersion { major, latest: version.to_string(), lts_name });
    }

    // 按 major 降序（新版在前）
    results.sort_by(|a, b| b.major.cmp(&a.major));
    ok(results, "OK")
}

#[tauri::command]
pub async fn nvm_run_stream(
    app: tauri::AppHandle,
    nvm_state: tauri::State<'_, NvmState>,
    registry: tauri::State<'_, CancelRegistry>,
    request_id: String,
    action: String,  // "install" | "uninstall"
    version: String,
) -> Result<ApiResponse<bool>, String> {
    let Some(info) = resolve_manager(&nvm_state) else {
        return Ok(err("NVM_NOT_FOUND", "NVM_NOT_FOUND"));
    };

    if action != "install" && action != "uninstall" {
        return Ok(err("INVALID_ACTION", "INVALID_ACTION"));
    }

    let (program, args): (String, Vec<String>) = if info.kind == "fnm" {
        (info.path.clone(), vec![action.clone(), version.clone()])
    } else {
        let bash_cmd = format!(
            "source '{}' --no-use && nvm {} {}",
            info.path, action, version
        );
        ("bash".to_string(), vec!["-c".to_string(), bash_cmd])
    };

    let cancel_flag = Arc::new(Mutex::new(false));
    {
        let mut map = registry.0.lock().unwrap();
        map.insert(request_id.clone(), Arc::clone(&cancel_flag));
    }

    emit_brew_log(&app, BrewLogEvent {
        request_id: request_id.clone(),
        stage: "start".to_string(),
        stream: None, line: None, success: None,
    });

    let mut cmd = tokio::process::Command::new(&program);
    cmd.args(&args).stdout(Stdio::piped()).stderr(Stdio::piped());
    if info.kind == "fnm" { cmd.env("NO_COLOR", "1"); }

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            cleanup_registry(&registry, &request_id);
            return Ok(err("SPAWN_FAILED", &format!("SPAWN_FAILED: {e}")));
        }
    };

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let (tx, mut rx) = tokio::sync::mpsc::channel::<(String, String)>(128);
    let tx_out = tx.clone();
    tokio::spawn(async move {
        use tokio::io::{AsyncBufReadExt, BufReader};
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if tx_out.send(("stdout".to_string(), line)).await.is_err() { break; }
        }
    });
    let tx_err = tx.clone();
    tokio::spawn(async move {
        use tokio::io::{AsyncBufReadExt, BufReader};
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if tx_err.send(("stderr".to_string(), line)).await.is_err() { break; }
        }
    });
    drop(tx);

    let mut cancelled = false;
    let mut had_output = false;

    while let Some((stream, line)) = rx.recv().await {
        if *cancel_flag.lock().unwrap() {
            cancelled = true;
            let _ = child.kill().await;
            break;
        }
        had_output = true;
        emit_brew_log(&app, BrewLogEvent {
            request_id: request_id.clone(),
            stage: "line".to_string(),
            stream: Some(stream),
            line: Some(line),
            success: None,
        });
    }

    if cancelled {
        cleanup_registry(&registry, &request_id);
        emit_brew_log(&app, BrewLogEvent {
            request_id: request_id.clone(),
            stage: "end".to_string(),
            stream: None, line: None, success: Some(false),
        });
        return Ok(err("CANCELLED", "CANCELLED"));
    }

    let status = match child.wait().await {
        Ok(s) => s,
        Err(e) => {
            cleanup_registry(&registry, &request_id);
            return Ok(err("WAIT_FAILED", &format!("WAIT_FAILED: {e}")));
        }
    };

    if !status.success() && !had_output {
        emit_brew_log(&app, BrewLogEvent {
            request_id: request_id.clone(),
            stage: "line".to_string(),
            stream: Some("stderr".to_string()),
            line: Some(format!(
                "Command exited with no output (exit code: {})",
                status.code().map(|c| c.to_string()).unwrap_or_else(|| "unknown".to_string())
            )),
            success: None,
        });
    }

    emit_brew_log(&app, BrewLogEvent {
        request_id: request_id.clone(),
        stage: "end".to_string(),
        stream: None, line: None, success: Some(status.success()),
    });

    cleanup_registry(&registry, &request_id);

    if status.success() { Ok(ok(true, "OK")) }
    else { Ok(err("COMMAND_FAILED", "COMMAND_FAILED")) }
}

fn cleanup_registry(registry: &tauri::State<CancelRegistry>, request_id: &str) {
    let mut map = registry.0.lock().unwrap();
    map.remove(request_id);
}
