import { useEffect, useState } from "react";
import { tauriService } from "./services/tauriService";
import type { UsageSummary } from "./types/usage";
import "./App.css";

function App() {
  const [summary, setSummary] = useState<UsageSummary | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    tauriService
      .getUsageSummary()
      .then((data) => {
        setSummary(data);
        setLoading(false);
      })
      .catch(() => setLoading(false));
  }, []);

  const handleRefresh = async () => {
    setLoading(true);
    try {
      await tauriService.refreshData();
      const data = await tauriService.getUsageSummary();
      setSummary(data);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
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

      <section className="charts-placeholder">
        <h2>Charts</h2>
        <p>5-hour block chart and weekly trend chart will appear here.</p>
      </section>
    </main>
  );
}

export default App;
