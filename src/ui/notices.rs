//! The inbox: a flat ledger of sample updates with filters for the queues readers care about.

use rune::{Color, El, TextStyle, col, el, icon, row, text};
use rune_kit::{Button, ButtonVariant, ControlSize, Kit};

use crate::app::{App, NoticeFilter};
use crate::icons;
use crate::model::{Notice, Stage, ago};
use crate::ui::Viewport;
use crate::ui::shell::rule;
use crate::ui::widgets::{
    avatar, caption, icon_badge, micro, mono, mono_small, small, status_colors, status_mark,
    status_mark_compact, ui_medium,
};

pub fn page(s: &App, k: &Kit, vp: Viewport) -> El {
    let rows = s.notices();
    let counts = s.notice_counts();
    let now = s.now();
    let total: usize = counts[0];

    let mut tabs = row().key("notice-filters").gap(2.0).items_end();
    if !vp.mobile() {
        tabs = tabs.h(32.0);
    }
    for (i, filter) in NoticeFilter::ALL.into_iter().enumerate() {
        tabs = tabs.child(filter_tab(
            k,
            filter,
            counts[i],
            s.notice_filter == filter,
            vp.mobile(),
        ));
    }
    let tabs = if vp.mobile() {
        tabs.w_full().flex_wrap()
    } else {
        el().key("notice-filter-viewport")
            .grow(1.0)
            .min_w(0.0)
            .child(tabs)
            .scroll_x()
            .scroll_edge_fade(14.0)
            .label("Filter notifikasi")
    };

    let mut list = col().key("notice-list").w_full();
    if rows.is_empty() {
        list = list.child(empty(s, k));
    } else {
        for n in &rows {
            list = list.child(notice_row(s, k, n, now, vp));
            list = list.child(rule(k));
        }
    }

    let mut header = row().w_full().gap(10.0).center_y().child(tabs);
    if !vp.mobile() {
        header = header.h(32.0).child(
            text(format!("{:02} NOTIFIKASI", total))
                .font(mono_small(k))
                .color(k.c.text_faint),
        );
    }

    col()
        .key("notices-page")
        .w_full()
        .gap(if vp.mobile() { 8.0 } else { 10.0 })
        .child(header)
        .child(rule(k))
        .child(list)
}

fn filter_tab(k: &Kit, filter: NoticeFilter, count: usize, active: bool, compact: bool) -> El {
    let danger = matches!(filter, NoticeFilter::Resampling | NoticeFilter::NotPassed);
    let ink = if active {
        if danger { k.c.danger.fg } else { k.c.text_base }
    } else {
        k.c.text_faint
    };
    let bar = if active {
        if danger {
            k.c.danger.fg
        } else {
            k.c.text_accent
        }
    } else {
        Color::TRANSPARENT
    };
    col()
        .key(format!("notice-filter-{}", filter.label()))
        .role("radio")
        .label(filter.label())
        .checked(active)
        .focusable(true)
        .h(32.0)
        .justify_end()
        .hover(k.c.hover)
        .focus_ring(2.0, 1.0, k.c.border_focus)
        .cursor(rune::Cursor::Pointer)
        .on_press(move |s: &mut App| s.notice_filter = filter)
        .child(
            row()
                .px(4.0)
                .pb(7.0)
                .gap(5.0)
                .center_y()
                .child(
                    text(
                        if compact {
                            filter.tab_compact()
                        } else {
                            filter.tab()
                        }
                        .to_uppercase(),
                    )
                    .font(micro(k))
                    .color(ink)
                    .a11y_hidden(),
                )
                .child(
                    text(format!("{count:02}"))
                        .font(TextStyle {
                            size: 8.5,
                            weight: 500.0,
                            line_height: 12.0,
                            ..k.ty.mono
                        })
                        .color(if active { ink } else { k.c.text_faint }),
                ),
        )
        .child(el().h(2.0).w_full().fill(bar))
}

fn notice_row(s: &App, k: &Kit, n: &Notice, now: u64, vp: Viewport) -> El {
    if vp.mobile() {
        return mobile_notice_row(s, k, n, now);
    }
    let me = s.data.current_user;
    let unread = !n.read_by.contains(&me);
    let sender = s.data.user(n.from);
    let id = n.id;
    let accent = n
        .status
        .map(|status| status_colors(k, status).1)
        .unwrap_or(k.c.text_accent);

    let mut title_row = row()
        .w_full()
        .gap(7.0)
        .center_y()
        .child(
            text(n.title.clone())
                .font(ui_medium(k))
                .color(k.c.text_base),
        )
        .child(if unread { new_tag(k) } else { el() });
    if let Some(status) = n.status {
        title_row = title_row.child(status_mark(k, status));
    }
    title_row = title_row.child(el().grow(1.0)).child(
        text(ago(now, n.at))
            .font(mono_small(k))
            .color(k.c.text_faint),
    );

    let mut body = col().grow(1.0).min_w(0.0).gap(3.0).child(title_row);
    body = body.child(
        text(n.body.clone())
            .font(small(k))
            .color(k.c.text_muted)
            .wrap()
            .w_full(),
    );
    let mut meta = row()
        .mt(3.0)
        .gap(7.0)
        .center_y()
        .child(text(n.batch.clone()).font(mono(k)).color(k.c.text_muted));
    if let Some(stage) = n.stage {
        meta = meta
            .child(text("·").font(mono_small(k)).color(k.c.text_faint))
            .child(
                text(stage_tag(stage))
                    .font(TextStyle {
                        size: 8.5,
                        weight: 550.0,
                        line_height: 11.0,
                        tracking: 0.7,
                        ..k.ty.mono
                    })
                    .color(k.c.text_faint),
            );
    }
    meta = meta
        .child(text("·").font(mono_small(k)).color(k.c.text_faint))
        .child(
            text(format!(
                "dari {}",
                sender.map(|u| u.name.as_str()).unwrap_or("anggota")
            ))
            .font(caption(k))
            .color(k.c.text_faint),
        );
    body = body.child(meta);

    let mut row_body = row()
        .w_full()
        .py(12.0)
        .gap(11.0)
        .items_start()
        .child(
            el().w(2.0)
                .h(38.0)
                .fill(if unread { accent } else { Color::TRANSPARENT })
                .a11y_hidden(),
        )
        .child(match sender {
            Some(u) => avatar(k, u, 28.0),
            None => el().size(28.0, 28.0),
        })
        .child(body)
        .child(if unread {
            icon(&icons::CHEVRON_RIGHT, 12.0)
                .color(accent)
                .mt(7.0)
                .a11y_hidden()
        } else {
            el()
        });

    if unread {
        row_body = row_body
            .hover(k.c.layer_02)
            .press(k.c.pressed)
            .focus_ring(2.0, 1.0, k.c.border_focus)
            .cursor(rune::Cursor::Pointer)
            .role("button")
            .label(format!("Tandai dibaca: {}", n.title))
            .focusable(true)
            .on_press(move |s: &mut App| s.mark_read(id));
    }
    row_body
}

fn mobile_notice_row(s: &App, k: &Kit, n: &Notice, now: u64) -> El {
    let me = s.data.current_user;
    let unread = !n.read_by.contains(&me);
    let sender = s.data.user(n.from);
    let id = n.id;
    let accent = n
        .status
        .map(|status| status_colors(k, status).1)
        .unwrap_or(k.c.text_accent);

    let title = row()
        .w_full()
        .gap(6.0)
        .center_y()
        .child(
            el().grow(1.0).min_w(0.0).child(
                text(n.title.clone())
                    .font(ui_medium(k))
                    .color(k.c.text_base)
                    .truncate(),
            ),
        )
        .child(if unread { new_tag(k) } else { el() })
        .child(
            text(ago(now, n.at))
                .font(mono_small(k))
                .color(k.c.text_faint),
        );

    let mut body = col().grow(1.0).min_w(0.0).gap(3.0).child(title);
    if let Some(status) = n.status {
        body = body.child(status_mark_compact(k, status));
    }
    body = body.child(
        text(n.body.clone())
            .font(small(k))
            .color(k.c.text_muted)
            .wrap()
            .w_full(),
    );

    let mut meta = row()
        .w_full()
        .mt(2.0)
        .gap(6.0)
        .center_y()
        .child(text(n.batch.clone()).font(mono(k)).color(k.c.text_muted));
    if let Some(stage) = n.stage {
        meta = meta
            .child(text("·").font(mono_small(k)).color(k.c.text_faint))
            .child(
                text(stage_tag(stage))
                    .font(TextStyle {
                        size: 8.5,
                        weight: 550.0,
                        line_height: 11.0,
                        tracking: 0.7,
                        ..k.ty.mono
                    })
                    .color(k.c.text_faint),
            );
    }
    meta = meta
        .child(text("·").font(mono_small(k)).color(k.c.text_faint))
        .child(
            text(format!(
                "dari {}",
                sender.map(|u| u.name.as_str()).unwrap_or("anggota")
            ))
            .font(caption(k))
            .color(k.c.text_faint)
            .truncate(),
        );
    body = body.child(meta);

    let mut row_body = row()
        .w_full()
        .py(11.0)
        .gap(9.0)
        .items_start()
        .child(
            el().w(2.0)
                .h(36.0)
                .fill(if unread { accent } else { Color::TRANSPARENT })
                .a11y_hidden(),
        )
        .child(match sender {
            Some(u) => avatar(k, u, 24.0),
            None => el().size(24.0, 24.0),
        })
        .child(body);

    if unread {
        row_body = row_body
            .hover(k.c.layer_02)
            .press(k.c.pressed)
            .focus_ring(2.0, 1.0, k.c.border_focus)
            .cursor(rune::Cursor::Pointer)
            .role("button")
            .label(format!("Tandai dibaca: {}", n.title))
            .focusable(true)
            .on_press(move |s: &mut App| s.mark_read(id));
    }
    row_body
}

fn stage_tag(stage: Stage) -> String {
    stage.label().to_uppercase()
}

fn new_tag(k: &Kit) -> El {
    el().h(16.0)
        .px(5.0)
        .radius(1.0)
        .center()
        .fill(k.c.text_accent)
        .child(
            text("NEW")
                .font(TextStyle {
                    size: 8.0,
                    weight: 650.0,
                    line_height: 11.0,
                    tracking: 0.8,
                    ..k.ty.mono
                })
                .color(k.c.text_contrast),
        )
}

fn empty(s: &App, k: &Kit) -> El {
    let filtered = s.notice_filter != NoticeFilter::All;
    let mut c = col()
        .key("notices-empty")
        .w_full()
        .py(64.0)
        .gap(7.0)
        .center()
        .child(icon_badge(k, &icons::BELL, 40.0))
        .child(
            text("Tidak ada notifikasi di filter ini")
                .font(ui_medium(k))
                .color(k.c.text_base),
        )
        .child(
            text("Coba filter lain atau tandai semua notifikasi sebagai dibaca.")
                .font(small(k))
                .color(k.c.text_muted),
        );
    if filtered {
        c = c.child(
            el().mt(5.0).child(
                Button::new("Semua notifikasi")
                    .variant(ButtonVariant::Outline)
                    .control_size(ControlSize::Regular)
                    .on_press(|s: &mut App| s.notice_filter = NoticeFilter::All)
                    .build(k),
            ),
        );
    }
    c
}
