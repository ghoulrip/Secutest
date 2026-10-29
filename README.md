# Secutest

**Leak-detection web app** — Scan targets for XSS, SQL injection, and other vulnerabilities. Built with a **Rust backend** (Actix-web) and a **React/TypeScript frontend** (Vite).

---

## Development

This section covers local development setup for both backend and frontend on Windows.

### Prerequisites

- **Windows 10/11** with WSL2 available
- **Docker Desktop** (optional, for containerized development)

---

### Backend Development (Windows + WSL2)

The Rust backend cannot compile natively on Windows due to toolchain conflicts:

- **MSVC target**: missing `link.exe` (linker not in PATH)
- **GNU MinGW target**: broken — `dlltool.exe` / `-lgcc` causes linking failures

**Recommended approach**: Use WSL2 with a Linux toolchain that matches CI's Ubuntu runners.

#### 2.1 Install WSL2

Open PowerShell as Administrator and run:

```powershell
wsl --install
```

This installs WSL2 and the default Ubuntu distribution from the Microsoft Store. Restart your computer when prompted.

Verify WSL2 is running:

```powershell
wsl --list --verbose
```

You should see `Ubuntu` listed as `Running` with `WSL version: 2`.

#### 2.2 Install Linux Toolchain in WSL2

Open Ubuntu from the Start menu or run:

```powershell
wsl -d Ubuntu
```

Inside WSL2, install system dependencies and Rust:

```bash
sudo apt update && sudo apt install -y build-essential curl

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

source "$HOME/.cargo/env"
```

Verify the Rust toolchain:

```bash
rustc --version
cargo --version
```

The toolchain should match CI's Ubuntu runners (stable Rust on x86_64-unknown-linux-gnu).

#### 2.3 Clone or Open the Repository in WSL2

**Option A — Mount the existing Windows directory** (recommended for quick setup):

```bash
cd /mnt/c/dev/Secutest
```

**Option B — Clone into the WSL2 filesystem** (better performance):

```bash
git clone https://github.com/ghoulrip/Secutest.git ~/Secutest
cd ~/Secutest
```

Navigate to the backend directory:

```bash
cd backend
```

#### 2.4 Verify `cargo test` in WSL2

Run the backend test suite:

```bash
cargo test --manifest-path backend/Cargo.toml
```

All **tests** should pass (100+ tests across 4 modules):
- **models_test.rs**: 17 tests (serialization/deserialization)
- **scanner_test.rs**: 31 tests (XSS/SQLi detection, edge cases, subdomain extraction)
- **handlers_test.rs**: 60+ tests (handler integration, error handling, subdomain scanning, mockito)
- **integration_test.rs**: 5 tests (full scan flow, mockito integration, worker simulation)

Expected output (example):

```
test result: ok. 113 passed; 0 failed; 0 ignored; 0 measured
```

#### 2.5 Verify `cargo fmt --check` and `cargo clippy -D warnings`

Formatting:

```bash
cargo fmt --manifest-path backend/Cargo.toml -- --check
```

Linting:

```bash
cargo clippy --manifest-path backend/Cargo.toml -- -D warnings
```

Both should pass without errors on the Linux toolchain.

#### 2.6 Local Development Workflow

| Command | Purpose |
|---------|---------|
| `cargo build --manifest-path backend/Cargo.toml` | Compile the backend |
| `cargo test --manifest-path backend/Cargo.toml` | Run all 100+ backend tests |
| `cargo run --manifest-path backend/Cargo.toml` | Start the backend server (`localhost:8080`) |
| `cargo fmt --manifest-path backend/Cargo.toml -- --check` | Check code formatting |
| `cargo clippy --manifest-path backend/Cargo.toml -- -D warnings` | Run linter with warnings as errors |
| `cargo audit` | Security vulnerability scanning |

---

### Frontend Development

See `frontend/README.md` for frontend setup instructions (`npm install`, `npm run dev`, `npm run build`).

---

### CI/CD

All backend jobs run on **Linux Ubuntu runners** via GitHub Actions:

| Job | Command |
|-----|---------|
| `backend-build` | `cargo build --release` |
| `backend-test` | `cargo test` + `cargo fmt --check` + `cargo clippy -- -D warnings` |
| `backend-security` | `cargo audit` |

See `.github/workflows/ci.yml` for the full CI pipeline.

---

### Troubleshooting

| Issue | Solution |
|-------|----------|
| `cargo test` fails on Windows | Use WSL2 (this README section) instead of native Windows toolchain |
| `link.exe not found` | MSVC target linker missing — switch to WSL2 Linux toolchain |
| `dlltool.exe not found` | GNU MinGW broken — switch to WSL2 Linux toolchain |
| Rust not found in WSL2 | Run `source "$HOME/.cargo/env"` or restart WSL2 terminal |
| WSL2 not installed | Run `wsl --install` as Administrator in PowerShell |

---

### Team Context

- Issue: GEO-21 — Phase 2: Fix Windows Local Development (WSL2 Setup)
- Parent: GEO-20 — Phase 1 plan document and task breakdown
- Phase 2 task list: 2.1–2.6 (~4–5 hours)
- CI pipeline: `.github/workflows/ci.yml`