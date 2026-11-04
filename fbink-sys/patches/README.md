# Local patches to the pinned FBInk source

`build.rs` applies every patch in this directory to the FBInk source tree
before compiling it (both for the fetched snapshot and for a user-supplied
`FBINK_SOURCE_DIR` checkout — the latter is modified in place).

## Why

`fix-fbink_state_dump-varargs-non-kobo.patch` fixes an upstream bug that
**segfaults the process** in `fbink_state_dump()` on every non-Kobo target:
its `fprintf` passes only two values for the three
`isTolino`/`SunxiHasFBDamage`/`SunxiForceRota` format specifiers (the `#else`
branch), so all later values shift by one and `pixelFormat='%s'` receives an
integer (0 or 1) as a pointer. Reproduced with the stock FBInk 1.25.0 binary
(`fbink -e`) on a Kindle Basic 5 (22D) and at the pinned revision; the bug is
still present on NiLuJe/FBInk `master` as of 2026-11-04.

The patch is a plain unified diff (`patch -p1` / `git apply` compatible), so
it doubles as an upstream submission.

## Contract (loud by design)

`build.rs` splices each patch's hunks into the source as exact-text
replacements (no fuzz, no offsets): the preimage must occur exactly once.

| Tree state | Behavior |
|---|---|
| preimage matches | hunk applied, one `cargo:warning` |
| postimage already in place | silent no-op (rebuilds stay green) |
| anything else | **build fails**: the patch is stale |

A stale patch means the FBInk pin was bumped (or the source changed). Fix it
here — rebase the patch if the bug remains, delete it if upstream fixed the
bug (update `fb.rs`'s `state_dump` docs when you do). When iterating on a
patch against the cached snapshot, wipe the cache first (`cargo clean`, or
`rm -rf target/*/build/fbink-sys-src`): a tree patched with an older version
of the patch no longer classifies cleanly.
