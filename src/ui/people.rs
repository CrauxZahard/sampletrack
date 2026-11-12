//! Team access: one row per person, with the scopes they hold switched in place.

use rune::{El, TextStyle, col, el, icon, row, text};
use rune_kit::{Kit, glyphs};

use crate::app::App;
use crate::icons;
use crate::model::{Scope, Scopes, User};
use crate::ui::shell::rule;
use crate::ui::widgets::{avatar, caption, micro, mono, scope_toggle, small, ui_medium};

const W_SCOPES: f32 = 440.0;

pub fn page(s: &App, k: &Kit, _layer: &mut Vec<El>) -> El {
    if !s.is_admin() {
        return restricted(k);
    }
    col()
        .key("people-page")
        .w_full()
        .gap(16.0)
        .child(
            col()
                .w_full()
                .gap(2.0)
                .child(text("SCOPE AKSES TIM").font(micro(k)).color(k.c.text_faint))
                .child(
                    text("Admin mencakup Input, Update, dan Resample. Read-only mengosongkan semua izin.")
                        .font(caption(k))
                        .color(k.c.text_muted),
                ),
        )
        .child(team_sheet(s, k))
        .child(scope_note(k))
        .child(storage_block(s, k))
}

fn team_sheet(s: &App, k: &Kit) -> El {
    let mut c = col()
        .key("team-sheet")
        .w_full()
        .child(
            row()
                .w_full()
                .h(30.0)
                .gap(12.0)
                .center_y()
                .child(
                    el().grow(1.0)
                        .child(text("ANGGOTA").font(micro(k)).color(k.c.text_faint)),
                )
                .child(
                    el().w(W_SCOPES)
                        .child(text("SCOPE AKSES").font(micro(k)).color(k.c.text_faint)),
                ),
        )
        .child(rule(k));
    for user in &s.data.users {
        c = c.child(member_row(s, k, user));
        c = c.child(rule(k));
    }
    c
}

fn member_row(s: &App, k: &Kit, user: &User) -> El {
    let is_me = user.id == s.data.current_user;
    let last_admin = s.is_last_admin(user.id);
    let admin = user.scopes.is_admin();
    let id = user.id;

    let ident = row()
        .grow(1.0)
        .min_w(0.0)
        .gap(10.0)
        .center_y()
        .child(avatar(k, user, 30.0))
        .child(
            col()
                .grow(1.0)
                .min_w(0.0)
                .gap(1.0)
                .child(
                    row()
                        .gap(7.0)
                        .center_y()
                        .child(
                            text(user.name.clone())
                                .font(ui_medium(k))
                                .color(k.c.text_base)
                                .truncate(),
                        )
                        .child(if is_me { you_tag(k) } else { el() }),
                )
                .child(
                    text(format!("#{:02} · {}", user.id, user.scopes.summary()))
                        .font(caption(k))
                        .color(k.c.text_faint)
                        .truncate(),
                ),
        );

    let mut scopes = row().w(W_SCOPES).gap(4.0).center_y();
    for scope in Scope::ALL {
        let checked = user.scopes.holds(scope);
        let direct = matches!(scope, Scope::Admin | Scope::ReadOnly);
        let enabled = (!admin || direct) && !(last_admin && direct);
        scopes = scopes.child(
            scope_toggle(k, scope.short(), checked, enabled, move |s: &mut App| {
                s.toggle_scope(id, scope)
            })
            .key(format!("scope-{id}-{}", scope.short())),
        );
    }
    if last_admin {
        scopes = scopes.child(
            icon(&glyphs::LOCK, 12.0)
                .color(k.c.icon_faint)
                .help("Admin terakhir tidak dapat diturunkan."),
        );
    }

    row()
        .key(format!("member-{}", user.id))
        .w_full()
        .h(58.0)
        .gap(12.0)
        .center_y()
        .hover(k.c.layer_02)
        .child(ident)
        .child(scopes)
}

fn you_tag(k: &Kit) -> El {
    el().h(16.0)
        .px(5.0)
        .radius(1.0)
        .center()
        .border(0.5, k.c.border_base)
        .child(
            text("ANDA")
                .font(TextStyle {
                    size: 8.5,
                    weight: 600.0,
                    line_height: 11.0,
                    tracking: 0.8,
                    ..k.ty.mono
                })
                .color(k.c.text_muted),
        )
}

fn scope_note(k: &Kit) -> El {
    row()
        .key("scope-note")
        .w_full()
        .gap(7.0)
        .center_y()
        .child(
            icon(&icons::SHIELD, 12.0)
                .color(k.c.text_faint)
                .a11y_hidden(),
        )
        .child(
            text("Scope mengatur aksi, bukan hanya tampilan. Perubahan langsung tersimpan.")
                .font(caption(k))
                .color(k.c.text_faint),
        )
}

fn storage_block(s: &App, k: &Kit) -> El {
    row()
        .key("data-source")
        .w_full()
        .pt(14.0)
        .gap(12.0)
        .items_start()
        .child(el().w(2.0).h(46.0).fill(k.c.text_accent).a11y_hidden())
        .child(
            col()
                .grow(1.0)
                .gap(2.0)
                .child(text("SUMBER DATA").font(micro(k)).color(k.c.text_faint))
                .child(
                    text(s.storage_note())
                        .font(mono(k))
                        .color(k.c.text_muted)
                        .truncate(),
                )
                .child(
                    text("Lapisan Repository terpisah; siap diganti klien API tanpa mengubah tampilan.")
                        .font(caption(k))
                        .color(k.c.text_faint),
                ),
        )
}

fn restricted(k: &Kit) -> El {
    col()
        .key("people-restricted")
        .w_full()
        .py(70.0)
        .gap(8.0)
        .center()
        .child(
            icon(&icons::SHIELD, 28.0)
                .color(k.c.text_faint)
                .a11y_hidden(),
        )
        .child(
            text("Halaman khusus admin")
                .font(ui_medium(k))
                .color(k.c.text_base),
        )
        .child(
            text("Minta admin untuk membuka halaman ini atau mengubah scope Anda.")
                .font(small(k))
                .color(k.c.text_muted),
        )
}

pub fn _scopes_hint(_: Scopes) {}
