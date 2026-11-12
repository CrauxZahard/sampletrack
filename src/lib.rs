//! SampleTrack: an approval workspace for product samples, built with Rune.
//!
//! ```sh
//! cargo run -p sample-tracker
//! cargo run -p sample-tracker -- --snap samples.png --scene samples --theme dark --size 1360x860
//! ```
//!
//! The list is the product: every row records who input the sample and who last updated its
//! status. Scopes are enforced in the actions, not just hidden in the UI, and an update can be
//! sent as one message to any subset of the team.

mod app;
mod icons;
mod model;
mod store;
mod theme;
mod ui;

use rune::{AppSpec, Cx, El, KeyEvent, Tx};

fn view(s: &app::App, cx: &mut Cx<app::App>) -> El {
    ui::view(s, cx)
}

fn on_key(s: &mut app::App, ev: &KeyEvent, _: &mut Tx) -> bool {
    if ev.is_command('n') && s.dialog == app::Dialog::None && s.can_input() {
        s.open_new_sample();
        return true;
    }
    false
}

/// Opens the app. On the web this is the wasm-bindgen entry point; natively `main` calls it.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen(start))]
pub fn start() {
    rune::run(AppSpec {
        min_size: Some((1200.0, 700.0)),
        scenes: app::scenes(),
        poll: Some(app::App::poll),
        on_key: Some(on_key),
        ..AppSpec::new("SampleTrack", (1360.0, 860.0), app::App::boot, view)
    });
}
