import { useEffect, useState } from "react";
import type { DownloadProgressPayload } from "../types";
import { onDownloadProgress } from "../api/tauri";

interface ActiveDownload {
  repoId: string;
  filename: string;
  totalBytes: number;
  downloadedBytes: number;
}

interface DownloadManagerProps {
  activeDownloads: ActiveDownload[];
}

export default function DownloadManager({ activeDownloads }: DownloadManagerProps) {
  const [downloads, setDownloads] = useState<Map<string, ActiveDownload>>(
    new Map()
  );

  useEffect(() => {
    const unlisten = onDownloadProgress((progress: DownloadProgressPayload) => {
      const key = `${progress.repo_id}:${progress.filename}`;
      setDownloads((prev) => {
        const next = new Map(prev);
        next.set(key, {
          repoId: progress.repo_id,
          filename: progress.filename,
          totalBytes: progress.total_bytes,
          downloadedBytes: progress.downloaded_bytes,
        });
        return next;
      });
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  useEffect(() => {
    setDownloads((prev) => {
      const next = new Map(prev);
      for (const dl of activeDownloads) {
        const key = `${dl.repoId}:${dl.filename}`;
        if (!next.has(key)) {
          next.set(key, dl);
        }
      }
      return next;
    });
  }, [activeDownloads]);

  const downloadList = Array.from(downloads.values());

  if (downloadList.length === 0) {
    return (
      <div className="download-manager">
        <h2>Downloads</h2>
        <p>No active downloads.</p>
      </div>
    );
  }

  return (
    <div className="download-manager">
      <h2>Downloads</h2>
      {downloadList.map((dl) => {
        const progress =
          dl.totalBytes > 0
            ? Math.round((dl.downloadedBytes / dl.totalBytes) * 100)
            : 0;
        return (
          <div key={`${dl.repoId}:${dl.filename}`} className="download-item">
            <div className="download-info">
              <strong>{dl.filename}</strong>
              <span>{dl.repoId}</span>
            </div>
            <div className="progress-bar">
              <div
                className="progress-fill"
                style={{ width: `${progress}%` }}
              />
            </div>
            <div className="progress-text">
              {(dl.downloadedBytes / 1024 / 1024).toFixed(1)} MB /
              {(dl.totalBytes / 1024 / 1024).toFixed(1)} MB ({progress}%)
            </div>
          </div>
        );
      })}
    </div>
  );
}
