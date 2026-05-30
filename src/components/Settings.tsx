import { useEffect, useState } from "react";
import { tauriService } from "../services/tauriService";

interface Settings {
  claude_code_path: string;
  opencode_path: string;
  gemini_path: string;
  input_cost_per_1k: string;
  output_cost_per_1k: string;
  auto_start: boolean;
}

export default function Settings() {
  const [settings, setSettings] = useState<Settings>({
    claude_code_path: "",
    opencode_path: "",
    gemini_path: "",
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

  const handleSave = async () => {
    await tauriService.setSetting("claude_code_path", settings.claude_code_path);
    await tauriService.setSetting("opencode_path", settings.opencode_path);
    await tauriService.setSetting("gemini_path", settings.gemini_path);
    await tauriService.setSetting("input_cost_per_1k", settings.input_cost_per_1k);
    await tauriService.setSetting("output_cost_per_1k", settings.output_cost_per_1k);
    await tauriService.setSetting("auto_start", settings.auto_start ? "true" : "false");
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
          onChange={(e) => handleChange("auto_start", e.target.checked)}
        />
        <label htmlFor="auto-start">Auto-start on login</label>
      </div>

      <button onClick={handleSave}>Save</button>
      {saved && <span className="saved">Saved!</span>}
    </div>
  );
}
