import { useState, useEffect, useRef } from "react";
import type { ServerStatus, LocalModel } from "../types";
import {
  startServer,
  stopServer,
  getServerStatus,
  listLocalModels,
  onServerStatusChanged,
  onServerLog,
} from "../api/tauri";

export default function ServerStatusPanel() {
  const [status, setStatus] = useState<ServerStatus>({
    state: "Idle",
    loaded_model: null,
    port: 8080,
    pid: null,
    last_error: null,
  });
  const [localModels, setLocalModels] = useState<LocalModel[]>([]);
  const [selectedModel, setSelectedModel] = useState("");
  const [logs, setLogs] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);
  const logsRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    getServerStatus().then(setStatus).catch(console.error);
    listLocalModels().then(setLocalModels).catch(console.error);

    const unsubStatus = onServerStatusChanged((newStatus) => {
      setStatus(newStatus);
    });

    const unsubLog = onServerLog((line) => {
      setLogs((prev) => {
        const next = [...prev, line];
        if (next.length > 200) next.splice(0, next.length - 200);
        return next;
      });
    });

    return () => {
      unsubStatus.then((f) => f());
      unsubLog.then((f) => f());
    };
  }, []);

  useEffect(() => {
    if (logsRef.current) {
      logsRef.current.scrollTop = logsRef.current.scrollHeight;
    }
  }, [logs]);

  async function handleStart() {
    setError(null);
    try {
      await startServer(selectedModel);
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleStop() {
    setError(null);
    try {
      await stopServer();
    } catch (e) {
      setError(String(e));
    }
  }

  const stateColor =
    status.state === "Running"
      ? "green"
      : status.state === "Failed"
      ? "red"
      : status.state === "Starting"
      ? "orange"
      : "gray";

  return (
    <div className="server-status">
      <h2>Server Status</h2>

      <div className="status-indicator" style={{ color: stateColor }}>
        <span className="status-dot" style={{ backgroundColor: stateColor }} />
        <strong>{status.state}</strong>
        {status.loaded_model && <span> · {status.loaded_model}</span>}
        {status.port && <span> · Port {status.port}</span>}
      </div>

      {error && <div className="error">{error}</div>}
      {status.last_error && (
        <div className="error">Last error: {status.last_error}</div>
      )}

      <div className="server-controls">
        <select
          value={selectedModel}
          onChange={(e) => setSelectedModel(e.target.value)}
        >
          <option value="">Select a model...</option>
          {localModels.map((model) => (
            <option key={model.id} value={model.id}>
              {model.id} ({model.format})
            </option>
          ))}
        </select>

        <button
          onClick={handleStart}
          disabled={status.state === "Running" || status.state === "Starting" || !selectedModel}
        >
          Start Server
        </button>
        <button
          onClick={handleStop}
          disabled={status.state === "Idle" || status.state === "Failed"}
        >
          Stop Server
        </button>
      </div>

      <h3>Logs</h3>
      <div className="log-container" ref={logsRef}>
        {logs.length === 0 && <p className="no-logs">No logs yet.</p>}
        {logs.map((line, i) => (
          <div key={i} className="log-line">
            {line}
          </div>
        ))}
      </div>
    </div>
  );
}
