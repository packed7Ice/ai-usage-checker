import { useEffect, useState } from "react";
import { tauriService } from "./services/tauriService";
import type { UsageSummary, FiveHourBlock, WeeklySummary, DaySummary } from "./types/usage";
import "./App.css";

function App() {
  const [summary, setSummary] = useState<UsageSummary | null>(null);
  const [blocks, setBlocks] = useState<FiveHourBlock[]>([]);
  const [weekly, setWeekly] = useState<WeeklySummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<"blocks" | "weekly">("blocks");

  const loadData = async () => {
    setLoading(true);
    try {
      const [s, b, w] = await Promise.all([
        tauriService.getUsageSummary(),
        tauriService.getFiveHourBlocks("all", 7),
        tauriService.getWeeklySummary(),
      ]);
      setSummary(s);
      setBlocks(b);
      setWeekly(w);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadData();
  }, []);

  const handleRefresh = async () => {
    setLoading(true);
    try {
      await tauriService.refreshData();
      await loadData();
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  };

  const formatDate = (ts: number) => {
    const d = new Date(ts * 1000);
    return d.toLocaleString();
  };

  return (
    <main className="container">
      <header className="header">
        <h1>AI CLI Usage Tracker</h1>
        <button onClick={handleRefresh} disabled={loading}>
          {loading ? "Loading..." : "Refresh"}
        </button>
      </header>

      <section className="summary">
        <h2>Usage Summary</h2>
        {summary ? (
          <div className="cards">
            <div className="card">
              <span className="label">Input Tokens</span>
              <span className="value">{summary.total_input.toLocaleString()}</span>
            </div>
            <div className="card">
              <span className="label">Output Tokens</span>
              <span className="value">{summary.total_output.toLocaleString()}</span>
            </div>
            <div className="card">
              <span className="label">Cache Tokens</span>
              <span className="value">{summary.total_cache.toLocaleString()}</span>
            </div>
            <div className="card">
              <span className="label">Est. Cost</span>
              <span className="value">${summary.total_cost_usd.toFixed(2)}</span>
            </div>
          </div>
        ) : (
          <p>No data available yet.</p>
        )}
      </section>

      <div className="tabs">
        <button
          className={activeTab === "blocks" ? "active" : ""}
          onClick={() => setActiveTab("blocks")}
        >
          5-Hour Blocks
        </button>
        <button
          className={activeTab === "weekly" ? "active" : ""}
          onClick={() => setActiveTab("weekly")}
        >
          Weekly Summary
        </button>
      </div>

      {activeTab === "blocks" && (
        <section className="data-table">
          <h2>5-Hour Blocks (Last 7 Days)</h2>
          {blocks.length > 0 ? (
            <table>
              <thead>
                <tr>
                  <th>Tool</th>
                  <th>Period Start</th>
                  <th>Input</th>
                  <th>Output</th>
                  <th>Cache</th>
                </tr>
              </thead>
              <tbody>
                {blocks.map((b, i) => (
                  <tr key={i}>
                    <td>{b.tool}</td>
                    <td>{formatDate(b.block_start)}</td>
                    <td>{b.input_tokens.toLocaleString()}</td>
                    <td>{b.output_tokens.toLocaleString()}</td>
                    <td>{b.cache_tokens.toLocaleString()}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          ) : (
            <p>No block data available.</p>
          )}
        </section>
      )}

      {activeTab === "weekly" && weekly && (
        <section className="data-table">
          <h2>Weekly Trend (Last 4 Weeks)</h2>
          {weekly.per_day.length > 0 ? (
            <table>
              <thead>
                <tr>
                  <th>Date</th>
                  <th>Tool</th>
                  <th>Input</th>
                  <th>Output</th>
                </tr>
              </thead>
              <tbody>
                {weekly.per_day.map((day: DaySummary) =>
                  day.tool_breakdown.map((tool, idx) => (
                    <tr key={`${day.date}-${idx}`}>
                      {idx === 0 && (
                        <td rowSpan={day.tool_breakdown.length}>{day.date}</td>
                      )}
                      <td>{tool.tool}</td>
                      <td>{tool.input_tokens.toLocaleString()}</td>
                      <td>{tool.output_tokens.toLocaleString()}</td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          ) : (
            <p>No weekly data available.</p>
          )}
        </section>
      )}
    </main>
  );
}

export default App;
