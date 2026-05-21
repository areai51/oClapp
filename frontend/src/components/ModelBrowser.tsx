import { useState } from "react";
import type { ModelInfo, LocalModel } from "../types";
import { searchModels, listLocalModels } from "../api/tauri";

interface ModelBrowserProps {
  onDownloadStart?: (repoId: string, filename: string) => void;
}

export default function ModelBrowser({ onDownloadStart }: ModelBrowserProps) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<ModelInfo[]>([]);
  const [localModels, setLocalModels] = useState<LocalModel[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [repoId, setRepoId] = useState("");
  const [filename, setFilename] = useState("");

  async function handleSearch() {
    setLoading(true);
    setError(null);
    try {
      const models = await searchModels(query);
      setResults(models);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  async function refreshLocalModels() {
    try {
      const models = await listLocalModels();
      setLocalModels(models);
    } catch (e) {
      setError(String(e));
    }
  }

  function handleDownload(model: ModelInfo) {
    const fn = model.tags.find((t) => t.endsWith(".gguf"))
      ? `${model.id.split("/").pop() || "model"}.gguf`
      : "";
    if (fn && onDownloadStart) {
      onDownloadStart(model.id, fn);
    }
  }

  function handleDirectDownload() {
    if (repoId && filename && onDownloadStart) {
      onDownloadStart(repoId, filename);
    }
  }

  return (
    <div className="model-browser">
      <h2>Model Browser</h2>

      <div className="search-section">
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search HuggingFace..."
          onKeyDown={(e) => e.key === "Enter" && handleSearch()}
        />
        <button onClick={handleSearch} disabled={loading}>
          {loading ? "Searching..." : "Search"}
        </button>
      </div>

      {error && <div className="error">{error}</div>}

      <div className="results">
        {results.map((model) => (
          <div key={model.id} className="model-card">
            <div className="model-header">
              <strong>{model.name}</strong>
              <span className="author">by {model.author}</span>
            </div>
            <div className="model-meta">
              {model.downloads.toLocaleString()} downloads · {model.likes} likes
            </div>
            <div className="model-tags">
              {model.tags.slice(0, 5).map((tag) => (
                <span key={tag} className="tag">
                  {tag}
                </span>
              ))}
            </div>
            <button onClick={() => handleDownload(model)}>Download</button>
          </div>
        ))}
      </div>

      <hr />

      <h3>Direct Download</h3>
      <div className="direct-download">
        <input
          type="text"
          value={repoId}
          onChange={(e) => setRepoId(e.target.value)}
          placeholder="org/model-name"
        />
        <input
          type="text"
          value={filename}
          onChange={(e) => setFilename(e.target.value)}
          placeholder="filename.gguf"
        />
        <button onClick={handleDirectDownload}>Download</button>
      </div>

      <hr />

      <h3>Local Models</h3>
      <button onClick={refreshLocalModels}>Refresh</button>
      <div className="local-models">
        {localModels.length === 0 && <p>No local models found.</p>}
        {localModels.map((model) => (
          <div key={model.id} className="local-model">
            <span>
              {model.id} ({model.format}) - {(model.size / 1024 / 1024).toFixed(1)}{" "}
              MB
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}
