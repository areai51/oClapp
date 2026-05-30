use crate::error::{OclappError, Result};
use crate::server::binary::BinaryFlavor;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::broadcast;
use tokio::time::{interval, Duration};

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub enum ServerState {
    Idle,
    Starting,
    Running,
    Failed,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ServerStatus {
    pub state: ServerState,
    pub loaded_model: Option<String>,
    pub port: u16,
    pub pid: Option<u32>,
    pub last_error: Option<String>,
}

pub struct ServerManager {
    status: ServerStatus,
    child: Option<Child>,
    log_tx: broadcast::Sender<String>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl ServerManager {
    pub fn new() -> Self {
        let (log_tx, _log_rx) = broadcast::channel(100);
        Self {
            status: ServerStatus {
                state: ServerState::Idle,
                loaded_model: None,
                port: 8080,
                pid: None,
                last_error: None,
            },
            child: None,
            log_tx,
            shutdown_tx: None,
        }
    }

    pub fn status(&self) -> &ServerStatus {
        &self.status
    }

    pub fn subscribe_logs(&self) -> broadcast::Receiver<String> {
        self.log_tx.subscribe()
    }

    pub async fn start(
        &mut self,
        binary_path: &std::path::Path,
        flavor: BinaryFlavor,
        args: &[String],
        model_name: &str,
        port: u16,
    ) -> Result<()> {
        if self.status.state == ServerState::Running {
            return Ok(());
        }

        self.status.state = ServerState::Starting;
        self.status.port = port;
        self.status.loaded_model = Some(model_name.to_string());
        self.status.last_error = None;

        let mut cmd = Command::new(binary_path);
        if flavor == BinaryFlavor::LlamaApp {
            cmd.arg("serve");
        }
        cmd.args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            self.status.state = ServerState::Failed;
            self.status.last_error = Some(e.to_string());
            OclappError::Server(format!("Failed to spawn server: {}", e))
        })?;

        self.status.pid = child.id();

        // Spawn stdout/stderr readers
        let log_tx = self.log_tx.clone();
        if let Some(stdout) = child.stdout.take() {
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let _ = log_tx.send(format!("[stdout] {}", line));
                }
            });
        }

        let log_tx = self.log_tx.clone();
        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let _ = log_tx.send(format!("[stderr] {}", line));
                }
            });
        }

        // Health check loop
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);

        let health_url = format!("http://127.0.0.1:{}/v1/models", port);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| OclappError::Network(e))?;

        let mut health_interval = interval(Duration::from_secs(2));
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 30; // 60 seconds max

        loop {
            tokio::select! {
                _ = health_interval.tick() => {
                    attempts += 1;

                    // Check if process died
                    match child.try_wait() {
                        Ok(Some(exit_status)) => {
                            self.status.state = ServerState::Failed;
                            self.status.last_error = Some(format!(
                                "Server process exited prematurely with status: {}",
                                exit_status
                            ));
                            self.child = Some(child);
                            return Err(OclappError::Server(
                                "Server process exited prematurely".to_string()
                            ));
                        }
                        Ok(None) => {
                            // Process still running, check HTTP health
                            match client.get(&health_url).send().await {
                                Ok(response) if response.status().is_success() => {
                                    self.status.state = ServerState::Running;
                                    self.child = Some(child);
                                    return Ok(());
                                }
                                _ => {
                                    if attempts >= MAX_ATTEMPTS {
                                        let _ = child.kill().await;
                                        self.status.state = ServerState::Failed;
                                        self.status.last_error = Some(
                                            "Health check timeout".to_string()
                                        );
                                        self.child = Some(child);
                                        return Err(OclappError::Server(
                                            "Server failed to respond within timeout".to_string()
                                        ));
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            self.status.state = ServerState::Failed;
                            self.status.last_error = Some(e.to_string());
                            self.child = Some(child);
                            return Err(OclappError::Server(format!(
                                "Failed to check process status: {}", e
                            )));
                        }
                    }
                }
                _ = &mut shutdown_rx => {
                    let _ = child.kill().await;
                    self.status.state = ServerState::Idle;
                    self.child = None;
                    return Ok(());
                }
            }
        }
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }

        if let Some(mut child) = self.child.take() {
            let _ = child.kill().await;
            let _ = child.wait().await;
        }

        self.status.state = ServerState::Idle;
        self.status.loaded_model = None;
        self.status.pid = None;
        Ok(())
    }

    pub async fn check_health(&mut self,
    ) -> Result<()> {
        if self.status.state != ServerState::Running {
            return Ok(());
        }

        if let Some(ref mut child) = self.child {
            match child.try_wait() {
                Ok(Some(_status)) => {
                    self.status.state = ServerState::Failed;
                    self.status.last_error = Some("Server process exited unexpectedly".to_string());
                    self.child = None;
                }
                Ok(None) => {}
                Err(e) => {
                    self.status.state = ServerState::Failed;
                    self.status.last_error = Some(e.to_string());
                    self.child = None;
                }
            }
        }

        Ok(())
    }
}

impl Default for ServerManager {
    fn default() -> Self {
        Self::new()
    }
}
