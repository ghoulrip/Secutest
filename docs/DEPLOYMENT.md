# Deployment & Operations Guide

Production readiness for the Secutest leak-detection service. Written for
DevOps / platform teams. See `README.md` for local development setup.

## Architecture

Secutest is a two-service application:

- **Backend** — Rust (Actix-web) API at `http://0.0.0.0:8080`, exposed `/health`.
- **Frontend** — React 19 + TypeScript (Vite), served as a static build.

Both run inside a single container; the container entrypoint starts the Rust
backend in the background and serves the production static build.

```
                 ┌───────────────────────────────────┐
   client  ───►  │  seculint/secutest (docker image)   │
                 │                                      │
                 │   backend :8080  ─────►  static /     │
                 │   vite preview :5173 ◄── dist/        │
                 └───────────────────────────────────┘
```

## Build & push

The `Deploy` workflow (`.github/workflows/deploy.yml`) builds and pushes to
Docker Hub. It is triggered on pushes that touch `*.yml` / `Dockerfile` /
`*.toml`, or manually via `workflow_dispatch`.

```bash
docker build -t seculint/secutest:latest .
docker tag seculint/secutest:latest <DOCKERHUB_USERNAME>/secutest:latest
docker push <DOCKERHUB_USERNAME>/secutest:latest
```

Required secrets (set in the GitHub repository/organization secrets):

- `DOCKERHUB_USERNAME`
- `DOCKERHUB_TOKEN`

## Runtime configuration

| Variable | Default | Purpose |
|----------|---------|---------|
| `PORT` | `8080` | Backend HTTP bind port. The backend binds `0.0.0.0:{PORT}`. |

No other runtime configuration is required. The frontend is served from the
committed `dist/` build; no dev server runs in the container.

## Health check

```bash
curl -fsS http://localhost:8080/health
# {"status":"healthy","timestamp":"2026-..."}
```

The deploy workflow runs a post-deploy verification step against
`http://localhost:8080/health`.

## Kubernetes (optional)

The `Deploy` workflow attempts a K8s deploy when a target is available
(`DEPLOY_TARGET` env is `production` by default, or `staging` on dispatch). If
no cluster context is present the step is a placeholder — confirm a real
manifest/deployment is wired before relying on auto-deploy.

## CI / CD pipeline

`.github/workflows/ci.yml` — runs on every push/PR to `main`:

| Job | Covers |
|-----|--------|
| `backend-build` | `cargo build --release` |
| `backend-test` | `cargo test` + `cargo fmt --check` + `cargo clippy -- -D warnings` |
| `backend-security` | `cargo audit` |
| `frontend-test` | `npm run build` + `npm test` (records execution time) |
| `frontend-lint` | `npx tsc --noEmit` |
| `frontend-security` | `npm audit --audit-level=high` |
| `docker-build` | multi-arch build via Buildx |
| `deploy-preview` | PR deploy preview |

Status badges in `README.md` link to the CI run and coverage reports:

- Frontend coverage — `frontend-coverage` job (v8 provider): statements /
  branches / functions / lines (target 70 / 60 / 70 / 70).
- Backend — `cargo` test pass count, `fmt`, `clippy`, `audit`.

## Monitoring & test timing

Backend and frontend test suites now record execution time to the CI
`$GITHUB_OUTPUT` context. Surface these as dashboard panels or alerts once the
values are consumed by a metric collector.

## Maintenance

- **Dependency refresh** — run `cargo update` (backend) / `npm audit fix`
  (frontend); keep `Cargo.lock` committed for reproducible builds.
- **Audit** — `cargo audit` (backend) and `npm audit --audit-level=high`
  (frontend) run in CI; treat HIGH+ findings as release blockers.
- **Images** — prune stale `latest` tags; the workflow also pushes
  `<image>:<github.sha>` for reproducible rollbacks.
- **Secrets rotation** — rotate `DOCKERHUB_TOKEN` via the same rotation
  cadence as other credentials; it never leaves the runner.
