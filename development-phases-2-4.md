# Secutest Development Phases 2–4

This document outlines all remaining work for Phases 2 through 4 of the Secutest leak-detection web application. Phase 1 (CI verification on Linux) is complete.

**Source of truth**: `C:\dev\Secutest\development-status-report.md`, commit `7c6a624` (`fix: restore Cargo files, remove duplicative scan_handlers, apply rustfmt`).

---

## Phase 2: Fix Windows Local Development

**Status**: In progress  
**Priority**: Medium  
**Dependency**: Phase 1 complete; Phase 3 depends on Phase 2 being usable  
**Blocked**: Local `cargo test` cannot compile on Windows

### Problem
`cargo test` cannot compile locally on Windows due to toolchain conflicts:
- MSVC target: missing `link.exe` (linker not in PATH)
- GNU MinGW target: broken — `dlltool.exe` / `-lgcc` causes linking failures

This blocks local backend development and debugging but does **not** block CI (CI runs on Linux Ubuntu runners).

### Fix Options

| Option | Description | Pros | Cons |
|--------|-------------|------|------|
| A. Install MinGW | Provides `dlltool.exe` for GNU target linking | Keeps Windows native dev | Still diverges from CI (Ubuntu) |
| B. **WSL2 with Linux toolchain** (recommended) | Use Ubuntu via WSL2; run cargo with Linux toolchain | Matches CI's Ubuntu runner exactly; avoids Windows toolchain conflicts | Requires WSL2 setup + learning curve |

### Task List — Phase 2

| # | Task | Steps | Effort |
|---|------|-------|--------|
| 2.1 | Install WSL2 | Install WSL2 (`wsl --install`), install Ubuntu distribution from Microsoft Store | 1 hour |
| 2.2 | Install Linux toolchain in WSL2 | `sudo apt update && sudo apt install build-essential curl`; install Rust via `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y` | 1 hour |
| 2.3 | Clone/open repo in WSL2 | Clone `ghoulrip/Secutest` into WSL2 filesystem (`\\wsl$\Ubuntu\home\<user>\Secutest`) or mount `C:\dev\Secutest` via `/mnt/c` | 30 min |
| 2.4 | Verify `cargo test` in WSL2 | Run `cargo test --manifest-path backend/Cargo.toml`; confirm 21 tests pass | 1 hour |
| 2.5 | Verify `cargo fmt --check` and `cargo clippy -D warnings` | Confirm formatting and linting pass on Linux toolchain | 30 min |
| 2.6 | Document local dev instructions | Add Windows/WSL2 setup steps to README (`backend/` development section) | 30 min |

**Phase 2 total estimated effort**: ~4–5 hours

---

## Phase 3: Expand Test Coverage

**Status**: Backlog  
**Priority**: Medium-Low  
**Dependency**: Phase 2 must be working first (backend tests run locally); Phase 4 depends on Phase 3 coverage data

### Source
QA report recommendations plus items tracked in `development-status-report.md`:
- Integration tests for full scan flow (frontend → backend → report)
- E2E tests with Playwright or Cypress
- Backend coverage reporting (tarpaulin or lcov)
- More scanner edge-case tests
- Sub-domain scanning tests
- Additional error handling tests

### Task List — Phase 3

| # | Task | Specific test areas to expand | Effort |
|---|------|-------------------------------|--------|
| 3.1 | Add backend scanner edge-case tests | `backend/src/tests/scanner_test.rs`: add malformed HTML, mixed XSS+SQLi patterns, unicode payloads, long bodies, concurrent scans | 2 hours |
| 3.2 | Add backend error-handling tests | `backend/src/tests/handlers_test.rs`: malformed JSON body, missing fields, duplicate scan IDs, progress > 1.0, invalid UUID job IDs | 2 hours |
| 3.3 | Add sub-domain scanning tests | Scanner service (`backend/src/services/scanner.rs`): subdomain enumeration edge cases, wildcard DNS, non-responsive subdomains | 3 hours |
| 3.4 | Add backend integration test for full scan flow | End-to-end within backend: create scan → worker runs `run_http_scan` → status progresses queued → running → completed → results retrievable | 3 hours |
| 3.5 | Add frontend → backend integration tests | Frontend (`frontend/src/`): form submission → `startScan` API call → polling → report rendering; mock backend responses | 3 hours |
| 3.6 | Add E2E tests (Playwright or Cypress) | Full scan flow through the running app: submit URL, wait for scan, view report, verify findings tiles; run against docker-compose stack | 5 hours |
| 3.7 | Add backend coverage reporting | Install `cargo-tarpaulin` or configure `llvm-cov`/lcov; generate coverage report; add coverage job to CI; enforce thresholds (e.g., 70% statements) | 3 hours |

**Phase 3 total estimated effort**: ~21 hours

---

## Phase 4: Production Readiness

**Status**: Backlog  
**Priority**: Nice to Have  
**Dependency**: Phase 3 (coverage data needed for badges)

### Task List — Phase 4

| # | Task | Details | Effort |
|---|------|---------|--------|
| 4.1 | Add CI coverage badges to README | Add backend (tarpaulin/lcov) and frontend coverage badges to `README.md`, linked to CI artifacts or Codecov/Coveralls | 1 hour |
| 4.2 | Add test execution time measurement | Instrument `cargo test` and `npm test` timing; record and track execution time in CI; alert on regressions | 2 hours |
| 4.3 | Clean up untracked files | Remove: `post-status.js`, `push_attempt.log`, `result.txt`, `status-update.json`, `strip-target.sh` (verify none are needed first) | 30 min |
| 4.4 | Review and remove duplicative `scan_handlers` | Confirm `backend/src/handlers/mod.rs` already contains the full handler set (`health_check`, `create_scan`, `list_scans`, `get_scan_status`, `get_scan_results`, `update_scan_progress`, `scan_worker`, `AppState`); remove any duplicative/dead code | 1 hour |
| 4.5 | Final production readiness review | Confirm Docker multi-stage build, deploy workflow, env config, and health-check endpoint all verified | 2 hours |

**Phase 4 total estimated effort**: ~6–7 hours

---

## Prioritization and Dependencies

```
Phase 1 (complete) ──┐
                     ▼
             Phase 2 (Windows dev fix)
                     │  prerequisite for local backend test runs
                     ▼
             Phase 3 (expand test coverage)
                     │  coverage data feeds badges
                     ▼
             Phase 4 (production readiness)
```

### Dependency notes
1. **Phase 2 → Phase 3**: Backend unit/integration tests must run locally before adding more of them. WSL2 setup unblocks `cargo test`.
2. **Phase 3 → Phase 4**: Coverage reporting (3.7) produces the data needed for CI coverage badges (4.1).
3. **Phase 4 cleanup (4.3, 4.4)** can be done independently of test work, but is grouped here for completeness.

### Recommended execution order
1. Phase 2 (all tasks) — unblocks local backend development.
2. Phase 3 tasks 3.1–3.4 (backend-focused) — can run in WSL2.
3. Phase 3 tasks 3.5–3.7 (frontend + coverage).
4. Phase 4 tasks in order; cleanup (4.3, 4.4) first since it's low effort and clears the working tree.

---

## Overall Estimated Effort

| Phase | Effort | Priority |
|-------|--------|----------|
| Phase 2: Fix Windows Local Development | ~4–5 hours | Medium |
| Phase 3: Expand Test Coverage | ~21 hours | Medium-Low |
| Phase 4: Production Readiness | ~6–7 hours | Nice to Have |
| **Total** | **~31–33 hours** | — |

---

## Notes

- The untracked files listed for cleanup (`post-status.js`, `push_attempt.log`, `result.txt`, `status-update.json`, `strip-target.sh`) appear at the repo root per `git status` at commit `7c6a624`. Verify contents before deletion.
- The handler deduplication review (4.4) references `backend/src/handlers/mod.rs`, which already contains `scan_handlers` module with the full handler set; confirm no separate `scan_handlers.rs` file remains before removing anything.
