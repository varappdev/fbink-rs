//! Hello world: open the framebuffer, print, refresh & wait.
//!
//! Run it on a device with an eInk framebuffer; host builds are for
//! compile-testing only. See the README for cross-compiling recipes.

use fbink::{Config, Fbink, Font, LAST_MARKER, WaveformMode};

fn main() -> fbink::Result<()> {
    let fb = Fbink::open()?;

    // Init once (from the main thread!), keeping FBInk's setup chatter quiet.
    fb.init(&Config::new().quiet(true))?;

    // A first line, in the default font & position.
    fb.print("Hello from Rust!", &Config::new())?;

    // And a bigger, centered one, two rows below.
    fb.print(
        "fbink-rs",
        &Config::new()
            .row(2)
            .centered(true)
            .font(Font::Unscii)
            .fontmult(2),
    )?;

    // Full-screen refresh (0x0 @ (0, 0) means the whole screen), then wait
    // for it to actually hit the glass before exiting.
    fb.refresh(
        0,
        0,
        0,
        0,
        &Config::new().waveform(WaveformMode::Gc16).flashing(true),
    )?;
    fb.wait_for_complete(LAST_MARKER)?;

    Ok(())
}
