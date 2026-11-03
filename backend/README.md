# Secutest Backend

Rust backend for the Secutest security scanner (Actix-Web, Postgres).

## Local development (Windows)

Build with the **WSL2 Ubuntu toolchain** — it matches CI (Ubuntu runner) so builds are reproducible.

```bash
./dev.sh              # cargo test
./dev.sh test --release
./dev.sh bash         # drop into the Ubuntu shell in backend/
```

Inside WSL, the repo is at `/mnt/c/dev/Secutest/backend`.

> Why not Rust on Windows directly? The Windows `cargo`/`rustc` aren't on PATH, and the
> MSVC target needs `link.exe` while the MinGW target has linker issues. WSL2 avoids both
> and is what CI uses.
