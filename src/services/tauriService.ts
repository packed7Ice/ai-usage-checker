import { invoke } from "@tauri-apps/api/core";
import type {
  FiveHourBlock,
  UsageSummary,
  WeeklySummary,
} from "../types/usage";

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
};
