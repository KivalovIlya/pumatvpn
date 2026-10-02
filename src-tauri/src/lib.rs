use std::{
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::Mutex,
};

use serde::Serialize;
use tauri::{Manager, RunEvent, State};

struct SingBoxState(Mutex<Option<Child>>);

#[derive(Serialize)]
struct AppStatus {
    running: bool,
    binary_found: bool,
    config_found: bool,
}

fn executable_name() -> &'static str {
    if cfg!(windows) {
        "sing-box.exe"
    } else {
        "sing-box"
    }
}

fn find_file(name: &str) -> Option<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join(name));
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join(name));

        // During `tauri dev`, the working directory is normally the
        // project root. This also covers running from src-tauri/.
        if cwd.file_name().is_some_and(|n| n == "src-tauri") {
            if let Some(parent) = cwd.parent() {
                candidates.push(parent.join(name));
            }
        }
    }

    candidates.into_iter().find(|path| path.is_file())
}

fn binary_path() -> Result<PathBuf, String> {
    find_file(executable_name()).ok_or_else(|| {
        format!(
            "{} not found. Put {} next to the PumatVPN executable.",
            executable_name(),
            executable_name()
        )
    })
}

fn config_path() -> Result<PathBuf, String> {
    find_file("config.json")
        .ok_or_else(|| "config.json not found. Put it next to the PumatVPN executable.".to_string())
}

fn validate_config(binary: &Path, config: &Path) -> Result<(), String> {
    let output = Command::new(binary)
        .args(["check", "-c"])
        .arg(config)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to run sing-box check: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let details = if !stderr.is_empty() { stderr } else { stdout };

        Err(if details.is_empty() {
            format!("sing-box config check failed with {}", output.status)
        } else {
            format!("Config check failed: {details}")
        })
    }
}

#[tauri::command]
fn get_status(state: State<'_, SingBoxState>) -> AppStatus {
    let mut child = state.0.lock().expect("sing-box state mutex poisoned");
    let running = match child.as_mut() {
        Some(process) => match process.try_wait() {
            Ok(Some(_)) => {
                *child = None;
                false
            }
            Ok(None) => true,
            Err(_) => true,
        },
        None => false,
    };

    AppStatus {
        running,
        binary_found: find_file(executable_name()).is_some(),
        config_found: find_file("config.json").is_some(),
    }
}

#[tauri::command]
fn check_config() -> Result<(), String> {
    let binary = binary_path()?;
    let config = config_path()?;
    validate_config(&binary, &config)
}

#[tauri::command]
fn connect(state: State<'_, SingBoxState>) -> Result<(), String> {
    let binary = binary_path()?;
    let config = config_path()?;

    let mut child = state.0.lock().expect("sing-box state mutex poisoned");

    if let Some(process) = child.as_mut() {
        if process.try_wait().map_err(|e| e.to_string())?.is_none() {
            return Ok(());
        }
    }
    *child = None;

    validate_config(&binary, &config)?;

    let process = Command::new(&binary)
        .args(["run", "-c"])
        .arg(&config)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to start sing-box: {e}"))?;

    *child = Some(process);
    Ok(())
}

#[tauri::command]
fn disconnect(state: State<'_, SingBoxState>) -> Result<(), String> {
    let mut child = state.0.lock().expect("sing-box state mutex poisoned");

    if let Some(mut process) = child.take() {
        if process.try_wait().map_err(|e| e.to_string())?.is_none() {
            process
                .kill()
                .map_err(|e| format!("Failed to stop sing-box: {e}"))?;
            let _ = process.wait();
        }
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(SingBoxState(Mutex::new(None)))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_status,
            check_config,
            connect,
            disconnect
        ])
        .build(tauri::generate_context!())
        .expect("error while building PumatVPN")
        .run(|app, event| {
            if matches!(event, RunEvent::Exit) {
                if let Some(state) = app.try_state::<SingBoxState>() {
                    if let Ok(mut child) = state.0.lock() {
                        if let Some(mut process) = child.take() {
                            let _ = process.kill();
                            let _ = process.wait();
                        }
                    }
                }
            }
        });
}
