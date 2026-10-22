# Phase 3: Expand Test Coverage - Implementation Plan

## Overview
This plan implements the test coverage expansion tasks from GEO-17, organized by the development-phases-2-4.md task list.

## Current Test State
- **Backend**: 21 tests (5 models + 5 scanner + 11 handlers) - infrastructure ready, CI untested
- **Frontend**: 39 tests passing at 98.8% statement coverage
- **CI**: Configured but backend tests not yet verified

## Task Implementation Order

### 3.1: Backend Scanner Edge-Case Tests (2 hours) - **✅ COMPLETE**
**File**: `backend/src/tests/scanner_test.rs`
The following edge case tests are already implemented:
- Malformed HTML responses (lines 47-54)
- Mixed XSS + SQL injection patterns in same response (lines 56-65)
- Unicode payloads (emoji, RTL overrides, etc.) (lines 67-80)
- Very long response bodies (DoS resilience) (lines 82-93)
- Empty/whitespace-only bodies (lines 95-100)
- Null bytes in response (lines 102-109)
- Multiple XSS instances (lines 145-152)
- Event handlers (lines 154-167)

### 3.2: Backend Error-Handling Tests (2 hours) - **✅ COMPLETE**
**File**: `backend/src/tests/handlers_test.rs`
The following error handling tests are already implemented:
- Malformed JSON request (lines 438-450)
- Missing URL field (lines 452-463)
- Invalid URL format (lines 465-482)
- Very long URLs (lines 484-500)
- Null bytes in URL (lines 502-516)
- Invalid UUID job IDs (lines 518-562)

### 3.2: Backend Error-Handling Tests (2 hours)
**File**: `backend/src/tests/handlers_test.rs`
Add tests for:
- Malformed JSON request body
- Missing required fields (url)
- Invalid URL formats
- Duplicate scan IDs (should not happen but test isolation)
- Progress values > 1.0 or < 0.0
- Invalid UUID job IDs in path params
- Very long URLs
- Null bytes in URL

### 3.3: Sub-Domain Scanning Tests (3 hours)
**File**: `backend/src/services/scanner.rs` + `backend/src/tests/scanner_test.rs`
- Add subdomain enumeration logic to scanner
- Test wildcard DNS handling
- Test non-responsive subdomains
- Test subdomain scan options (include_subdomains flag)

### 3.4: Backend Integration Test - Full Scan Flow (3 hours) - **✅ COMPLETE**
**File**: `backend/src/tests/integration_test.rs`
The following integration tests are already implemented:
- Full scan flow from create to completed (lines 42-142)
- Mock HTTP server testing (lines 144-203)
- Failed scan status handling (lines 205-238)
- Multiple independent scan progress (lines 249-341)
- Background worker simulation (lines 344-441)

### 3.5: Frontend → Backend Integration Tests (3 hours)
**File**: `frontend/src/pages/ScanPage.integration.test.tsx` (new)
- Form submission → startScan API call → polling → report rendering
- Mock backend responses with MSW or vitest mocks

### 3.6: E2E Tests with Playwright (5 hours)
**File**: `frontend/e2e/` (new directory)
- Full scan flow through running app
- Submit URL, wait for scan, view report, verify findings
- Run against docker-compose stack

### 3.7: Backend Coverage Reporting (3 hours)
**Tool**: cargo-tarpaulin or llvm-cov
- Generate coverage report
- Add coverage job to CI
- Enforce thresholds (e.g., 70% statements)

---

## Dependencies
- Phase 2 (Windows dev fix) must be working for local backend test runs
- But CI runs on Linux so tests can be written and pushed without local execution

## Notes
- Write tests first, CI will verify them
- Use mockito for HTTP mocking in backend integration tests
- For E2E tests, consider Playwright over Cypress for better TypeScript support