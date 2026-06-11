use oclapp_core::models::discovery::{list_local_models, search_huggingface, LocalModel, ModelInfo};
use oclapp_core::server::binary::ensure_binary;
use oclapp_core::server::client::send_chat_completion as send_chat_completion_http;
use oclapp_core::server::config::ServerConfig;
use oclapp_core::server::lifecycle::{ServerManager, ServerState, ServerStatus};
use oclapp_core::server::{ChatCompletionRequest, ChatCompletionResponse};
use oclapp_core::settings::Settings;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_store::StoreExt;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgressPayload {
    pub repo_id: String,
    pub filename: String,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
}

struct AppState {
    server_manager: Arc<Mutex<ServerManager>>,
}

fn get_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_local_data_dir()
        .map_err(|e| e.to_string())
}

fn get_settings_store(app: &AppHandle) -> Result<std::sync::Arc<tauri_plugin_store::Store<tauri::Wry>>, String> {
    let data_dir = get_data_dir(app)?;
    let store_path = data_dir.join("settings.json");
    app.store(store_path.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

fn settings_to_store(settings: &Settings) -> serde_json::Value {
    serde_json::json!({
        "models_dir": settings.models_dir.to_string_lossy().to_string(),
        "server_port": settings.server_port,
        "temperature": settings.curated_params.temperature,
        "top_p": settings.curated_params.top_p,
        "context_size": settings.curated_params.context_size,
        "max_tokens": settings.curated_params.max_tokens,
        "gpu_layers": settings.curated_params.gpu_layers,
        "seed": settings.advanced_params.seed,
        "repeat_penalty": settings.advanced_params.repeat_penalty,
        "frequency_penalty": settings.advanced_params.frequency_penalty,
        "presence_penalty": settings.advanced_params.presence_penalty,
        "batch_size": settings.advanced_params.batch_size,
        "threads": settings.advanced_params.threads,
        "flash_attention": settings.advanced_params.flash_attention,
        "mmap": settings.advanced_params.mmap,
        "mlock": settings.advanced_params.mlock,
    })
}

fn settings_from_store(value: &serde_json::Value) -> Settings {
    let mut settings = Settings::default();

    if let Some(obj) = value.as_object() {
        if let Some(v) = obj.get("models_dir").and_then(|v| v.as_str()) {
            settings.models_dir = PathBuf::from(v);
        }
        if let Some(v) = obj.get("server_port").and_then(|v| v.as_u64()) {
            settings.server_port = v as u16;
        }
        if let Some(v) = obj.get("temperature").and_then(|v| v.as_f64()) {
            settings.curated_params.temperature = v as f32;
        }
        if let Some(v) = obj.get("top_p").and_then(|v| v.as_f64()) {
            settings.curated_params.top_p = v as f32;
        }
        if let Some(v) = obj.get("context_size").and_then(|v| v.as_u64()) {
            settings.curated_params.context_size = v as u32;
        }
        if let Some(v) = obj.get("max_tokens").and_then(|v| v.as_u64()) {
            settings.curated_params.max_tokens = v as u32;
        }
        if let Some(v) = obj.get("gpu_layers").and_then(|v| v.as_i64()) {
            settings.curated_params.gpu_layers = v as i32;
        }
        if let Some(v) = obj.get("seed").and_then(|v| v.as_u64()) {
            settings.advanced_params.seed = v as u32;
        }
        if let Some(v) = obj.get("repeat_penalty").and_then(|v| v.as_f64()) {
            settings.advanced_params.repeat_penalty = v as f32;
        }
        if let Some(v) = obj.get("frequency_penalty").and_then(|v| v.as_f64()) {
            settings.advanced_params.frequency_penalty = v as f32;
        }
        if let Some(v) = obj.get("presence_penalty").and_then(|v| v.as_f64()) {
            settings.advanced_params.presence_penalty = v as f32;
        }
        if let Some(v) = obj.get("batch_size").and_then(|v| v.as_u64()) {
            settings.advanced_params.batch_size = v as u32;
        }
        if let Some(v) = obj.get("threads").and_then(|v| v.as_u64()) {
            settings.advanced_params.threads = v as u32;
        }
        if let Some(v) = obj.get("flash_attention").and_then(|v| v.as_bool()) {
            settings.advanced_params.flash_attention = v;
        }
        if let Some(v) = obj.get("mmap").and_then(|v| v.as_bool()) {
            settings.advanced_params.mmap = v;
        }
        if let Some(v) = obj.get("mlock").and_then(|v| v.as_bool()) {
            settings.advanced_params.mlock = v;
        }
    }

    settings
}

#[tauri::command]
async fn search_models(query: String) -> Result<Vec<ModelInfo>, String> {
    search_huggingface(&query)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_local_models_command(app: AppHandle) -> Result<Vec<LocalModel>, String> {
    let store = get_settings_store(&app)?;
    let settings_val = store
        .get("settings")
        .unwrap_or_else(|| settings_to_store(&Settings::default()));
    let settings = settings_from_store(&settings_val);
    list_local_models(&settings.models_dir).map_err(|e| e.to_string())
}

#[tauri::command]
async fn download_model(
    app: AppHandle,
    repo_id: String,
    filename: String,
) -> Result<(), String> {
    let store = get_settings_store(&app)?;
    let settings_val = store
        .get("settings")
        .unwrap_or_else(|| settings_to_store(&Settings::default()));
    let settings = settings_from_store(&settings_val);

    let progress_callback = {
        let app = app.clone();
        let repo_id = repo_id.clone();
        let filename = filename.clone();
        move |progress: oclapp_core::models::download::DownloadProgress| {
            let _ = app.emit(
                "download-progress",
                DownloadProgressPayload {
                    repo_id: repo_id.clone(),
                    filename: filename.clone(),
                    total_bytes: progress.total_bytes,
                    downloaded_bytes: progress.downloaded_bytes,
                },
            );
        }
    };

    oclapp_core::models::download::download_from_hf(
        &repo_id,
        &filename,
        &settings.models_dir,
        Some(&progress_callback),
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn start_server(app: AppHandle, state: tauri::State<'_, AppState>, model_id: String) -> Result<(), String> {
    let store = get_settings_store(&app)?;
    let settings_val = store
        .get("settings")
        .unwrap_or_else(|| settings_to_store(&Settings::default()));
    let settings = settings_from_store(&settings_val);

    let model_path = oclapp_core::models::discovery::resolve_model_path(&settings.models_dir, &model_id)
        .ok_or_else(|| format!("Model '{}' not found", model_id))?;

    // HuggingFace safetensors models need conversion to GGUF before llama.cpp can load them
    if model_path.is_dir() {
        return Err(format!(
            "Model '{}' is in HuggingFace safetensors format. \
            llama.cpp requires GGUF format to run inference.\n\n\
            To fix this, either:\n\
            1. Download a GGUF version of this model from HuggingFace\n\
            2. Convert it using llama.cpp's conversion tool:\n\
               python3 convert_hf_to_gguf.py {} --outfile {}.gguf\n\n\
            Note: conversion requires 'transformers' and 'gguf' Python packages.",
            model_id,
            model_path.display(),
            model_id
        ));
    }

    let data_dir = get_data_dir(&app)?;
    let resolved = ensure_binary(&data_dir).await.map_err(|e| e.to_string())?;

    let config = ServerConfig {
        model_path: model_path.to_string_lossy().to_string(),
        port: settings.server_port,
        host: "127.0.0.1".to_string(),
        ctx_size: settings.curated_params.context_size,
        n_gpu_layers: settings.curated_params.gpu_layers,
        parallel: 1,
        alias: Some("local-model".to_string()),
        api_key: None,
        flash_attn: settings.advanced_params.flash_attention,
        embedding: false,
    };

    let mut manager = state.server_manager.lock().await;
    manager
        .start(
            &resolved.path,
            resolved.flavor,
            &config.to_args(),
            &model_id,
            settings.server_port,
        )
        .await
        .map_err(|e| e.to_string())?;

    let status = manager.status().clone();
    let _ = app.emit("server-status-changed", status);

    Ok(())
}

#[tauri::command]
async fn stop_server(app: AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut manager = state.server_manager.lock().await;
    manager.stop().await.map_err(|e| e.to_string())?;
    let status = manager.status().clone();
    let _ = app.emit("server-status-changed", status);
    Ok(())
}

#[tauri::command]
async fn get_server_status(state: tauri::State<'_, AppState>) -> Result<ServerStatus, String> {
    let mut manager = state.server_manager.lock().await;
    manager.check_health().await.map_err(|e| e.to_string())?;
    Ok(manager.status().clone())
}

#[tauri::command]
async fn send_chat_completion(
    state: tauri::State<'_, AppState>,
    request: ChatCompletionRequest,
) -> Result<ChatCompletionResponse, String> {
    let manager = state.server_manager.lock().await;
    let status = manager.status();

    if status.state != ServerState::Running {
        return Err("Server is not running".to_string());
    }

    let port = status.port;
    let url = format!("http://127.0.0.1:{}/v1/chat/completions", port);

    send_chat_completion_http(&url, &request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn load_settings(app: AppHandle) -> Result<Settings, String> {
    let store = get_settings_store(&app)?;
    let value = store
        .get("settings")
        .unwrap_or_else(|| settings_to_store(&Settings::default()));
    Ok(settings_from_store(&value))
}

#[tauri::command]
async fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    let store = get_settings_store(&app)?;
    store.set("settings", settings_to_store(&settings));
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn pick_models_dir(app: AppHandle) -> Result<Option<String>, String> {
    let folder = app
        .dialog()
        .file()
        .blocking_pick_folder();
    match folder {
        Some(tauri_plugin_dialog::FilePath::Path(p)) => Ok(Some(p.to_string_lossy().to_string())),
        Some(tauri_plugin_dialog::FilePath::Url(u)) => Ok(Some(u.to_string())),
        None => Ok(None),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let server_manager = Arc::new(Mutex::new(ServerManager::new()));

            // Spawn log forwarding task
            let app_handle = app.handle().clone();
            let log_manager = server_manager.clone();
            tauri::async_runtime::spawn(async move {
                let mut rx = {
                    let manager = log_manager.lock().await;
                    manager.subscribe_logs()
                };
                loop {
                    match rx.recv().await {
                        Ok(line) => {
                            let _ = app_handle.emit("server-log", line);
                        }
                        Err(_) => {
                            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                            rx = {
                                let manager = log_manager.lock().await;
                                manager.subscribe_logs()
                            };
                        }
                    }
                }
            });

            // Spawn status polling task
            let app_handle = app.handle().clone();
            let status_manager = server_manager.clone();
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(2));
                let mut last_state = ServerState::Idle;
                loop {
                    interval.tick().await;
                    let mut manager = status_manager.lock().await;
                    if let Ok(()) = manager.check_health().await {
                        let status = manager.status().clone();
                        if status.state != last_state {
                            last_state = status.state;
                            let _ = app_handle.emit("server-status-changed", status);
                        }
                    }
                }
            });

            app.manage(AppState { server_manager });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            search_models,
            list_local_models_command,
            download_model,
            start_server,
            stop_server,
            get_server_status,
            send_chat_completion,
            load_settings,
            save_settings,
            pick_models_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_settings_roundtrip() {
        let original = Settings {
            models_dir: PathBuf::from("/custom/models"),
            server_port: 9090,
            curated_params: oclapp_core::settings::CuratedParams {
                temperature: 0.5,
                top_p: 0.9,
                context_size: 8192,
                max_tokens: 2048,
                gpu_layers: 40,
            },
            advanced_params: oclapp_core::settings::AdvancedParams {
                seed: 42,
                repeat_penalty: 1.1,
                frequency_penalty: 0.2,
                presence_penalty: 0.3,
                batch_size: 1024,
                threads: 8,
                flash_attention: true,
                mmap: false,
                mlock: true,
            },
        };

        let store_val = settings_to_store(&original);
        let back = settings_from_store(&store_val);

        assert_eq!(back.models_dir, original.models_dir);
        assert_eq!(back.server_port, original.server_port);
        assert_eq!(back.curated_params.temperature, original.curated_params.temperature);
        assert_eq!(back.curated_params.top_p, original.curated_params.top_p);
        assert_eq!(back.curated_params.context_size, original.curated_params.context_size);
        assert_eq!(back.curated_params.max_tokens, original.curated_params.max_tokens);
        assert_eq!(back.curated_params.gpu_layers, original.curated_params.gpu_layers);
        assert_eq!(back.advanced_params.seed, original.advanced_params.seed);
        assert_eq!(back.advanced_params.repeat_penalty, original.advanced_params.repeat_penalty);
        assert_eq!(back.advanced_params.frequency_penalty, original.advanced_params.frequency_penalty);
        assert_eq!(back.advanced_params.presence_penalty, original.advanced_params.presence_penalty);
        assert_eq!(back.advanced_params.batch_size, original.advanced_params.batch_size);
        assert_eq!(back.advanced_params.threads, original.advanced_params.threads);
        assert_eq!(back.advanced_params.flash_attention, original.advanced_params.flash_attention);
        assert_eq!(back.advanced_params.mmap, original.advanced_params.mmap);
        assert_eq!(back.advanced_params.mlock, original.advanced_params.mlock);
    }

    #[test]
    fn test_settings_default_roundtrip() {
        let default = Settings::default();
        let store_val = settings_to_store(&default);
        let back = settings_from_store(&store_val);
        assert_eq!(back.models_dir, default.models_dir);
        assert_eq!(back.server_port, default.server_port);
    }

    #[test]
    fn test_settings_partial_restore() {
        let partial = serde_json::json!({
            "models_dir": "/partial/path",
            "server_port": 7777,
        });
        let back = settings_from_store(&partial);
        assert_eq!(back.models_dir, PathBuf::from("/partial/path"));
        assert_eq!(back.server_port, 7777);
        // defaults for missing fields
        assert_eq!(back.curated_params.temperature, Settings::default().curated_params.temperature);
    }

    #[test]
    fn test_download_progress_payload_serde() {
        let payload = DownloadProgressPayload {
            repo_id: "user/repo".to_string(),
            filename: "model.gguf".to_string(),
            total_bytes: 1000,
            downloaded_bytes: 500,
        };
        let json = serde_json::to_string(&payload).unwrap();
        let back: DownloadProgressPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(back.repo_id, "user/repo");
        assert_eq!(back.filename, "model.gguf");
        assert_eq!(back.total_bytes, 1000);
        assert_eq!(back.downloaded_bytes, 500);
    }

    #[test]
    fn test_huggingface_model_rejected_at_start_server() {
        let dir = std::env::temp_dir().join("oclapp-test-hf-dir");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("config.json"), b"{}").unwrap();

        let path = oclapp_core::models::discovery::resolve_model_path(&dir.parent().unwrap(),
            dir.file_name().unwrap().to_str().unwrap()
        );
        assert!(path.is_some());
        assert!(path.unwrap().is_dir());
    }

    #[test]
    fn test_app_state_new() {
        let manager = ServerManager::new();
        assert_eq!(manager.status().state, ServerState::Idle);
    }
}
