//! Bounce: a ball bouncing around the screen, the eInk way.
//!
//! A moving-object demo for the safe bindings: each frame erases the
//! previous ball, redraws it at its new position (a filled circle, one
//! scanline at a time), and refreshes the region the ball has swept
//! recently, in the fast DU waveform (light ghosting, never flashes).
//!
//! Why the *swept* region and not just old ∪ new: any submission rate above
//! the panel's waveform rate gets frames coalesced or dropped by the driver
//! (it displays the newest state of whatever survives). A per-frame dirty
//! rect only erases one ball position back, so a skipped frame's erase never
//! reaches the panel and its position stays painted black — long black
//! streaks along the path. Refreshing the whole recent sweep means every
//! processed update re-erases everything the skipped ones would have, no
//! matter which frames the driver keeps.
//!
//! Frames are paced against the wall clock, *never* against the panel:
//! fencing every update with [`Fbink::wait_for_complete`] would throttle the
//! loop to the waveform's completion rate, as that wait blocks until the
//! update has *fully* completed — longer than it stays visibly refreshing
//! (a DU update completes in a few hundred ms on most panels). Instead,
//! updates are submitted and immediately moved on from: the EPDC's
//! QUEUE_AND_MERGE scheme folds in-flight frames together, so the panel
//! always shows the newest ball position as fast as the waveform allows.
//! [`Fbink::wait_for_complete`] is only used to fence the *flashing*
//! refreshes at both ends of the run (draining in-flight frames before the
//! exit flash), per FBInk's guidance: blocking around a flashing update keeps
//! it from being affected by the surrounding ones.
//!
//! The screen is snapshotted ([`Fbink::dump`]) before the first pixel is
//! painted and restored ([`Fbink::restore`]) on exit, so the device's own
//! UI is exactly as we found it: it doesn't know it needs to redraw itself.
//!
//! Usage: `bounce [SECONDS] [FPS]` (defaults: 15 seconds at the built-in
//! [`FPS`] rate; 0 = run until Ctrl-C). Run it on a device with an eInk
//! framebuffer; host builds are for compile-testing only. See the README
//! for cross-compiling recipes.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use fbink::{Config, Fbink, Font, LAST_MARKER, Rect, WaveformMode};

/// Ball radius, in pixels.
const RADIUS: i32 = 60;
/// Default target frame rate (overridable via the `FPS` argument; wall-clock
/// paced — the panel shows as many distinct frames per second as its DU
/// waveform physically allows).
const FPS: u64 = 10;
/// Ball speed, in pixels per frame (x, y).
const SPEED: (f32, f32) = (19.0, 15.0);

fn main() -> fbink::Result<()> {
    let mut args = std::env::args().skip(1);
    let secs: u64 = args.next().and_then(|a| a.parse().ok()).unwrap_or(15);
    let fps: u64 = args.next().and_then(|a| a.parse().ok()).unwrap_or(FPS);
    let frames = if secs == 0 { u64::MAX } else { secs * fps };

    let fb = Fbink::open()?;
    // Init once (from the main thread!), keeping FBInk's setup chatter quiet.
    fb.init(&Config::new().quiet(true))?;

    // Snapshot the current screen (the device's own UI) before painting
    // anything: it gets restored on exit, because the UI won't redraw
    // itself just because we were here.
    let snapshot = match fb.dump() {
        Ok(dump) => Some(dump),
        Err(e) => {
            eprintln!("warning: dump failed ({e}); the screen will be cleared on exit");
            None
        }
    };

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
    let frame_budget = Duration::from_micros(1_000_000 / fps);
    // Ball boxes from the last half second: every update repaints their union,
    // so a processed frame re-erases whatever the skipped ones would have.
    // Half a second comfortably covers one waveform (~250-450ms) worth of
    // coalesced frames at any sane frame rate.
    let sweep = (fps / 2).clamp(4, 64) as usize;
    let mut trail: VecDeque<Rect> = VecDeque::with_capacity(sweep + 1);
    let mut done: u64 = 0;
    // Per-phase wall-clock accounting, to see where a frame actually goes.
    let (mut t_fill, mut t_submit, mut t_pace) = (Duration::ZERO, Duration::ZERO, Duration::ZERO);
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
        let t = Instant::now();
        if let Some(&prev) = trail.back() {
            fb.fill_rect_gray(&paint, Some(&prev), false, 0xFF)?;
        }
        let bbox = draw_ball(&fb, &paint, cx as i32, cy as i32)?;
        t_fill += t.elapsed();

        // Refresh the swept region (see the module docs): the union of the
        // ball's boxes over the last half second, in fast DU mode.
        trail.push_back(bbox);
        while trail.len() > sweep {
            trail.pop_front();
        }
        let dirty = trail.iter().copied().reduce(union).unwrap_or(bbox);
        let t = Instant::now();
        fb.refresh_rect(&dirty, &du)?;
        t_submit += t.elapsed();
        done += 1;

        // Pace against the wall clock. Never wait for the panel here: the
        // loop would drop to the waveform's *completion* rate (~3fps on many
        // devices), while submitted frames pile up nowhere — the EPDC merges
        // them (QUEUE_AND_MERGE) and paints the newest ball position at its
        // own pace.
        let t = Instant::now();
        if let Some(rest) = frame_budget.checked_sub(t0.elapsed()) {
            std::thread::sleep(rest);
        }
        t_pace += t.elapsed();
    }

    // Leave the screen exactly as we found it: repaint the UI snapshot.
    // `restore` honors the config's refresh flags, like `refresh`.
    let bye = Config::new().waveform(WaveformMode::Gc16);
    // Per FBInk's guidance, fence *before* a flashing update: drain in-flight
    // animation frames first, so the flash isn't affected by them. This is
    // also where the panel's backlog surfaces (the DU completion latency):
    // best-effort, as the restore below matters more than the fence.
    let t = Instant::now();
    if let Err(e) = fb.wait_for_complete(LAST_MARKER) {
        eprintln!("warning: wait_for_complete failed ({e}); restore may pick up stray frames");
    }
    let drain = t.elapsed();
    match &snapshot {
        Some(dump) => fb.restore(&bye, dump)?,
        // Fallback when there is no snapshot: blank screen + refresh.
        None => {
            fb.cls(&Config::new().no_refresh(true), None, false)?;
            fb.refresh(0, 0, 0, 0, &bye)?;
        }
    }
    fb.wait_for_complete(LAST_MARKER)?;

    let elapsed = start.elapsed();
    println!(
        "bounced {} frames in {:?} (~{:.1} fps)",
        done,
        elapsed,
        done as f64 / elapsed.as_secs_f64()
    );
    if done > 0 {
        let per = |d: Duration| d.as_millis() as f64 / done as f64;
        println!(
            "  per frame: fill {:.1}ms, submit {:.1}ms, pacing {:.1}ms",
            per(t_fill),
            per(t_submit),
            per(t_pace)
        );
    }
    println!(
        "  panel backlog drained before restore: {}ms",
        drain.as_millis()
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
