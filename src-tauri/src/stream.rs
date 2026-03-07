use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use crate::brew::{resolve_brew, emit_brew_log, is_valid_pkg_name};
use crate::types::{err, ok, ApiResponse, BrewLogEvent, BrewState};

/// Global registry of active cancellation flags, keyed by request_id.
/// Each flag is set to `true` when the client requests cancellation.
pub struct CancelRegistry(pub Mutex<HashMap<String, Arc<Mutex<bool>>>>);

impl CancelRegistry {
    pub fn new() -> Self {
        CancelRegistry(Mutex::new(HashMap::new()))
    }
}

#[tauri::command]
pub fn cancel_brew_stream(registry: tauri::State<CancelRegistry>, request_id: String) {
    let map = registry.0.lock().unwrap();
    if let Some(flag) = map.get(&request_id) {
        *flag.lock().unwrap() = true;
    }
}

#[tauri::command]
pub async fn brew_run_stream(
    app: tauri::AppHandle,
    brew_state: tauri::State<'_, BrewState>,
    registry: tauri::State<'_, CancelRegistry>,
    request_id: String,
    action: String,
    name: Option<String>,
    kind: Option<String>,
) -> Result<ApiResponse<bool>, String> {
    let brew = match resolve_brew(&brew_state) {
        Some(v) => v,
        None => return Ok(err("BREW_NOT_FOUND", "BREW_NOT_FOUND")),
    };

    // Build args from action
    let mut args: Vec<String> = vec![];
    match action.as_str() {
        "doctor" => args.push("doctor".to_string()),
        "update" => args.push("update".to_string()),
        "cleanup" => {
            args.push("cleanup".to_string());
            args.push("-s".to_string());
        }
        "upgrade_all" => args.push("upgrade".to_string()),
        "tap" | "untap" => {
            let Some(tap_name) = name else {
                return Ok(err("INVALID_NAME", "INVALID_NAME"));
            };
            if tap_name.trim().is_empty() {
                return Ok(err("INVALID_NAME", "INVALID_NAME"));
            }
            args.push(action.clone());
            args.push(tap_name);
        }
        "info" | "install" | "uninstall" | "upgrade" | "pin" | "unpin" => {
            let Some(pkg_name) = name else {
                return Ok(err("INVALID_NAME", "INVALID_NAME"));
            };
            if !is_valid_pkg_name(&pkg_name) {
                return Ok(err("INVALID_NAME", "INVALID_NAME"));
            }
            let k = kind.unwrap_or_else(|| "formula".to_string());
            if k != "formula" && k != "cask" {
                return Ok(err("INVALID_KIND", "INVALID_KIND"));
            }
            match action.as_str() {
                "info" => {
                    args.push("info".to_string());
                    if k == "cask" { args.push("--cask".to_string()); }
                }
                "install" => {
                    args.push("install".to_string());
                    if k == "cask" { args.push("--cask".to_string()); }
                }
                "uninstall" => {
                    args.push("uninstall".to_string());
                    if k == "cask" { args.push("--cask".to_string()); }
                }
                "upgrade" => {
                    args.push("upgrade".to_string());
                    if k == "cask" { args.push("--cask".to_string()); }
                }
                "pin" => {
                    args.push("pin".to_string());
                }
                "unpin" => {
                    args.push("unpin".to_string());
                }
                _ => unreachable!(),
            }
            args.push(pkg_name);
        }
        _ => {
            return Ok(err("INVALID_ACTION", "INVALID_ACTION"));
        }
    }

    // Register a cancellation flag for this request
    let cancel_flag = Arc::new(Mutex::new(false));
    {
        let mut map = registry.0.lock().unwrap();
        map.insert(request_id.clone(), Arc::clone(&cancel_flag));
    }

    emit_brew_log(
        &app,
        BrewLogEvent {
            request_id: request_id.clone(),
            stage: "start".to_string(),
            stream: None,
            line: None,
            success: None,
        },
    );

    let mut child = match tokio::process::Command::new(&brew)
        .args(args.iter().map(String::as_str))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            cleanup_registry(&registry, &request_id);
            return Ok(err("SPAWN_FAILED", &format!("SPAWN_FAILED: {e}")));
        }
    };

    let stdout = match child.stdout.take() {
        Some(s) => s,
        None => {
            cleanup_registry(&registry, &request_id);
            return Ok(err("STDOUT_PIPE_FAILED", "STDOUT_PIPE_FAILED"));
        }
    };
    let stderr = match child.stderr.take() {
        Some(s) => s,
        None => {
            cleanup_registry(&registry, &request_id);
            return Ok(err("STDERR_PIPE_FAILED", "STDERR_PIPE_FAILED"));
        }
    };

    // Use a single channel with tagged (stream, line) messages for ordering.
    // Both readers send into the same channel so the main loop processes them
    // in arrival order — eliminating the race condition.
    let (tx, mut rx) = tokio::sync::mpsc::channel::<(String, String)>(128);

    let tx_out = tx.clone();
    tokio::spawn(async move {
        use tokio::io::{AsyncBufReadExt, BufReader};
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if tx_out.send(("stdout".to_string(), line)).await.is_err() {
                break;
            }
        }
    });

    let tx_err = tx.clone();
    tokio::spawn(async move {
        use tokio::io::{AsyncBufReadExt, BufReader};
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if tx_err.send(("stderr".to_string(), line)).await.is_err() {
                break;
            }
        }
    });

    drop(tx); // channel closes when both spawned tasks finish

    let mut had_output = false;
    let mut cancelled = false;

    while let Some((stream, line)) = rx.recv().await {
        // Check cancellation flag
        if *cancel_flag.lock().unwrap() {
            cancelled = true;
            // Kill the child process
            let _ = child.kill().await;
            break;
        }
        had_output = true;
        emit_brew_log(
            &app,
            BrewLogEvent {
                request_id: request_id.clone(),
                stage: "line".to_string(),
                stream: Some(stream),
                line: Some(line),
                success: None,
            },
        );
    }

    if cancelled {
        cleanup_registry(&registry, &request_id);
        emit_brew_log(
            &app,
            BrewLogEvent {
                request_id: request_id.clone(),
                stage: "end".to_string(),
                stream: None,
                line: None,
                success: Some(false),
            },
        );
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
        emit_brew_log(
            &app,
            BrewLogEvent {
                request_id: request_id.clone(),
                stage: "line".to_string(),
                stream: Some("stderr".to_string()),
                line: Some(format!(
                    "Command exited with no output (exit code: {})",
                    status.code().map(|c| c.to_string()).unwrap_or_else(|| "unknown".to_string())
                )),
                success: None,
            },
        );
    }

    emit_brew_log(
        &app,
        BrewLogEvent {
            request_id: request_id.clone(),
            stage: "end".to_string(),
            stream: None,
            line: None,
            success: Some(status.success()),
        },
    );

    cleanup_registry(&registry, &request_id);

    if status.success() {
        Ok(ok(true, "OK"))
    } else {
        Ok(err("COMMAND_FAILED", "COMMAND_FAILED"))
    }
}

fn cleanup_registry(registry: &tauri::State<CancelRegistry>, request_id: &str) {
    let mut map = registry.0.lock().unwrap();
    map.remove(request_id);
}
