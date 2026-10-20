# Frontend ↔ Backend API Contract

This document defines the contract between the Secutest frontend (GEO-5) and the
backend (GEO-6). It is intended as a shared reference until the Rust backend
lands; either side may propose changes by filing a follow-up task.

## Endpoints

### POST /api/scans

Start a new scan.

**Request body:**

```ts
{
  url: string                        // required; website URL to scan
  options?: {
    includeSubdomains?: boolean      // default false
    followRedirects?: boolean        // default true
    maxDepth?: number                // default 1
  }
}
```

**Response 202:**

```ts
{
  scanId: string                     // opaque scan identifier
}
```

### GET /api/scans/{scanId}

Get the status/result of a scan. Poll this endpoint while the scan runs.

**Response 200:**

```ts
interface Scan {
  scanId: string
  url: string
  status: 'queued' | 'running' | 'completed' | 'failed'
  progress: number                   // 0-100
  startedAt: string                  // ISO timestamp
  completedAt?: string               // ISO timestamp
  error?: string                     // set when status === 'failed'
  summary?: {
    total: number
    critical: number
    high: number
    medium: number
    low: number
    info: number
    riskScore: number
  }
  findings?: Finding[]               // populated when status === 'completed'
}
```

### Finding / Solution

```ts
interface Finding {
  id: string
  title: string
  severity: 'critical' | 'high' | 'medium' | 'low' | 'info'
  category: string
  description: string
  location?: string      // e.g. URL/path where the finding was detected
  impact?: string        // human-readable impact statement
  evidence?: string[]
  solution?: Solution
}

interface Solution {
  summary: string
  steps: string[]
  references: { title: string; url: string }[]
}
```

## Frontend behaviour

- `POST /api/scans` → navigate to `/api/scans/{scanId}` report page via URL hash
- `GET /api/scans/{scanId}` → polled every 1.5s until `completed`/`failed` (120s timeout)
- On `failed`: show `error` message
- On `completed`: render summary tiles + findings list with severity badges and solutions
