import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  ModelInfo,
  LocalModel,
  ServerStatus,
  Settings,
  DownloadProgressPayload,
} from "../types";

export async function searchModels(query: string): Promise<ModelInfo[]> {
  return invoke("search_models", { query });
}

export async function listLocalModels(): Promise<LocalModel[]> {
  return invoke("list_local_models_command");
}

export async function downloadModel(
  repoId: string,
  filename: string
): Promise<void> {
  return invoke("download_model", { repoId, filename });
}

export async function startServer(modelId: string): Promise<void> {
  return invoke("start_server", { modelId });
}

export async function stopServer(): Promise<void> {
  return invoke("stop_server");
}

export async function getServerStatus(): Promise<ServerStatus> {
  return invoke("get_server_status");
}

export async function loadSettings(): Promise<Settings> {
  return invoke("load_settings");
}

export async function saveSettings(settings: Settings): Promise<void> {
  return invoke("save_settings", { settings });
}

export async function pickModelsDir(): Promise<string | null> {
  return invoke("pick_models_dir");
}

export function onServerStatusChanged(
  callback: (status: ServerStatus) => void
) {
  return listen<ServerStatus>("server-status-changed", (event) => {
    callback(event.payload);
  });
}

export function onServerLog(callback: (line: string) => void) {
  return listen<string>("server-log", (event) => {
    callback(event.payload);
  });
}

export function onDownloadProgress(
  callback: (progress: DownloadProgressPayload) => void
) {
  return listen<DownloadProgressPayload>("download-progress", (event) => {
    callback(event.payload);
  });
}
