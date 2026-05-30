use clap::{Parser, Subcommand};
use oclapp_core::models::discovery::{list_local_models, resolve_model_path};
use oclapp_core::server::binary::ensure_binary;
use oclapp_core::server::config::ServerConfig;
use oclapp_core::server::lifecycle::ServerManager;
use oclapp_core::settings::load_settings;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::info;

#[derive(Parser)]
#[command(name = "oClapp")]
#[command(about = "Local LLM inference desktop app powered by llama.cpp")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch a coding agent with a local model
    Launch {
        /// Coding agent to launch (claude, codex, pi, copilot, etc.)
        #[arg(value_name = "TOOL")]
        tool: String,

        /// Model name to use for inference
        #[arg(short, long, value_name = "MODEL")]
        model: String,

        /// Server port (default: 8080)
        #[arg(short, long, value_name = "PORT")]
        port: Option<u16>,
    },

    /// Start the inference server directly
    Serve {
        /// Model name to load
        #[arg(short, long, value_name = "MODEL")]
        model: String,

        /// Server port (default: 8080)
        #[arg(short, long, value_name = "PORT")]
        port: Option<u16>,
    },

    /// Show server status
    Status,

    /// List locally available models
    Models,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Launch { tool, model, port } => {
            if let Err(e) = launch_tool(&tool, &model, port).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Serve { model, port } => {
            if let Err(e) = serve_model(&model, port).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Status => {
            println!("Server status: Not yet implemented");
        }
        Commands::Models => {
            if let Err(e) = list_models().await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn resolve_tool_executable(tool: &str) -> Option<String> {
    match tool.to_lowercase().as_str() {
        "claude" | "claude-code" => Some("claude".to_string()),
        "codex" => Some("codex".to_string()),
        "pi" => Some("pi".to_string()),
        "copilot" => Some("gh".to_string()),
        _ => {
            // Try to use the tool name as-is if it exists in PATH
            Some(tool.to_string())
        }
    }
}

fn get_tool_display_name(tool: &str) -> &str {
    match tool.to_lowercase().as_str() {
        "claude" | "claude-code" => "Claude Code",
        "codex" => "Codex CLI",
        "pi" => "Pi CLI",
        "copilot" => "GitHub Copilot CLI",
        _ => tool,
    }
}

async fn launch_tool(tool: &str, model: &str, port: Option<u16>) -> Result<(), Box<dyn std::error::Error>> {
    let settings = load_settings().await?;
    let models_dir = &settings.models_dir;

    // Verify model exists locally
    let model_path = resolve_model_path(models_dir, model)
        .ok_or_else(|| format!("Model '{}' not found in {}", model, models_dir.display()))?;

    if model_path.is_dir() {
        return Err(format!(
            "Model '{}' is in HuggingFace safetensors format. llama.cpp requires GGUF format.\n\
            Convert it with: python3 convert_hf_to_gguf.py {} --outfile {}.gguf",
            model, model_path.display(), model
        ).into());
    }

    info!("Launching {} with model: {}", get_tool_display_name(tool), model);

    // Ensure llama.cpp binary is available
    let data_dir = dirs::data_local_dir()
        .ok_or("Could not determine local data directory")?
        .join("com.oclapp.desktop");
    let resolved = ensure_binary(&data_dir).await?;

    // Build server config
    let server_port = port.unwrap_or(settings.server_port);
    let config = ServerConfig {
        model_path: model_path.to_string_lossy().to_string(),
        port: server_port,
        host: "127.0.0.1".to_string(),
        ctx_size: settings.curated_params.context_size,
        n_gpu_layers: settings.curated_params.gpu_layers,
        parallel: 1,
        alias: Some("local-model".to_string()),
        api_key: None,
        flash_attn: settings.advanced_params.flash_attention,
        embedding: false,
    };

    // Start server
    let mut manager = ServerManager::new();
    info!("Starting llama.cpp server on port {}...", server_port);
    manager
        .start(
            &resolved.path,
            resolved.flavor,
            &config.to_args(),
            model,
            server_port,
        )
        .await?;

    info!("Server started successfully");

    // Set environment variables for the target tool
    let openai_base_url = format!("http://127.0.0.1:{}/v1", server_port);
    let tool_exe = resolve_tool_executable(tool)
        .ok_or_else(|| format!("Unknown tool: {}", tool))?;

    // Check if tool exists in PATH
    if which::which(&tool_exe).is_err() {
        eprintln!("Warning: '{}' not found in PATH. Make sure it's installed.", tool_exe);
    }

    // Setup shutdown flag
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = shutdown.clone();
    ctrlc::set_handler(move || {
        shutdown_clone.store(true, Ordering::SeqCst);
    })?;

    // Spawn the tool
    info!("Spawning {}...", tool_exe);
    let mut tool_cmd = std::process::Command::new(&tool_exe);
    tool_cmd
        .env("OPENAI_BASE_URL", &openai_base_url)
        .env("OPENAI_API_KEY", "ollama") // llama-server accepts any key by default
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let mut child = tool_cmd.spawn()?;

    // Poll for tool exit or shutdown signal
    loop {
        if shutdown.load(Ordering::SeqCst) {
            info!("Received interrupt signal, shutting down...");
            let _ = child.kill();
            break;
        }

        match child.try_wait()? {
            Some(status) => {
                info!("Tool exited with status: {}", status);
                break;
            }
            None => {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    }

    // Stop server
    manager.stop().await?;
    info!("Server stopped");

    Ok(())
}

async fn serve_model(model: &str, port: Option<u16>) -> Result<(), Box<dyn std::error::Error>> {
    let settings = load_settings().await?;
    let models_dir = &settings.models_dir;

    let model_path = resolve_model_path(models_dir, model)
        .ok_or_else(|| format!("Model '{}' not found in {}", model, models_dir.display()))?;

    if model_path.is_dir() {
        return Err(format!(
            "Model '{}' is in HuggingFace safetensors format. llama.cpp requires GGUF format.\n\
            Convert it with: python3 convert_hf_to_gguf.py {} --outfile {}.gguf",
            model, model_path.display(), model
        ).into());
    }

    let data_dir = dirs::data_local_dir()
        .ok_or("Could not determine local data directory")?
        .join("com.oclapp.desktop");
    let resolved = ensure_binary(&data_dir).await?;

    let server_port = port.unwrap_or(settings.server_port);
    let config = ServerConfig {
        model_path: model_path.to_string_lossy().to_string(),
        port: server_port,
        host: "127.0.0.1".to_string(),
        ctx_size: settings.curated_params.context_size,
        n_gpu_layers: settings.curated_params.gpu_layers,
        parallel: 1,
        alias: Some("local-model".to_string()),
        api_key: None,
        flash_attn: settings.advanced_params.flash_attention,
        embedding: false,
    };

    let mut manager = ServerManager::new();
    info!("Starting llama.cpp server on port {}...", server_port);
    manager
        .start(&resolved.path, resolved.flavor, &config.to_args(), model, server_port)
        .await?;

    println!("Server running at http://127.0.0.1:{}/v1", server_port);
    println!("Press Ctrl+C to stop");

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = shutdown.clone();
    ctrlc::set_handler(move || {
        shutdown_clone.store(true, Ordering::SeqCst);
    })?;

    while !shutdown.load(Ordering::SeqCst) {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    manager.stop().await?;
    println!("Server stopped");

    Ok(())
}

async fn list_models() -> Result<(), Box<dyn std::error::Error>> {
    let settings = load_settings().await?;
    let models = list_local_models(&settings.models_dir)?;

    if models.is_empty() {
        println!("No models found in {}", settings.models_dir.display());
        println!("Use the desktop app to download models from HuggingFace.");
    } else {
        println!("Local models in {}:\n", settings.models_dir.display());
        for model in models {
            let size_mb = model.size as f64 / (1024.0 * 1024.0);
            println!(
                "  {} ({}) - {:.1} MB",
                model.id,
                model.format,
                size_mb
            );
        }
    }

    Ok(())
}
