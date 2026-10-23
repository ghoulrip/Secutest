# Secutest Local Development Setup Guide

**Version:** 1.0  
**Last Updated:** 2026-10-22  
**Related Issues:** GEO-27 (Phase 2.6), GEO-21 (Phase 2), GEO-20 (Phase 1)

---

## Overview

This guide documents the complete Windows local development setup for the Secutest leak-detection web application. The backend is built with Rust (Actix-web) and cannot compile natively on Windows due to toolchain conflicts. The recommended approach is to use **WSL2 with Ubuntu** and a Linux toolchain that matches CI's Ubuntu runners.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Install WSL2](#1-install-wsl2)
3. [Install Linux Toolchain in WSL2](#2-install-linux-toolchain-in-wsl2)
4. [Access Repository in WSL2](#3-access-repository-in-wsl2)
5. [Verify Build & Test Commands](#4-verify-build--test-commands)
6. [Local Development Workflow](#5-local-development-workflow)
7. [Frontend Development](#6-frontend-development)
8. [Troubleshooting](#7-troubleshooting)
9. [CI/CD Alignment](#8-cicd-alignment)

---

## Prerequisites

- **Windows 10 (1903+) or Windows 11** with WSL2 support
- **Administrator access** on Windows machine
- **Internet connection** for downloading packages
- **Docker Desktop** (optional, for containerized development)

---

## 1. Install WSL2

### 1.1 Open PowerShell as Administrator

Right-click the Start button → **Windows Terminal (Admin)** or search "PowerShell" → right-click → **Run as Administrator**

### 1.2 Install WSL2 with Ubuntu

```powershell
wsl --install
```

This command:
- Enables the Virtual Machine Platform and WSL features
- Downloads and installs the latest WSL2 kernel
- Sets WSL2 as the default version
- Installs Ubuntu as the default Linux distribution from the Microsoft Store

### 1.3 Restart Your Computer

When prompted, restart your computer to complete the installation.

### 1.4 Verify WSL2 Installation

After restart, open PowerShell and run:

```powershell
wsl --list --verbose
```

Expected output:
```
  NAME      STATE           VERSION
* Ubuntu    Running         2
```

The `VERSION` column should show `2` and `STATE` should be `Running`.

### 1.5 Set Ubuntu as Default (if multiple distros)

```powershell
wsl --set-default Ubuntu
```

---

## 2. Install Linux Toolchain in WSL2

### 2.1 Open Ubuntu Terminal

Launch **Ubuntu** from the Start menu or run:

```powershell
wsl -d Ubuntu
```

### 2.2 Update Package Lists

```bash
sudo apt update
```

### 2.3 Install System Dependencies

```bash
sudo apt install -y build-essential curl libssl-dev pkg-config
```

| Package | Purpose |
|---------|---------|
| `build-essential` | GCC, G++, make, and other build tools |
| `curl` | Download Rust installer |
| `libssl-dev` | OpenSSL development headers (required by many Rust crates) |
| `pkg-config` | Library discovery for native dependencies |

### 2.4 Install Rust Toolchain

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
```

This installs:
- `rustc` (Rust compiler)
- `cargo` (package manager and build tool)
- `rustup` (toolchain manager)

### 2.5 Configure Shell Environment

```bash
source "$HOME/.cargo/env"
```

**To make this permanent**, add to your shell profile:

```bash
echo 'source "$HOME/.cargo/env"' >> ~/.bashrc
echo 'source "$HOME/.cargo/env"' >> ~/.profile
```

### 2.6 Verify Rust Installation

```bash
rustc --version
cargo --version
rustup --version
```

Expected output (versions may vary):
```
rustc 1.82.0 (stable)
cargo 1.82.0
rustup 1.27.0
```

The toolchain should match CI's Ubuntu runners: **stable Rust on `x86_64-unknown-linux-gnu`**.

---

## 3. Access Repository in WSL2

### Option A: Mount Existing Windows Directory (Recommended for Quick Setup)

If your repository is already on Windows at `C:\dev\Secutest`:

```bash
cd /mnt/c/dev/Secutest
```

**Pros:** No re-clone needed, files sync instantly with Windows editors  
**Cons:** Slightly slower filesystem performance on `/mnt/c`

### Option B: Clone into WSL2 Filesystem (Better Performance)

```bash
# Navigate to WSL2 home directory
cd ~

# Clone the repository
git clone https://github.com/ghoulrip/Secutest.git
cd Secutest
```

**Pros:** Native Linux filesystem performance, faster cargo operations  
**Cons:** Need to set up Git credentials in WSL2; Windows editors access via `\\wsl$\Ubuntu\home\<user>\Secutest`

### 3.1 Navigate to Backend Directory

```bash
cd backend
```

---

## 4. Verify Build & Test Commands

### 4.1 Run Full Test Suite

```bash
cargo test --manifest-path backend/Cargo.toml
```

**Expected Result:** All **21 tests** should pass:

```
running 21 tests
test tests::models_test::test_scan_job_serialization ... ok
test tests::models_test::test_scan_status_serialization ... ok
test tests::models_test::test_severity_level_serialization ... ok
test tests::models_test::test_scan_job_deserialization ... ok
test tests::models_test::test_scan_with_findings ... ok
test tests::scanner_test::test_analyze_xss_simple ... ok
test tests::scanner_test::test_analyze_sql_injection ... ok
test tests::scanner_test::test_analyze_no_vulnerabilities ... ok
test tests::scanner_test::test_analyze_multiple_patterns ... ok
test tests::scanner_test::test_analyze_edge_case_unicode ... ok
test tests::handlers_test::test_health_check ... ok
test tests::handlers_test::test_create_scan_valid ... ok
test tests::handlers_test::test_create_scan_invalid_url ... ok
test tests::handlers_test::test_list_scans ... ok
test tests::handlers_test::test_get_scan_status ... ok
test tests::handlers_test::test_get_scan_results ... ok
test tests::handlers_test::test_update_scan_progress ... ok
test tests::handlers_test::test_scan_worker ... ok
test tests::handlers_test::test_create_scan_malformed_json ... ok
test tests::handlers_test::test_create_scan_missing_url ... ok
test tests::handlers_test::test_invalid_uuid_job_id ... ok
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured
```

**Test breakdown:**
- **5 model serialization tests** (`models_test.rs`)
- **5 scanner logic tests** (`scanner_test.rs`)
- **11 handler integration tests** (`handlers_test.rs`)

### 4.2 Verify Code Formatting

```bash
cargo fmt --manifest-path backend/Cargo.toml -- --check
```

**Expected:** No output (exit code 0) = formatting is correct

If formatting issues exist:
```bash
cargo fmt --manifest-path backend/Cargo.toml
```

### 4.3 Verify Linting (Clippy)

```bash
cargo clippy --manifest-path backend/Cargo.toml -- -D warnings
```

**Expected:** No warnings or errors (exit code 0)

### 4.4 Quick Build Verification

```bash
cargo build --manifest-path backend/Cargo.toml
```

**Expected:** Successful compilation with no errors

---

## 5. Local Development Workflow

### 5.1 Common Commands Reference

| Command | Purpose |
|---------|---------|
| `cargo build --manifest-path backend/Cargo.toml` | Compile the backend (debug) |
| `cargo build --release --manifest-path backend/Cargo.toml` | Compile optimized release build |
| `cargo test --manifest-path backend/Cargo.toml` | Run all 21 backend tests |
| `cargo test --manifest-path backend/Cargo.toml -- <filter>` | Run specific test(s) |
| `cargo run --manifest-path backend/Cargo.toml` | Start backend server on `localhost:8080` |
| `cargo fmt --manifest-path backend/Cargo.toml -- --check` | Check code formatting |
| `cargo fmt --manifest-path backend/Cargo.toml` | Auto-format code |
| `cargo clippy --manifest-path backend/Cargo.toml -- -D warnings` | Run linter (warnings = errors) |
| `cargo clippy --manifest-path backend/Cargo.toml --fix -- -D warnings` | Auto-fix clippy warnings |
| `cargo audit` | Security vulnerability scanning (requires `cargo install cargo-audit`) |
| `cargo doc --manifest-path backend/Cargo.toml --open` | Generate and open documentation |

### 5.2 Running the Backend Server

```bash
cd /path/to/Secutest/backend
cargo run --manifest-path backend/Cargo.toml
```

Server starts at: **http://localhost:8080**

**Health check:**
```bash
curl http://localhost:8080/health
# Expected: {"status":"ok"}
```

### 5.3 IDE/Editor Setup (VS Code Recommended)

1. **Install VS Code** on Windows
2. **Install "WSL" extension** (ms-vscode-remote.remote-wsl)
3. **Open project in WSL2:**
   - In WSL2 terminal: `code .`
   - Or in VS Code: `Ctrl+Shift+P` → "WSL: Connect to WSL" → open `/home/<user>/Secutest` or `/mnt/c/dev/Secutest`
4. **Install Rust Analyzer extension** (inside WSL2 session)
5. **Configure format on save** (settings.json):
   ```json
   {
     "rust-analyzer.checkOnSave": true,
     "editor.formatOnSave": true,
     "[rust]": {
       "editor.defaultFormatter": "rust-lang.rust-analyzer"
     }
   }
   ```

---

## 6. Frontend Development

The frontend runs natively on Windows (Node.js/Vite) and proxies API calls to the WSL2 backend.

### 6.1 Prerequisites on Windows

- **Node.js 22.12.0+** (match CI version)
- **npm** (comes with Node.js)

```powershell
# Verify Node version
node --version
# Should be >= 22.12.0
```

### 6.2 Setup Frontend

```powershell
# In Windows PowerShell (not WSL2)
cd C:\dev\Secutest\frontend
npm install
```

### 6.3 Run Frontend Dev Server

```powershell
npm run dev
```

- Dev server: **http://localhost:5173**
- Vite proxies `/api/*` → `http://localhost:8080` (WSL2 backend)

### 6.4 Frontend Commands

| Command | Purpose |
|---------|---------|
| `npm run dev` | Start dev server with HMR |
| `npm run build` | Production build to `dist/` |
| `npm run preview` | Preview production build |
| `npm test` | Run 39 Vitest tests (98.8% coverage) |
| `npm run lint` | TypeScript type checking (`tsc --noEmit`) |

---

## 7. Troubleshooting

### 7.1 WSL2 Issues

| Issue | Solution |
|-------|----------|
| `wsl --install` fails | Enable Virtual Machine Platform: `dism.exe /online /enable-feature /featurename:VirtualMachinePlatform /all /norestart` then restart |
| WSL2 runs very slow | Check if antivirus is scanning `/mnt/c`; add exclusion for project directory |
| `wsl -d Ubuntu` hangs | Run `wsl --shutdown` in PowerShell, then restart Ubuntu |
| Ubuntu not in Microsoft Store | Manual install: `wsl --install -d Ubuntu-22.04` |

### 7.2 Rust/Cargo Issues

| Issue | Solution |
|-------|----------|
| `cargo` command not found | Run `source "$HOME/.cargo/env"` or restart WSL2 terminal |
| `linker cc not found` | Install build-essential: `sudo apt install -y build-essential` |
| OpenSSL linking errors | Install libssl-dev: `sudo apt install -y libssl-dev pkg-config` |
| Rust version mismatch with CI | Run `rustup update stable` to match CI's stable toolchain |
| Cargo cache corrupted | Run `cargo clean` then rebuild |

### 7.3 Build & Test Failures

| Issue | Solution |
|-------|----------|
| `cargo test` fails with 0 tests | Ensure you're in `/path/to/Secutest/backend` or use `--manifest-path` |
| Tests pass locally but fail in CI | Check CI uses Ubuntu runners; ensure same Rust stable version |
| `cargo fmt --check` fails | Run `cargo fmt` to auto-fix, then commit changes |
| `cargo clippy` shows warnings | Fix warnings or use `#[allow(...)]` with justification |

### 7.4 Backend-Frontend Integration

| Issue | Solution |
|-------|----------|
| Frontend "could not reach backend" | Ensure backend is running: `cargo run` in WSL2; check `http://localhost:8080/health` |
| CORS errors | Vite dev server proxies `/api` to backend; no CORS config needed locally |
| API requests timeout | Check Windows firewall allows port 8080; WSL2 port forwarding is automatic |

### 7.5 Git Issues

| Issue | Solution |
|-------|----------|
| Line ending warnings | Configure Git in WSL2: `git config --global core.autocrlf input` |
| SSH key not found in WSL2 | Copy Windows keys: `cp /mnt/c/Users/<user>/.ssh/* ~/.ssh/` and `chmod 600 ~/.ssh/id_*` |
| Git credential helper | Use Windows credential manager: `git config --global credential.helper "/mnt/c/Program\ Files/Git/mingw64/bin/git-credential-manager.exe"` |

---

## 8. CI/CD Alignment

### 8.1 CI Environment

All backend CI jobs run on **GitHub Actions Ubuntu runners**:

| Job | Commands |
|-----|----------|
| `backend-build` | `cargo build --release` |
| `backend-test` | `cargo test` + `cargo fmt --check` + `cargo clippy -- -D warnings` |
| `backend-security` | `cargo audit` |

### 8.2 Local-CI Parity Checklist

Before pushing, verify locally in WSL2:

- [ ] `cargo test` — all 21 tests pass
- [ ] `cargo fmt --check` — no formatting issues
- [ ] `cargo clippy -D warnings` — no lint warnings
- [ ] `cargo build --release` — release build succeeds
- [ ] `cargo audit` — no critical vulnerabilities (optional, install `cargo-audit`)

### 8.3 CI Configuration Reference

See `.github/workflows/ci.yml` for the complete pipeline definition.

---

## Appendix: Quick Reference Card

### One-Line Setup (Copy-Paste)

```bash
# Run inside WSL2 Ubuntu terminal
sudo apt update && sudo apt install -y build-essential curl libssl-dev pkg-config
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
cd /mnt/c/dev/Secutest/backend  # or ~/Secutest/backend
cargo test
cargo fmt --check
cargo clippy -D warnings
```

### Useful WSL2 Paths

| Description | Path |
|-------------|------|
| Windows C: drive | `/mnt/c/` |
| Windows user home | `/mnt/c/Users/<username>/` |
| WSL2 user home | `~` or `/home/<username>/` |
| Access WSL2 from Windows Explorer | `\\wsl$\Ubuntu\` |
| Project (Windows mount) | `/mnt/c/dev/Secutest` |
| Project (WSL2 native) | `~/Secutest` |

### Version Compatibility Matrix

| Component | Local (WSL2) | CI (GitHub Actions) |
|-----------|--------------|---------------------|
| OS | Ubuntu 22.04/24.04 | Ubuntu 22.04/24.04 |
| Rust | Stable (rustup) | Stable (rustup) |
| Target | x86_64-unknown-linux-gnu | x86_64-unknown-linux-gnu |
| Node.js (frontend) | 22.12.0+ (Windows) | 22.17 (CI) |

---

## Support & Contributing

- **Issues**: Report setup issues at [GitHub Issues](https://github.com/ghoulrip/Secutest/issues)
- **Documentation Updates**: Submit PRs to improve this guide
- **Related Documentation**:
  - [README.md](README.md) — Project overview & quick start
  - [development-phases-2-4.md](development-phases-2-4.md) — Phase planning
  - [QA/TEST_FRAMEWORK.md](QA/TEST_FRAMEWORK.md) — Test framework details

---

*This document was created as part of GEO-27 Phase 2.6: Document Local Development Instructions. It consolidates steps from Phase 2.1 through 2.5 into a single reference for team onboarding.*