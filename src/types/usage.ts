export interface FiveHourBlock {
  tool: string;
  block_start: number;
  block_end: number;
  input_tokens: number;
  output_tokens: number;
  cache_tokens: number;
  cost_usd: number;
}

export interface ToolUsage {
  tool: string;
  input_tokens: number;
  output_tokens: number;
  cache_tokens: number;
  cost_usd: number;
}

export interface DaySummary {
  date: string;
  tool_breakdown: ToolUsage[];
}

export interface WeeklySummary {
  week_start: number;
  per_day: DaySummary[];
  total_input: number;
  total_output: number;
  total_cost_usd: number;
}

export interface UsageSummary {
  total_input: number;
  total_output: number;
  total_cache: number;
  total_cost_usd: number;
}
