//! The Arunika mascot on the resting pill: a little angry fire keeping the
//! clock company, its pose reading the island's state (plugged in, an update
//! waiting, a focus timer running, music playing, the small hours).
//!
//! The art ships in the exe; each pose decodes once, lazily, through
//! [`crate::png`], and the cached bitmap is reused after. The flame motion
//! only moves and scales the draw rect — nothing allocates per frame, and no
//! timer or thread starts here: the resting pill's ambient frames, the same
//! `st.now` clock the equalizer answers to, drive it. It stands still on
//! battery saver, as the equalizer's bars do, and the app loops already park
//! ambient frames while the session is locked, the display is off or the
//! island is hidden; expanded, the island draws no resting pill, so nothing
//! moves there either.

use crate::gfx::{Image, ImageData};
use crate::state::AppState;
use crate::ui::Ui;
use poni_core::geom::{Point, Rect};
use poni_core::settings::IdleStyle;
use std::sync::OnceLock;

/// The mascot rests this tall on the pill (DIPs); its width follows the art.
pub const H: f32 = 21.0;
/// Between the clock (or the pill's right-hand content) and the mascot.
pub const GAP: f32 = 6.0;
/// The flame: a slow vertical bob and a subtler scale flicker, one sine family.
const BOB_DIP: f32 = 1.0;
const FLICKER: f32 = 0.03;
const FLAME_HZ: f64 = 1.2;
const FLICKER_PHASE: f64 = 1.3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pose {
    Angry = 0,
    Sleepy = 1,
    Laughing = 2,
    Focused = 3,
    Charging = 4,
    Waving = 5,
}

/// The pose's PNG, straight from the exe.
fn bytes(p: Pose) -> &'static [u8] {
    match p {
        Pose::Angry => include_bytes!("../res/mascot/01_angry.png"),
        Pose::Sleepy => include_bytes!("../res/mascot/07_sleepy.png"),
        Pose::Laughing => include_bytes!("../res/mascot/09_laughing_tears.png"),
        Pose::Focused => include_bytes!("../res/mascot/13_focused_angry.png"),
        Pose::Charging => include_bytes!("../res/mascot/16_charging.png"),
        Pose::Waving => include_bytes!("../res/mascot/20_waving.png"),
    }
}

/// What the island's state says the mascot feels, most urgent first: plugged
/// in, an update waiting, a focus timer running, music playing, the small
/// hours — otherwise its resting grumpy self.
fn pose(st: &AppState) -> Pose {
    let p = &st.power;
    if p.has_battery && (p.charging || p.on_ac) {
        // Plugged in counts, as the resting battery has it (a full battery
        // on its charger keeps the bolt).
        Pose::Charging
    } else if st.update.offer.is_some() || st.update.ready {
        // The state the update notice announces from.
        Pose::Waving
    } else if st.pomodoro.is_active() {
        Pose::Focused
    } else if st.media.is_some() {
        // A session with something playing or paused.
        Pose::Laughing
    } else if st.local.hour >= 23 || st.local.hour < 5 {
        Pose::Sleepy
    } else {
        Pose::Angry
    }
}

/// The mascot keeps the clock company in the looks that rest a clock; the
/// media activity and the other live surfaces have their own content, and
/// there it stays out of the way.
pub fn shows(st: &AppState) -> bool {
    matches!(st.settings.idle_style, IdleStyle::Clock | IdleStyle::Status)
}

/// The pose decoded for drawing: each pose decodes exactly once, here, and
/// later frames reuse the cached bitmap (an `Arc` clone, no pixels copied).
fn image(p: Pose) -> Option<Image> {
    static CACHE: [OnceLock<Option<Image>>; 6] =
        [OnceLock::new(), OnceLock::new(), OnceLock::new(), OnceLock::new(), OnceLock::new(), OnceLock::new()];
    CACHE[p as usize].get_or_init(|| decode(p)).clone()
}

/// PNG (straight RGBA) → the renderer's premultiplied BGRA.
fn decode(p: Pose) -> Option<Image> {
    let d = crate::png::decode(bytes(p)).ok()?;
    let mut px = Vec::with_capacity(d.pixels.len());
    for c in d.pixels.chunks_exact(4) {
        let (r, g, b, a) = (c[0] as u32, c[1] as u32, c[2] as u32, c[3] as u32);
        px.extend_from_slice(&[((b * a + 127) / 255) as u8, ((g * a + 127) / 255) as u8, ((r * a + 127) / 255) as u8, a as u8]);
    }
    Some(ImageData::new(d.width, d.height, px))
}

/// The mascot's resting width for this state: 0 where it doesn't show or the
/// art wouldn't decode. The layout pass and the drawing agree through this.
pub fn w(st: &AppState) -> f32 {
    if !shows(st) {
        return 0.0;
    }
    image(pose(st)).map_or(0.0, |i| H * i.width as f32 / i.height as f32)
}

/// The pill grows by this much next to its clock while the mascot shows.
pub fn slot(st: &AppState) -> f32 {
    let w = w(st);
    if w > 0.0 { GAP + w } else { 0.0 }
}

/// Whether the flame moves right now: the equalizer's own rule — nothing
/// moves on battery saver. (Locked, display off and a hidden island park
/// ambient frames in the app loops, [`crate::activities::is_live`]'s caller.)
pub fn animates(st: &AppState) -> bool {
    shows(st) && !st.power.saver
}

/// Draw the mascot's current pose into its resting box `r` (`w` x [`H`]),
/// centred, the flame motion derived from `now`.
pub fn draw(ui: &mut Ui, st: &AppState, r: Rect) {
    let Some(img) = image(pose(st)) else { return };
    let wave = std::f64::consts::TAU * FLAME_HZ * st.now;
    let bob = BOB_DIP * wave.sin() as f32;
    let s = 1.0 + FLICKER * (wave + FLICKER_PHASE).sin() as f32;
    let c = Point::new(r.center().x, r.center().y + bob);
    ui.c.image(&img, Rect::from_center(c, r.w * s, r.h * s), 0.0);
}
