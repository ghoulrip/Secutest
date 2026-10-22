#!/bin/sh
# Strip backend/target/ from git index (build artifacts)
set -e
git rm -r --cached --ignore-unmatch backend/target >/dev/null 2>&1 || true