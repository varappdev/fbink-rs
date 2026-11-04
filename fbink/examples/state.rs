//! Dump every FBInk `State` field: device identification & quirks, screen &
//! font geometry, pen colors...
//!
//! Run it on a device to see real values; on a host without a framebuffer,
//! `Fbink::open()` simply fails.

use fbink::{Config, Fbink};

fn main() -> fbink::Result<()> {
    let fb = Fbink::open()?;
    fb.init(&Config::new().quiet(true))?;

    let state = fb.state(&Config::new());
    println!("{state:#?}");

    // The same thing, in FBInk's shell-consumable format:
    eprintln!("--- fbink_state_dump() ---");
    fb.state_dump(&Config::new());

    Ok(())
}
