import { useState, useEffect } from "react";
import type { Settings } from "../types";
import { loadSettings, saveSettings, pickModelsDir } from "../api/tauri";

export default function SettingsPanel() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [loading, setLoading] = useState(false);
  const [saved, setSaved] = useState(false);
  const [showAdvanced, setShowAdvanced] = useState(false);

  useEffect(() => {
    loadSettings()
      .then((s) => setSettings(s))
      .catch((e) => console.error("Failed to load settings:", e));
  }, []);

  async function handlePickDir() {
    const dir = await pickModelsDir();
    if (dir && settings) {
      setSettings({ ...settings, models_dir: dir });
    }
  }

  async function handleSave() {
    if (!settings) return;
    setLoading(true);
    try {
      await saveSettings(settings);
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (e) {
      console.error("Failed to save settings:", e);
    } finally {
      setLoading(false);
    }
  }

  if (!settings) {
    return <div>Loading settings...</div>;
  }

  return (
    <div className="settings-panel">
      <h2>Settings</h2>

      <div className="setting-group">
        <label>Models Directory</label>
        <div className="dir-picker">
          <input type="text" value={settings.models_dir} readOnly />
          <button onClick={handlePickDir}>Browse...</button>
        </div>
      </div>

      <div className="setting-group">
        <label>Server Port</label>
        <input
          type="number"
          value={settings.server_port}
          onChange={(e) =>
            setSettings({
              ...settings,
              server_port: parseInt(e.target.value, 10),
            })
          }
        />
      </div>

      <h3>Curated Parameters</h3>

      <div className="setting-group">
        <label>Temperature</label>
        <input
          type="number"
          step="0.1"
          min="0"
          max="2"
          value={settings.curated_params.temperature}
          onChange={(e) =>
            setSettings({
              ...settings,
              curated_params: {
                ...settings.curated_params,
                temperature: parseFloat(e.target.value),
              },
            })
          }
        />
      </div>

      <div className="setting-group">
        <label>Top P</label>
        <input
          type="number"
          step="0.05"
          min="0"
          max="1"
          value={settings.curated_params.top_p}
          onChange={(e) =>
            setSettings({
              ...settings,
              curated_params: {
                ...settings.curated_params,
                top_p: parseFloat(e.target.value),
              },
            })
          }
        />
      </div>

      <div className="setting-group">
        <label>Context Size</label>
        <input
          type="number"
          step="512"
          min="512"
          value={settings.curated_params.context_size}
          onChange={(e) =>
            setSettings({
              ...settings,
              curated_params: {
                ...settings.curated_params,
                context_size: parseInt(e.target.value, 10),
              },
            })
          }
        />
      </div>

      <div className="setting-group">
        <label>Max Tokens</label>
        <input
          type="number"
          step="128"
          min="1"
          value={settings.curated_params.max_tokens}
          onChange={(e) =>
            setSettings({
              ...settings,
              curated_params: {
                ...settings.curated_params,
                max_tokens: parseInt(e.target.value, 10),
              },
            })
          }
        />
      </div>

      <div className="setting-group">
        <label>GPU Layers</label>
        <input
          type="number"
          step="1"
          value={settings.curated_params.gpu_layers}
          onChange={(e) =>
            setSettings({
              ...settings,
              curated_params: {
                ...settings.curated_params,
                gpu_layers: parseInt(e.target.value, 10),
              },
            })
          }
        />
        <small>-1 for all layers</small>
      </div>

      <button
        className="toggle-advanced"
        onClick={() => setShowAdvanced(!showAdvanced)}
      >
        {showAdvanced ? "Hide" : "Show"} Advanced Parameters
      </button>

      {showAdvanced && (
        <div className="advanced-params">
          <div className="setting-group">
            <label>Seed</label>
            <input
              type="number"
              value={settings.advanced_params.seed}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  advanced_params: {
                    ...settings.advanced_params,
                    seed: parseInt(e.target.value, 10),
                  },
                })
              }
            />
          </div>

          <div className="setting-group">
            <label>Repeat Penalty</label>
            <input
              type="number"
              step="0.1"
              value={settings.advanced_params.repeat_penalty}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  advanced_params: {
                    ...settings.advanced_params,
                    repeat_penalty: parseFloat(e.target.value),
                  },
                })
              }
            />
          </div>

          <div className="setting-group">
            <label>Frequency Penalty</label>
            <input
              type="number"
              step="0.1"
              value={settings.advanced_params.frequency_penalty}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  advanced_params: {
                    ...settings.advanced_params,
                    frequency_penalty: parseFloat(e.target.value),
                  },
                })
              }
            />
          </div>

          <div className="setting-group">
            <label>Presence Penalty</label>
            <input
              type="number"
              step="0.1"
              value={settings.advanced_params.presence_penalty}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  advanced_params: {
                    ...settings.advanced_params,
                    presence_penalty: parseFloat(e.target.value),
                  },
                })
              }
            />
          </div>

          <div className="setting-group">
            <label>Batch Size</label>
            <input
              type="number"
              step="64"
              value={settings.advanced_params.batch_size}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  advanced_params: {
                    ...settings.advanced_params,
                    batch_size: parseInt(e.target.value, 10),
                  },
                })
              }
            />
          </div>

          <div className="setting-group">
            <label>Threads</label>
            <input
              type="number"
              step="1"
              value={settings.advanced_params.threads}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  advanced_params: {
                    ...settings.advanced_params,
                    threads: parseInt(e.target.value, 10),
                  },
                })
              }
            />
          </div>

          <div className="setting-group checkbox">
            <label>
              <input
                type="checkbox"
                checked={settings.advanced_params.flash_attention}
                onChange={(e) =>
                  setSettings({
                    ...settings,
                    advanced_params: {
                      ...settings.advanced_params,
                      flash_attention: e.target.checked,
                    },
                  })
                }
              />
              Flash Attention
            </label>
          </div>

          <div className="setting-group checkbox">
            <label>
              <input
                type="checkbox"
                checked={settings.advanced_params.mmap}
                onChange={(e) =>
                  setSettings({
                    ...settings,
                    advanced_params: {
                      ...settings.advanced_params,
                      mmap: e.target.checked,
                    },
                  })
                }
              />
              Memory Map (MMAP)
            </label>
          </div>

          <div className="setting-group checkbox">
            <label>
              <input
                type="checkbox"
                checked={settings.advanced_params.mlock}
                onChange={(e) =>
                  setSettings({
                    ...settings,
                    advanced_params: {
                      ...settings.advanced_params,
                      mlock: e.target.checked,
                    },
                  })
                }
              />
              Memory Lock (MLOCK)
            </label>
          </div>
        </div>
      )}

      <div className="settings-actions">
        <button onClick={handleSave} disabled={loading}>
          {loading ? "Saving..." : "Save Settings"}
        </button>
        {saved && <span className="saved-indicator">Saved!</span>}
      </div>
    </div>
  );
}
