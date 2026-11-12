//! A bench-sheet theme: warm paper and graphite ink with a safety-orange accent, and corners
//! measured in pixels rather than pills. It follows the Swiss/data-dense direction from
//! ui-ux-pro-max, tuned for a quality-control workspace.

use std::sync::OnceLock;

use rune::color::hex;
use rune_kit::{Kit, PaletteFocus, PaletteSeed};

pub fn kit(dark: bool) -> &'static Kit {
    static DARK: OnceLock<Kit> = OnceLock::new();
    static LIGHT: OnceLock<Kit> = OnceLock::new();
    if dark {
        DARK.get_or_init(|| build(true))
    } else {
        LIGHT.get_or_init(|| build(false))
    }
}

fn build(dark: bool) -> Kit {
    let c = |s: &str| hex(s).expect("a theme colour");
    let seed = if dark {
        PaletteSeed {
            bg: c("#12110e"),
            fg: c("#ede7da"),
            accent: c("#ff7a45"),
        }
    } else {
        PaletteSeed {
            bg: c("#f1ede4"),
            fg: c("#17140f"),
            accent: c("#c2410c"),
        }
    };
    let mut k = Kit::from_palette_focus(seed, PaletteFocus::Accent);
    k.radius.xs = 1.0;
    k.radius.sm = 2.0;
    k.radius.md = 3.0;
    k.radius.lg = 4.0;
    k.radius.xl = 6.0;
    k
}
