//! The window frame: a narrow ledger sidebar, a masthead, and the page viewport.

use rune::tok::motion;
use rune::{El, ScrollSpec, TextStyle, col, el, icon, row, text};
use rune_kit::{Button, ButtonVariant, ControlSize, IconButton, IconButtonVariant, Kit};

use crate::app::{App, Dialog, Page};
use crate::icons;
use crate::ui::Viewport;
use crate::ui::notices::page as notices_page;
use crate::ui::people::page as people_page;
use crate::ui::samples::page as samples_page;
use crate::ui::widgets::{
    avatar, caption, count_tag, hairline, heading, micro, nav_item, scope_summary, small, ui_medium,
};

pub fn shell(s: &App, k: &Kit, layer: &mut Vec<El>, vp: Viewport) -> El {
    if vp.mobile() {
        return mobile_shell(s, k, layer, vp);
    }
    let main = col()
        .key("main")
        .grow(1.0)
        .min_w(0.0)
        .h_full()
        .child(header(s, k))
        .child(content(s, k, layer, vp));
    row()
        .key("window")
        .w_full()
        .h_full()
        .fill(k.c.page)
        .child(sidebar(s, k))
        .child(main)
}

fn sidebar(s: &App, k: &Kit) -> El {
    let me = s.me();
    let unread = s.unread();
    let brand = row()
        .key("brand")
        .h(76.0)
        .px(16.0)
        .gap(10.0)
        .center_y()
        .child(
            el().size(22.0, 22.0)
                .radius(1.0)
                .center()
                .fill(k.c.bg_accent)
                .a11y_hidden()
                .child(icon(&icons::STACK, 13.0).color(k.c.text_contrast)),
        )
        .child(
            col()
                .gap(1.0)
                .child(
                    text("SAMPLETRACK")
                        .font(TextStyle {
                            size: 11.0,
                            weight: 650.0,
                            line_height: 15.0,
                            tracking: 1.4,
                            ..k.ty.ui
                        })
                        .color(k.c.text_base),
                )
                .child(
                    text("QC WORKSPACE")
                        .font(TextStyle {
                            size: 8.5,
                            weight: 500.0,
                            line_height: 12.0,
                            tracking: 1.2,
                            ..k.ty.mono
                        })
                        .color(k.c.text_faint),
                ),
        );

    let nav = col()
        .key("nav")
        .w_full()
        .px(10.0)
        .gap(2.0)
        .child(
            text("RUANG KERJA")
                .font(micro(k))
                .color(k.c.text_faint)
                .padding(8.0),
        )
        .child(nav_item(
            k,
            "nav-samples",
            &icons::STACK,
            "Sample produk",
            Page::Samples,
            s.page == Page::Samples,
            None,
        ))
        .child(nav_item(
            k,
            "nav-notices",
            &icons::BELL,
            "Notifikasi",
            Page::Notices,
            s.page == Page::Notices,
            Some(unread),
        ));
    let nav = if s.is_admin() {
        nav.child(nav_item(
            k,
            "nav-people",
            &icons::USERS,
            "Pengguna & akses",
            Page::People,
            s.page == Page::People,
            None,
        ))
    } else {
        nav
    };

    let storage = col()
        .key("storage")
        .w_full()
        .px(8.0)
        .gap(2.0)
        .help(s.storage_note())
        .child(
            text("LOCAL")
                .font(TextStyle {
                    size: 8.5,
                    weight: 600.0,
                    line_height: 12.0,
                    tracking: 1.1,
                    ..k.ty.mono
                })
                .color(k.c.text_faint),
        )
        .child(
            text("Tersimpan di perangkat ini")
                .font(caption(k))
                .color(k.c.text_muted)
                .truncate(),
        );

    let user_card = row()
        .key("current-user")
        .role("button")
        .label(format!("Ganti pengguna. Sekarang {}.", me.name))
        .focusable(true)
        .w_full()
        .h(52.0)
        .px(7.0)
        .gap(9.0)
        .center_y()
        .radius(2.0)
        .fill(k.c.layer_02)
        .hover(k.c.layer_03)
        .press(k.c.pressed)
        .focus_ring(2.0, 1.0, k.c.border_focus)
        .cursor(rune::Cursor::Pointer)
        .on_press(|s: &mut App| {
            s.open_select = None;
            s.dialog = Dialog::SwitchUser;
        })
        .child(avatar(k, me, 28.0))
        .child(
            col()
                .grow(1.0)
                .min_w(0.0)
                .gap(1.0)
                .child(
                    text(me.name.clone())
                        .font(ui_medium(k))
                        .color(k.c.text_base)
                        .truncate(),
                )
                .child(scope_summary(k, me.scopes).truncate()),
        )
        .child(
            icon(&icons::CHEVRON_RIGHT, 12.0)
                .color(k.c.icon_faint)
                .a11y_hidden(),
        );

    col()
        .key("sidebar")
        .w(216.0)
        .h_full()
        .shrink()
        .fill(k.c.layer_01)
        .child(brand)
        .child(nav)
        .child(el().grow(1.0))
        .child(
            col()
                .w_full()
                .gap(8.0)
                .px(10.0)
                .pb(10.0)
                .child(storage)
                .child(user_card),
        )
}

fn brand_mark(k: &Kit, size: f32) -> El {
    el().size(size, size)
        .radius(1.0)
        .center()
        .fill(k.c.bg_accent)
        .a11y_hidden()
        .child(icon(&icons::STACK, size * 0.58).color(k.c.text_contrast))
}

fn bell_button(k: &Kit, unread: usize) -> El {
    IconButton::new(&icons::BELL)
        .variant(IconButtonVariant::Ghost)
        .control_size(ControlSize::Regular)
        .label("Notifikasi")
        .help(if unread == 0 {
            "Tidak ada notifikasi baru".to_owned()
        } else {
            format!("{unread} notifikasi belum dibaca")
        })
        .on_press(|s: &mut App| s.page = Page::Notices)
        .build(k)
}

fn theme_button(k: &Kit, dark_now: bool) -> El {
    IconButton::new(if dark_now { &icons::SUN } else { &icons::MOON })
        .variant(IconButtonVariant::Ghost)
        .control_size(ControlSize::Regular)
        .label(if dark_now {
            "Mode terang"
        } else {
            "Mode gelap"
        })
        .on_press_tx(move |s: &mut App, tx| {
            s.data.dark = Some(!dark_now);
            s.persist();
            tx.color_motion(motion::CALM);
        })
        .build(k)
}

// ---- phone shell --------------------------------------------------------

fn mobile_shell(s: &App, k: &Kit, layer: &mut Vec<El>, vp: Viewport) -> El {
    col()
        .key("window")
        .w_full()
        .h_full()
        .fill(k.c.page)
        .child(mobile_header(s, k))
        .child(content(s, k, layer, vp))
        .child(mobile_nav(s, k))
}

fn mobile_header(s: &App, k: &Kit) -> El {
    let me = s.me();
    let unread = s.unread();
    let dark_now = k.dark;

    let user = row()
        .key("mobile-user")
        .role("button")
        .label(format!("Ganti pengguna. Sekarang {}.", me.name))
        .focusable(true)
        .h(34.0)
        .px(3.0)
        .radius(2.0)
        .center_y()
        .hover(k.c.hover)
        .press(k.c.pressed)
        .focus_ring(2.0, 1.0, k.c.border_focus)
        .cursor(rune::Cursor::Pointer)
        .on_press(|s: &mut App| {
            s.open_select = None;
            s.dialog = Dialog::SwitchUser;
        })
        .child(avatar(k, me, 26.0));

    let mut brand = row()
        .key("mobile-brand")
        .w_full()
        .h(52.0)
        .px(14.0)
        .gap(9.0)
        .center_y()
        .child(brand_mark(k, 20.0))
        .child(
            text("SAMPLETRACK")
                .font(TextStyle {
                    size: 11.0,
                    weight: 650.0,
                    line_height: 15.0,
                    tracking: 1.2,
                    ..k.ty.ui
                })
                .color(k.c.text_base),
        )
        .child(el().grow(1.0))
        .child(bell_button(k, unread));
    if unread > 0 {
        brand = brand.child(count_tag(k, unread, true));
    }
    brand = brand.child(theme_button(k, dark_now)).child(user);

    let (title, subtitle) = s.page_title();
    let mut title_row = row()
        .w_full()
        .px(14.0)
        .pb(12.0)
        .gap(10.0)
        .items_end()
        .child(
            col()
                .grow(1.0)
                .min_w(0.0)
                .gap(1.0)
                .child(text(title).font(heading(k)).color(k.c.text_base).truncate())
                .child(
                    text(subtitle)
                        .font(small(k))
                        .color(k.c.text_muted)
                        .truncate(),
                ),
        );
    if let Some(action) = mobile_action(s, k) {
        title_row = title_row.child(action);
    }

    col()
        .key("mobile-header")
        .w_full()
        .fill(k.c.layer_01)
        .child(brand)
        .child(title_row)
        .child(el().h(1.0).w_full().fill(hairline(k)).a11y_hidden())
}

fn mobile_action(s: &App, k: &Kit) -> Option<El> {
    match s.page {
        Page::Samples => {
            let can = s.can_input();
            let mut button = IconButton::new(&icons::PLUS)
                .variant(IconButtonVariant::Contrast)
                .control_size(ControlSize::Regular)
                .label("Sample baru")
                .disabled(!can);
            if !can {
                button = button.help("Scope Anda tidak dapat menambah sample.");
            }
            Some(button.on_press(|s: &mut App| s.open_new_sample()).build(k))
        }
        Page::People if s.is_admin() => Some(
            IconButton::new(&icons::PLUS)
                .variant(IconButtonVariant::Contrast)
                .control_size(ControlSize::Regular)
                .label("Tambah pengguna")
                .on_press(|s: &mut App| s.open_new_user())
                .build(k),
        ),
        Page::Notices => {
            let unread = s.unread();
            Some(
                IconButton::new(&icons::CHECK)
                    .variant(IconButtonVariant::Neutral)
                    .control_size(ControlSize::Regular)
                    .label("Tandai semua dibaca")
                    .disabled(unread == 0)
                    .on_press(|s: &mut App| s.mark_all_read())
                    .build(k),
            )
        }
        _ => None,
    }
}

fn mobile_nav(s: &App, k: &Kit) -> El {
    let mut tabs = row().key("mobile-tabs").w_full().h(58.0).center_y();
    tabs = tabs.child(mobile_tab(
        k,
        "tab-samples",
        &icons::STACK,
        "Sample",
        Page::Samples,
        s.page == Page::Samples,
    ));
    tabs = tabs.child(mobile_tab(
        k,
        "tab-notices",
        &icons::BELL,
        "Notifikasi",
        Page::Notices,
        s.page == Page::Notices,
    ));
    if s.is_admin() {
        tabs = tabs.child(mobile_tab(
            k,
            "tab-people",
            &icons::USERS,
            "Pengguna",
            Page::People,
            s.page == Page::People,
        ));
    }
    col()
        .key("mobile-nav")
        .w_full()
        .fill(k.c.layer_01)
        .child(el().h(1.0).w_full().fill(hairline(k)).a11y_hidden())
        .child(tabs)
}

fn mobile_tab(
    k: &Kit,
    key: &str,
    glyph: &'static rune::Icon,
    label: &str,
    page: Page,
    active: bool,
) -> El {
    let ink = if active {
        k.c.text_accent
    } else {
        k.c.text_faint
    };
    col()
        .key(key)
        .role("button")
        .label(label)
        .focusable(true)
        .grow(1.0)
        .h_full()
        .center()
        .gap(3.0)
        .hover(k.c.hover)
        .press(k.c.pressed)
        .focus_ring(2.0, 1.0, k.c.border_focus)
        .cursor(rune::Cursor::Pointer)
        .on_press(move |s: &mut App| s.page = page)
        .child(icon(glyph, 18.0).color(ink).a11y_hidden())
        .child(
            text(label)
                .font(TextStyle {
                    size: 9.5,
                    weight: 550.0,
                    line_height: 12.0,
                    tracking: 0.2,
                    ..k.ty.ui
                })
                .color(ink),
        )
}

fn header(s: &App, k: &Kit) -> El {
    let (title, subtitle) = s.page_title();
    let unread = s.unread();
    let dark_now = k.dark;
    let mut actions = row().gap(6.0).center_y();

    actions = actions.child(bell_button(k, unread));
    if unread > 0 {
        actions = actions.child(count_tag(k, unread, true));
    }

    actions = actions.child(theme_button(k, dark_now));

    match s.page {
        Page::Samples => {
            let can = s.can_input();
            let mut b = Button::new("Sample baru")
                .variant(ButtonVariant::Contrast)
                .control_size(ControlSize::Regular)
                .icon(&icons::PLUS)
                .label_with(|font, ink| {
                    text("NEW SAMPLE")
                        .font(TextStyle {
                            size: 10.0,
                            weight: 620.0,
                            line_height: 14.0,
                            tracking: 0.9,
                            ..font
                        })
                        .color(ink)
                        .a11y_hidden()
                })
                .disabled(!can);
            if !can {
                b = b.help("Scope Anda tidak dapat menambah sample.");
            }
            actions = actions.child(b.on_press(|s: &mut App| s.open_new_sample()).build(k));
        }
        Page::People if s.is_admin() => {
            actions = actions.child(
                Button::new("Tambah pengguna")
                    .variant(ButtonVariant::Contrast)
                    .control_size(ControlSize::Regular)
                    .icon(&icons::PLUS)
                    .label_with(|font, ink| {
                        text("TAMBAH PENGGUNA")
                            .font(TextStyle {
                                size: 10.0,
                                weight: 620.0,
                                line_height: 14.0,
                                tracking: 0.9,
                                ..font
                            })
                            .color(ink)
                            .a11y_hidden()
                    })
                    .on_press(|s: &mut App| s.open_new_user())
                    .build(k),
            );
        }
        Page::Notices => {
            actions = actions.child(
                Button::new("Tandai semua dibaca")
                    .variant(ButtonVariant::Outline)
                    .control_size(ControlSize::Regular)
                    .disabled(unread == 0)
                    .on_press(|s: &mut App| s.mark_all_read())
                    .build(k),
            );
        }
        _ => {}
    }

    col()
        .key("header")
        .w_full()
        .child(
            row()
                .key("header-row")
                .h(86.0)
                .w_full()
                .px(26.0)
                .gap(16.0)
                .center_y()
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .gap(2.0)
                        .child(
                            text("QUALITY CONTROL / SAMPLE LOG")
                                .font(micro(k))
                                .color(k.c.text_faint),
                        )
                        .child(text(title).font(heading(k)).color(k.c.text_base))
                        .child(text(subtitle).font(small(k)).color(k.c.text_muted)),
                )
                .child(actions),
        )
        .child(el().h(1.0).w_full().fill(hairline(k)).a11y_hidden())
}

fn content(s: &App, k: &Kit, layer: &mut Vec<El>, vp: Viewport) -> El {
    if s.page == Page::Samples {
        return col()
            .key("content")
            .w_full()
            .grow(1.0)
            .min_h(0.0)
            .child(samples_page(s, k, vp));
    }
    let page: El = match s.page {
        Page::People => people_page(s, k, layer, vp),
        Page::Notices => notices_page(s, k, vp),
        Page::Samples => unreachable!(),
    };
    let scroller = page
        .w_full()
        .grow(1.0)
        .min_h(0.0)
        .padding(vp.pad())
        .gap(if vp.mobile() { 12.0 } else { 18.0 })
        .scroll(ScrollSpec {
            stick_bottom: false,
        });
    col()
        .key("content")
        .w_full()
        .grow(1.0)
        .min_h(0.0)
        .child(scroller)
}

pub fn rule(k: &Kit) -> El {
    el().h(1.0).w_full().fill(hairline(k)).a11y_hidden()
}
