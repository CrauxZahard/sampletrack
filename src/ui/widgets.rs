//! Shared visual primitives: warm paper surfaces, square monograms, scope toggles and the
//! typographic scale. No pills, no tinted icon tiles; hierarchy comes from type and rules.

use rune::{Color, El, Icon, TextStyle, col, el, icon, row, text};
use rune_kit::Kit;

use crate::app::{App, Page};
use crate::model::{Scopes, Status, User};

pub fn ui(k: &Kit) -> TextStyle {
    TextStyle {
        size: 13.0,
        weight: 450.0,
        line_height: 20.0,
        ..k.ty.ui
    }
}

pub fn ui_medium(k: &Kit) -> TextStyle {
    TextStyle {
        size: 13.0,
        weight: 560.0,
        line_height: 19.0,
        ..k.ty.ui
    }
}

pub fn small(k: &Kit) -> TextStyle {
    TextStyle {
        size: 11.5,
        weight: 450.0,
        line_height: 17.0,
        ..k.ty.small
    }
}

pub fn caption(k: &Kit) -> TextStyle {
    TextStyle {
        size: 10.5,
        weight: 450.0,
        line_height: 15.0,
        ..k.ty.small
    }
}

/// Uppercase micro-labels: column heads, eyebrows, scope tags.
pub fn micro(k: &Kit) -> TextStyle {
    TextStyle {
        size: 9.5,
        weight: 620.0,
        line_height: 13.0,
        tracking: 0.9,
        ..k.ty.small
    }
}

pub fn heading(k: &Kit) -> TextStyle {
    TextStyle {
        size: 25.0,
        weight: 610.0,
        line_height: 31.0,
        tracking: -0.55,
        ..k.ty.ui
    }
}

pub fn number(k: &Kit) -> TextStyle {
    TextStyle {
        size: 30.0,
        weight: 590.0,
        line_height: 34.0,
        tracking: -0.9,
        ..k.ty.ui
    }
}

pub fn mono(k: &Kit) -> TextStyle {
    TextStyle {
        size: 11.0,
        weight: 450.0,
        line_height: 17.0,
        ..k.ty.mono
    }
}

pub fn mono_small(k: &Kit) -> TextStyle {
    TextStyle {
        size: 9.5,
        weight: 450.0,
        line_height: 13.0,
        ..k.ty.mono
    }
}

pub fn accent_soft(k: &Kit) -> Color {
    k.alpha(k.c.text_accent, if k.dark { 0.14 } else { 0.08 })
}

pub fn hairline(k: &Kit) -> Color {
    k.c.border_weak
}

/// A square monogram. Identity comes from the name beside it, not a rainbow of tints.
pub fn avatar(k: &Kit, user: &User, size: f32) -> El {
    let ink = if user.scopes.is_admin() {
        k.c.text_accent
    } else {
        k.c.text_muted
    };
    el().size(size, size)
        .radius(2.0)
        .center()
        .fill(k.c.layer_02)
        .border(0.5, k.c.border_base)
        .a11y_hidden()
        .child(
            text(user.initials())
                .font(TextStyle {
                    size: (size * 0.34).max(8.5),
                    weight: 600.0,
                    line_height: size,
                    ..k.ty.mono
                })
                .color(ink),
        )
}

pub fn status_colors(k: &Kit, status: Status) -> (Color, Color) {
    match status {
        Status::Unanalyzed => (k.c.layer_02, k.c.text_muted),
        Status::Approved => (k.c.success.bg, k.c.success.fg),
        Status::ApprovedHuds => (k.c.success.bg, k.c.success.fg),
        Status::WaitingHuds => (k.c.warning.bg, k.c.warning.fg),
        Status::Resampling => (k.c.danger.bg, k.c.danger.fg),
        Status::Resampled => (k.c.layer_02, k.c.text_accent),
        Status::NotQualified => (k.c.danger.bg, k.c.danger.fg),
    }
}

/// A square status mark and a micro label. No filled pill; the list stays a ledger.
pub fn status_mark(k: &Kit, status: Status) -> El {
    let (_, fg) = status_colors(k, status);
    row()
        .h(20.0)
        .gap(6.0)
        .center_y()
        .role("status")
        .label(status.label())
        .child(el().size(6.0, 6.0).fill(fg).a11y_hidden())
        .child(
            text(status.label())
                .font(TextStyle {
                    size: 10.0,
                    weight: 560.0,
                    line_height: 14.0,
                    tracking: 0.1,
                    ..k.ty.ui
                })
                .color(fg)
                .a11y_hidden(),
        )
}

/// A phone-sized status mark: the same colour signal with a label that fits beside a product
/// name. The full label stays in the row's accessibility label.
pub fn status_mark_compact(k: &Kit, status: Status) -> El {
    let (_, fg) = status_colors(k, status);
    row()
        .h(18.0)
        .gap(5.0)
        .center_y()
        .role("status")
        .label(status.label())
        .child(el().size(5.0, 5.0).fill(fg).a11y_hidden())
        .child(
            text(short_status(status))
                .font(TextStyle {
                    size: 9.5,
                    weight: 560.0,
                    line_height: 12.0,
                    ..k.ty.ui
                })
                .color(fg),
        )
}

pub fn short_status(status: Status) -> &'static str {
    match status {
        Status::Unanalyzed => "Belum",
        Status::Approved => "Approved",
        Status::ApprovedHuds => "Appr. HUDS",
        Status::WaitingHuds => "Menunggu",
        Status::Resampling => "Resample",
        Status::Resampled => "Sudah",
        Status::NotQualified => "Tidak lolos",
    }
}

pub fn nav_item(
    k: &Kit,
    key: &str,
    glyph: &'static Icon,
    label: &str,
    page: Page,
    active: bool,
    count: Option<usize>,
) -> El {
    let ink = if active {
        k.c.text_base
    } else {
        k.c.text_muted
    };
    let icon_ink = if active {
        k.c.text_accent
    } else {
        k.c.icon_faint
    };
    let mut r = row()
        .key(key)
        .role("button")
        .label(label)
        .focusable(true)
        .h(34.0)
        .w_full()
        .px(8.0)
        .gap(9.0)
        .center_y()
        .radius(2.0)
        .fill(if active {
            k.c.layer_02
        } else {
            Color::TRANSPARENT
        })
        .hover(k.c.hover)
        .press(k.c.pressed)
        .focus_ring(2.0, 1.0, k.c.border_focus)
        .cursor(rune::Cursor::Pointer)
        .on_press(move |s: &mut App| s.page = page)
        .child(el().size(2.0, 15.0).fill(if active {
            k.c.text_accent
        } else {
            Color::TRANSPARENT
        }))
        .child(icon(glyph, 14.0).color(icon_ink).a11y_hidden())
        .child(text(label).font(ui(k)).color(ink).a11y_hidden());
    r = r.child(el().grow(1.0));
    if let Some(n) = count.filter(|n| *n > 0) {
        r = r.child(count_tag(k, n, active));
    }
    r
}

/// A small numeric square. Used for unread counts and remark counts.
pub fn count_tag(k: &Kit, n: usize, filled: bool) -> El {
    let (bg, ink, border) = if filled {
        (k.c.text_accent, k.c.text_contrast, Color::TRANSPARENT)
    } else {
        (Color::TRANSPARENT, k.c.text_muted, k.c.border_strong)
    };
    el().h(16.0)
        .min_w(16.0)
        .px(4.0)
        .radius(1.0)
        .center()
        .fill(bg)
        .border(0.5, border)
        .a11y_hidden()
        .child(
            text(n.to_string())
                .font(TextStyle {
                    size: 9.5,
                    weight: 600.0,
                    line_height: 13.0,
                    ..k.ty.mono
                })
                .color(ink),
        )
}

pub fn icon_badge(k: &Kit, glyph: &'static Icon, size: f32) -> El {
    el().size(size, size)
        .radius(2.0)
        .center()
        .fill(k.c.layer_02)
        .border(0.5, k.c.border_base)
        .a11y_hidden()
        .child(icon(glyph, size * 0.48).color(k.c.icon_muted))
}

pub fn field(k: &Kit, label: &str, input: El, hint: Option<&str>) -> El {
    let mut c = col().w_full().gap(6.0).child(
        text(label.to_uppercase())
            .font(micro(k))
            .color(k.c.text_faint),
    );
    c = c.child(input);
    if let Some(h) = hint {
        c = c.child(text(h).font(caption(k)).color(k.c.text_faint));
    }
    c
}

/// An editable scope toggle: a square check and an uppercase label.
pub fn scope_toggle(
    k: &Kit,
    label: &str,
    checked: bool,
    enabled: bool,
    on_press: impl Fn(&mut App) + 'static,
) -> El {
    let ink = if checked {
        k.c.text_base
    } else {
        k.c.text_muted
    };
    let mut e = row()
        .h(25.0)
        .px(6.0)
        .gap(5.0)
        .center_y()
        .radius(1.0)
        .fill(if checked {
            accent_soft(k)
        } else {
            Color::TRANSPARENT
        })
        .border(
            0.5,
            if checked {
                k.alpha(k.c.text_accent, 0.5)
            } else {
                k.c.border_base
            },
        )
        .role("checkbox")
        .label(label.to_owned())
        .checked(checked)
        .focusable(enabled)
        .child(
            el().size(11.0, 11.0)
                .radius(1.0)
                .center()
                .fill(if checked {
                    k.c.text_accent
                } else {
                    Color::TRANSPARENT
                })
                .border(
                    1.0,
                    if checked {
                        Color::TRANSPARENT
                    } else {
                        k.c.border_strong
                    },
                )
                .a11y_hidden()
                .child(if checked {
                    icon(&rune_kit::glyphs::CHECK, 8.0).color(k.c.text_contrast)
                } else {
                    el()
                }),
        )
        .child(
            text(label.to_uppercase())
                .font(micro(k))
                .color(ink)
                .a11y_hidden(),
        );
    if enabled {
        e = e
            .hover(k.c.hover)
            .press(k.c.pressed)
            .focus_ring(2.0, 1.0, k.c.border_focus)
            .cursor(rune::Cursor::Pointer)
            .on_press(on_press);
    } else {
        e = e.layer_opacity(0.4);
    }
    e
}

/// A read-only line summarising a person's scopes.
pub fn scope_summary(k: &Kit, scopes: Scopes) -> El {
    let tint = if scopes.is_admin() {
        k.c.text_accent
    } else {
        k.c.text_muted
    };
    text(scopes.summary()).font(caption(k)).color(tint)
}
