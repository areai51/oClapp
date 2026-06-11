use crate::error::{OclappError, Result};
use crate::server::{ChatCompletionRequest, ChatCompletionResponse};
use reqwest::Client;
use std::time::Duration;

pub async fn send_chat_completion(
    url: &str,
    request: &ChatCompletionRequest,
) -> Result<ChatCompletionResponse> {
    let client = Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(OclappError::Network)?;

    let response = client
        .post(url)
        .json(request)
        .send()
        .await
        .map_err(OclappError::Network)?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(OclappError::Server(format!("Server error: {}", error_text)));
    }

    let chat_response: ChatCompletionResponse = response
        .json()
        .await
        .map_err(OclappError::Network)?;

    Ok(chat_response)
}