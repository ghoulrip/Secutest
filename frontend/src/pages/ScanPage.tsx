import { useState, useCallback } from 'react'
import { startScan, pollScan } from '../api/client'
import type { Scan } from '../types'
import './ScanPage.css'

export default function ScanPage({ onScanComplete }: { onScanComplete: (scanId: string) => void }) {
  const [url, setUrl] = useState('')
  const [scan, setScan] = useState<Scan | null>(null)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = useCallback(async (e: React.FormEvent) => {
    e.preventDefault()
    if (!url) return

    setIsSubmitting(true)
    setError(null)
    setScan(null)

    try {
      const { scanId } = await startScan({ url })
      await pollScan(scanId, setScan)
      if (onScanComplete) onScanComplete(scanId)
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Scan failed'
      setError(message)
    } finally {
      setIsSubmitting(false)
    }
  }, [url, onScanComplete])

  return (
    <div className="scan-container">
      <div className="scan-header">
        <h2>🔍 Website Leak Detection</h2>
        <p>Enter a website URL to scan for security leaks, vulnerabilities, and misconfigurations.</p>
      </div>

      <form onSubmit={handleSubmit} className="scan-form">
        <div className="input-wrapper">
          <input
            type="url"
            placeholder="https://example.com"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            disabled={isSubmitting}
            className="url-input"
          />
        </div>
        <button type="submit" disabled={isSubmitting} className="scan-btn">
          {isSubmitting ? 'Scanning...' : 'Start Scan'}
        </button>
      </form>

      {error && (
        <div className="error-message">
          <h4>Scan Failed</h4>
          <p>{error}</p>
        </div>
      )}

      {scan && (
        <div className="scan-status">
          {scan.status === 'queued' && <div className="status-item queued">Queued...</div>}
          {scan.status === 'running' && <div className="status-item running">
            <div className="spinner" /> Running scan ({scan.progress}%)...
          </div>}
          {scan.status === 'completed' && <div className="status-item completed">
            <h4>✅ Scan Complete</h4>
            {scan.summary && <ScanSummary summary={scan.summary} />}
            {scan.findings && scan.findings.length > 0 && <FindingsList findings={scan.findings} />}
            {onScanComplete && (
              <button className="view-report-btn" onClick={() => onScanComplete(scan.scanId)}>
                View Full Report
              </button>
            )}
          </div>}
          {scan.status === 'failed' && <div className="status-item failed">
            <h4>❌ Scan Failed</h4>
            <p>{scan.error}</p>
          </div>}
        </div>
      )}

      <div className="scan-tips">
        <h4>💡 Scanning Tips</h4>
        <ul>
          <li>Start with the main domain (e.g., https://example.com)</li>
          <li>Scans may take a few minutes depending on site complexity</li>
          <li>Results include actionable recommendations</li>
        </ul>
      </div>
    </div>
  )
}

function ScanSummary({ summary }: { summary: Scan['summary'] }) {
  const total = summary?.total || 0
  if (total === 0) return <p>No issues found.</p>

  return (
    <div className="summary-grid">
      <div className="summary-item">
        <span className="count critical">{summary?.critical || 0}</span>
        <span className="label">Critical</span>
      </div>
      <div className="summary-item">
        <span className="count high">{summary?.high || 0}</span>
        <span className="label">High</span>
      </div>
      <div className="summary-item">
        <span className="count medium">{summary?.medium || 0}</span>
        <span className="label">Medium</span>
      </div>
      <div className="summary-item">
        <span className="count low">{summary?.low || 0}</span>
        <span className="label">Low</span>
      </div>
      <div className="summary-item">
        <span className="count info">{summary?.info || 0}</span>
        <span className="label">Info</span>
      </div>
    </div>
  )
}

function FindingsList({ findings }: { findings: Scan['findings'] }) {
  if (!findings || findings.length === 0) return <p>No detailed findings available.</p>

  return (
    <div className="findings-list">
      <h4>Findings</h4>
      {findings.map((finding) => (
        <div key={finding.id} className="finding-item">
          <div className="finding-header">
            <h5>{finding.title}</h5>
            <span className={`severity-badge ${finding.severity}`}>{finding.severity.toUpperCase()}</span>
          </div>
          <p className="description">{finding.description}</p>
          <div className="location-impact">
            {finding.location && <span className="location">📍 {finding.location}</span>}
            {finding.impact && <span className="impact">⚡ {finding.impact}</span>}
          </div>
          {finding.solution && (
            <div className="solution">
              <h6>🔧 Solution</h6>
              <p>{finding.solution.summary}</p>
              <ul>
                {finding.solution.steps.map((step, idx) => <li key={idx}>{step}</li>)}
              </ul>
            </div>
          )}
        </div>
      ))}
    </div>
  )
}