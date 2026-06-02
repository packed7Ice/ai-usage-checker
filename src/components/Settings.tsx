import { useEffect, useState } from "react";
import { tauriService } from "../services/tauriService";

interface Settings {
  claude_code_path: string;
  opencode_path: string;
  gemini_path: string;
  additional_claude_code_paths: string;
  additional_opencode_paths: string;
  additional_gemini_paths: string;
  input_cost_per_1k: string;
  output_cost_per_1k: string;
  auto_start: boolean;
}

export default function Settings() {
  const [settings, setSettings] = useState<Settings>({
    claude_code_path: "",
    opencode_path: "",
    gemini_path: "",
    additional_claude_code_paths: "",
    additional_opencode_paths: "",
    additional_gemini_paths: "",
    input_cost_per_1k: "0.003",
    output_cost_per_1k: "0.015",
    auto_start: false,
  });
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    tauriService.getSettings().then((s) => {
      if (s) setSettings(s);
    });
  }, []);

  const handleChange = (key: keyof Settings, value: string | boolean) => {
    setSettings((prev) => ({ ...prev, [key]: value }));
    setSaved(false);
  };

  const handleAutoStartToggle = async (checked: boolean) => {
    handleChange("auto_start", checked);
    try {
      await tauriService.setAutoStart(checked);
    } catch (e) {
      console.error("Failed to toggle auto-start:", e);
    }
  };

  const handleSave = async () => {
    await tauriService.setSettings(settings);
    setSaved(true);
  };

  return (
    <div className="settings">
      <h2>Settings</h2>

      <div className="field">
        <label>Claude Code Log Path</label>
        <input
          type="text"
          value={settings.claude_code_path}
          onChange={(e) => handleChange("claude_code_path", e.target.value)}
          placeholder="Leave empty for default"
        />
      </div>

      <div className="field">
        <label>Opencode DB Path</label>
        <input
          type="text"
          value={settings.opencode_path}
          onChange={(e) => handleChange("opencode_path", e.target.value)}
          placeholder="Leave empty for default"
        />
      </div>

      <div className="field">
        <label>Gemini CLI Log Path</label>
        <input
          type="text"
          value={settings.gemini_path}
          onChange={(e) => handleChange("gemini_path", e.target.value)}
          placeholder="Leave empty for default"
        />
      </div>

      <div className="field">
        <label>Additional Claude Code Paths (Google Drive synced)</label>
        <textarea
          rows={3}
          value={settings.additional_claude_code_paths}
          onChange={(e) => handleChange("additional_claude_code_paths", e.target.value)}
          placeholder="C:\Users\...\Google Drive\ai-logs\pc-b\.claude&#10;Or comma-separated paths"
        />
        <small>Separate multiple paths with commas or new lines</small>
      </div>

      <div className="field">
        <label>Additional Opencode DB Paths (Google Drive synced)</label>
        <textarea
          rows={3}
          value={settings.additional_opencode_paths}
          onChange={(e) => handleChange("additional_opencode_paths", e.target.value)}
          placeholder="C:\Users\...\Google Drive\ai-logs\pc-b\opencode.db&#10;Or comma-separated paths"
        />
        <small>Separate multiple paths with commas or new lines</small>
      </div>

      <div className="field">
        <label>Additional Gemini CLI Paths (Google Drive synced)</label>
        <textarea
          rows={3}
          value={settings.additional_gemini_paths}
          onChange={(e) => handleChange("additional_gemini_paths", e.target.value)}
          placeholder="C:\Users\...\Google Drive\ai-logs\pc-b\.gemini\tmp&#10;Or comma-separated paths"
        />
        <small>Separate multiple paths with commas or new lines</small>
      </div>

      <div className="field">
        <label>Input Cost per 1K tokens ($)</label>
        <input
          type="text"
          value={settings.input_cost_per_1k}
          onChange={(e) => handleChange("input_cost_per_1k", e.target.value)}
        />
      </div>

      <div className="field">
        <label>Output Cost per 1K tokens ($)</label>
        <input
          type="text"
          value={settings.output_cost_per_1k}
          onChange={(e) => handleChange("output_cost_per_1k", e.target.value)}
        />
      </div>

      <div className="field inline">
        <input
          id="auto-start"
          type="checkbox"
          checked={settings.auto_start}
          onChange={(e) => handleAutoStartToggle(e.target.checked)}
        />
        <label htmlFor="auto-start">Auto-start on login</label>
      </div>

      <button onClick={handleSave}>Save</button>
      {saved && <span className="saved">Saved!</span>}
    </div>
  );
}
