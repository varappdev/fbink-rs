//! Bounce: a ball bouncing around the screen, the eInk way.
//!
//! A moving-object demo for the safe bindings: each frame erases the
//! previous ball, redraws it at its new position (a filled circle, one
//! scanline at a time), and refreshes only the dirty region, in the fast
//! DU waveform (light ghosting, never flashes), paced by
//! [`Fbink::wait_for_complete`].
//!
//! Usage: `bounce [SECONDS]` (default 15; 0 = run until Ctrl-C).
//! Run it on a device with an eInk framebuffer; host builds are for
//! compile-testing only. See the README for cross-compiling recipes.

use std::time::{Duration, Instant};

use fbink::{Config, Fbink, Font, Rect, WaveformMode, LAST_MARKER};

/// Ball radius, in pixels.
const RADIUS: i32 = 60;
/// Target frame rate (eInk DU updates top out around 15-20 fps anyway).
const FPS: u64 = 20;
/// Ball speed, in pixels per frame (x, y).
const SPEED: (f32, f32) = (19.0, 15.0);

fn main() -> fbink::Result<()> {
    let secs: u64 = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(15);
    let frames = if secs == 0 { u64::MAX } else { secs * FPS };

    let fb = Fbink::open()?;
    // Init once (from the main thread!), keeping FBInk's setup chatter quiet.
    fb.init(&Config::new().quiet(true))?;

    // Start from a clean slate: clear, then caption; the ball plays below it.
    // `get_last_rect` tells us where the printed line actually landed.
    fb.cls(&Config::new().no_refresh(true), None, false)?;
    let title = Config::new().row(0).centered(true).font(Font::Unscii);
    fb.print("fbink-rs bounce demo", &title)?;
    fb.refresh(
        0,
        0,
        0,
        0,
        &Config::new().waveform(WaveformMode::Gc16).flashing(true),
    )?;
    fb.wait_for_complete(LAST_MARKER)?;
    let caption = fbink::get_last_rect(false);

    // Playfield: the full view, minus the caption row.
    let st = fb.state(&Config::new());
    let (w, h) = (st.view_width as i32, st.view_height as i32);
    let top = i32::from(caption.top) + i32::from(caption.height) + RADIUS;
    let top = if top + RADIUS >= h { 0 } else { top }; // tiny screens

    let r = RADIUS as f32;
    let (mut cx, mut cy) = (w as f32 / 3.0, (top + h) as f32 / 2.0);
    let (mut vx, mut vy) = SPEED;

    // `no_refresh`: fills are pure pixel writes; we refresh the dirty region
    // ourselves (and per-scanline refreshes would be rejected as too small).
    let paint = Config::new().no_refresh(true);
    let du = Config::new().waveform(WaveformMode::Du);
    let frame_budget = Duration::from_micros(1_000_000 / FPS);
    let mut ball: Option<Rect> = None;
    let mut done: u64 = 0;
    let start = Instant::now();

    for _ in 0..frames {
        let t0 = Instant::now();

        // Bounce off the playfield edges.
        cx += vx;
        cy += vy;
        if cx < r {
            cx = r;
            vx = -vx;
        } else if cx > w as f32 - r {
            cx = w as f32 - r;
            vx = -vx;
        }
        if cy < top as f32 + r {
            cy = top as f32 + r;
            vy = -vy;
        } else if cy > h as f32 - r {
            cy = h as f32 - r;
            vy = -vy;
        }

        // Erase the previous ball (white), then draw the new one (black).
        if let Some(prev) = ball {
            fb.fill_rect_gray(&paint, Some(&prev), false, 0xFF)?;
        }
        let bbox = draw_ball(&fb, &paint, cx as i32, cy as i32)?;

        // Refresh just the old ∪ new region, in fast DU mode.
        let dirty = match ball.replace(bbox) {
            Some(prev) => union(prev, bbox),
            None => bbox,
        };
        fb.refresh_rect(&dirty, &du)?;
        fb.wait_for_complete(LAST_MARKER)?;
        done += 1;

        // Pace the animation (wait_for_complete already blocks a while).
        if let Some(rest) = frame_budget.checked_sub(t0.elapsed()) {
            std::thread::sleep(rest);
        }
    }

    // Clear the screen & do a full flashing refresh to flush any ghosting.
    fb.cls(&Config::new().no_refresh(true), None, false)?;
    fb.print("demo done!", &title)?;
    fb.refresh(
        0,
        0,
        0,
        0,
        &Config::new().waveform(WaveformMode::Gc16).flashing(true),
    )?;
    fb.wait_for_complete(LAST_MARKER)?;

    println!(
        "bounced {} frames in {:?} (~{} fps)",
        done,
        start.elapsed(),
        FPS
    );
    Ok(())
}

/// Draw a filled circle at (cx, cy) and return its bounding box.
fn draw_ball(fb: &Fbink, cfg: &Config, cx: i32, cy: i32) -> fbink::Result<Rect> {
    for dy in -RADIUS..=RADIUS {
        // Scanline half-width, from the circle equation.
        let hw = ((RADIUS * RADIUS - dy * dy) as f32).sqrt() as i32;
        if hw < 1 {
            continue;
        }
        let scanline = Rect::new((cx - hw) as u16, (cy + dy) as u16, (hw * 2) as u16, 1);
        fb.fill_rect_gray(cfg, Some(&scanline), false, 0x00)?;
    }
    let d = (RADIUS * 2) as u16;
    Ok(Rect::new((cx - RADIUS) as u16, (cy - RADIUS) as u16, d, d))
}

/// The smallest [`Rect`] containing both `a` and `b`.
fn union(a: Rect, b: Rect) -> Rect {
    let left = a.left.min(b.left);
    let top = a.top.min(b.top);
    let right = (a.left as u32 + a.width as u32).max(b.left as u32 + b.width as u32);
    let bottom = (a.top as u32 + a.height as u32).max(b.top as u32 + b.height as u32);
    Rect::new(
        left,
        top,
        (right - left as u32) as u16,
        (bottom - top as u32) as u16,
    )
}
