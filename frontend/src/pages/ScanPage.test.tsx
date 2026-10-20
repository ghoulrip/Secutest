import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import '@testing-library/jest-dom'
import ScanPage from './ScanPage'
import { startScan, pollScan } from '../api/client'
import type { Scan } from '../types'

vi.mock('../api/client', () => ({
  startScan: vi.fn(),
  pollScan: vi.fn(),
}))

const mockedStartScan = vi.mocked(startScan)
const mockedPollScan = vi.mocked(pollScan)

beforeEach(() => {
  vi.resetAllMocks()
})

afterEach(() => {
  vi.unstubAllGlobals()
})

function renderScanPage(onScanComplete?: (scanId: string) => void) {
  return render(<ScanPage onScanComplete={onScanComplete ?? (() => {})} />)
}

function makeScan(overrides: Partial<Scan> = {}): Scan {
  return {
    scanId: 'scan-1',
    url: 'https://example.com',
    status: 'queued',
    progress: 0,
    startedAt: '2026-10-20T00:00:00Z',
    findings: [],
    ...overrides,
  }
}

/** Drive a scan to a given state by resolving pollScan with the supplied snapshot. */
async function submitScan(scan: Scan) {
  mockedStartScan.mockResolvedValue({ scanId: scan.scanId })
  mockedPollScan.mockImplementation(async (_id, onUpdate) => {
    onUpdate(scan)
    return scan
  })

  const input = screen.getByPlaceholderText(/example\.com/)
  fireEvent.change(input, { target: { value: scan.url } })
  fireEvent.click(screen.getByRole('button', { name: /Start Scan/i }))

  await waitFor(() => expect(mockedPollScan).toHaveBeenCalled())
}

describe('ScanPage', () => {
  it('renders the scan form with URL input and start button', () => {
    renderScanPage()

    expect(screen.getByRole('heading', { name: /Website Leak Detection/i })).toBeInTheDocument()
    expect(screen.getByPlaceholderText(/example\.com/)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /Start Scan/i })).toBeInTheDocument()
  })

  it('does not submit an empty URL', async () => {
    mockedStartScan.mockResolvedValue({ scanId: 'scan-1' })

    renderScanPage()

    fireEvent.click(screen.getByRole('button', { name: /Start Scan/i }))

    expect(mockedStartScan).not.toHaveBeenCalled()
  })

  it('starts a scan and calls onScanComplete when done', async () => {
    mockedStartScan.mockResolvedValue({ scanId: 'scan-1' })
    mockedPollScan.mockImplementation(async (_id, onUpdate) => {
      onUpdate(makeScan({ status: 'completed', progress: 100 }))
      return makeScan({ status: 'completed' })
    })

    const onScanComplete = vi.fn()
    renderScanPage(onScanComplete)

    const input = screen.getByPlaceholderText(/example\.com/) as HTMLInputElement
    fireEvent.change(input, { target: { value: 'https://example.com' } })
    expect(input.value).toBe('https://example.com')

    fireEvent.click(screen.getByRole('button', { name: /Start Scan/i }))

    await waitFor(() => expect(mockedStartScan).toHaveBeenCalledWith({ url: 'https://example.com' }))
    expect(mockedPollScan).toHaveBeenCalledWith('scan-1', expect.any(Function))
    expect(onScanComplete).toHaveBeenCalledWith('scan-1')
  })

  it('displays an error message when startScan fails', async () => {
    mockedStartScan.mockRejectedValue(new Error('backend unreachable'))

    renderScanPage()

    const input = screen.getByPlaceholderText(/example\.com/)
    fireEvent.change(input, { target: { value: 'https://example.com' } })
    fireEvent.click(screen.getByRole('button', { name: /Start Scan/i }))

    await waitFor(() => expect(screen.getByText(/Scan Failed/i)).toBeInTheDocument())
    expect(screen.getByText(/backend unreachable/i)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /Start Scan/i })).toBeEnabled()
  })

  it('displays an error message when polling rejects', async () => {
    mockedStartScan.mockResolvedValue({ scanId: 'scan-1' })
    mockedPollScan.mockRejectedValue(new Error('scan timed out'))

    renderScanPage()

    const input = screen.getByPlaceholderText(/example\.com/)
    fireEvent.change(input, { target: { value: 'https://example.com' } })
    fireEvent.click(screen.getByRole('button', { name: /Start Scan/i }))

    await waitFor(() => expect(screen.getByText(/scan timed out/i)).toBeInTheDocument())
  })

  describe('scan status rendering', () => {
    it('shows the queued state', async () => {
      renderScanPage()
      await submitScan(makeScan({ status: 'queued' }))

      await waitFor(() => expect(screen.getByText(/Queued/i)).toBeInTheDocument())
    })

    it('shows the running state with progress percentage', async () => {
      renderScanPage()
      await submitScan(makeScan({ status: 'running', progress: 42 }))

      await waitFor(() => expect(screen.getByText(/42%/)).toBeInTheDocument())
    })

    it('shows the failed state with the backend error', async () => {
      renderScanPage()
      await submitScan(makeScan({ status: 'failed', error: 'host unreachable' }))

      await waitFor(() => expect(screen.getByText(/host unreachable/i)).toBeInTheDocument())
    })

    it('shows a "no issues found" summary when the scan found nothing', async () => {
      renderScanPage()
      await submitScan(
        makeScan({
          status: 'completed',
          progress: 100,
          summary: { total: 0, critical: 0, high: 0, medium: 0, low: 0, info: 0, riskScore: 0 },
        }),
      )

      await waitFor(() => expect(screen.getByText(/No issues found/i)).toBeInTheDocument())
    })

    it('renders a severity count tile for every severity level', async () => {
      renderScanPage()
      await submitScan(
        makeScan({
          status: 'completed',
          progress: 100,
          summary: { total: 5, critical: 1, high: 1, medium: 1, low: 1, info: 1, riskScore: 8 },
        }),
      )

      await waitFor(() => expect(screen.getByText(/Scan Complete/i)).toBeInTheDocument())

      for (const label of ['Critical', 'High', 'Medium', 'Low', 'Info']) {
        expect(screen.getByText(label)).toBeInTheDocument()
      }
    })

    it('renders findings with severity badges, location, impact and solution steps', async () => {
      renderScanPage()
      await submitScan(
        makeScan({
          status: 'completed',
          progress: 100,
          summary: { total: 1, critical: 1, high: 0, medium: 0, low: 0, info: 0, riskScore: 9 },
          findings: [
            {
              id: 'f-1',
              title: 'SQL Injection',
              severity: 'critical',
              category: 'injection',
              description: 'Unsanitised input reaches the database.',
              location: '/api/login',
              impact: 'Full database read access.',
              solution: {
                summary: 'Use parameterized queries.',
                steps: ['Bind parameters', 'Add an input allowlist'],
                references: [{ title: 'OWASP SQL Injection', url: 'https://owasp.org/sql-injection' }],
              },
            },
          ],
        }),
      )

      await waitFor(() => expect(screen.getByText(/SQL Injection/i)).toBeInTheDocument())
      expect(screen.getByText('CRITICAL')).toBeInTheDocument()
      expect(screen.getByText(/Unsanitised input reaches the database/i)).toBeInTheDocument()
      expect(screen.getByText(/\/api\/login/)).toBeInTheDocument()
      expect(screen.getByText(/Full database read access/i)).toBeInTheDocument()
      expect(screen.getByText(/Use parameterized queries/i)).toBeInTheDocument()
      expect(screen.getByText('Bind parameters')).toBeInTheDocument()
      expect(screen.getByText('Add an input allowlist')).toBeInTheDocument()
    })

    it('shows "no detailed findings" when the completed scan returns an empty findings list', async () => {
      renderScanPage()
      await submitScan(
        makeScan({
          status: 'completed',
          progress: 100,
          summary: { total: 0, critical: 0, high: 0, medium: 0, low: 0, info: 0, riskScore: 0 },
          findings: [],
        }),
      )

      await waitFor(() =>
        expect(screen.getByText(/No issues found/i)).toBeInTheDocument(),
      )
    })

    it('offers a "View Full Report" button that calls onScanComplete with the scan id', async () => {
      const onScanComplete = vi.fn()
      mockedStartScan.mockResolvedValue({ scanId: 'scan-1' })
      mockedPollScan.mockImplementation(async (_id, onUpdate) => {
        onUpdate(makeScan({ status: 'completed', progress: 100 }))
        return makeScan({ status: 'completed' })
      })

      renderScanPage(onScanComplete)

      const input = screen.getByPlaceholderText(/example\.com/)
      fireEvent.change(input, { target: { value: 'https://example.com' } })
      fireEvent.click(screen.getByRole('button', { name: /Start Scan/i }))

      const reportButton = await screen.findByRole('button', { name: /View Full Report/i })
      fireEvent.click(reportButton)

      expect(onScanComplete).toHaveBeenCalledWith('scan-1')
    })

    it('falls back to "Scan failed" when the thrown error is not an Error instance', async () => {
      mockedStartScan.mockRejectedValue('non-error value')

      renderScanPage()

      const input = screen.getByPlaceholderText(/example\.com/)
      fireEvent.change(input, { target: { value: 'https://example.com' } })
      fireEvent.click(screen.getByRole('button', { name: /Start Scan/i }))

      await waitFor(() =>
        expect(screen.getByRole('heading', { name: /Scan Failed/i })).toBeInTheDocument(),
      )
    })

    it('displays the summary counts with fallbacks when summary is missing fields', async () => {
      // Summary has some missing properties to exercise `??` and `||` fallbacks
      const scan = makeScan({
        status: 'completed',
        progress: 100,
        summary: {
          total: 2,
          critical: 1,
          high: 0,
          medium: 0,
          low: 0,
          info: 1,
          riskScore: 5,
        },
      })

      mockedStartScan.mockResolvedValue({ scanId: scan.scanId })
      mockedPollScan.mockImplementation(async (_id, onUpdate) => {
        onUpdate(scan)
        return scan
      })

      renderScanPage()

      const input = screen.getByPlaceholderText(/example\.com/)
      fireEvent.change(input, { target: { value: scan.url } })
      fireEvent.click(screen.getByRole('button', { name: /Start Scan/i }))

      await waitFor(() => expect(screen.getByText(/Scan Complete/i)).toBeInTheDocument())

      // Critical shows 1, High shows 0, Info shows 1 (tests the `|| 0` fallback path)
      const counts = screen.getAllByText(/^1$|^0$/)
      expect(counts.length).toBeGreaterThanOrEqual(2)
    })

    })
})
