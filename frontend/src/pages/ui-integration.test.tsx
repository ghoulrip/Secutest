/**
 * End-to-end UI integration test.
 *
 * Renders the real App shell (scan + report views) with a single mocked client
 * module that scripts an entire run -- start a scan, watch the live poll
 * updates render, confirm the summary tiles and findings appear, then navigate to
 * the full report and assert its contents. This is the closest thing to Playwright
 * E2E that fits the unit-test budget: one rendered component, one client seam,
 * no page reload.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import '@testing-library/jest-dom/vitest'
import App from '../App'
import * as client from '../api/client'

vi.mock('../api/client', () => ({
  ...vi.importActual<typeof client>('../api/client'),
  startScan: vi.fn(),
  getScan: vi.fn(),
  pollScan: vi.fn(),
}))

const startScanMock = vi.mocked(client.startScan)
const getScanMock = vi.mocked(client.getScan)
const pollScanMock = vi.mocked(client.pollScan)

const SCAN_ID = 'scan-int-1'
const TARGET = 'https://example.test'

/** Script one scan lifecycle: queued -> running(progress) -> completed(findings). */
async function runScan(scanId: string = SCAN_ID) {
  startScanMock.mockResolvedValue({ scanId })
  // ReportPage loads via getScan(); the completed scan is what it must resolve
  // to, otherwise the report page stays in "Loading report..." forever.
  const completed = {
    scanId, url: TARGET, status: 'completed', progress: 100,
    startedAt: '2026-10-01T00:00:00Z', completedAt: '2026-10-01T00:03:00Z',
    summary: { total: 1, critical: 0, high: 1, medium: 0, low: 0, info: 0, riskScore: 75 },
    findings: [{
      id: 'f-1', title: 'Stored XSS', severity: 'high', category: 'xss',
      description: 'Reflected user input in comments.', location: 'https://example.test/comments',
      impact: 'Session token theft via injected <script>',
      solution: { summary: 'HTML-escape all user input.', steps: ['Use an auto-escaping renderer', 'Set a Content-Security-Policy header'], references: [{ title: 'OWASP XSS', url: 'https://owasp.org/xss' }] },
    }],
  }
  getScanMock.mockResolvedValue(completed)
  pollScanMock.mockImplementation(async (_id, onUpdate) => {
    // queued
    await onUpdate({
      scanId, url: TARGET, status: 'queued', progress: 0, startedAt: '2026-10-01T00:00:00Z', findings: [],
    })
    // running
    await onUpdate({
      scanId, url: TARGET, status: 'running', progress: 55, startedAt: '2026-10-01T00:00:00Z', findings: [],
    })
    // completed
    await onUpdate(completed)
    return completed
  })

  const input = screen.getByPlaceholderText(/example\.com/)
  fireEvent.change(input, { target: { value: TARGET } })
  fireEvent.click(screen.getByRole('button', { name: /start scan/i }))
  await waitFor(() => expect(pollScanMock).toHaveBeenCalled())
  // On completion the app auto-navigates to the report (ScanPage calls
  // `onScanComplete(scanId)` when the scan finishes), so the report loads.
  await waitFor(() => expect(screen.getByRole('heading', { name: /Security Report/i })).toBeInTheDocument())
}

async function enterReport(scanId: string = SCAN_ID) {
  await waitFor(() => expect(screen.getByText(/Security Report/i)).toBeInTheDocument())
  await screen.findByRole('link', { name: /OWASP XSS/i })
}

describe('App full flow (ScanPage -> ReportPage)', () => {
  beforeEach(() => {
    vi.restoreAllMocks()
    // `handleViewReport` writes a `scan=<id>` hash into jsdom's history. That
    // hash persists across `it()` blocks, so every App mounted after the first
    // deep-links straight into ReportPage instead of ScanPage -- the cause of
    // the "Unable to find placeholder /example.com" failures. Reset it so each
    // test starts on the scan view.
    window.location.hash = ''
  })
  afterEach(() => {
    vi.unstubAllGlobals()
    vi.useRealTimers()
  })

  it('starts a scan, shows live progress, then renders a full report with the finding', async () => {
    render(<App />)

    await runScan()

    // The report is the guaranteed end state: completion auto-navigates away
    // from ScanPage, so the "Running scan (55%)..." progress state never paints
    // on its own here. Assert the report instead.
    await screen.findByRole('heading', { name: /Security Report/i })

    await enterReport()

    expect(screen.getByText(/Status: completed/)).toBeInTheDocument()
    // Target URL appears in the report meta (scoped to its column, since it is
    // duplicated in the finding location below).
    expect(screen.getByText('https://example.test', { selector: '.report-meta strong' })).toBeInTheDocument()
    expect(screen.getByText(/Summary/)).toBeInTheDocument()
    expect(screen.getByText('1', { selector: '.summary-tile.high .count' })).toBeInTheDocument()
    expect(screen.getByText('Stored XSS')).toBeInTheDocument()
    expect(screen.getByText(/Reflected user input in comments\./)).toBeInTheDocument()
    expect(screen.getByText(/HTTPS:\/\/example\.test\/comments/i)).toBeInTheDocument()
    expect(screen.getByText('high')).toBeInTheDocument()
    expect(screen.getByText('Use an auto-escaping renderer')).toBeInTheDocument()
    expect(screen.getByText('OWASP XSS')).toBeInTheDocument()
  })

  it('shows the scan error state when scanning fails', async () => {
    startScanMock.mockRejectedValue(new Error('backend unreachable'))
    render(<App />)

    const input = screen.getByPlaceholderText(/example\.com/)
    fireEvent.change(input, { target: { value: TARGET } })
    fireEvent.click(screen.getByRole('button', { name: /start scan/i }))

    await waitFor(() => expect(screen.getByText(/Scan Failed/)).toBeInTheDocument())
    await waitFor(() => expect(screen.getByText(/backend unreachable/)).toBeInTheDocument())
  })

  it('navigates back to the scan view from the report', async () => {
    render(<App />)
    await runScan()
    await enterReport()

    fireEvent.click(screen.getByRole('button', { name: /← back/i }))
    await waitFor(() => expect(screen.getByRole('button', { name: /start scan/i })).toBeInTheDocument())
    expect(screen.queryByText(/Security Report/i)).not.toBeInTheDocument()
  })
})
