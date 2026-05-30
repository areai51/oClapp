import { useState } from "react";
import ModelBrowser from "./components/ModelBrowser";
import DownloadManager from "./components/DownloadManager";
import SettingsPanel from "./components/SettingsPanel";
import ServerStatusPanel from "./components/ServerStatus";
import { downloadModel } from "./api/tauri";
import "./App.css";

type Tab = "models" | "downloads" | "settings" | "server";

interface ActiveDownload {
  repoId: string;
  filename: string;
  totalBytes: number;
  downloadedBytes: number;
}

export default function App() {
  const [activeTab, setActiveTab] = useState<Tab>("models");
  const [activeDownloads, setActiveDownloads] = useState<ActiveDownload[]>([]);

  async function handleDownloadStart(repoId: string, filename: string) {
    setActiveDownloads((prev) => {
      if (prev.some((d) => d.repoId === repoId && d.filename === filename)) {
        return prev;
      }
      return [...prev, { repoId, filename, totalBytes: 0, downloadedBytes: 0 }];
    });
    setActiveTab("downloads");
    try {
      await downloadModel(repoId, filename);
    } catch (e) {
      console.error("Download failed:", e);
      setActiveDownloads((prev) =>
        prev.filter((d) => !(d.repoId === repoId && d.filename === filename))
      );
    }
  }

  return (
    <div className="app">
      <aside className="sidebar">
        <div className="logo">
          <h1>oClapp</h1>
        </div>
        <nav className="tabs">
          <button
            className={activeTab === "models" ? "active" : ""}
            onClick={() => setActiveTab("models")}
          >
            Models
          </button>
          <button
            className={activeTab === "downloads" ? "active" : ""}
            onClick={() => setActiveTab("downloads")}
          >
            Downloads
            {activeDownloads.length > 0 && (
              <span className="badge">{activeDownloads.length}</span>
            )}
          </button>
          <button
            className={activeTab === "server" ? "active" : ""}
            onClick={() => setActiveTab("server")}
          >
            Server
          </button>
          <button
            className={activeTab === "settings" ? "active" : ""}
            onClick={() => setActiveTab("settings")}
          >
            Settings
          </button>
        </nav>
      </aside>

      <main className="content">
        {activeTab === "models" && (
          <ModelBrowser onDownloadStart={handleDownloadStart} />
        )}
        {activeTab === "downloads" && (
          <DownloadManager activeDownloads={activeDownloads} />
        )}
        {activeTab === "server" && <ServerStatusPanel />}
        {activeTab === "settings" && <SettingsPanel />}
      </main>
    </div>
  );
}
