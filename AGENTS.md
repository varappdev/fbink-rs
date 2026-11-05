# AGENTS.md

Rust bindings for FBInk: `fbink-sys` (raw bindgen FFI; compiles the tethered C
source at build time) + `fbink` (safe wrapper). Quickstart and the
feature / `FBINK_TARGET` tables: `README.md`.

## Where truth lives

| Source | Holds |
|---|---|
| `fbink-sys/build.rs` | the pin (FBInk + submodule commits), feature→define map, `FBINK_TARGET`/`FBINK_SOURCE_DIR`/`FBINK_FETCH`, fetch & cache logic |
| pinned `fbink.h` | authoritative API docs; read before wrapping calls or touching signatures (cached at `target/<profile>/build/fbink-sys-src/src/fbink/fbink.h`, or fetch raw from GitHub at the pin) |
| `fbink-sys/patches/` | fixes spliced into the C source before compiling; `patches/README.md` = the patch contract |
| `README.md` | user-facing docs; update alongside public API changes |

## Verify (host; no eInk hardware needed)

- `cargo build && cargo test` — unit, doctests and the fbink-sys smoke test.
- `cargo clippy --all-targets`, `cargo doc --no-deps` — both clean, intra-doc links resolve.
- Feature matrix: bare `--no-default-features`, then `--features <f>` for `draw`,
  `opentype`, `input`, `image`, `button-scan`, `unifont`, and `opentype,input`.
- `FBINK_TARGET=kobo cargo test -p fbink --lib` — flavor switching recompiles.
- Cross-build: `./scripts/build-demo-arm.sh [--release]` (zig cc, static musl,
  `FBINK_TARGET=kindle`); runtime testing on the jailbroken Kindle VM —
  `kindle-vm` skill (rig symlinked as `eink-emulator/`, deploy recipe in the
  script's header). On-device patterns (screen dump/restore lifecycle,
  partial-refresh animation): `bounce.rs`'s comments.

## Conventions

- Every raw value comes from `sys::*` consts (bindgen output); signatures use
  the header's fixed-width `*_T` types.
- No `libc` dep: `error.rs` hardcodes Linux errno numbers. Negative returns map
  through `Error` (`-1` first, as `Failure` — it is ambiguous). Two families
  invert the sign convention: `reinit`, `set_fb_info`, `sunxi_ntx_enforce_rota`
  return **positive bitmasks on success** (0 = nothing changed), while `rota_*`
  reports **positive errno as failure** (valid results are `0-3`; see
  `rota_result` in `fb.rs`).
- Features are declared in three places that change together:
  `fbink-sys/Cargo.toml` (definition + implications), `fbink-sys/build.rs`
  (defines + per-feature C files), `fbink/Cargo.toml` (forwarding).
- Only `ot` & `input` are feature-gated modules; every other call compiles
  always and returns `Error::NotSupported` at runtime when FBInk lacks the
  feature or device support (`FBINK_TARGET` is a device flavor, not a target
  triple).

## Changing the pin

When bumping `FBINK_COMMIT` in `fbink-sys/build.rs`:

1. Diff old vs new `fbink.h`; adjust `fbink`'s enums & signatures to match.
2. Each `PATCHES` entry must keep applying exactly (preimage matches once) or
   be rebased/dropped — the build failing loudly here is by design. While
   iterating on a patch, wipe the source cache first (`cargo clean` or
   `rm -rf target/*/build/fbink-sys-src`). Check whether the `fbink_state_dump`
   varargs segfault got fixed upstream; if so, drop the patch and update
   `fb.rs`'s `state_dump` docs.
3. Re-run the verify list.

## Workflow

- Commits: Conventional Commits (`caveman-commit` skill); review new diffs with
  `caveman-review` before declaring done.
