use oclapp_core::server::config::ServerConfig;

#[test]
fn test_server_config_to_args_basic() {
    let config = ServerConfig {
        model_path: "/models/llama.gguf".to_string(),
        port: 8080,
        host: "127.0.0.1".to_string(),
        ctx_size: 4096,
        n_gpu_layers: 35,
        parallel: 2,
        alias: Some("my-model".to_string()),
        api_key: None,
        flash_attn: false,
        embedding: false,
    };

    let args = config.to_args();
    let args_str: Vec<String> = args.clone();

    assert!(args_str.contains(&"-m".to_string()));
    assert!(args_str.contains(&"/models/llama.gguf".to_string()));
    assert!(args_str.contains(&"--port".to_string()));
    assert!(args_str.contains(&"8080".to_string()));
    assert!(args_str.contains(&"--host".to_string()));
    assert!(args_str.contains(&"127.0.0.1".to_string()));
    assert!(args_str.contains(&"-c".to_string()));
    assert!(args_str.contains(&"4096".to_string()));
    assert!(args_str.contains(&"-ngl".to_string()));
    assert!(args_str.contains(&"35".to_string()));
    assert!(args_str.contains(&"-np".to_string()));
    assert!(args_str.contains(&"2".to_string()));
    assert!(args_str.contains(&"-a".to_string()));
    assert!(args_str.contains(&"my-model".to_string()));

    // api-key should NOT be present
    assert!(!args_str.contains(&"--api-key".to_string()));
    // flash_attn should NOT be present
    assert!(!args_str.contains(&"-fa".to_string()));
    // embedding should NOT be present
    assert!(!args_str.contains(&"--embedding".to_string()));
}

#[test]
fn test_server_config_to_args_with_api_key() {
    let config = ServerConfig {
        model_path: "/models/mistral.gguf".to_string(),
        port: 9090,
        host: "0.0.0.0".to_string(),
        ctx_size: 2048,
        n_gpu_layers: 0,
        parallel: 1,
        alias: None,
        api_key: Some("secret123".to_string()),
        flash_attn: true,
        embedding: true,
    };

    let args = config.to_args();
    let args_str: Vec<String> = args.clone();

    assert!(args_str.contains(&"--api-key".to_string()));
    assert!(args_str.contains(&"secret123".to_string()));
    assert!(args_str.contains(&"-fa".to_string()));
    assert!(args_str.contains(&"--embedding".to_string()));
}

#[test]
fn test_server_config_serde_roundtrip() {
    let config = ServerConfig {
        model_path: "/models/test.gguf".to_string(),
        port: 8080,
        host: "127.0.0.1".to_string(),
        ctx_size: 8192,
        n_gpu_layers: 40,
        parallel: 4,
        alias: Some("alias".to_string()),
        api_key: Some("key".to_string()),
        flash_attn: true,
        embedding: true,
    };

    let json = serde_json::to_string(&config).unwrap();
    let back: ServerConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.model_path, back.model_path);
    assert_eq!(config.port, back.port);
    assert_eq!(config.host, back.host);
    assert_eq!(config.ctx_size, back.ctx_size);
    assert_eq!(config.n_gpu_layers, back.n_gpu_layers);
    assert_eq!(config.parallel, back.parallel);
    assert_eq!(config.alias, back.alias);
    assert_eq!(config.api_key, back.api_key);
    assert_eq!(config.flash_attn, back.flash_attn);
    assert_eq!(config.embedding, back.embedding);
}
