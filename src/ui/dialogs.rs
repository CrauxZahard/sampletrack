//! Modal surfaces: add a sample, update one (and notify the team), switch the simulated
//! account, and add a member.

use rune::{Color, El, ScrollSpec, TextStyle, col, el, icon, row, text};
use rune_kit::{
    Button, ButtonVariant, ControlSize, Dialog, IconButton, IconButtonVariant, Kit, TextInput,
    glyphs,
};

use crate::app::{App, Dialog as Screen};
use crate::icons;
use crate::model::{Remark, Scope, Stage, Status, User, ago};
use crate::ui::widgets::{
    avatar, caption, count_tag, hairline, micro, scope_toggle, status_colors, ui, ui_medium,
};

pub fn build(s: &App, k: &Kit, layer: &mut Vec<El>) {
    match s.dialog {
        Screen::None => {}
        Screen::NewSample => new_sample(s, k, layer),
        Screen::Sample(id) => {
            if s.data.sample(id).is_some() {
                update_sample(s, k, layer, id);
            }
        }
        Screen::SwitchUser => switch_user(s, k, layer),
        Screen::NewUser => new_user(s, k, layer),
    }
}

fn shell(
    key: &str,
    title: impl Into<String>,
    description: impl Into<String>,
    width: f32,
) -> Dialog {
    Dialog::new(key, title)
        .description(description)
        .width(width)
        .fit()
        .open(true)
        .on_close(|s: &mut App| s.close_dialog())
}

fn body(children: impl IntoIterator<Item = El>) -> El {
    col()
        .key("dialog-body")
        .w_full()
        .px(16.0)
        .pb(16.0)
        .gap(15.0)
        .children(children)
}

fn actions(k: &Kit, cancel: &str, primary: Option<El>) -> El {
    let mut r = row().w_full().justify_end().gap(8.0).pt(3.0).child(
        Button::new(cancel)
            .variant(ButtonVariant::Outline)
            .control_size(ControlSize::Regular)
            .on_press(|s: &mut App| s.close_dialog())
            .build(k),
    );
    if let Some(p) = primary {
        r = r.child(p);
    }
    r
}

// ---- new sample ---------------------------------------------------------

fn new_sample(s: &App, k: &Kit, layer: &mut Vec<El>) {
    let me = s.me();
    let ready = s.new_sample_ready();
    let dup = !s.new_batch.text.trim().is_empty() && s.batch_taken(&s.new_batch.text, None);
    let product = TextInput::new(&s.new_product)
        .placeholder("cth. Paracetamol 500 mg")
        .label("Nama produk")
        .width(None)
        .on_edit(|s: &mut App| &mut s.new_product)
        .build(k)
        .autofocus();
    let batch = TextInput::new(&s.new_batch)
        .placeholder("cth. MO-2613-021")
        .label("Nomor batch / MO")
        .width(None)
        .invalid(dup)
        .on_edit(|s: &mut App| &mut s.new_batch)
        .build(k);
    let note = TextInput::new(&s.new_remarks)
        .placeholder("Remarks awal (opsional)")
        .label("Remarks")
        .width(None)
        .on_edit(|s: &mut App| &mut s.new_remarks)
        .build(k);

    let mut stage_row = row().w_full().gap(4.0);
    for stage in Stage::ALL {
        stage_row = stage_row.child(stage_pick(k, stage, s.new_stage == stage));
    }

    let mut fields = col()
        .w_full()
        .gap(13.0)
        .child(field(k, "Nama produk", product, None))
        .child(field(
            k,
            "Nomor batch / MO",
            batch,
            Some(if dup {
                "Batch ini sudah terdaftar."
            } else {
                "Harus unik di antara sample yang ada."
            }),
        ))
        .child(field(k, "Stage", stage_row, None))
        .child(field(k, "Remarks", note, None));

    fields = fields.child(
        row()
            .w_full()
            .padding(10.0)
            .gap(10.0)
            .radius(2.0)
            .border(0.5, hairline(k))
            .child(avatar(k, me, 26.0))
            .child(
                col()
                    .grow(1.0)
                    .gap(1.0)
                    .child(
                        text("Sample input by")
                            .font(caption(k))
                            .color(k.c.text_faint),
                    )
                    .child(text(me.name.clone()).font(ui(k)).color(k.c.text_base)),
            )
            .child(
                row()
                    .gap(5.0)
                    .center_y()
                    .child(
                        icon(&glyphs::LOCK, 12.0)
                            .color(k.c.icon_faint)
                            .a11y_hidden(),
                    )
                    .child(text("Otomatis").font(caption(k)).color(k.c.text_faint)),
            ),
    );

    let primary = Button::new("Simpan sample")
        .variant(ButtonVariant::Contrast)
        .control_size(ControlSize::Regular)
        .disabled(!ready)
        .on_press(|s: &mut App| s.save_new_sample())
        .build(k)
        .key("save-new");

    shell(
        "new-sample",
        "Sample baru",
        "Status awal Belum Analisa; stage menentukan jalur produksi.",
        500.0,
    )
    .click_outside(true)
    .child(body([fields, actions(k, "Batal", Some(primary))]))
    .build(k, layer);
}

// ---- update sample ------------------------------------------------------

fn update_sample(s: &App, k: &Kit, layer: &mut Vec<El>, id: u64) {
    let Some(sample) = s.data.sample(id) else {
        return;
    };
    let can_update = s.can_update();
    let is_final = sample.status.is_final();
    let can_comment = s.can_comment() && !is_final;
    let can_decide = can_update && !is_final;
    let now = s.now();
    let input_by = s.data.user(sample.input_by);
    let updated_by = s.data.user(sample.updated_by);
    let product = sample.product.clone();
    let batch = sample.batch.clone();
    let stage = sample.stage.label();
    let status = sample.status;

    let facts = row()
        .w_full()
        .radius(2.0)
        .border(0.5, hairline(k))
        .child(fact(
            k,
            "SAMPLE INPUT BY",
            input_by,
            ago(now, sample.input_at),
        ))
        .child(el().w(1.0).h(44.0).fill(hairline(k)).a11y_hidden())
        .child(fact(
            k,
            "SAMPLE UPDATED BY",
            updated_by,
            ago(now, sample.updated_at),
        ));

    let mut statuses = col().w_full().gap(7.0);
    for st in Status::DECISIONS {
        statuses = statuses.child(choice(
            k,
            st.label(),
            st.description(),
            status_colors(k, st).1,
            st == s.edit_status,
            can_decide,
            move |s: &mut App| s.edit_status = st,
        ));
    }
    let mut status_col = col()
        .w(232.0)
        .gap(7.0)
        .child(section(k, "KEPUTUSAN ANALISA"))
        .child(statuses);
    if is_final {
        status_col = status_col.child(locked_note(
            k,
            "Status sudah final. Tidak ada perubahan status atau notifikasi lagi.",
        ));
    } else if !can_update {
        status_col = status_col.child(locked_note(k, "Scope Anda hanya dapat melihat status."));
    } else if s.edit_status == Status::Unanalyzed {
        status_col = status_col.child(
            text("Pilih satu hasil analisa untuk menyimpan.")
                .font(caption(k))
                .color(k.c.text_faint),
        );
    }

    // Remarks are a thread, like a cell's comments: every sample keeps all of them, and the
    // thread grows in place without closing the dialog.
    let mut thread = col()
        .key("remark-thread")
        .w_full()
        .gap(6.0)
        .max_h(228.0)
        .scroll(ScrollSpec {
            stick_bottom: false,
        });
    if sample.remarks.is_empty() {
        thread = thread.child(
            text("Belum ada remarks.")
                .font(caption(k))
                .color(k.c.text_faint)
                .padding(8.0),
        );
    } else {
        for remark in sample.remarks.iter().rev() {
            thread = thread.child(remark_bubble(s, k, remark));
        }
    }

    let mut remarks_col = col()
        .grow(1.0)
        .min_w(220.0)
        .gap(8.0)
        .child(
            row()
                .w_full()
                .gap(6.0)
                .center_y()
                .child(section(k, "REMARKS"))
                .child(count_tag(k, sample.remark_count(), true)),
        )
        .child(thread);

    if can_comment {
        let input = TextInput::new(&s.remark_draft)
            .placeholder("Tulis remarks...")
            .label("Remarks baru")
            .width(Some(246.0))
            .on_edit(|s: &mut App| &mut s.remark_draft)
            .on_submit(move |s: &mut App, _| s.add_remark(id))
            .build(k)
            .key("remark-input");
        let send = IconButton::new(&icons::SEND)
            .variant(IconButtonVariant::Contrast)
            .control_size(ControlSize::Regular)
            .label("Kirim remarks")
            .disabled(s.remark_draft.text.trim().is_empty())
            .on_press(move |s: &mut App| s.add_remark(id))
            .build(k)
            .key("add-remark");
        remarks_col =
            remarks_col.child(row().w_full().gap(8.0).center_y().child(input).child(send));
    } else {
        let message = if is_final {
            "Sample sudah final: remarks hanya dapat dibaca."
        } else {
            "Scope read-only: remarks hanya dapat dibaca."
        };
        remarks_col = remarks_col.child(locked_note(k, message));
    }

    let primary = if can_decide {
        Some(
            Button::new("Simpan update")
                .variant(ButtonVariant::Contrast)
                .control_size(ControlSize::Regular)
                .icon(&icons::SEND)
                .disabled(s.edit_status == Status::Unanalyzed)
                .on_press(|s: &mut App| s.save_update())
                .build(k)
                .key("save-update"),
        )
    } else {
        None
    };

    let mut columns = row()
        .w_full()
        .gap(16.0)
        .items_start()
        .child(status_col)
        .child(remarks_col);
    if can_decide {
        columns = columns.child(notify_col(s, k));
    }

    let body = body([
        facts,
        columns,
        actions(k, if can_decide { "Batal" } else { "Tutup" }, primary),
    ]);

    shell(
        "update-sample",
        if is_final {
            format!("Detail {}", product)
        } else {
            format!("Update {}", product)
        },
        format!("{batch} \u{00b7} {stage} \u{00b7} {}", status.label()),
        if can_decide { 840.0 } else { 660.0 },
    )
    .click_outside(true)
    .child(body)
    .build(k, layer);
}

/// The bulk message the update can carry: pick any subset of the other people.
fn notify_col(s: &App, k: &Kit) -> El {
    let others: Vec<&User> = s
        .data
        .users
        .iter()
        .filter(|u| u.id != s.data.current_user)
        .collect();
    let all = s.all_others_selected();
    let mut col = col().w(238.0).shrink().gap(5.0).child(
        row()
            .w_full()
            .gap(8.0)
            .center_y()
            .child(section(k, "KIRIM NOTIFIKASI"))
            .child(el().grow(1.0))
            .child(
                Button::new(if all { "Kosongkan" } else { "Pilih semua" })
                    .variant(ButtonVariant::Ghost)
                    .control_size(ControlSize::Small)
                    .on_press(move |s: &mut App| s.set_all_recipients(!all))
                    .build(k),
            ),
    );
    for user in others {
        col = col.child(recipient(k, user, s.recipients.contains(&user.id)));
    }
    col.child(
        text(format!("{} penerima dipilih.", s.recipients.len()))
            .font(caption(k))
            .color(k.c.text_faint),
    )
}

fn remark_bubble(s: &App, k: &Kit, remark: &Remark) -> El {
    let author = s.data.user(remark.by);
    let mut text_col = col().grow(1.0).min_w(0.0).gap(1.0).child(
        row()
            .w_full()
            .gap(6.0)
            .center_y()
            .child(
                text(
                    author
                        .map(|u| u.name.clone())
                        .unwrap_or_else(|| "Anggota".into()),
                )
                .font(TextStyle {
                    size: 11.0,
                    weight: 560.0,
                    line_height: 15.0,
                    ..k.ty.small
                })
                .color(k.c.text_base)
                .truncate(),
            )
            .child(el().grow(1.0))
            .child(
                text(ago(s.now(), remark.at))
                    .font(TextStyle {
                        size: 9.5,
                        line_height: 13.0,
                        ..k.ty.small
                    })
                    .color(k.c.text_faint),
            ),
    );
    text_col = text_col.child(
        text(remark.text.clone())
            .font(caption(k))
            .color(k.c.text_muted)
            .wrap()
            .w_full(),
    );

    row()
        .key(format!("remark-{}", remark.id))
        .w_full()
        .py(7.0)
        .gap(8.0)
        .items_start()
        .child(el().w(1.0).h(32.0).fill(k.c.border_strong).a11y_hidden())
        .child(match author {
            Some(u) => avatar(k, u, 20.0),
            None => el().size(20.0, 20.0),
        })
        .child(text_col)
}

fn locked_note(k: &Kit, message: &str) -> El {
    row()
        .w_full()
        .padding(8.0)
        .gap(7.0)
        .radius(1.0)
        .border(0.5, hairline(k))
        .child(
            icon(&glyphs::LOCK, 12.0)
                .color(k.c.icon_faint)
                .a11y_hidden(),
        )
        .child(
            text(message)
                .font(caption(k))
                .color(k.c.text_muted)
                .wrap()
                .grow(1.0),
        )
}
fn fact(k: &Kit, label: &str, user: Option<&User>, when: String) -> El {
    let mut c = col()
        .key(format!("fact-{label}"))
        .grow(1.0)
        .min_w(0.0)
        .padding(9.0)
        .gap(3.0)
        .child(text(label).font(micro(k)).color(k.c.text_faint));
    match user {
        Some(u) => {
            c = c.child(
                row().gap(6.0).center_y().child(avatar(k, u, 21.0)).child(
                    text(u.name.clone())
                        .font(caption(k))
                        .color(k.c.text_base)
                        .truncate(),
                ),
            );
        }
        None => {
            c = c.child(text("Tidak dikenal").font(caption(k)).color(k.c.text_muted));
        }
    }
    c.child(text(when).font(caption(k)).color(k.c.text_faint))
}

fn recipient(k: &Kit, user: &User, on: bool) -> El {
    let id = user.id;
    row()
        .key(format!("recipient-{id}"))
        .role("checkbox")
        .label(user.name.clone())
        .checked(on)
        .focusable(true)
        .w_full()
        .min_w(0.0)
        .h(34.0)
        .px(4.0)
        .gap(7.0)
        .center_y()
        .radius(1.0)
        .hover(k.c.hover)
        .press(k.c.pressed)
        .focus_ring(2.0, 1.0, k.c.border_focus)
        .cursor(rune::Cursor::Pointer)
        .on_press(move |s: &mut App| s.toggle_recipient(id))
        .child(avatar(k, user, 18.0))
        .child(
            col()
                .grow(1.0)
                .min_w(0.0)
                .gap(0.0)
                .child(
                    text(user.name.clone())
                        .font(caption(k))
                        .color(k.c.text_base)
                        .truncate(),
                )
                .child(
                    text(user.scopes.summary())
                        .font(TextStyle {
                            size: 8.5,
                            weight: 450.0,
                            line_height: 11.0,
                            ..k.ty.mono
                        })
                        .color(k.c.text_faint)
                        .truncate(),
                ),
        )
        .child(check_box(k, on))
}

fn check_box(k: &Kit, on: bool) -> El {
    el().size(13.0, 13.0)
        .radius(1.0)
        .center()
        .fill(if on {
            k.c.text_accent
        } else {
            Color::TRANSPARENT
        })
        .border(
            1.0,
            if on {
                Color::TRANSPARENT
            } else {
                k.c.border_strong
            },
        )
        .a11y_hidden()
        .child(if on {
            icon(&glyphs::CHECK, 8.0).color(k.c.text_contrast)
        } else {
            el()
        })
}

// ---- switch user --------------------------------------------------------

fn switch_user(s: &App, k: &Kit, layer: &mut Vec<El>) {
    let mut list = col().key("user-list").w_full();
    for user in &s.data.users {
        let id = user.id;
        let active = id == s.data.current_user;
        list = list.child(
            row()
                .key(format!("switch-{}", user.id))
                .role("button")
                .label(format!("Masuk sebagai {}", user.name))
                .focusable(true)
                .w_full()
                .h(46.0)
                .px(6.0)
                .gap(9.0)
                .center_y()
                .radius(1.0)
                .fill(if active {
                    k.c.layer_02
                } else {
                    Color::TRANSPARENT
                })
                .hover(k.c.hover)
                .press(k.c.pressed)
                .focus_ring(2.0, 1.0, k.c.border_focus)
                .cursor(rune::Cursor::Pointer)
                .on_press(move |s: &mut App| s.switch_user(id))
                .child(
                    el().w(2.0)
                        .h(22.0)
                        .fill(if active {
                            k.c.text_accent
                        } else {
                            Color::TRANSPARENT
                        })
                        .a11y_hidden(),
                )
                .child(avatar(k, user, 28.0))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .gap(1.0)
                        .child(
                            text(user.name.clone())
                                .font(ui_medium(k))
                                .color(k.c.text_base),
                        )
                        .child(
                            text(user.scopes.summary())
                                .font(caption(k))
                                .color(k.c.text_faint),
                        ),
                )
                .child(if active {
                    icon(&glyphs::CHECK, 13.0).color(k.c.text_accent)
                } else {
                    el()
                }),
        );
        list = list.child(el().h(1.0).w_full().fill(hairline(k)).a11y_hidden());
    }

    let note = row()
        .w_full()
        .padding(9.0)
        .gap(8.0)
        .radius(1.0)
        .border(0.5, hairline(k))
        .child(icon(&icons::SHIELD, 12.0).color(k.c.text_faint).a11y_hidden())
        .child(
            text("Mode demo: berpindah akun mensimulasikan login. Scope mengikuti pengguna yang aktif.")
                .font(caption(k))
                .color(k.c.text_muted)
                .wrap()
                .grow(1.0),
        );

    shell(
        "switch-user",
        "Ganti pengguna",
        "Pilih siapa yang sedang memakai workspace.",
        430.0,
    )
    .child(body([list, note, actions(k, "Tutup", None)]))
    .build(k, layer);
}

// ---- new user -----------------------------------------------------------

fn new_user(s: &App, k: &Kit, layer: &mut Vec<El>) {
    let name = TextInput::new(&s.new_name)
        .placeholder("cth. Nabila Putri")
        .label("Nama anggota")
        .width(None)
        .on_edit(|s: &mut App| &mut s.new_name)
        .build(k)
        .autofocus();

    let admin = s.new_scopes.is_admin();
    let mut scopes_row = row().w_full().gap(4.0);
    for scope in Scope::ALL {
        let checked = s.new_scopes.holds(scope);
        let enabled = !admin || scope == Scope::Admin;
        scopes_row = scopes_row.child(scope_toggle(
            k,
            scope.short(),
            checked,
            enabled,
            move |s: &mut App| s.new_scopes.toggle(scope),
        ));
    }
    let scopes_block = col().w_full().gap(6.0).child(scopes_row).child(
        text(s.new_scopes.summary())
            .font(caption(k))
            .color(k.c.text_muted),
    );

    let ready = s.new_user_ready();
    let primary = Button::new("Tambah anggota")
        .variant(ButtonVariant::Contrast)
        .control_size(ControlSize::Regular)
        .disabled(!ready)
        .on_press(|s: &mut App| s.save_new_user())
        .build(k);

    shell(
        "new-user",
        "Anggota baru",
        "Scope bisa diubah lagi nanti oleh admin.",
        500.0,
    )
    .child(body([
        col()
            .w_full()
            .gap(7.0)
            .child(section(k, "IDENTITAS"))
            .child(field(
                k,
                "Nama anggota",
                name,
                Some("Nama harus unik di antara anggota tim."),
            )),
        col()
            .w_full()
            .gap(7.0)
            .child(section(k, "SCOPE IZIN"))
            .child(scopes_block),
        actions(k, "Batal", Some(primary)),
    ]))
    .build(k, layer);
}

// ---- shared pieces ------------------------------------------------------

fn field(k: &Kit, label: &str, input: El, hint: Option<&str>) -> El {
    crate::ui::widgets::field(k, label, input, hint)
}

fn section(k: &Kit, label: &str) -> El {
    text(label.to_uppercase())
        .font(micro(k))
        .color(k.c.text_faint)
}

fn stage_pick(k: &Kit, stage: Stage, selected: bool) -> El {
    row()
        .key(format!("stage-{}", stage.label()))
        .role("radio")
        .label(stage.label())
        .checked(selected)
        .focusable(true)
        .grow(1.0)
        .h(28.0)
        .center()
        .radius(1.0)
        .fill(if selected {
            k.c.layer_02
        } else {
            Color::TRANSPARENT
        })
        .border(
            1.0,
            if selected {
                k.c.text_accent
            } else {
                k.c.border_weak
            },
        )
        .hover(k.c.hover)
        .press(k.c.pressed)
        .focus_ring(2.0, 1.0, k.c.border_focus)
        .cursor(rune::Cursor::Pointer)
        .on_press(move |s: &mut App| s.new_stage = stage)
        .child(
            text(stage.label().to_uppercase())
                .font(micro(k))
                .color(if selected {
                    k.c.text_base
                } else {
                    k.c.text_muted
                })
                .a11y_hidden(),
        )
}

fn choice(
    k: &Kit,
    title: &str,
    description: &str,
    tint: Color,
    selected: bool,
    enabled: bool,
    on_press: impl Fn(&mut App) + 'static,
) -> El {
    let mut r = row()
        .key(format!("choice-{title}"))
        .role("radio")
        .label(title.to_owned())
        .checked(selected)
        .focusable(enabled)
        .w_full()
        .padding(9.0)
        .gap(9.0)
        .radius(2.0)
        .fill(if selected {
            k.c.layer_02
        } else {
            Color::TRANSPARENT
        })
        .border(
            if selected { 1.0 } else { 0.5 },
            if selected {
                k.alpha(tint, 0.8)
            } else {
                k.c.border_weak
            },
        )
        .child(
            el().size(14.0, 14.0)
                .radius(1.0)
                .border(1.0, if selected { tint } else { k.c.border_strong })
                .center()
                .a11y_hidden()
                .child(if selected {
                    el().size(6.0, 6.0).fill(tint)
                } else {
                    el()
                }),
        )
        .child(
            col()
                .grow(1.0)
                .min_w(0.0)
                .gap(1.0)
                .child(text(title).font(ui_medium(k)).color(if enabled {
                    k.c.text_base
                } else {
                    k.c.text_faint
                }))
                .child(
                    text(description)
                        .font(caption(k))
                        .color(if enabled {
                            k.c.text_muted
                        } else {
                            k.c.text_faint
                        })
                        .wrap(),
                ),
        )
        .child(if selected {
            icon(&glyphs::CHECK, 12.0).color(tint).a11y_hidden()
        } else {
            el()
        });
    if enabled {
        r = r
            .hover(k.c.hover)
            .press(k.c.pressed)
            .focus_ring(2.0, 1.0, k.c.border_focus)
            .cursor(rune::Cursor::Pointer)
            .on_press(on_press);
    } else {
        r = r.layer_opacity(0.55);
    }
    r
}
