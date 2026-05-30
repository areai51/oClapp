use oclapp_core::models::discovery::{list_local_models, resolve_model_path, ModelFormat};
use std::fs::{self, File};
use std::io::Write;
use tempfile::TempDir;

fn create_file(dir: &TempDir, name: &str, content: &[u8]) {
    let path = dir.path().join(name);
    let mut file = File::create(&path).unwrap();
    file.write_all(content).unwrap();
}

fn create_dir(dir: &TempDir, name: &str) {
    fs::create_dir(dir.path().join(name)).unwrap();
}

#[test]
fn test_list_local_models_empty_dir() {
    let dir = TempDir::new().unwrap();
    let models = list_local_models(dir.path()).unwrap();
    assert!(models.is_empty());
}

#[test]
fn test_list_local_models_missing_dir() {
    let dir = TempDir::new().unwrap();
    let missing = dir.path().join("does-not-exist");
    let models = list_local_models(&missing).unwrap();
    assert!(models.is_empty());
}

#[test]
fn test_list_local_models_gguf_file() {
    let dir = TempDir::new().unwrap();
    create_file(&dir, "llama-2-7b.gguf", b"fake gguf data");
    let models = list_local_models(dir.path()).unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "llama-2-7b");
    assert_eq!(models[0].format, ModelFormat::Gguf);
    assert_eq!(models[0].size, 14);
}

#[test]
fn test_list_local_models_mlx_file() {
    let dir = TempDir::new().unwrap();
    create_file(&dir, "mistral-7b.mlx", b"fake mlx data");
    let models = list_local_models(dir.path()).unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "mistral-7b");
    assert_eq!(models[0].format, ModelFormat::Mlx);
}

#[test]
fn test_list_local_models_safetensors_file() {
    let dir = TempDir::new().unwrap();
    create_file(&dir, "model.safetensors", b"fake safetensors");
    let models = list_local_models(dir.path()).unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "model");
    assert_eq!(models[0].format, ModelFormat::Safetensors);
}

#[test]
fn test_list_local_models_huggingface_dir_with_config() {
    let dir = TempDir::new().unwrap();
    create_dir(&dir, "gemma-4-4b");
    create_file(&dir, "gemma-4-4b/config.json", b"{}");
    create_file(&dir, "gemma-4-4b/model.safetensors", b"fake");

    let models = list_local_models(dir.path()).unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "gemma-4-4b");
    assert_eq!(models[0].format, ModelFormat::Safetensors);
}

#[test]
fn test_list_local_models_huggingface_dir_without_config() {
    let dir = TempDir::new().unwrap();
    create_dir(&dir, "qwen-7b");
    create_file(&dir, "qwen-7b/model.safetensors", b"fake");

    let models = list_local_models(dir.path()).unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "qwen-7b");
}

#[test]
fn test_list_local_models_huggingface_dir_size() {
    let dir = TempDir::new().unwrap();
    create_dir(&dir, "deepseek-67b");
    create_file(&dir, "deepseek-67b/model-00001-of-00003.safetensors", &[0u8; 1000]);
    create_file(&dir, "deepseek-67b/model-00002-of-00003.safetensors", &[0u8; 1000]);
    create_file(&dir, "deepseek-67b/model-00003-of-00003.safetensors", &[0u8; 1000]);

    let models = list_local_models(dir.path()).unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].size, 3000);
}

#[test]
fn test_list_local_models_mixed_formats() {
    let dir = TempDir::new().unwrap();
    create_file(&dir, "model1.gguf", b"a");
    create_file(&dir, "model2.mlx", b"b");
    create_file(&dir, "model3.safetensors", b"c");
    create_dir(&dir, "model4-hf");
    create_file(&dir, "model4-hf/config.json", b"{}");
    create_file(&dir, "model4-hf/model.safetensors", b"d");
    create_file(&dir, "readme.txt", b"not a model");

    let models = list_local_models(dir.path()).unwrap();
    assert_eq!(models.len(), 4);
}

#[test]
fn test_resolve_model_path_gguf() {
    let dir = TempDir::new().unwrap();
    create_file(&dir, "llama.gguf", b"");
    let path = resolve_model_path(dir.path(), "llama");
    assert!(path.is_some());
    assert_eq!(path.unwrap().file_name().unwrap(), "llama.gguf");
}

#[test]
fn test_resolve_model_path_mlx() {
    let dir = TempDir::new().unwrap();
    create_file(&dir, "mistral.mlx", b"");
    let path = resolve_model_path(dir.path(), "mistral");
    assert!(path.is_some());
}

#[test]
fn test_resolve_model_path_safetensors() {
    let dir = TempDir::new().unwrap();
    create_file(&dir, "model.safetensors", b"");
    let path = resolve_model_path(dir.path(), "model");
    assert!(path.is_some());
}

#[test]
fn test_resolve_model_path_huggingface_dir() {
    let dir = TempDir::new().unwrap();
    create_dir(&dir, "gemma-4b");
    create_file(&dir, "gemma-4b/config.json", b"{}");
    let path = resolve_model_path(dir.path(), "gemma-4b");
    assert!(path.is_some());
    assert!(path.unwrap().is_dir());
}

#[test]
fn test_resolve_model_path_missing() {
    let dir = TempDir::new().unwrap();
    let path = resolve_model_path(dir.path(), "nonexistent");
    assert!(path.is_none());
}

#[test]
fn test_model_format_display() {
    assert_eq!(format!("{}", ModelFormat::Gguf), "GGUF");
    assert_eq!(format!("{}", ModelFormat::Mlx), "MLX");
    assert_eq!(format!("{}", ModelFormat::Safetensors), "Safetensors");
    assert_eq!(format!("{}", ModelFormat::Unknown), "Unknown");
}

#[test]
fn test_model_format_serde_roundtrip() {
    let formats = vec![
        ModelFormat::Gguf,
        ModelFormat::Mlx,
        ModelFormat::Safetensors,
        ModelFormat::Unknown,
    ];
    for fmt in formats {
        let json = serde_json::to_string(&fmt).unwrap();
        let back: ModelFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(fmt, back);
    }
}
