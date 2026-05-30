use oclapp_core::models::discovery::resolve_model_path;
use oclapp_core::server::lifecycle::ServerManager;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_resolve_model_path_for_huggingface_dir() {
    let dir = TempDir::new().unwrap();
    let hf_dir = dir.path().join("gemma-4b");
    fs::create_dir(&hf_dir).unwrap();
    fs::write(hf_dir.join("config.json"), b"{}").unwrap();

    let path = resolve_model_path(dir.path(), "gemma-4b");
    assert!(path.is_some());
    let p = path.unwrap();
    assert!(p.is_dir());
    assert_eq!(p.file_name().unwrap(), "gemma-4b");
}

#[test]
fn test_resolve_model_path_for_gguf_file() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("model.gguf"), b"").unwrap();

    let path = resolve_model_path(dir.path(), "model");
    assert!(path.is_some());
    assert!(!path.unwrap().is_dir());
}

#[test]
fn test_server_manager_new_idle() {
    let manager = ServerManager::new();
    let status = manager.status();
    assert_eq!(status.state, oclapp_core::server::lifecycle::ServerState::Idle);
    assert!(status.loaded_model.is_none());
    assert_eq!(status.port, 8080);
    assert!(status.pid.is_none());
    assert!(status.last_error.is_none());
}
