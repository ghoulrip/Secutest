import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import '@testing-library/jest-dom'
import ReportPage from './ReportPage'
import { getScan } from '../api/client'
import type { Scan } from '../types'

vi.mock('../api/client', () => ({
  getScan: vi.fn(),
}))

const mockedGetScan = vi.mocked(getScan)

beforeEach(() => {
  vi.resetAllMocks()
})

afterEach(() => {
  vi.unstubAllGlobals()
})

function makeScan(overrides: Partial<Scan> = {}): Scan {
  return {
    scanId: 'scan-1',
    url: 'https://example.com',
    status: 'completed',
    progress: 100,
    startedAt: '2026-10-20T00:00:00Z',
    completedAt: '2026-10-20T00:05:00Z',
    findings: [],
    summary: { total: 0, critical: 0, high: 0, medium: 0, low: 0, info: 0, riskScore: 0 },
    ...overrides,
  }
}

function renderReportPage(scanId = 'scan-1', onBack = vi.fn()) {
  return render(<ReportPage scanId={scanId} onBack={onBack} />)
}


describe('ReportPage', () => {
  it('shows loading state initially', () => {
    mockedGetScan.mockImplementation(() => new Promise(() => {})) // never resolves

    renderReportPage()

    expect(screen.getByText(/Loading report/i)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /← Back/i })).toBeInTheDocument()
  })

  it('shows error state when getScan rejects', async () => {
    mockedGetScan.mockRejectedValue(new Error('network down'))

    renderReportPage()

    await waitFor(() => expect(screen.getByText(/Failed to load report/i)).toBeInTheDocument())
    await waitFor(() => expect(screen.getByText(/network down/i)).toBeInTheDocument())
    expect(screen.getByRole('button', { name: /← Back/i })).toBeInTheDocument()
  })

  it('shows error state when thrown error is not an Error instance', async () => {
    mockedGetScan.mockRejectedValue('unknown failure')

    renderReportPage()

    await waitFor(() => expect(screen.getAllByText(/Failed to load report/i).length).toBeGreaterThan(0))
    expect(screen.getByRole('button', { name: /← Back/i })).toBeInTheDocument()
  })

  describe('completed scan rendering', () => {
    beforeEach(() => {
      mockedGetScan.mockResolvedValue(makeScan())
    })

    it('renders report header with url, status and timestamps', async () => {
      renderReportPage()

      await waitFor(() => expect(screen.getByText(/Security Report/i)).toBeInTheDocument())
      expect(screen.getByText(/example\.com/)).toBeInTheDocument()
      expect(screen.getByText(/Status: completed/i)).toBeInTheDocument()
      expect(screen.getByText(/Started:/i)).toBeInTheDocument()
      expect(screen.getByText(/Completed:/i)).toBeInTheDocument()
    })

    it('renders summary tiles with severity counts', async () => {
      mockedGetScan.mockResolvedValue(
        makeScan({
          summary: { total: 6, critical: 2, high: 1, medium: 2, low: 1, info: 0, riskScore: 75 },
        }),
      )

      renderReportPage()

      await waitFor(() => expect(screen.getByText(/Summary/i)).toBeInTheDocument())

      expect(screen.getByText('2', { selector: '.summary-tile.critical .count' })).toBeInTheDocument()
      expect(screen.getByText('1', { selector: '.summary-tile.high .count' })).toBeInTheDocument()
      expect(screen.getByText('2', { selector: '.summary-tile.medium .count' })).toBeInTheDocument()
      expect(screen.getByText('1', { selector: '.summary-tile.low .count' })).toBeInTheDocument()
      expect(screen.getByText('0', { selector: '.summary-tile.info .count' })).toBeInTheDocument()
    })

    it('shows "No detailed findings" when findings list is empty', async () => {
      renderReportPage()

      await waitFor(() => expect(screen.getByText(/No detailed findings available/i)).toBeInTheDocument())
    })

    it('renders findings with severity badges, meta tags, solution and references', async () => {
      mockedGetScan.mockResolvedValue(
        makeScan({
          findings: [
            {
              id: 'f-1',
              title: 'XSS Vulnerability',
              severity: 'high',
              category: 'xss',
              description: 'User input reflected without escaping.',
              location: '/search',
              impact: 'Session hijacking possible.',
              solution: {
                summary: 'Encode output and set CSP.',
                steps: ['Use auto-escaping templates', 'Add Content-Security-Policy header'],
                references: [
                  { title: 'OWASP XSS', url: 'https://owasp.org/xss' },
                  { title: 'MDN CSP', url: 'https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP' },
                ],
              },
            },
          ],
        }),
      )

      renderReportPage()

      await waitFor(() => expect(screen.getByText(/XSS Vulnerability/i)).toBeInTheDocument())

      expect(screen.getByText('high')).toBeInTheDocument()
      expect(screen.getByText(/User input reflected without escaping/i)).toBeInTheDocument()
      expect(screen.getByText(/Category: xss/i)).toBeInTheDocument()
      expect(screen.getByText(/\/search/)).toBeInTheDocument()
      expect(screen.getByText(/Session hijacking possible/i)).toBeInTheDocument()
      expect(screen.getByText(/Encode output and set CSP/i)).toBeInTheDocument()
      expect(screen.getByText('Use auto-escaping templates')).toBeInTheDocument()
      expect(screen.getByText('Add Content-Security-Policy header')).toBeInTheDocument()
      expect(screen.getByRole('link', { name: /OWASP XSS/i })).toHaveAttribute('href', 'https://owasp.org/xss')
      expect(screen.getByRole('link', { name: /MDN CSP/i })).toHaveAttribute('href', 'https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP')
    })

    it('renders multiple findings correctly', async () => {
      mockedGetScan.mockResolvedValue(
        makeScan({
          findings: [
            { id: 'f-1', title: 'Finding 1', severity: 'critical', category: 'cat1', description: 'd1' },
            { id: 'f-2', title: 'Finding 2', severity: 'low', category: 'cat2', description: 'd2' },
          ],
        }),
      )

      renderReportPage()

      await waitFor(() => expect(screen.getByText('Finding 1')).toBeInTheDocument())
      expect(screen.getByText('Finding 2')).toBeInTheDocument()
      expect(screen.getAllByText(/critical|low/i).length).toBeGreaterThanOrEqual(2)
    })

    it('handles findings without optional fields (location, impact, solution)', async () => {
      mockedGetScan.mockResolvedValue(
        makeScan({
          findings: [
            {
              id: 'f-1',
              title: 'Minimal Finding',
              severity: 'medium',
              category: 'test',
              description: 'Just a description.',
            },
          ],
        }),
      )

      renderReportPage()

      await waitFor(() => expect(screen.getByText('Minimal Finding')).toBeInTheDocument())
      expect(screen.getByText('medium')).toBeInTheDocument()
      expect(screen.queryByText(/location/i)).not.toBeInTheDocument()
      expect(screen.queryByText(/impact/i)).not.toBeInTheDocument()
      expect(screen.queryByText(/solution/i)).not.toBeInTheDocument()
    })

    it('calls onBack when back button is clicked', async () => {
      const onBack = vi.fn()
      renderReportPage('scan-1', onBack)

      await waitFor(() => expect(screen.getByRole('button', { name: /← Back/i })).toBeInTheDocument())
      fireEvent.click(screen.getByRole('button', { name: /← Back/i }))

      expect(onBack).toHaveBeenCalledTimes(1)
    })
  })

  describe('non-completed scan states', () => {
    it('does not render summary section for non-completed scans', async () => {
      mockedGetScan.mockResolvedValue(makeScan({ status: 'running', progress: 50 }))

      renderReportPage()

      await waitFor(() => expect(screen.getByText(/Security Report/i)).toBeInTheDocument())
      expect(screen.queryByText(/Summary/i)).not.toBeInTheDocument()
    })

    it('shows completedAt only when available', async () => {
      mockedGetScan.mockResolvedValue(
        makeScan({ status: 'running', completedAt: undefined }),
      )

      renderReportPage()

      await waitFor(() => expect(screen.getByText(/Security Report/i)).toBeInTheDocument())
      expect(screen.queryByText(/Completed:/i)).not.toBeInTheDocument()
    })
  })
})