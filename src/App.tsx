import { useState } from "react";
import type { AnalysisResult, ImportedTrack } from "./bindings";
import { api } from "./lib/api";

function App() {
  const [tracks, setTracks] = useState<ImportedTrack[]>([]);
  const [analysis, setAnalysis] = useState<AnalysisResult[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const importMockTrack = async () => {
    setBusy(true);
    setError(null);
    try {
      const imported = await api.importTracks([
        `C:/music/Demo ${tracks.length + 1}.wav`,
      ]);
      setTracks((prev) => [...prev, ...imported]);
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  };

  const runAnalyze = async () => {
    if (tracks.length === 0) return;
    setBusy(true);
    setError(null);
    try {
      const results = await api.analyzeTracks(tracks.map((t) => t.id));
      setAnalysis(results);
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  };

  return (
    <main className="placeholder">
      <h1>Album Mastering Studio</h1>
      <p className="tagline">Private Windows desktop mastering app.</p>

      <section className="phase-block">
        <h2>Phase 1 — IPC proof</h2>
        <p>
          The Track Master surface is built in Phase 2. This screen verifies
          that the frontend can call Rust-side typed commands and receive
          well-shaped responses.
        </p>

        <div className="actions">
          <button type="button" onClick={importMockTrack} disabled={busy}>
            Import mock track
          </button>
          <button
            type="button"
            onClick={runAnalyze}
            disabled={busy || tracks.length === 0}
          >
            Analyze
          </button>
        </div>

        {error && <p className="err">Error: {error}</p>}

        {tracks.length > 0 && (
          <div className="card">
            <h3>Imported tracks ({tracks.length})</h3>
            <ul>
              {tracks.map((t) => (
                <li key={t.id}>
                  <strong>{t.display_name}</strong>{" "}
                  <span className="hint">({t.source_format})</span>
                </li>
              ))}
            </ul>
          </div>
        )}

        {analysis.length > 0 && (
          <div className="card">
            <h3>Analysis (mock values)</h3>
            <ul>
              {analysis.map((a) => (
                <li key={a.track_id}>
                  <strong>{a.track_id.slice(0, 8)}…</strong>
                  <div className="metering">
                    <span>LUFS {a.lufs_integrated.toFixed(1)}</span>
                    <span>TP {a.true_peak_dbtp.toFixed(2)} dBTP</span>
                    <span>DR {a.dynamic_range_lu.toFixed(1)} LU</span>
                    <span>W {a.stereo_width.toFixed(2)}</span>
                  </div>
                </li>
              ))}
            </ul>
          </div>
        )}
      </section>

      <p className="footer">
        See <code>docs/IMPLEMENTATION_PLAN.md</code> for the phase map.
      </p>
    </main>
  );
}

export default App;
