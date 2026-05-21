use crate::error::{OclappError, Result};
use std::process::Stdio;
use tokio::process::{Child, Command};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ServerState {
    Idle,
    Starting,
    Running,
    Failed,
}

pub struct ServerManager {
    state: ServerState,
    child: Option<Child>,
}

impl ServerManager {
    pub fn new() -> Self {
        Self {
            state: ServerState::Idle,
            child: None,
        }
    }

    pub fn state(&self) -> ServerState {
        self.state
    }

    pub async fn start(&mut self, binary_path: &std::path::Path, args: &[String]) -> Result<()> {
        if self.state == ServerState::Running {
            return Ok(());
        }
        self.state = ServerState::Starting;

        let mut cmd = Command::new(binary_path);
        cmd.args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let child = cmd.spawn().map_err(|e| OclappError::Server(format!("Failed to spawn server: {}", e)))?;
        self.child = Some(child);
        self.state = ServerState::Running;
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
        self.state = ServerState::Idle;
        Ok(())
    }
}

impl Default for ServerManager {
    fn default() -> Self {
        Self::new()
    }
}
