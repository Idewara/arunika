//! The idle Eyes look: the reference video's eyes, played back frame by frame
//! on the closed island.
//!
//! The 60 frames ship in the exe; each decodes once, lazily, through
//! [`crate::png`], and the cached bitmap is reused after (an `Arc` clone, no
//! pixels copied, nothing allocated per frame). The frame shown is a pure
//! function of `st.now` at 15 fps — the same clock the equalizer answers to —
//! so playback never drifts and no timer or thread starts here. It stands
//! still on battery saver, as the equalizer's bars do, and the app loops
//! already park ambient frames while the session is locked or the display is
//! off; expanded, the island draws no resting pill, so nothing moves there
//! either. The frames' black background is part of the art: it sits on the
//! black pill, drawn unrounded.

use crate::gfx::{Image, ImageData};
use crate::state::AppState;
use crate::ui::Ui;
use poni_core::geom::Rect;
use poni_core::settings::IdleStyle;
use std::sync::OnceLock;

/// The reference video: 60 frames, one 4-second loop.
pub const FPS: f64 = 15.0;
pub const FRAMES: usize = 60;
/// The frames' native size; the drawn rect keeps this aspect.
const FRAME_W: f32 = 280.0;
const FRAME_H: f32 = 79.0;

const EMPTY: OnceLock<Option<Image>> = OnceLock::new();
static CACHE: [OnceLock<Option<Image>>; FRAMES] = [EMPTY; FRAMES];

/// Frame `i`'s PNG, straight from the exe.
fn bytes(i: usize) -> &'static [u8] {
    match i {
        0 => include_bytes!("../res/eyes/frame_00.png"),
        1 => include_bytes!("../res/eyes/frame_01.png"),
        2 => include_bytes!("../res/eyes/frame_02.png"),
        3 => include_bytes!("../res/eyes/frame_03.png"),
        4 => include_bytes!("../res/eyes/frame_04.png"),
        5 => include_bytes!("../res/eyes/frame_05.png"),
        6 => include_bytes!("../res/eyes/frame_06.png"),
        7 => include_bytes!("../res/eyes/frame_07.png"),
        8 => include_bytes!("../res/eyes/frame_08.png"),
        9 => include_bytes!("../res/eyes/frame_09.png"),
        10 => include_bytes!("../res/eyes/frame_10.png"),
        11 => include_bytes!("../res/eyes/frame_11.png"),
        12 => include_bytes!("../res/eyes/frame_12.png"),
        13 => include_bytes!("../res/eyes/frame_13.png"),
        14 => include_bytes!("../res/eyes/frame_14.png"),
        15 => include_bytes!("../res/eyes/frame_15.png"),
        16 => include_bytes!("../res/eyes/frame_16.png"),
        17 => include_bytes!("../res/eyes/frame_17.png"),
        18 => include_bytes!("../res/eyes/frame_18.png"),
        19 => include_bytes!("../res/eyes/frame_19.png"),
        20 => include_bytes!("../res/eyes/frame_20.png"),
        21 => include_bytes!("../res/eyes/frame_21.png"),
        22 => include_bytes!("../res/eyes/frame_22.png"),
        23 => include_bytes!("../res/eyes/frame_23.png"),
        24 => include_bytes!("../res/eyes/frame_24.png"),
        25 => include_bytes!("../res/eyes/frame_25.png"),
        26 => include_bytes!("../res/eyes/frame_26.png"),
        27 => include_bytes!("../res/eyes/frame_27.png"),
        28 => include_bytes!("../res/eyes/frame_28.png"),
        29 => include_bytes!("../res/eyes/frame_29.png"),
        30 => include_bytes!("../res/eyes/frame_30.png"),
        31 => include_bytes!("../res/eyes/frame_31.png"),
        32 => include_bytes!("../res/eyes/frame_32.png"),
        33 => include_bytes!("../res/eyes/frame_33.png"),
        34 => include_bytes!("../res/eyes/frame_34.png"),
        35 => include_bytes!("../res/eyes/frame_35.png"),
        36 => include_bytes!("../res/eyes/frame_36.png"),
        37 => include_bytes!("../res/eyes/frame_37.png"),
        38 => include_bytes!("../res/eyes/frame_38.png"),
        39 => include_bytes!("../res/eyes/frame_39.png"),
        40 => include_bytes!("../res/eyes/frame_40.png"),
        41 => include_bytes!("../res/eyes/frame_41.png"),
        42 => include_bytes!("../res/eyes/frame_42.png"),
        43 => include_bytes!("../res/eyes/frame_43.png"),
        44 => include_bytes!("../res/eyes/frame_44.png"),
        45 => include_bytes!("../res/eyes/frame_45.png"),
        46 => include_bytes!("../res/eyes/frame_46.png"),
        47 => include_bytes!("../res/eyes/frame_47.png"),
        48 => include_bytes!("../res/eyes/frame_48.png"),
        49 => include_bytes!("../res/eyes/frame_49.png"),
        50 => include_bytes!("../res/eyes/frame_50.png"),
        51 => include_bytes!("../res/eyes/frame_51.png"),
        52 => include_bytes!("../res/eyes/frame_52.png"),
        53 => include_bytes!("../res/eyes/frame_53.png"),
        54 => include_bytes!("../res/eyes/frame_54.png"),
        55 => include_bytes!("../res/eyes/frame_55.png"),
        56 => include_bytes!("../res/eyes/frame_56.png"),
        57 => include_bytes!("../res/eyes/frame_57.png"),
        58 => include_bytes!("../res/eyes/frame_58.png"),
        59 => include_bytes!("../res/eyes/frame_59.png"),
        _ => unreachable!("frame index is taken mod FRAMES"),
    }
}

/// The frame to show at `now` (mono seconds): frame 0 at boot, then 15 a
/// second, wrapping every 4 s. Negative times (not really reachable) clamp to
/// frame 0, `as usize` saturates.
pub fn frame_index(now: f64) -> usize {
    (now * FPS) as usize % FRAMES
}

/// Frame `i` decoded for drawing: each frame decodes exactly once, here, and
/// later frames reuse the cached bitmap. Frame 0 decodes on first draw.
fn frame(i: usize) -> Option<Image> {
    CACHE[i].get_or_init(|| decode(bytes(i))).clone()
}

/// PNG (straight RGBA) → the renderer's premultiplied BGRA.
fn decode(png: &[u8]) -> Option<Image> {
    let d = crate::png::decode(png).ok()?;
    let mut px = Vec::with_capacity(d.pixels.len());
    for c in d.pixels.chunks_exact(4) {
        let (r, g, b, a) = (c[0] as u32, c[1] as u32, c[2] as u32, c[3] as u32);
        px.extend_from_slice(&[((b * a + 127) / 255) as u8, ((g * a + 127) / 255) as u8, ((r * a + 127) / 255) as u8, a as u8]);
    }
    Some(ImageData::new(d.width, d.height, px))
}

/// Whether the eyes' frames advance right now: the eyes look is what's
/// showing and nothing moves on battery saver — the equalizer's own rule.
/// (Locked, display off and a hidden island park ambient frames in the app
/// loops, [`crate::activities::is_live`]'s caller.)
pub fn animates(st: &AppState) -> bool {
    crate::activities::idle_look(st) == IdleStyle::Eyes && !st.power.saver
}

/// Draw the current frame into the eyes' slot `rect`, fit inside it at the
/// frames' 280:79 aspect, centred. The frame's black blends into the black
/// pill. Rounded like the capsule so the art's square corners never poke
/// past the pill's transparent ends.
pub fn draw(ui: &mut Ui, st: &AppState, rect: Rect) {
    let Some(img) = frame(frame_index(st.now)) else { return };
    let s = (rect.w / FRAME_W).min(rect.h / FRAME_H);
    let r = Rect::from_center(rect.center(), FRAME_W * s, FRAME_H * s);
    ui.c.image(&img, r, r.h / 2.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_index_steps_at_15_fps_and_wraps_each_loop() {
        assert_eq!(frame_index(0.0), 0, "frame 0 first");
        assert_eq!(frame_index(1.0), 15);
        assert_eq!(frame_index(4.0), 0, "the 4-second loop wraps to frame 0");
        assert_eq!(frame_index(4.0 - 1.0 / FPS), FRAMES - 1, "last frame before the wrap");
        assert_eq!(frame_index(5.0), 15, "later loops keep stepping");
        // A moment before the loop seam and the seam itself are neighbours.
        assert_eq!(frame_index(3.999), 59);
        assert_eq!(frame_index(4.001), 0);
        // Negative times clamp to frame 0 (the `as usize` saturation).
        assert_eq!(frame_index(-1.0), 0);
    }
}
