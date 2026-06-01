import { invoke } from "@tauri-apps/api/core";
import type {
  FiveHourBlock,
  UsageSummary,
  WeeklySummary,
} from "../types/usage";

export interface AppSettings {
  claude_code_path: string;
  opencode_path: string;
  gemini_path: string;
  input_cost_per_1k: string;
  output_cost_per_1k: string;
  auto_start: boolean;
}

/**
 * Tauri バックエンドへのコマンド呼び出しを抽象化するサービス層
 */
export const tauriService = {
  async greet(name: string): Promise<string> {
    return await invoke("greet", { name });
  },

  async getUsageSummary(): Promise<UsageSummary> {
    return await invoke<UsageSummary>("get_usage_summary");
  },

  async getFiveHourBlocks(
    tool: string = "all",
    days: number = 7
  ): Promise<FiveHourBlock[]> {
    return await invoke<FiveHourBlock[]>("get_five_hour_blocks", {
      tool,
      days,
    });
  },

  async getWeeklySummary(): Promise<WeeklySummary> {
    return await invoke<WeeklySummary>("get_weekly_summary");
  },

  async refreshData(): Promise<void> {
    return await invoke("refresh_data");
  },

  async getSettings(): Promise<AppSettings> {
    return await invoke<AppSettings>("get_settings");
  },

  async setSetting(key: string, value: string): Promise<void> {
    return await invoke("set_setting", { key, value });
  },

  async setSettings(settings: AppSettings): Promise<void> {
    return await invoke("set_settings", { settings });
  },

  async setAutoStart(enable: boolean): Promise<void> {
    const cmd = enable
      ? "plugin:autostart|enable"
      : "plugin:autostart|disable";
    return await invoke(cmd);
  },
};
