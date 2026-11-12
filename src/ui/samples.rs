//! The sample log: a fixed metric strip over a virtualized ledger. Only the rows in view are
//! built, so the cost of a million samples is the query, not the frame.

use std::borrow::Cow;

use rune::{Color, El, GrowthSpec, ScrollSpec, TextStyle, col, el, icon, row, text};
use rune_kit::{Button, ButtonVariant, ControlSize, Kit, TextInput, glyphs};

use crate::app::{App, Filter, SortKey};
use crate::icons;
use crate::model::ago;
use crate::ui::shell::rule;
use crate::ui::widgets::{
    avatar, caption, hairline, icon_badge, micro, mono, mono_small, number, small, status_mark,
    ui_medium,
};

const GAP: f32 = 10.0;
const ROW_HEIGHT: f32 = 50.0;
const W_BATCH: f32 = 104.0;
const W_STAGE: f32 = 78.0;
const W_STATUS: f32 = 164.0;
const W_INPUT: f32 = 100.0;
const W_UPDATED: f32 = 118.0;
const W_ACTION: f32 = 16.0;
const FLEX_PRODUCT: f32 = 1.0;
const FLEX_REMARKS: f32 = 1.25;

pub fn page(s: &App, k: &Kit) -> El {
    col()
        .key("samples-page")
        .w_full()
        .h_full()
        .min_h(0.0)
        .padding(26.0)
        .gap(14.0)
        .child(metrics(s, k))
        .child(log(s, k))
}

fn metrics(s: &App, k: &Kit) -> El {
    let c = s.counts();
    row()
        .key("metrics")
        .w_full()
        .h(90.0)
        .radius(k.radius.md)
        .fill(k.c.layer_01)
        .border(0.5, hairline(k))
        .child(metric(k, "TOTAL", c.total, "Semua batch", k.c.text_accent))
        .child(metric_rule(k))
        .child(metric(
            k,
            "BELUM ANALISA",
            c.unanalyzed,
            "Baru masuk",
            k.c.text_muted,
        ))
        .child(metric_rule(k))
        .child(metric(
            k,
            "MENUNGGU HUDS",
            c.waiting,
            "Butuh review",
            k.c.warning.fg,
        ))
        .child(metric_rule(k))
        .child(metric(
            k,
            "PERLU RESAMPLE",
            c.resampling,
            "Belum diambil",
            k.c.danger.fg,
        ))
        .child(metric_rule(k))
        .child(metric(
            k,
            "SUDAH RESAMPLE",
            c.resampled,
            "Sudah diambil",
            k.c.text_accent,
        ))
        .child(metric_rule(k))
        .child(metric(
            k,
            "APPROVED",
            c.approved,
            "Lolos approval",
            k.c.success.fg,
        ))
        .child(metric_rule(k))
        .child(metric(
            k,
            "APPROVED HUDS",
            c.approved_huds,
            "Lolos review HUDS",
            k.c.success.fg,
        ))
        .child(metric_rule(k))
        .child(metric(
            k,
            "TIDAK MEMENUHI",
            c.not_qualified,
            "Tidak lolos syarat",
            k.c.danger.fg,
        ))
}

fn metric(k: &Kit, label: &str, value: usize, note: &str, tint: Color) -> El {
    col()
        .key(format!("metric-{label}"))
        .grow(1.0)
        .min_w(0.0)
        .padding(13.0)
        .gap(2.0)
        .child(
            row()
                .gap(6.0)
                .center_y()
                .child(el().size(5.0, 5.0).fill(tint).a11y_hidden())
                .child(text(label).font(micro(k)).color(k.c.text_faint)),
        )
        .child(
            text(value.to_string())
                .font(TextStyle {
                    features: "'tnum' 1",
                    ..number(k)
                })
                .color(k.c.text_base),
        )
        .child(text(note).font(caption(k)).color(k.c.text_muted).truncate())
}

fn metric_rule(k: &Kit) -> El {
    el().w(1.0).h_full().fill(hairline(k)).a11y_hidden()
}

fn log(s: &App, k: &Kit) -> El {
    let shown = s.visible_len();
    let total = s.counts().total;
    let now = s.now();
    let kit = k.clone();
    let body = if shown == 0 {
        col()
            .key("log-empty")
            .w_full()
            .grow(1.0)
            .min_h(0.0)
            .center()
            .child(empty(s, k))
    } else {
        let rows = col()
            .key("log-rows")
            .w_full()
            .growth(GrowthSpec::default())
            .virtual_rows_version((s.visible_generation(), shown))
            .virtual_rows(
                shown,
                ROW_HEIGHT,
                |s: &App, i| Cow::Owned(s.visible_key(i)),
                move |s: &App, i| {
                    s.visible_sample(i)
                        .map(|sample| row_view(s, &kit, sample, now))
                        .unwrap_or_else(|| el())
                },
            );
        col()
            .key("log-body")
            .w_full()
            .grow(1.0)
            .min_h(0.0)
            .scroll(ScrollSpec {
                stick_bottom: false,
            })
            .child(rows)
    };

    col()
        .key("sample-log")
        .w_full()
        .grow(1.0)
        .min_h(0.0)
        .child(toolbar(s, k, shown, total))
        .child(rule(k))
        .child(header(s, k))
        .child(rule(k))
        .child(body)
        .child(rule(k))
        .child(footer(s, k, shown, total))
}

fn toolbar(s: &App, k: &Kit, shown: usize, total: usize) -> El {
    let search = TextInput::new(&s.search)
        .placeholder("Cari produk, batch, stage...")
        .label("Cari sample")
        .leading_icon(&glyphs::SEARCH)
        .width(Some(262.0))
        .on_edit(|s: &mut App| &mut s.search)
        .build(k);

    let mut tabs = row().key("filters").gap(2.0).h(32.0).items_end();
    for f in Filter::ALL {
        tabs = tabs.child(filter_tab(k, f, s.filter == f));
    }
    let tabs = el()
        .key("filter-viewport")
        .grow(1.0)
        .min_w(0.0)
        .child(tabs)
        .scroll_x()
        .scroll_edge_fade(14.0)
        .label("Filter status");

    row()
        .key("log-toolbar")
        .w_full()
        .h(48.0)
        .gap(10.0)
        .center_y()
        .child(search)
        .child(tabs)
        .child(
            text(format!("{shown:02}/{total:02}"))
                .font(mono(k))
                .color(k.c.text_faint),
        )
}

fn filter_tab(k: &Kit, f: Filter, active: bool) -> El {
    let label = f.tab().to_uppercase();
    col()
        .key(format!("filter-{}", f.label()))
        .role("radio")
        .label(f.label())
        .checked(active)
        .focusable(true)
        .h(32.0)
        .justify_end()
        .hover(k.c.hover)
        .focus_ring(2.0, 1.0, k.c.border_focus)
        .cursor(rune::Cursor::Pointer)
        .on_press(move |s: &mut App| s.filter = f)
        .child(
            row().px(6.0).pb(7.0).child(
                text(label)
                    .font(micro(k))
                    .color(if active {
                        k.c.text_base
                    } else {
                        k.c.text_faint
                    })
                    .a11y_hidden(),
            ),
        )
        .child(el().h(2.0).w_full().fill(if active {
            k.c.text_accent
        } else {
            Color::TRANSPARENT
        }))
}

fn header(s: &App, k: &Kit) -> El {
    row()
        .key("log-header")
        .w_full()
        .h(32.0)
        .gap(GAP)
        .center_y()
        .child(head_cell(
            k,
            "PRODUK",
            Some(SortKey::Product),
            s,
            None,
            FLEX_PRODUCT,
        ))
        .child(head_cell(
            k,
            "BATCH / MO",
            Some(SortKey::Batch),
            s,
            Some(W_BATCH),
            FLEX_PRODUCT,
        ))
        .child(head_cell(k, "STAGE", None, s, Some(W_STAGE), FLEX_PRODUCT))
        .child(head_cell(
            k,
            "STATUS",
            Some(SortKey::Status),
            s,
            Some(W_STATUS),
            FLEX_PRODUCT,
        ))
        .child(head_cell(k, "REMARKS", None, s, None, FLEX_REMARKS))
        .child(head_cell(
            k,
            "INPUT BY",
            None,
            s,
            Some(W_INPUT),
            FLEX_PRODUCT,
        ))
        .child(head_cell(
            k,
            "UPDATED BY",
            Some(SortKey::Updated),
            s,
            Some(W_UPDATED),
            FLEX_PRODUCT,
        ))
        .child(el().w(W_ACTION))
}

fn head_cell(
    k: &Kit,
    label: &str,
    key: Option<SortKey>,
    s: &App,
    width: Option<f32>,
    weight: f32,
) -> El {
    let active = key.is_some_and(|key| s.sort == key);
    let mut cell = el().h(22.0).center_y();
    cell = match width {
        Some(w) => cell.w(w),
        None => cell.grow(weight).min_w(0.0),
    };
    if let Some(key) = key {
        cell = cell
            .role("columnheader")
            .label(label.to_owned())
            .focusable(true)
            .cursor(rune::Cursor::Pointer)
            .hover(k.c.hover)
            .on_press(move |s: &mut App| s.sort_by(key));
        let mut content = row().w_full().gap(4.0).center_y().child(
            text(label)
                .font(micro(k))
                .color(if active {
                    k.c.text_base
                } else {
                    k.c.text_faint
                })
                .a11y_hidden(),
        );
        if active {
            content = content.child(
                icon(
                    if s.sort_desc {
                        &glyphs::CHEVRON_DOWN
                    } else {
                        &glyphs::CHEVRON_UP
                    },
                    9.0,
                )
                .color(k.c.text_accent)
                .a11y_hidden(),
            );
        }
        cell.child(content)
    } else {
        cell.child(text(label).font(micro(k)).color(k.c.text_faint))
    }
}

fn row_view(s: &App, k: &Kit, sample: &crate::model::Sample, now: u64) -> El {
    let id = sample.id;
    let product = sample.product.clone();
    let batch = sample.batch.clone();
    let stage = sample.stage.label().to_uppercase();
    let latest = sample
        .latest_remark()
        .map(|r| r.text.clone())
        .unwrap_or_default();
    let remark_count = sample.remark_count();
    let input_by = s.data.user(sample.input_by);
    let updated_by = s.data.user(sample.updated_by);

    let product_cell = el().grow(FLEX_PRODUCT).min_w(0.0).child(
        text(product.clone())
            .font(ui_medium(k))
            .color(k.c.text_base)
            .truncate(),
    );

    let batch_cell = el().w(W_BATCH).child(
        text(batch.clone())
            .font(mono(k))
            .color(k.c.text_muted)
            .truncate(),
    );

    let stage_cell = el()
        .w(W_STAGE)
        .child(text(stage.clone()).font(micro(k)).color(k.c.text_faint));

    let status_cell = el()
        .w(W_STATUS)
        .center_y()
        .child(status_mark(k, sample.status));

    let snippet = if latest.is_empty() {
        text("—").font(caption(k)).color(k.c.text_faint)
    } else {
        text(latest)
            .font(caption(k))
            .color(k.c.text_muted)
            .truncate()
    };
    let mut remarks_row = row().w_full().gap(6.0).center_y();
    if remark_count > 1 {
        remarks_row = remarks_row.child(
            text(format!("{remark_count}×"))
                .font(TextStyle {
                    size: 10.0,
                    weight: 550.0,
                    line_height: 13.0,
                    ..k.ty.mono
                })
                .color(k.c.text_accent),
        );
    }
    let mut remarks_cell = el()
        .grow(FLEX_REMARKS)
        .min_w(0.0)
        .child(remarks_row.child(el().grow(1.0).min_w(0.0).child(snippet)));
    if !sample.remarks.is_empty() {
        let mut thread = String::new();
        for (i, remark) in sample.remarks.iter().rev().enumerate() {
            if i > 0 {
                thread.push_str("\n\n");
            }
            let who = s
                .data
                .user(remark.by)
                .map(|u| u.name.clone())
                .unwrap_or_else(|| "Anggota".into());
            thread.push_str(&format!("{who} · {}\n{}", ago(now, remark.at), remark.text));
        }
        remarks_cell = remarks_cell.help(thread);
    }

    let input_cell = match input_by {
        Some(u) => el().w(W_INPUT).child(
            row().gap(5.0).center_y().child(avatar(k, u, 18.0)).child(
                text(u.name.clone())
                    .font(caption(k))
                    .color(k.c.text_muted)
                    .truncate(),
            ),
        ),
        None => el()
            .w(W_INPUT)
            .child(text("—").font(caption(k)).color(k.c.text_faint)),
    };

    let updated_cell = match updated_by {
        Some(u) => el().w(W_UPDATED).child(
            col()
                .gap(0.0)
                .child(
                    row().gap(5.0).center_y().child(avatar(k, u, 18.0)).child(
                        text(u.name.clone())
                            .font(caption(k))
                            .color(k.c.text_muted)
                            .truncate(),
                    ),
                )
                .child(
                    text(ago(now, sample.updated_at))
                        .font(mono_small(k))
                        .color(k.c.text_faint),
                ),
        ),
        None => el().w(W_UPDATED),
    };

    let action = el().w(W_ACTION).center().child(
        icon(&icons::CHEVRON_RIGHT, 11.0)
            .color(k.c.icon_faint)
            .a11y_hidden(),
    );

    row()
        .key(format!("sample-{id}"))
        .role("row")
        .label(format!(
            "{}, {}, {}, {}",
            product,
            batch,
            stage,
            sample.status.label()
        ))
        .focusable(true)
        .w_full()
        .h(ROW_HEIGHT)
        .gap(GAP)
        .center_y()
        .hover(k.c.layer_02)
        .press(k.c.pressed)
        .focus_ring(2.0, 1.0, k.c.border_focus)
        .cursor(rune::Cursor::Pointer)
        .on_press(move |s: &mut App| s.open_sample(id))
        .child(product_cell)
        .child(batch_cell)
        .child(stage_cell)
        .child(status_cell)
        .child(remarks_cell)
        .child(input_cell)
        .child(updated_cell)
        .child(action)
}

fn empty(s: &App, k: &Kit) -> El {
    let filtered = !s.search.text.trim().is_empty() || s.filter != Filter::All;
    let mut c = col()
        .key("samples-empty")
        .w_full()
        .py(52.0)
        .gap(7.0)
        .center()
        .child(icon_badge(k, &icons::EMPTY, 40.0))
        .child(
            text("Tidak ada sample yang cocok")
                .font(ui_medium(k))
                .color(k.c.text_base),
        )
        .child(
            text("Coba kata kunci lain atau ubah filter status.")
                .font(small(k))
                .color(k.c.text_muted),
        );
    if filtered {
        c = c.child(
            el().mt(5.0).child(
                Button::new("Reset filter")
                    .variant(ButtonVariant::Outline)
                    .control_size(ControlSize::Regular)
                    .on_press(|s: &mut App| s.reset_filters())
                    .build(k),
            ),
        );
    }
    c
}

fn footer(s: &App, k: &Kit, shown: usize, total: usize) -> El {
    let sort = match s.sort {
        SortKey::Updated => "updated",
        SortKey::Product => "product",
        SortKey::Batch => "batch",
        SortKey::Status => "status",
    };
    row()
        .key("log-footer")
        .w_full()
        .h(34.0)
        .gap(8.0)
        .center_y()
        .child(
            text(format!("{shown} DARI {total} SAMPLE"))
                .font(micro(k))
                .color(k.c.text_faint),
        )
        .child(el().grow(1.0))
        .child(
            text(format!(
                "SORT {sort} {}",
                if s.sort_desc { "DESC" } else { "ASC" }
            ))
            .font(TextStyle {
                size: 9.0,
                weight: 500.0,
                line_height: 12.0,
                tracking: 0.8,
                ..k.ty.mono
            })
            .color(k.c.text_faint),
        )
}
