import type { FiveHourBlock } from "../types/usage";
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
  data: FiveHourBlock[];
}

export default function FiveHourChart({ data }: Props) {
  if (data.length === 0) {
    return <p style={{ color: "#999" }}>No data available.</p>;
  }

  // 時系列順にソート
  const sorted = [...data].sort((a, b) => a.block_start - b.block_start);

  // ブロックごとに input / output の合計を集計
  const chartData = sorted.map((b) => ({
    label: new Date(b.block_start * 1000).toLocaleDateString(undefined, {
      month: "short",
      day: "numeric",
      hour: "2-digit",
    }),
    Input: b.input_tokens,
    Output: b.output_tokens,
    Cache: b.cache_tokens,
  }));

  return (
    <ResponsiveContainer width="100%" height={300}>
      <BarChart data={chartData}>
        <CartesianGrid strokeDasharray="3 3" stroke="#eee" />
        <XAxis dataKey="label" tick={{ fontSize: 11 }} />
        <YAxis tick={{ fontSize: 11 }} />
        <Tooltip
          formatter={(value: any) => (typeof value === "number" ? value.toLocaleString() : value)}
          contentStyle={{ fontSize: "0.85rem" }}
        />
        <Legend wrapperStyle={{ fontSize: "0.85rem" }} />
        <Bar dataKey="Input" fill="#007acc" />
        <Bar dataKey="Output" fill="#cc6b2c" />
        <Bar dataKey="Cache" fill="#34a853" />
      </BarChart>
    </ResponsiveContainer>
  );
}
