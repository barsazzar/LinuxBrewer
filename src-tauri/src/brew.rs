use std::process::Command;
use tauri::Emitter;

use crate::types::{ok, err, ApiResponse, BrewLogEvent, BrewPackage, BrewStatus};

pub(crate) fn detect_brew() -> Option<String> {
    let candidates = [
        "/home/linuxbrew/.linuxbrew/bin/brew",
        "/linuxbrew/.linuxbrew/bin/brew",
        "/usr/local/bin/brew",
        "/opt/homebrew/bin/brew",
        "brew",
    ];

    for c in candidates {
        let output = Command::new(c).arg("--version").output();
        if let Ok(out) = output {
            if out.status.success() {
                return Some(c.to_string());
            }
        }
    }
    None
}

pub(crate) fn is_valid_pkg_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '+' | '.' | '-' | '@' | '_'))
}

pub(crate) fn brew_output(args: &[&str]) -> Result<String, String> {
    let brew = detect_brew().ok_or_else(|| "brew not found".to_string())?;
    let out = Command::new(&brew)
        .args(args)
        .output()
        .map_err(|e| format!("failed to execute brew: {e}"))?;

    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).to_string())
    }
}

pub(crate) fn emit_brew_log(app: &tauri::AppHandle, event: BrewLogEvent) {
    let _ = app.emit("brew-log", event);
}

#[tauri::command]
pub fn brew_status() -> ApiResponse<BrewStatus> {
    let Some(path) = detect_brew() else {
        return err("BREW_NOT_FOUND", "未找到 brew，请先确认 Linuxbrew 安装路径");
    };

    let out = Command::new(&path).arg("--version").output();
    match out {
        Ok(result) if result.status.success() => {
            let version = String::from_utf8_lossy(&result.stdout)
                .lines()
                .next()
                .unwrap_or_default()
                .to_string();
            ok(
                BrewStatus {
                    brew_path: path,
                    version,
                },
                "brew 可用",
            )
        }
        Ok(result) => err(
            "BREW_VERSION_FAILED",
            &String::from_utf8_lossy(&result.stderr),
        ),
        Err(e) => err("BREW_EXEC_FAILED", &format!("执行 brew 失败: {e}")),
    }
}

#[tauri::command]
pub fn brew_list_installed() -> ApiResponse<Vec<BrewPackage>> {
    let formulas_raw = match brew_output(&["list", "--formula", "--versions"]) {
        Ok(v) => v,
        Err(e) => return err("BREW_LIST_FORMULA_FAILED", &e),
    };

    let casks_raw = match brew_output(&["list", "--cask", "--versions"]) {
        Ok(v) => v,
        Err(e) => return err("BREW_LIST_CASK_FAILED", &e),
    };

    let mut out: Vec<BrewPackage> = vec![];

    for line in formulas_raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let name = parts.next().unwrap_or_default().to_string();
        let version = parts.next().map(|v| v.to_string());
        out.push(BrewPackage {
            name,
            version,
            kind: "formula".to_string(),
        });
    }

    for line in casks_raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let name = parts.next().unwrap_or_default().to_string();
        let version = parts.next().map(|v| v.to_string());
        out.push(BrewPackage {
            name,
            version,
            kind: "cask".to_string(),
        });
    }

    out.sort_by(|a, b| a.name.cmp(&b.name));

    ok(out, "已获取安装列表")
}

#[tauri::command]
pub fn brew_outdated() -> ApiResponse<Vec<BrewPackage>> {
    let formulas_raw = match brew_output(&["outdated", "--formula", "--verbose"]) {
        Ok(v) => v,
        Err(e) => return err("BREW_OUTDATED_FORMULA_FAILED", &e),
    };
    let casks_raw = match brew_output(&["outdated", "--cask", "--verbose"]) {
        Ok(v) => v,
        Err(e) => return err("BREW_OUTDATED_CASK_FAILED", &e),
    };

    let mut out: Vec<BrewPackage> = vec![];

    for line in formulas_raw.lines() {
        let mut parts = line.split_whitespace();
        let name = parts.next().unwrap_or_default().to_string();
        if name.is_empty() {
            continue;
        }
        out.push(BrewPackage {
            name,
            version: None,
            kind: "formula".to_string(),
        });
    }

    for line in casks_raw.lines() {
        let mut parts = line.split_whitespace();
        let name = parts.next().unwrap_or_default().to_string();
        if name.is_empty() {
            continue;
        }
        out.push(BrewPackage {
            name,
            version: None,
            kind: "cask".to_string(),
        });
    }

    out.sort_by(|a, b| a.name.cmp(&b.name));
    ok(out, "已获取可升级列表")
}

#[tauri::command]
pub fn brew_install(name: String, kind: String) -> ApiResponse<String> {
    if !is_valid_pkg_name(&name) {
        return err("INVALID_NAME", "包名不合法");
    }
    if kind != "formula" && kind != "cask" {
        return err("INVALID_KIND", "kind 仅支持 formula 或 cask");
    }

    let args = if kind == "cask" {
        vec!["install", "--cask", name.as_str()]
    } else {
        vec!["install", name.as_str()]
    };

    match brew_output(&args) {
        Ok(stdout) => ok(stdout, "安装完成"),
        Err(stderr) => err("INSTALL_FAILED", &stderr),
    }
}

#[tauri::command]
pub fn brew_uninstall(name: String, kind: String) -> ApiResponse<String> {
    if !is_valid_pkg_name(&name) {
        return err("INVALID_NAME", "包名不合法");
    }
    if kind != "formula" && kind != "cask" {
        return err("INVALID_KIND", "kind 仅支持 formula 或 cask");
    }

    let args = if kind == "cask" {
        vec!["uninstall", "--cask", name.as_str()]
    } else {
        vec!["uninstall", name.as_str()]
    };

    match brew_output(&args) {
        Ok(stdout) => ok(stdout, "卸载完成"),
        Err(stderr) => err("UNINSTALL_FAILED", &stderr),
    }
}

#[tauri::command]
pub fn brew_upgrade_all() -> ApiResponse<String> {
    match brew_output(&["upgrade"]) {
        Ok(stdout) => ok(stdout, "全部升级完成"),
        Err(stderr) => err("UPGRADE_ALL_FAILED", &stderr),
    }
}

#[tauri::command]
pub fn brew_upgrade_single(name: String, kind: String) -> ApiResponse<String> {
    if !is_valid_pkg_name(&name) {
        return err("INVALID_NAME", "包名不合法");
    }
    if kind != "formula" && kind != "cask" {
        return err("INVALID_KIND", "kind 仅支持 formula 或 cask");
    }

    let args = if kind == "cask" {
        vec!["upgrade", "--cask", name.as_str()]
    } else {
        vec!["upgrade", name.as_str()]
    };

    match brew_output(&args) {
        Ok(stdout) => ok(stdout, "升级完成"),
        Err(stderr) => err("UPGRADE_FAILED", &stderr),
    }
}

#[tauri::command]
pub fn brew_search(query: String) -> ApiResponse<Vec<BrewPackage>> {
    if query.trim().is_empty() {
        return err("EMPTY_QUERY", "搜索词不能为空");
    }

    let formula_results = brew_output(&["search", "--formula", query.as_str()]).unwrap_or_default();
    let cask_results = brew_output(&["search", "--cask", query.as_str()]).unwrap_or_default();

    let mut pkgs: Vec<BrewPackage> = Vec::new();
    for line in formula_results.lines() {
        let name = line.trim().to_string();
        if !name.is_empty() && !name.starts_with('=') {
            pkgs.push(BrewPackage {
                name,
                version: None,
                kind: "formula".to_string(),
            });
        }
    }
    for line in cask_results.lines() {
        let name = line.trim().to_string();
        if !name.is_empty() && !name.starts_with('=') {
            pkgs.push(BrewPackage {
                name,
                version: None,
                kind: "cask".to_string(),
            });
        }
    }

    if pkgs.is_empty() {
        err("NO_RESULTS", "未找到匹配的包")
    } else {
        ok(pkgs, "搜索完成")
    }
}

#[tauri::command]
pub fn brew_info(name: String, kind: String) -> ApiResponse<String> {
    if !is_valid_pkg_name(&name) {
        return err("INVALID_NAME", "包名不合法");
    }
    if kind != "formula" && kind != "cask" {
        return err("INVALID_KIND", "kind 仅支持 formula 或 cask");
    }

    let args = if kind == "cask" {
        vec!["info", "--cask", name.as_str()]
    } else {
        vec!["info", name.as_str()]
    };

    match brew_output(&args) {
        Ok(stdout) => ok(stdout, "已获取包详情"),
        Err(stderr) => err("INFO_FAILED", &stderr),
    }
}

#[tauri::command]
pub fn brew_doctor() -> ApiResponse<String> {
    match brew_output(&["doctor"]) {
        Ok(stdout) => ok(stdout, "doctor 执行完成"),
        Err(stderr) => err("DOCTOR_FAILED", &stderr),
    }
}

#[tauri::command]
pub fn brew_tap_list() -> ApiResponse<Vec<String>> {
    match brew_output(&["tap"]) {
        Ok(stdout) => {
            let taps: Vec<String> = stdout
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect();
            ok(taps, "加载成功")
        }
        Err(stderr) => err("TAP_LIST_FAILED", &stderr),
    }
}
