//! Views. Every function is a pure read of [`App`], except the handlers it attaches.

mod dialogs;
mod notices;
mod people;
mod samples;
mod shell;
mod widgets;

use rune::{Cx, El, icon};
use rune_kit::{Kit, Toast, ToastRegion};

use crate::app::{App, Tone};
use crate::theme;

/// The window's last laid-out size. Breakpoints and dialog widths read it; a resize re-runs the
/// view through the layout read, so the shell can swap to the phone layout without a rebuild.
#[derive(Clone, Copy, Debug)]
pub struct Viewport {
    pub width: f32,
    pub height: f32,
}

impl Viewport {
    pub const DESKTOP: Viewport = Viewport {
        width: 1360.0,
        height: 860.0,
    };

    pub fn mobile(self) -> bool {
        self.width < 1000.0
    }

    pub fn pad(self) -> f32 {
        if self.mobile() { 14.0 } else { 26.0 }
    }

    /// A dialog width that leaves a margin on phones and keeps the desktop panels calm.
    pub fn dialog_width(self, desktop: f32) -> f32 {
        if self.mobile() {
            (self.width - 24.0).clamp(300.0, desktop)
        } else {
            desktop
        }
    }

    pub fn dialog_body_height(self) -> f32 {
        (self.height - 190.0).max(260.0)
    }
}

pub fn view(s: &App, cx: &mut Cx<App>) -> El {
    let dark = s.data.dark.unwrap_or_else(|| cx.system_dark());
    let k = theme::kit(dark);
    let size = cx.layout_size("window");
    let vp = match size {
        Some([width, height]) => Viewport { width, height },
        None => Viewport::DESKTOP,
    };
    let mut layer = Vec::new();
    let body = shell::shell(s, k, &mut layer, vp);
    dialogs::build(s, k, &mut layer, vp);
    if let Some(t) = &s.toast {
        layer.push(toast(k, t));
    }
    rune_kit::root(body, layer)
}

fn toast(k: &Kit, t: &crate::app::ToastNote) -> El {
    let (tint, glyph) = match t.tone {
        Tone::Success => (k.c.success.fg, &crate::icons::CHECK),
        Tone::Info => (k.c.text_accent, &crate::icons::BELL),
        Tone::Warning => (k.c.warning.fg, &crate::icons::CLOCK),
    };
    let toast = Toast::new("app-toast", t.title.clone())
        .description(t.body.clone())
        .lead(icon(glyph, 16.0).color(tint))
        .closable()
        .on_close(|s: &mut App| s.dismiss_toast())
        .build(k);
    ToastRegion::new().toast(toast).build(k)
}
