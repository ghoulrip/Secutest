/**
 * App-level navigation coverage.
 *
 * Exercises the bits of the shared App shell that the page-level tests never
 * reach: the mount-time hash deep-link (App's deep-link effect) and the nav-bar
 * brand / "Scan" link handlers that route back to the scan view. These are the
 * small branches that would otherwise sit uncovered.
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

describe('App navigation', () => {
  beforeEach(() => {
    vi.restoreAllMocks()
    window.location.hash = ''
  })
  afterEach(() => {
    vi.unstubAllGlobals()
    window.location.hash = ''
  })

  it('deep-links into the report when a hash is present on mount', async () => {
    window.location.hash = 'scan=int-9'
    render(<App />)
    // The deep-link effect routes straight to ReportPage (renders its back
    // button) before any interaction; the heading only appears once getScan
    // resolves, so assert on the back button to prove the route fired.
    await waitFor(() => expect(screen.getByRole('button', { name: /← back/i })).toBeInTheDocument())
  })

  it('routes back to the scan view via the nav-brand / Scan link', async () => {
    window.location.hash = 'scan=int-9'
    render(<App />)
    await waitFor(() => expect(screen.getByRole('button', { name: /← back/i })).toBeInTheDocument())

    // Nav-brand ("🔍 Secutest") is the back-to-scan entry point.
    fireEvent.click(screen.getByRole('button', { name: /🔍 Secutest/i }))
    expect(screen.getByText('🔍 Website Leak Detection')).toBeInTheDocument()
    expect(screen.queryByRole('heading', { name: /Security Report/i })).not.toBeInTheDocument()
  })

  it('routes back to the scan view via the Scan link', async () => {
    window.location.hash = 'scan=int-9'
    render(<App />)
    await waitFor(() => expect(screen.getByRole('button', { name: /← back/i })).toBeInTheDocument())

    fireEvent.click(screen.getByRole('link', { name: /Scan/i }))
    expect(screen.getByText('🔍 Website Leak Detection')).toBeInTheDocument()
    expect(screen.queryByRole('heading', { name: /Security Report/i })).not.toBeInTheDocument()
  })

  it('routes to the report via the Report nav link when clicked', async () => {
    window.location.hash = 'scan=int-9'
    render(<App />)
    await waitFor(() => expect(screen.getByRole('button', { name: /← back/i })).toBeInTheDocument())

    fireEvent.click(screen.getByRole('link', { name: /Report/i }))
    // handleViewReport re-points the hash so deep-link navigation round-trips.
    expect(window.location.hash.replace('#', '')).toBe('scan=int-9')
    await waitFor(() => expect(screen.getByRole('button', { name: /← back/i })).toBeInTheDocument())
  })
})
