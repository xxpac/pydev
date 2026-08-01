use serde::Serialize;
use tauri::{AppHandle, Emitter};

use pydev_core::config::Config;
use pydev_core::installers::python;
use pydev_core::netcheck::{self, EndpointResult};
use pydev_core::pathenv::{self, PathPreview};
use pydev_core::platform;
use pydev_core::progress::{LogLevel, Reporter, Stage};
use pydev_core::status::{self, EnvStatus};
use pydev_core::{orchestrate, Result as CoreResult};

/// Reporter that forwards core log/progress to the webview via events.
#[derive(Clone)]
struct TauriReporter {
    app: AppHandle,
}

#[derive(Serialize, Clone)]
struct LogPayload {
    level: LogLevel,
    message: String,
}

impl Reporter for TauriReporter {
    fn emit_log(&self, level: LogLevel, message: &str) {
        let _ = self.app.emit(
            "install://log",
            LogPayload {
                level,
                message: message.to_string(),
            },
        );
    }

    fn emit_stage(&self, stage: Stage) {
        let _ = self.app.emit("install://progress", stage);
    }
}

/// Run a blocking core routine off the UI thread, wiring up a reporter that
/// streams events, and map any error to a string for the frontend.
async fn run_blocking<T, F>(app: AppHandle, f: F) -> Result<T, String>
where
    F: FnOnce(&dyn Reporter) -> CoreResult<T> + Send + 'static,
    T: Send + 'static,
{
    tauri::async_runtime::spawn_blocking(move || {
        let reporter = TauriReporter { app };
        f(&reporter)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

fn config_path() -> std::path::PathBuf {
    pydev_core::config::default_config_path()
}

#[tauri::command]
fn detect_platform() -> serde_json::Value {
    serde_json::json!({ "os": platform::os_key(), "arch": platform::arch() })
}

#[tauri::command]
fn get_config() -> Config {
    Config::load_or_default(&config_path())
}

#[tauri::command]
fn save_config(config: Config) -> Result<(), String> {
    config.save_to_path(&config_path()).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_python_versions(config: Config) -> Vec<String> {
    python::list_versions(&config)
}

#[tauri::command]
async fn detect_status(config: Config, check_latest: bool) -> Result<EnvStatus, String> {
    tauri::async_runtime::spawn_blocking(move || status::detect(&config, check_latest))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn path_preview(config: Config) -> Result<PathPreview, String> {
    pathenv::preview(&config).map_err(|e| e.to_string())
}

#[tauri::command]
async fn test_network(app: AppHandle, config: Config) -> Result<Vec<EndpointResult>, String> {
    run_blocking(app, move |r| netcheck::run(&config, r)).await
}

#[tauri::command]
async fn one_click(app: AppHandle, config: Config) -> Result<(), String> {
    run_blocking(app, move |r| orchestrate::one_click(&config, r)).await
}

#[tauri::command]
async fn install_uv(app: AppHandle, config: Config) -> Result<(), String> {
    run_blocking(app, move |r| orchestrate::install_uv(&config, r)).await
}

#[tauri::command]
async fn install_python(app: AppHandle, config: Config) -> Result<(), String> {
    run_blocking(app, move |r| orchestrate::install_python(&config, r)).await
}

#[tauri::command]
async fn install_vscode(app: AppHandle, config: Config) -> Result<(), String> {
    run_blocking(app, move |r| orchestrate::install_vscode(&config, r)).await
}

#[tauri::command]
async fn install_extensions(app: AppHandle, config: Config) -> Result<(), String> {
    run_blocking(app, move |r| orchestrate::install_extensions(&config, r)).await
}

#[tauri::command]
async fn apply_path(app: AppHandle, config: Config) -> Result<(), String> {
    run_blocking(app, move |r| orchestrate::apply_path(&config, r)).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            detect_platform,
            get_config,
            save_config,
            list_python_versions,
            detect_status,
            path_preview,
            test_network,
            one_click,
            install_uv,
            install_python,
            install_vscode,
            install_extensions,
            apply_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running the pydev application");
}
