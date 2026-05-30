export interface ModelInfo {
  id: string;
  name: string;
  author: string;
  tags: string[];
  downloads: number;
  likes: number;
  last_modified: string;
}

export interface LocalModel {
  id: string;
  path: string;
  size: number;
  format: "Gguf" | "Mlx" | "Safetensors" | "Unknown";
}

export interface ServerStatus {
  state: "Idle" | "Starting" | "Running" | "Failed";
  loaded_model: string | null;
  port: number;
  pid: number | null;
  last_error: string | null;
}

export interface CuratedParams {
  temperature: number;
  top_p: number;
  context_size: number;
  max_tokens: number;
  gpu_layers: number;
}

export interface AdvancedParams {
  seed: number;
  repeat_penalty: number;
  frequency_penalty: number;
  presence_penalty: number;
  batch_size: number;
  threads: number;
  flash_attention: boolean;
  mmap: boolean;
  mlock: boolean;
}

export interface Settings {
  models_dir: string;
  server_port: number;
  curated_params: CuratedParams;
  advanced_params: AdvancedParams;
}

export interface DownloadProgressPayload {
  repo_id: string;
  filename: string;
  total_bytes: number;
  downloaded_bytes: number;
}
