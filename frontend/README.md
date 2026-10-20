# Secutest Frontend

Web UI for the Secutest website leak-detection app. Built with **React 19**, **TypeScript**, and **Vite**.

## Structure

```
src/
  App.tsx            # App shell + navigation + view switching
  main.tsx           # React entry point
  App.css            # Global design tokens (colors, spacing, radius)
  types.ts           # TS types for scans, findings, solutions, severities
  api/client.ts      # API client contract + pollScan() helper
  pages/ScanPage.tsx # Scan entry form + inline results
  pages/ReportPage.tsx # Full scan report with findings + solutions
```

## API contract (with Backend Developer, GEO-6)

| Method | Path            | Request            | Response         |
|--------|-----------------|--------------------|------------------|
| POST   | `/api/scans`    | `{ url, options? }`| `{ scanId }`     |
| GET    | `/api/scans/:id`| —                  | `Scan`           |
| GET    | `/api/scans`    | —                  | `Scan[]`         |

`GET /api/scans/:id` returns `{ status, summary, findings }`. The frontend polls this endpoint while a scan runs (`client.ts`, `pollScan()`).

## Development

```bash
cd frontend
npm install            # one-time
npm run dev            # dev server on http://localhost:5173
npm run build          # production build to dist/
```

The Vite dev server proxies `/api` to `http://localhost:8080` (see `vite.config.ts`), so no CORS configuration is needed while developing against a local backend.

## Notes

- Until the Rust backend (GEO-6) is ready, the UI will surface a "could not reach the scan backend" error on submit. Point `VITE_API_BASE` at a deployed backend if needed.
- React Native mobile counterpart (per role spec) is out of scope for this build task.

## Team context

- Issue: [GEO-5](/GEO/issues/GEO-5) — Frontend Developer: Build web UI for leak detection app
- Plan: [GEO-4 plan document](/GEO/issues/GEO-4#document-plan)
