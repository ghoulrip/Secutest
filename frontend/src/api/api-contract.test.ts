/**
 * Black-box API contract integration tests.
 *
 * These drive the real ./client.ts against an in-memory fetch shim so no
 * network is touched. They exercise the full start -> poll -> resolve lifecycle,
 * not just individual functions.
 *
 * Contract:
 *   POST /api/scans            -> { scanId }
 *   GET  /api/scans/{scanId}   -> Scan
 *   GET  /api/scans            -> Scan[]
 *   error body shape           -> { error: string }
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { ApiError, startScan, getScan, listScans, pollScan, type ScanStatus } from './client'

/**
 * Build an in-memory backend whose canned responses are served in order for the
 * matched request path. A single test scripts the exact sequence of calls a real
 * backend drives across a scan lifecycle.
 */
function fakeBackend() {
  const queue: { path: string; status: number; body: unknown; headers?: Headers }[] = []
  const calls: { path: string; method?: string; body?: unknown }[] = []

  const fetchMock = vi.fn(async (pathOrUrl: string, init: RequestInit = {}) => {
    const path = pathOrUrl.replace(/^https?:\/\/[^/]+/, '')
    const entry = queue.shift()
    if (!entry) throw new Error('Unexpected backend call to ' + path)
    calls.push({ path, method: init.method, body: init.body ? JSON.parse(init.body as string) : undefined })
    const ok = entry.status >= 200 && entry.status < 300
    const headers = new Headers({ 'Content-Type': 'application/json', ...(entry.headers ?? {}) })
    return {
      ok,
      status: entry.status,
      statusText: 'ok',
      headers,
      json: async () => entry.body,
    } as Response
  })

  return { fetchMock, queue, calls }
}

/**
 * Resolve the poll loop's setTimeout immediately so multiple poll iterations run
 * on microtasks rather than blocking on the 1500ms real interval.
 */
async function withFastPoll<T>(fn: () => Promise<T>): Promise<T> {
  const realSetTimeout = globalThis.setTimeout
  globalThis.setTimeout = ((resolve: () => void) => {
    Promise.resolve().then(resolve)
    return 0
  }) as unknown as typeof globalThis.setTimeout
  try {
    return await fn()
  } finally {
    globalThis.setTimeout = realSetTimeout
  }
}

describe('API contract integration', () => {
  beforeEach(() => {
    vi.restoreAllMocks()
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('serves POST /api/scans -> { scanId } and echoes the request body', async () => {
    const { fetchMock, queue, calls } = fakeBackend()
    queue.push({ path: '/api/scans', status: 201, body: { scanId: 'scan-abc' } })
    vi.stubGlobal('fetch', fetchMock)

    const { scanId } = await startScan({ url: 'https://example.com', options: { maxDepth: 3 } })

    expect(scanId).toBe('scan-abc')
    expect(calls[0]).toEqual({
      path: '/api/scans',
      method: 'POST',
      body: { url: 'https://example.com', options: { maxDepth: 3 } },
    })
  })

  it('drives a complete scan lifecycle: queued -> running -> completed', async () => {
    const { fetchMock, queue } = fakeBackend()
    // First poll returns queued -> running -> completed. (No leading /api/scans
    // entry here: that was the test pushing a bogus 201 `{scanId}` into the poll
    // queue, which pollScan's first getScan() consumed as an undefined status.)
    queue.push({
      path: '/api/scans/s1',
      status: 200,
      body: { scanId: 's1', status: 'queued', progress: 0, startedAt: '2026-10-01T00:00:00Z', findings: [] },
    })
    queue.push({
      path: '/api/scans/s1',
      status: 200,
      body: { scanId: 's1', status: 'running', progress: 40, startedAt: '2026-10-01T00:00:00Z', findings: [] },
    })
    queue.push({
      path: '/api/scans/s1',
      status: 200,
      body: {
        scanId: 's1',
        status: 'completed',
        progress: 100,
        startedAt: '2026-10-01T00:00:00Z',
        completedAt: '2026-10-01T00:02:00Z',
        findings: [],
        summary: { total: 2, critical: 1, high: 1, medium: 0, low: 1, info: 0, riskScore: 71 },
      },
    })
    vi.stubGlobal('fetch', fetchMock)

    const seen: ScanStatus[] = []
    const controller = new AbortController()
    const result = await withFastPoll(() => pollScan('s1', (s) => seen.push(s.status), controller.signal))

    expect(seen).toEqual(['queued', 'running', 'completed'])
    expect(result.status).toBe('completed')
    expect(result.summary?.total).toBe(2)
    expect(result.summary?.riskScore).toBe(71)
    expect(fetchMock).toHaveBeenCalledTimes(3) // 3 polls: queued -> running -> completed
  })

  it('returns the list of scans and issues a plain GET', async () => {
    const { fetchMock, queue, calls } = fakeBackend()
    queue.push({
      path: '/api/scans',
      status: 200,
      body: [
        { scanId: 's1', status: 'completed', findings: [] },
        { scanId: 's2', status: 'failed', findings: [], error: 'boom' },
      ],
    })
    vi.stubGlobal('fetch', fetchMock)

    const scans = await listScans()

    expect(scans.map((s) => s.scanId)).toEqual(['s1', 's2'])
    expect(calls[0].path).toBe('/api/scans')
    expect(calls[0].method).toBeUndefined()
  })

  it('throws an ApiError carrying status 404 when a scan is not found', async () => {
    const { fetchMock, queue, calls } = fakeBackend()
    queue.push({ path: '/api/scans/ghost', status: 404, body: { error: 'not found' } })
    vi.stubGlobal('fetch', fetchMock)

    const thrown = await getScan('ghost').catch((e) => e as ApiError)

    expect(thrown).toBeInstanceOf(ApiError)
    expect(thrown.message).toBe('not found')
    expect(thrown.status).toBe(404)
    expect(calls[0]).toEqual({ path: '/api/scans/ghost', method: undefined, body: undefined })
  })

  it('percent-encodes scan ids so non-trivial ids are URL-safe', async () => {
    const { fetchMock, queue, calls } = fakeBackend()
    queue.push({ path: '/api/scans/has', status: 200, body: { scanId: 'has', status: 'completed', findings: [] } })
    vi.stubGlobal('fetch', fetchMock)

    await getScan('has 2/scope')

    expect(calls[0].path).toBe('/api/scans/has%202%2Fscope')
  })

  it('rejects the scan if the backend returns a non-2xx during polling', async () => {
    const { fetchMock, queue } = fakeBackend()
    queue.push({ path: '/api/scans', status: 201, body: { scanId: 's2' } })
    queue.push({ path: '/api/scans/s2', status: 500, body: { error: 'scanner crashed' } })
    vi.stubGlobal('fetch', fetchMock)

    const controller = new AbortController()
    const outcome = await withFastPoll(() => pollScan('s2', () => {}, controller.signal)).then(
      () => 'RESOLVED',
      () => 'REJECTED',
    )

    expect(outcome).toBe('REJECTED')
  })
})
