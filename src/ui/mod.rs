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

pub fn view(s: &App, cx: &mut Cx<App>) -> El {
    let dark = s.data.dark.unwrap_or_else(|| cx.system_dark());
    let k = theme::kit(dark);
    let mut layer = Vec::new();
    let body = shell::shell(s, k, &mut layer);
    dialogs::build(s, k, &mut layer);
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
