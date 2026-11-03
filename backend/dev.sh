#!/usr/bin/env bash
# Run cargo commands in the Ubuntu WSL2 toolchain for the Secutest backend.
# WSL2 matches CI (Ubuntu runner) so builds are reproducible on Windows.
#
# Works whether invoked from Windows (PowerShell/Git Bash) or from inside Ubuntu WSL:
#   ./dev.sh                  # cargo test
#   ./dev.sh clippy           # cargo clippy --all-targets
#   ./dev.sh bash             # drop into an Ubuntu shell in backend/
set -euo pipefail

cd "$(dirname "$0")"
REPO="$(pwd)"

# Inside WSL we run cargo directly; from Windows we reach the Ubuntu distro via wsl.exe.
if [ -n "${WSL_DISTRO_NAME:-}" ] || grep -qi microsoft /proc/version 2>/dev/null; then
  exec bash -c "cd '$REPO' && cargo ${*:-test}"
fi

WSL="/c/Windows/system32/wsl.exe -d Ubuntu -e bash -lc"
cmd="${1:-test}"
shift || true

if [ "$cmd" = "bash" ]; then
  exec "$WSL" "bash"
fi

exec "$WSL" "cd '$REPO' && cargo $cmd ${*:-}"
