use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub model_path: String,
    pub port: u16,
    pub host: String,
    pub ctx_size: u32,
    pub n_gpu_layers: i32,
    pub parallel: u32,
    pub alias: Option<String>,
    pub api_key: Option<String>,
    pub flash_attn: bool,
    pub embedding: bool,
}

impl ServerConfig {
    pub fn to_args(&self) -> Vec<String> {
        let mut args = vec![
            "-m".to_string(), self.model_path.clone(),
            "--port".to_string(), self.port.to_string(),
            "--host".to_string(), self.host.clone(),
            "-c".to_string(), self.ctx_size.to_string(),
            "-ngl".to_string(), self.n_gpu_layers.to_string(),
            "-np".to_string(), self.parallel.to_string(),
        ];
        if let Some(alias) = &self.alias {
            args.push("-a".to_string());
            args.push(alias.clone());
        }
        if let Some(api_key) = &self.api_key {
            args.push("--api-key".to_string());
            args.push(api_key.clone());
        }
        if self.flash_attn {
            args.push("-fa".to_string());
        }
        if self.embedding {
            args.push("--embedding".to_string());
        }
        args
    }
}
