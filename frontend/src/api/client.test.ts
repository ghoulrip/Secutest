import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { startScan, getScan, listScans, pollScan, ApiError } from './client'

/** Build a minimal `Response`-like object for the fetch mock. */
function jsonResponse(body: unknown, init: { ok?: boolean; status?: number; statusText?: string } = {}) {
  return {
    ok: init.ok ?? true,
    status: init.status ?? 200,
    statusText: init.statusText ?? 'OK',
    json: async () => body,
  } as unknown as Response
}

describe('api/client', () => {
  beforeEach(() => {
    vi.restoreAllMocks()
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    vi.useRealTimers()
  })

  describe('startScan', () => {
    it('POSTs the url to /api/scans and returns the scanId', async () => {
      const fetchMock = vi.fn().mockResolvedValue(jsonResponse({ scanId: 'scan-1' }))
      vi.stubGlobal('fetch', fetchMock)

      const result = await startScan({ url: 'https://example.com' })

      expect(result).toEqual({ scanId: 'scan-1' })
      expect(fetchMock).toHaveBeenCalledTimes(1)

      const [url, init] = fetchMock.mock.calls[0]
      expect(url).toBe('/api/scans')
      expect(init.method).toBe('POST')
      expect(JSON.parse(init.body)).toEqual({ url: 'https://example.com' })
    })

    it('throws ApiError with a friendly message when the backend is unreachable', async () => {
      vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new TypeError('Failed to fetch')))

      await expect(startScan({ url: 'https://example.com' })).rejects.toThrow(
        'Could not reach the scan backend. Is it running on port 8080?',
      )
    })

    it('surfaces the backend error payload on a non-2xx response', async () => {
      vi.stubGlobal(
        'fetch',
        vi.fn().mockResolvedValue(
          jsonResponse({ error: 'invalid url' }, { ok: false, status: 400, statusText: 'Bad Request' }),
        ),
      )

      const error = await startScan({ url: 'not-a-url' }).catch((e) => e)

      expect(error).toBeInstanceOf(ApiError)
      expect(error.message).toBe('invalid url')
      expect(error.status).toBe(400)
    })

    it('falls back to statusText when the error body is not JSON', async () => {
      vi.stubGlobal(
        'fetch',
        vi.fn().mockResolvedValue({
          ok: false,
          status: 502,
          statusText: 'Bad Gateway',
          json: async () => {
            throw new Error('not json')
          },
        } as unknown as Response),
      )

      const error = await startScan({ url: 'https://example.com' }).catch((e) => e)

      expect(error).toBeInstanceOf(ApiError)
      expect(error.message).toBe('Bad Gateway')
      expect(error.status).toBe(502)
    })
  })

  describe('getScan', () => {
    it('GETs /api/scans/{scanId}', async () => {
      const fetchMock = vi.fn().mockResolvedValue(
        jsonResponse({
          scanId: 'scan-1',
          url: 'https://example.com',
          status: 'completed',
          progress: 100,
          startedAt: '2026-10-20T00:00:00Z',
          findings: [],
        }),
      )
      vi.stubGlobal('fetch', fetchMock)

      const scan = await getScan('scan-1')

      expect(scan.scanId).toBe('scan-1')
      expect(scan.status).toBe('completed')
      expect(fetchMock.mock.calls[0][0]).toBe('/api/scans/scan-1')
      expect(fetchMock.mock.calls[0][1].method).toBeUndefined()
    })

    it('encodes the scan id so it is safe in a URL', async () => {
      const fetchMock = vi.fn().mockResolvedValue(jsonResponse({ scanId: 'a/b' }))
      vi.stubGlobal('fetch', fetchMock)

      await getScan('a/b')

      expect(fetchMock.mock.calls[0][0]).toBe('/api/scans/a%2Fb')
    })
  })

  describe('listScans', () => {
    it('GETs /api/scans and returns the array', async () => {
      const fetchMock = vi.fn().mockResolvedValue(
        jsonResponse([
          { scanId: '1', status: 'completed' },
          { scanId: '2', status: 'failed' },
        ]),
      )
      vi.stubGlobal('fetch', fetchMock)

      const scans = await listScans()

      expect(scans).toHaveLength(2)
      expect(fetchMock.mock.calls[0][0]).toBe('/api/scans')
    })
  })

  describe('pollScan', () => {
    it('reports every update and resolves when the scan completes', async () => {
      const fetchMock = vi
        .fn()
        .mockResolvedValueOnce(jsonResponse({ scanId: '1', status: 'queued', progress: 0 }))
        .mockResolvedValueOnce(jsonResponse({ scanId: '1', status: 'running', progress: 40 }))
        .mockResolvedValueOnce(
          jsonResponse({
            scanId: '1',
            status: 'completed',
            progress: 100,
            summary: { total: 3, critical: 1, high: 1, medium: 1, low: 0, info: 0 },
            findings: [],
          }),
        )
      vi.stubGlobal('fetch', fetchMock)

      const seen: string[] = []
      const result = await pollScan('1', (scan) => seen.push(scan.status))

      expect(seen).toEqual(['queued', 'running', 'completed'])
      expect(result.status).toBe('completed')
      expect(result.summary?.total).toBe(3)
      expect(fetchMock).toHaveBeenCalledTimes(3)
    })

    it('resolves without further polling when the scan fails', async () => {
      const fetchMock = vi.fn().mockResolvedValue(
        jsonResponse({ scanId: '1', status: 'failed', error: 'target unreachable' }),
      )
      vi.stubGlobal('fetch', fetchMock)

      const result = await pollScan('1', () => {})

      expect(result.status).toBe('failed')
      expect(result.error).toBe('target unreachable')
      expect(fetchMock).toHaveBeenCalledTimes(1)
    })

    it('rejects when the signal is already aborted', async () => {
      const fetchMock = vi.fn()
      vi.stubGlobal('fetch', fetchMock)

      const controller = new AbortController()
      controller.abort()

      await expect(pollScan('1', () => {}, controller.signal)).rejects.toThrow(
        'Scan polling aborted',
      )
      expect(fetchMock).not.toHaveBeenCalled()
    })

    it('propagates API errors raised while polling', async () => {
      vi.stubGlobal(
        'fetch',
        vi.fn().mockResolvedValue(
          jsonResponse({ error: 'scan not found' }, { ok: false, status: 404 }),
        ),
      )

      await expect(pollScan('missing', () => {})).rejects.toThrow('scan not found')
    })

    it('times out when the scan never finishes', async () => {
      vi.useFakeTimers()
      vi.stubGlobal('fetch', vi.fn().mockResolvedValue(jsonResponse({ scanId: '1', status: 'running' })))

      const promise = pollScan('1', () => {})
      const assertion = expect(promise).rejects.toThrow('Timed out waiting for scan to finish')

      await vi.advanceTimersByTimeAsync(200_000)
      await assertion
    })
  })
})
