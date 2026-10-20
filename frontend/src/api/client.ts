import type { Scan, ScanRequest } from '../types'

/**
 * API contract proposed for the Rust backend (GEO-6).
 *
 *   POST /api/scans            -> { scanId }
 *   GET  /api/scans/{scanId}   -> Scan
 *   GET  /api/scans            -> Scan[]
 *
 * Requests are relative so the Vite dev server proxy (see vite.config.ts)
 * forwards them to the backend on port 8080 without CORS handling.
 * Set VITE_API_BASE to point at a deployed backend directly.
 */

const API_BASE = (import.meta.env.VITE_API_BASE as string | undefined) ?? ''

const POLL_INTERVAL_MS = 1500
const POLL_TIMEOUT_MS = 120_000

export class ApiError extends Error {
  status?: number

  constructor(message: string, status?: number) {
    super(message)
    this.name = 'ApiError'
    this.status = status
  }
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  let res: Response
  try {
    res = await fetch(`${API_BASE}${path}`, {
      headers: { 'Content-Type': 'application/json' },
      ...init,
    })
  } catch {
    throw new ApiError(
      'Could not reach the scan backend. Is it running on port 8080?',
    )
  }

  if (!res.ok) {
    let detail = res.statusText
    try {
      const payload = await res.json()
      if (payload?.error) detail = payload.error
    } catch {
      // Non-JSON error body: keep statusText.
    }
    throw new ApiError(detail, res.status)
  }

  return (await res.json()) as T
}

export function startScan(payload: ScanRequest): Promise<{ scanId: string }> {
  return request('/api/scans', { method: 'POST', body: JSON.stringify(payload) })
}

export function getScan(scanId: string): Promise<Scan> {
  return request(`/api/scans/${encodeURIComponent(scanId)}`)
}

export function listScans(): Promise<Scan[]> {
  return request('/api/scans')
}

export async function pollScan(
  scanId: string,
  onUpdate: (scan: Scan) => void,
  signal?: AbortSignal,
): Promise<Scan> {
  const deadline = Date.now() + POLL_TIMEOUT_MS

  for (;;) {
    if (signal?.aborted) throw new ApiError('Scan polling aborted')

    const scan = await getScan(scanId)
    onUpdate(scan)

    if (scan.status === 'completed' || scan.status === 'failed') return scan
    if (Date.now() > deadline) throw new ApiError('Timed out waiting for scan to finish')

    await new Promise((resolve) => setTimeout(resolve, POLL_INTERVAL_MS))
  }
}
