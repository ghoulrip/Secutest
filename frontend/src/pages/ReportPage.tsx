import { useEffect, useState } from 'react'
import { getScan } from '../api/client'
import type { Scan } from '../types'
import './ReportPage.css'

export default function ReportPage({ scanId, onBack }: { scanId: string; onBack: () => void }) {
  const [scan, setScan] = useState<Scan | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let cancelled = false
    ;(async () => {
      try {
        const data = await getScan(scanId)
        if (!cancelled) setScan(data)
      } catch (err) {
        if (!cancelled) setError(err instanceof Error ? err.message : 'Failed to load report')
      }
    })()
    return () => { cancelled = true }
  }, [scanId])

  if (error) {
    return (
      <div className="report-page">
        <button className="back-btn" onClick={onBack}>← Back</button>
        <div className="error-state">
          <h3>Failed to load report</h3>
          <p>{error}</p>
        </div>
      </div>
    )
  }

  if (!scan) {
    return (
      <div className="report-page">
        <button className="back-btn" onClick={onBack}>← Back</button>
        <div className="loading-state">Loading report...</div>
      </div>
    )
  }

  return (
    <div className="report-page">
      <button className="back-btn" onClick={onBack}>← Back to scans</button>

      <div className="report-header">
        <h2>📄 Security Report</h2>
        <div className="report-meta">
          <span>Target: <strong>{scan.url}</strong></span>
          <span>Status: {scan.status}</span>
          <span>Started: {new Date(scan.startedAt).toLocaleString()}</span>
          {scan.completedAt && <span>Completed: {new Date(scan.completedAt).toLocaleString()}</span>}
        </div>
      </div>

      {scan.status === 'completed' && scan.summary && (
        <section className="report-section">
          <h3>Summary</h3>
          <div className="summary-grid">
            <div className="summary-tile critical">
              <span className="count">{scan.summary.critical}</span>
              <span className="label">Critical</span>
            </div>
            <div className="summary-tile high">
              <span className="count">{scan.summary.high}</span>
              <span className="label">High</span>
            </div>
            <div className="summary-tile medium">
              <span className="count">{scan.summary.medium}</span>
              <span className="label">Medium</span>
            </div>
            <div className="summary-tile low">
              <span className="count">{scan.summary.low}</span>
              <span className="label">Low</span>
            </div>
            <div className="summary-tile info">
              <span className="count">{scan.summary.info}</span>
              <span className="label">Info</span>
            </div>
          </div>
        </section>
      )}

      <section className="report-section">
        <h3>Findings ({scan.findings?.length || 0})</h3>
        {scan.findings && scan.findings.length > 0 ? (
          <div className="findings-list">
            {scan.findings.map((f) => (
              <div key={f.id} className="finding-card">
                <div className="finding-card-header">
                  <h4>{f.title}</h4>
                  <span className={`severity-badge ${f.severity}`}>{f.severity}</span>
                </div>
                <p className="finding-desc">{f.description}</p>
                <div className="finding-meta">
                  <span className="meta-tag">Category: {f.category}</span>
                  {f.location && <span className="meta-tag">📍 {f.location}</span>}
                  {f.impact && <span className="meta-tag">Impact: {f.impact}</span>}
                </div>
                {f.solution && (
                  <div className="solution-card">
                    <h5>🔧 Solution</h5>
                    <p>{f.solution.summary}</p>
                    {f.solution.steps.length > 0 && (
                      <ol>
                        {f.solution.steps.map((s, i) => <li key={i}>{s}</li>)}
                      </ol>
                    )}
                    {f.solution.references && f.solution.references.length > 0 && (
                      <div className="refs">
                        {f.solution.references.map((r, i) => (
                          <a key={i} href={r.url} target="_blank" rel="noopener noreferrer" className="ref-link">
                            {r.title}
                          </a>
                        ))}
                      </div>
                    )}
                  </div>
                )}
              </div>
            ))}
          </div>
        ) : (
          <p className="no-findings">No detailed findings available.</p>
        )}
      </section>
    </div>
  )
}