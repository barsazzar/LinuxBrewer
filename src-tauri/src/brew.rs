use std::collections::HashMap;
use std::process::Command;
use tauri::Emitter;

use crate::types::{ok, err, ApiResponse, BrewLogEvent, BrewPackage, BrewState, BrewStatus};

/// Detect brew binary from standard candidate locations.
fn detect_brew_binary() -> Option<String> {
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

/// Resolve brew path: prefer custom path, then cached, then auto-detect.
/// Caches the result so subsequent calls are free.
pub(crate) fn resolve_brew(state: &BrewState) -> Option<String> {
    // If there is a custom path set, use it (bypass cache)
    {
        let cp = state.custom_path.lock().unwrap();
        if let Some(ref path) = *cp {
            return Some(path.clone());
        }
    }
    // Return cached auto-detected path
    {
        let cached = state.cached_path.lock().unwrap();
        if let Some(ref path) = *cached {
            return Some(path.clone());
        }
    }
    // Auto-detect and cache
    let detected = detect_brew_binary();
    {
        let mut cached = state.cached_path.lock().unwrap();
        *cached = detected.clone();
    }
    detected
}

pub(crate) fn is_valid_pkg_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '+' | '.' | '-' | '@' | '_'))
}

pub(crate) fn brew_output(brew: &str, args: &[&str]) -> Result<String, String> {
    let out = Command::new(brew)
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

// ── Commands ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn set_brew_path(state: tauri::State<BrewState>, path: Option<String>) -> ApiResponse<bool> {
    // Validate the custom path if provided
    if let Some(ref p) = path {
        let trimmed = p.trim();
        if !trimmed.is_empty() {
            let output = Command::new(trimmed).arg("--version").output();
            match output {
                Ok(out) if out.status.success() => {}
                _ => return err("INVALID_BREW_PATH", "INVALID_BREW_PATH"),
            }
        }
    }
    // Store custom path and invalidate all caches
    {
        let mut cp = state.custom_path.lock().unwrap();
        *cp = path.and_then(|p| {
            let t = p.trim().to_string();
            if t.is_empty() { None } else { Some(t) }
        });
    }
    { *state.cached_path.lock().unwrap() = None; }
    { *state.cached_version.lock().unwrap() = None; }
    ok(true, "OK")
}

#[tauri::command]
pub fn brew_status(state: tauri::State<BrewState>) -> ApiResponse<BrewStatus> {
    let Some(path) = resolve_brew(&state) else {
        return err("BREW_NOT_FOUND", "BREW_NOT_FOUND");
    };

    // Return cached version if available (avoids re-running `brew --version` on every refresh)
    if let Some(version) = state.cached_version.lock().unwrap().clone() {
        return ok(BrewStatus { brew_path: path, version }, "OK");
    }

    let out = Command::new(&path).arg("--version").output();
    match out {
        Ok(result) if result.status.success() => {
            let version = String::from_utf8_lossy(&result.stdout)
                .lines()
                .next()
                .unwrap_or_default()
                .to_string();
            *state.cached_version.lock().unwrap() = Some(version.clone());
            ok(BrewStatus { brew_path: path, version }, "OK")
        }
        Ok(result) => err("BREW_VERSION_FAILED", &String::from_utf8_lossy(&result.stderr)),
        Err(e) => err("BREW_EXEC_FAILED", &format!("BREW_EXEC_FAILED: {e}")),
    }
}

#[tauri::command]
pub fn brew_list_installed(state: tauri::State<BrewState>) -> ApiResponse<Vec<BrewPackage>> {
    let Some(brew) = resolve_brew(&state) else {
        return err("BREW_NOT_FOUND", "BREW_NOT_FOUND");
    };

    let formulas_raw = match brew_output(&brew, &["list", "--formula", "--versions"]) {
        Ok(v) => v,
        Err(e) => return err("BREW_LIST_FORMULA_FAILED", &e),
    };

    let casks_raw = match brew_output(&brew, &["list", "--cask", "--versions"]) {
        Ok(v) => v,
        Err(e) => return err("BREW_LIST_CASK_FAILED", &e),
    };

    let mut out: Vec<BrewPackage> = vec![];

    for line in formulas_raw.lines() {
        if line.trim().is_empty() { continue; }
        let mut parts = line.split_whitespace();
        let name = parts.next().unwrap_or_default().to_string();
        let version = parts.next().map(|v| v.to_string());
        out.push(BrewPackage { name, version, new_version: None, kind: "formula".to_string(), description: None });
    }

    for line in casks_raw.lines() {
        if line.trim().is_empty() { continue; }
        let mut parts = line.split_whitespace();
        let name = parts.next().unwrap_or_default().to_string();
        let version = parts.next().map(|v| v.to_string());
        out.push(BrewPackage { name, version, new_version: None, kind: "cask".to_string(), description: None });
    }

    out.sort_by(|a, b| a.name.cmp(&b.name));
    ok(out, "OK")
}

#[tauri::command]
pub fn brew_outdated(state: tauri::State<BrewState>) -> ApiResponse<Vec<BrewPackage>> {
    let Some(brew) = resolve_brew(&state) else {
        return err("BREW_NOT_FOUND", "BREW_NOT_FOUND");
    };

    // --verbose output format: "name (current_ver) < new_ver"
    // e.g. "wget (1.21.3) != 1.21.4" or "wget (1.21.3) < 1.21.4"
    let formulas_raw = match brew_output(&brew, &["outdated", "--formula", "--verbose"]) {
        Ok(v) => v,
        Err(e) => return err("BREW_OUTDATED_FORMULA_FAILED", &e),
    };
    let casks_raw = match brew_output(&brew, &["outdated", "--cask", "--verbose"]) {
        Ok(v) => v,
        Err(e) => return err("BREW_OUTDATED_CASK_FAILED", &e),
    };

    let parse_outdated_line = |line: &str, kind: &str| -> Option<BrewPackage> {
        let mut parts = line.split_whitespace();
        let name = parts.next()?.to_string();
        if name.is_empty() { return None; }
        // Parse "(current)" and "< new_ver" or "!= new_ver"
        let current = parts.next()
            .map(|s| s.trim_matches(|c| c == '(' || c == ')').to_string());
        // Skip the separator "(!=, <, etc.)"
        let _sep = parts.next();
        let new_ver = parts.next().map(|s| s.to_string());
        Some(BrewPackage {
            name,
            version: current,
            new_version: new_ver,
            kind: kind.to_string(),
            description: None,
        })
    };

    let mut out: Vec<BrewPackage> = vec![];
    for line in formulas_raw.lines() {
        if let Some(pkg) = parse_outdated_line(line, "formula") {
            out.push(pkg);
        }
    }
    for line in casks_raw.lines() {
        if let Some(pkg) = parse_outdated_line(line, "cask") {
            out.push(pkg);
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    ok(out, "OK")
}

#[tauri::command]
pub fn brew_search(state: tauri::State<BrewState>, query: String) -> ApiResponse<Vec<BrewPackage>> {
    if query.trim().is_empty() {
        return err("EMPTY_QUERY", "EMPTY_QUERY");
    }
    let Some(brew) = resolve_brew(&state) else {
        return err("BREW_NOT_FOUND", "BREW_NOT_FOUND");
    };

    let formula_results = brew_output(&brew, &["search", "--formula", query.as_str()]).unwrap_or_default();
    let cask_results = brew_output(&brew, &["search", "--cask", query.as_str()]).unwrap_or_default();

    let mut pkgs: Vec<BrewPackage> = Vec::new();
    for line in formula_results.lines() {
        let name = line.trim().to_string();
        if !name.is_empty() && !name.starts_with('=') {
            pkgs.push(BrewPackage { name, version: None, new_version: None, kind: "formula".to_string(), description: None });
        }
    }
    for line in cask_results.lines() {
        let name = line.trim().to_string();
        if !name.is_empty() && !name.starts_with('=') {
            pkgs.push(BrewPackage { name, version: None, new_version: None, kind: "cask".to_string(), description: None });
        }
    }

    if pkgs.is_empty() {
        return err("NO_RESULTS", "NO_RESULTS");
    }

    // Enrich up to 30 results with version and description via a single `brew info --json=v2` call.
    // Limiting to 30 keeps the command-line short and response time reasonable.
    let names: Vec<&str> = pkgs.iter().take(30).map(|p| p.name.as_str()).collect();
    let mut info_args = vec!["info", "--json=v2"];
    info_args.extend_from_slice(&names);
    if let Ok(info_json) = brew_output(&brew, &info_args) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&info_json) {
            let mut info_map: HashMap<String, (Option<String>, Option<String>)> = HashMap::new();
            if let Some(formulae) = val["formulae"].as_array() {
                for f in formulae {
                    let name = f["name"].as_str().unwrap_or("").to_string();
                    let desc = f["desc"].as_str().map(|s| s.to_string());
                    let version = f["versions"]["stable"].as_str().map(|s| s.to_string());
                    info_map.insert(name, (version, desc));
                }
            }
            if let Some(casks) = val["casks"].as_array() {
                for c in casks {
                    let name = c["token"].as_str().unwrap_or("").to_string();
                    let desc = c["desc"].as_str().map(|s| s.to_string());
                    let version = c["version"].as_str().map(|s| s.to_string());
                    info_map.insert(name, (version, desc));
                }
            }
            for pkg in &mut pkgs {
                if let Some((version, desc)) = info_map.get(&pkg.name) {
                    pkg.version = version.clone();
                    pkg.description = desc.clone();
                }
            }
        }
    }

    ok(pkgs, "OK")
}

#[tauri::command]
pub fn brew_tap_list(state: tauri::State<BrewState>) -> ApiResponse<Vec<String>> {
    let Some(brew) = resolve_brew(&state) else {
        return err("BREW_NOT_FOUND", "BREW_NOT_FOUND");
    };
    match brew_output(&brew, &["tap"]) {
        Ok(stdout) => {
            let taps: Vec<String> = stdout
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect();
            ok(taps, "OK")
        }
        Err(stderr) => err("TAP_LIST_FAILED", &stderr),
    }
}

// ── Unused legacy commands (kept for compatibility) ───────────────────────────

#[tauri::command]
pub fn brew_install(state: tauri::State<BrewState>, name: String, kind: String) -> ApiResponse<String> {
    if !is_valid_pkg_name(&name) { return err("INVALID_NAME", "INVALID_NAME"); }
    if kind != "formula" && kind != "cask" { return err("INVALID_KIND", "INVALID_KIND"); }
    let Some(brew) = resolve_brew(&state) else { return err("BREW_NOT_FOUND", "BREW_NOT_FOUND"); };
    let args = if kind == "cask" { vec!["install", "--cask", name.as_str()] } else { vec!["install", name.as_str()] };
    match brew_output(&brew, &args) {
        Ok(stdout) => ok(stdout, "OK"),
        Err(stderr) => err("INSTALL_FAILED", &stderr),
    }
}

#[tauri::command]
pub fn brew_uninstall(state: tauri::State<BrewState>, name: String, kind: String) -> ApiResponse<String> {
    if !is_valid_pkg_name(&name) { return err("INVALID_NAME", "INVALID_NAME"); }
    if kind != "formula" && kind != "cask" { return err("INVALID_KIND", "INVALID_KIND"); }
    let Some(brew) = resolve_brew(&state) else { return err("BREW_NOT_FOUND", "BREW_NOT_FOUND"); };
    let args = if kind == "cask" { vec!["uninstall", "--cask", name.as_str()] } else { vec!["uninstall", name.as_str()] };
    match brew_output(&brew, &args) {
        Ok(stdout) => ok(stdout, "OK"),
        Err(stderr) => err("UNINSTALL_FAILED", &stderr),
    }
}

#[tauri::command]
pub fn brew_upgrade_all(state: tauri::State<BrewState>) -> ApiResponse<String> {
    let Some(brew) = resolve_brew(&state) else { return err("BREW_NOT_FOUND", "BREW_NOT_FOUND"); };
    match brew_output(&brew, &["upgrade"]) {
        Ok(stdout) => ok(stdout, "OK"),
        Err(stderr) => err("UPGRADE_ALL_FAILED", &stderr),
    }
}

#[tauri::command]
pub fn brew_upgrade_single(state: tauri::State<BrewState>, name: String, kind: String) -> ApiResponse<String> {
    if !is_valid_pkg_name(&name) { return err("INVALID_NAME", "INVALID_NAME"); }
    if kind != "formula" && kind != "cask" { return err("INVALID_KIND", "INVALID_KIND"); }
    let Some(brew) = resolve_brew(&state) else { return err("BREW_NOT_FOUND", "BREW_NOT_FOUND"); };
    let args = if kind == "cask" { vec!["upgrade", "--cask", name.as_str()] } else { vec!["upgrade", name.as_str()] };
    match brew_output(&brew, &args) {
        Ok(stdout) => ok(stdout, "OK"),
        Err(stderr) => err("UPGRADE_FAILED", &stderr),
    }
}

#[tauri::command]
pub fn brew_info(state: tauri::State<BrewState>, name: String, kind: String) -> ApiResponse<String> {
    if !is_valid_pkg_name(&name) { return err("INVALID_NAME", "INVALID_NAME"); }
    if kind != "formula" && kind != "cask" { return err("INVALID_KIND", "INVALID_KIND"); }
    let Some(brew) = resolve_brew(&state) else { return err("BREW_NOT_FOUND", "BREW_NOT_FOUND"); };
    let args = if kind == "cask" { vec!["info", "--cask", name.as_str()] } else { vec!["info", name.as_str()] };
    match brew_output(&brew, &args) {
        Ok(stdout) => ok(stdout, "OK"),
        Err(stderr) => err("INFO_FAILED", &stderr),
    }
}

#[tauri::command]
pub fn brew_doctor(state: tauri::State<BrewState>) -> ApiResponse<String> {
    let Some(brew) = resolve_brew(&state) else { return err("BREW_NOT_FOUND", "BREW_NOT_FOUND"); };
    match brew_output(&brew, &["doctor"]) {
        Ok(stdout) => ok(stdout, "OK"),
        Err(stderr) => err("DOCTOR_FAILED", &stderr),
    }
}
