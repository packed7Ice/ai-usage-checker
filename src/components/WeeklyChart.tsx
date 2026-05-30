import type { WeeklySummary } from "../types/usage";
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from "recharts";

interface Props {
  data: WeeklySummary | null;
}

const TOOL_COLORS: Record<string, string> = {
  claude_code: "#cc6b2c",
  opencode: "#007acc",
  gemini: "#34a853",
};

export default function WeeklyChart({ data }: Props) {
  if (!data || data.per_day.length === 0) {
    return <p style={{ color: "#999" }}>No data available.</p>;
  }

  // 日付ごとにツール別トークン数を集計
  const chartData = data.per_day.map((day) => {
    const row: Record<string, number | string> = {
      date: day.date,
    };
    for (const tool of day.tool_breakdown) {
      row[tool.tool] = tool.input_tokens + tool.output_tokens;
    }
    return row;
  });

  // 存在するツールのリストを抽出
  const tools = Array.from(
    new Set(data.per_day.flatMap((d) => d.tool_breakdown.map((t) => t.tool)))
  );

  return (
    <ResponsiveContainer width="100%" height={300}>
      <BarChart data={chartData}>
        <CartesianGrid strokeDasharray="3 3" stroke="#eee" />
        <XAxis dataKey="date" tick={{ fontSize: 11 }} />
        <YAxis tick={{ fontSize: 11 }} />
        <Tooltip
          formatter={(value: any) => (typeof value === "number" ? value.toLocaleString() : value)}
          contentStyle={{ fontSize: "0.85rem" }}
        />
        <Legend wrapperStyle={{ fontSize: "0.85rem" }} />
        {tools.map((tool) => (
          <Bar
            key={tool}
            dataKey={tool}
            stackId="a"
            fill={TOOL_COLORS[tool] || "#8884d8"}
          />
        ))}
      </BarChart>
    </ResponsiveContainer>
  );
}
