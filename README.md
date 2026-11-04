# fbink-rs

Rust bindings for [FBInk](https://github.com/NiLuJe/FBInk) ("FrameBuffer eInker"), a C
library to print text & images to eInk Linux framebuffers (Kindle, Kobo, reMarkable,
PocketBook, generic Linux fb).

Two crates:

- **`fbink-sys`** — raw `unsafe` FFI bindings. The C sources and bindings are generated
  at build time from a pinned FBInk revision, so they can never drift apart.
- **`fbink`** — safe, idiomatic wrapper: RAII handles & dumps, `Result` errors, fluent
  `Config` builder, typed enums.

Build requirements: a C compiler, `libclang` (for bindgen) and `curl` or `wget` (to fetch
the pinned FBInk sources, cached under `target/`).

## Quickstart (safe API)

```rust
use fbink::{Config, Fbink, Font, LAST_MARKER};

fn main() -> fbink::Result<()> {
    let fb = Fbink::open()?;                 // or Fbink::auto() to let FBInk manage the fd
    fb.init(&Config::new().quiet(true))?;    // once, from the main thread
    fb.print("Hello, world!", &Config::new().row(2).font(Font::Unscii))?;
    fb.refresh(0, 0, 0, 0, &Config::new())?; // full-screen refresh (0x0 @ (0, 0))
    fb.wait_for_complete(LAST_MARKER)?;
    Ok(())
}
```

See `fbink/examples/hello.rs` (`cargo run -p fbink --example hello`),
`fbink/examples/state.rs` (dump of every `State` field) and
`fbink/examples/bounce.rs` (an animated bouncing ball: dirty-region refreshes
in the fast DU waveform). Errors are typed
(`Error::NotSupported`, `Error::Invalid`, …); feature- or platform-unsupported calls fail
with `Error::NotSupported` at runtime.

### Features

Cargo features mirror FBInk's own `MINIMAL=1 DRAW=1 BITMAP=1 …` build flags (with FBInk's
implications); without any feature you get a true MINIMAL build:

| Feature | FBInk define | What it enables |
|---|---|---|
| `draw` | `FBINK_WITH_DRAW` | drawing primitives |
| `bitmap` | `FBINK_WITH_BITMAP` | fixed-cell text + base font (implies `draw`) |
| `fonts` | `FBINK_WITH_FONTS` | all bundled fonts (implies `bitmap`) |
| `unifont` | `FBINK_WITH_UNIFONT` | GNU Unifont (implies `fonts`, +2MB!) |
| `image` | `FBINK_WITH_IMAGE` | images & framebuffer dumps (implies `draw`) |
| `opentype` | `FBINK_WITH_OPENTYPE` | TrueType/OpenType text (implies `draw`) |
| `input` | `FBINK_WITH_INPUT` | input device scanning |
| `button-scan` | `FBINK_WITH_BUTTON_SCAN` | deprecated Kobo USBMS scanner (implies `draw`) |
| `serde` | — | `Serialize`/`Deserialize` on the public types |

Default = all except `unifont`, `button-scan` & `serde` (i.e. a standard full-featured FBInk).

The fieldless enums also implement `Display` & `FromStr` round-tripping their variant
names (`"Gc16"`, `"UnsciiAlt"`, …) plus `EnumIter`/`EnumCount` (via `strum`), and errors
implement `std::error::Error` via `thiserror`. With `serde` enabled, `Config` serializes
as a map of builder-method names with variant-name enums (`{"row": 2, "font": "Unscii"}`);
missing fields deserialize as `Config::new()` defaults.

### `FBINK_TARGET`

| Value | Device flavor |
|---|---|
| `linux` (default) | generic Linux fb (no eInk refresh support) |
| `kobo` | Kobo |
| `kindle` / `kindle-legacy` | Kindle (einkfb for the legacy flavor) |
| `cervantes` | Cervantes |
| `remarkable` | reMarkable |
| `pocketbook` | PocketBook |

This selects FBInk's *device* target, not the Rust target triple: the code only ever runs
on Linux.

## Compiling the raw-API example

`fbink-sys/examples/fbink-sys-demo.rs` is a small showcase of the raw API: init & state
query, text in several fonts, a grayscale ramp, an inverted rect, a progress bar, an eInk
refresh — and a clean exit that restores the screen to how it was found
(`fbink_dump`/`fbink_restore`).

### On the host

```sh
cargo build -p fbink-sys --example fbink-sys-demo
```

Set `FBINK_TARGET` to the platform you're building for (`linux`, `kobo`, `kindle`,
`kindle-legacy`, `cervantes`, `remarkable`, `pocketbook`); it defaults to `linux`, which
builds fine but has no eInk refresh support. Host builds are for compile-testing — the
binary needs a real eInk framebuffer to do anything useful.

### Cross-compiling for a Kindle

Kindles run ARMv7 hard-float Linux, so the recipe below builds a **static** binary
(musl) that runs on any of them. No cross-gcc or device sysroot needed —
[`zig`](https://ziglang.org) stands in for the C toolchain.

Prerequisites (once):

```sh
rustup target add armv7-unknown-linux-musleabihf
# and have `zig` in PATH
```

Then:

```sh
./scripts/build-demo-arm.sh              # or: ./scripts/build-demo-arm.sh --release
```

Output: `target/armv7-unknown-linux-musleabihf/<profile>/examples/fbink-sys-demo`

The script (and `scripts/zig-cc-arm`) take care of the fiddly parts:

- `zig cc -target arm-linux-musleabihf` as the C compiler for FBInk and as the linker
  (filters out the `--target=` flags cc-rs/rustc add, which zig's parser rejects);
- `BINDGEN_EXTRA_CLANG_ARGS` pointing at zig's musl include dirs, so bindgen's ARM parse
  of `fbink.h` doesn't trip over the host's glibc headers;
- `RUSTFLAGS=-C link-self-contained=no` — zig owns the C runtime (otherwise musl's
  `crt1.o` is linked twice);
- `FBINK_TARGET=kindle`.

For a Kobo (or any other flavor), build with `FBINK_TARGET=kobo` — same target triple.

Running the binary on a device or emulator (jailbroken Kindle, `scp` it over SSH, run it
on `/mnt/us`, …) is left as an exercise; note that on a stock Kindle `/mnt/us` is mounted
`noexec`.

## License

FBInk is [GPL-3.0-or-later](https://github.com/NiLuJe/FBInk/blob/master/LICENSE); statically
linking it (as `fbink-sys` does) makes the resulting binary subject to the same terms.