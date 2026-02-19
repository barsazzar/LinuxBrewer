use std::process::Stdio;

use crate::brew::{detect_brew, emit_brew_log, is_valid_pkg_name};
use crate::types::{err, ok, ApiResponse, BrewLogEvent};

#[tauri::command]
pub async fn brew_run_stream(
    app: tauri::AppHandle,
    request_id: String,
    action: String,
    name: Option<String>,
    kind: Option<String>,
) -> ApiResponse<bool> {
    let brew = match detect_brew() {
        Some(v) => v,
        None => return err("BREW_NOT_FOUND", "未找到 brew"),
    };

    let mut args: Vec<String> = vec![];
    match action.as_str() {
        "doctor" => args.push("doctor".to_string()),
        "upgrade_all" => args.push("upgrade".to_string()),
        "tap" | "untap" => {
            let Some(tap_name) = name else {
                return err("INVALID_NAME", "缺少 tap 名称");
            };
            if tap_name.trim().is_empty() {
                return err("INVALID_NAME", "tap 名称不能为空");
            }
            if action == "tap" {
                args.push("tap".to_string());
            } else {
                args.push("untap".to_string());
            }
            args.push(tap_name);
        }
        "info" | "install" | "uninstall" | "upgrade" => {
            let Some(pkg_name) = name else {
                return err("INVALID_NAME", "缺少包名");
            };
            if !is_valid_pkg_name(&pkg_name) {
                return err("INVALID_NAME", "包名不合法");
            }
            let k = kind.unwrap_or_else(|| "formula".to_string());
            if k != "formula" && k != "cask" {
                return err("INVALID_KIND", "kind 仅支持 formula 或 cask");
            }
            match action.as_str() {
                "info" => {
                    args.push("info".to_string());
                    if k == "cask" {
                        args.push("--cask".to_string());
                    }
                }
                "install" => {
                    args.push("install".to_string());
                    if k == "cask" {
                        args.push("--cask".to_string());
                    }
                }
                "uninstall" => {
                    args.push("uninstall".to_string());
                    if k == "cask" {
                        args.push("--cask".to_string());
                    }
                }
                "upgrade" => {
                    args.push("upgrade".to_string());
                    if k == "cask" {
                        args.push("--cask".to_string());
                    }
                }
                _ => unreachable!(),
            }
            args.push(pkg_name);
        }
        _ => {
            return err(
                "INVALID_ACTION",
                "action 仅支持 info / doctor / install / uninstall / upgrade / upgrade_all",
            )
        }
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
        Err(e) => return err("SPAWN_FAILED", &format!("启动 brew 失败: {e}")),
    };

    let stdout = match child.stdout.take() {
        Some(s) => s,
        None => return err("STDOUT_PIPE_FAILED", "无法读取 stdout"),
    };
    let stderr = match child.stderr.take() {
        Some(s) => s,
        None => return err("STDERR_PIPE_FAILED", "无法读取 stderr"),
    };

    let (tx, mut rx) = tokio::sync::mpsc::channel::<(String, String)>(64);

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

    drop(tx);

    let mut had_output = false;
    while let Some((stream, line)) = rx.recv().await {
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

    let status = match child.wait().await {
        Ok(s) => s,
        Err(e) => return err("WAIT_FAILED", &format!("等待命令结束失败: {e}")),
    };

    if !status.success() && !had_output {
        emit_brew_log(
            &app,
            BrewLogEvent {
                request_id: request_id.clone(),
                stage: "line".to_string(),
                stream: Some("stderr".to_string()),
                line: Some(format!(
                    "命令退出但没有输出（exit code: {}）",
                    status
                        .code()
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "unknown".to_string())
                )),
                success: None,
            },
        );
    }

    emit_brew_log(
        &app,
        BrewLogEvent {
            request_id,
            stage: "end".to_string(),
            stream: None,
            line: None,
            success: Some(status.success()),
        },
    );

    if status.success() {
        ok(true, "命令执行完成")
    } else {
        err("COMMAND_FAILED", "命令执行失败，请查看输出")
    }
}
