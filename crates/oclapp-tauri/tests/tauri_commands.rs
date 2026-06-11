use oclapp_core::models::discovery::resolve_model_path;
use oclapp_core::server::lifecycle::ServerManager;
use oclapp_core::server::client::send_chat_completion as send_chat_completion_http;
use oclapp_core::server::{ChatCompletionRequest, ChatCompletionResponse, ChatMessage, ChatCompletionChoice};
use serde_json::json;
use std::fs;
use tempfile::TempDir;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

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

#[test]
fn test_chat_message_serde() {
    let msg = oclapp_core::server::lifecycle::ChatMessage {
        role: "user".to_string(),
        content: "Hello, world!".to_string(),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: oclapp_core::server::lifecycle::ChatMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(back.role, "user");
    assert_eq!(back.content, "Hello, world!");
}

#[test]
fn test_chat_completion_request_serde() {
    let req = oclapp_core::server::lifecycle::ChatCompletionRequest {
        model: "local-model".to_string(),
        messages: vec![
            oclapp_core::server::lifecycle::ChatMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant".to_string(),
            },
            oclapp_core::server::lifecycle::ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            },
        ],
        temperature: Some(0.7),
        top_p: Some(0.9),
        max_tokens: Some(100),
        stream: Some(false),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: oclapp_core::server::lifecycle::ChatCompletionRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.model, "local-model");
    assert_eq!(back.messages.len(), 2);
    assert_eq!(back.temperature, Some(0.7));
    assert_eq!(back.top_p, Some(0.9));
    assert_eq!(back.max_tokens, Some(100));
    assert_eq!(back.stream, Some(false));
}

#[test]
fn test_chat_completion_response_serde() {
    let resp = oclapp_core::server::lifecycle::ChatCompletionResponse {
        id: "chatcmpl-123".to_string(),
        object: "chat.completion".to_string(),
        created: 1234567890,
        model: "local-model".to_string(),
        choices: vec![oclapp_core::server::lifecycle::ChatCompletionChoice {
            index: 0,
            message: oclapp_core::server::lifecycle::ChatMessage {
                role: "assistant".to_string(),
                content: "Hello! How can I help you?".to_string(),
            },
            finish_reason: Some("stop".to_string()),
        }],
        usage: json!({
            "prompt_tokens": 10,
            "completion_tokens": 20,
            "total_tokens": 30
        }),
    };
    let json = serde_json::to_string(&resp).unwrap();
    let back: oclapp_core::server::lifecycle::ChatCompletionResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id, "chatcmpl-123");
    assert_eq!(back.model, "local-model");
    assert_eq!(back.choices.len(), 1);
    assert_eq!(back.choices[0].message.role, "assistant");
    assert_eq!(back.choices[0].finish_reason, Some("stop".to_string()));
}

#[tokio::test]
async fn test_send_chat_completion_http_success() {
    let mock_server = MockServer::start().await;

    let expected_response = ChatCompletionResponse {
        id: "chatcmpl-test".to_string(),
        object: "chat.completion".to_string(),
        created: 1234567890,
        model: "local-model".to_string(),
        choices: vec![ChatCompletionChoice {
            index: 0,
            message: ChatMessage {
                role: "assistant".to_string(),
                content: "Hello! I'm a test response.".to_string(),
            },
            finish_reason: Some("stop".to_string()),
        }],
        usage: json!({
            "prompt_tokens": 10,
            "completion_tokens": 15,
            "total_tokens": 25
        }),
    };

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&expected_response))
        .mount(&mock_server)
        .await;

    let request = ChatCompletionRequest {
        model: "local-model".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }],
        temperature: Some(0.7),
        top_p: Some(0.9),
        max_tokens: Some(100),
        stream: Some(false),
    };

    let url = format!("{}/v1/chat/completions", mock_server.uri());
    let response = send_chat_completion_http(&url, &request).await;

    assert!(response.is_ok());
    let chat_response = response.unwrap();
    assert_eq!(chat_response.id, "chatcmpl-test");
    assert_eq!(chat_response.choices[0].message.content, "Hello! I'm a test response.");
}

#[tokio::test]
async fn test_send_chat_completion_http_server_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&mock_server)
        .await;

    let request = ChatCompletionRequest {
        model: "local-model".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }],
        temperature: None,
        top_p: None,
        max_tokens: None,
        stream: None,
    };

    let url = format!("{}/v1/chat/completions", mock_server.uri());
    let response = send_chat_completion_http(&url, &request).await;

    assert!(response.is_err());
    assert!(response.unwrap_err().to_string().contains("Server error: Internal Server Error"));
}
