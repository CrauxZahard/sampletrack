//! The app's own icons. They are plain SVG bodies on a 16 point grid, drawn in the colour of
//! the element that holds them, like rune-kit's glyphs.

use rune::Icon;

pub static STACK: Icon = Icon {
    name: "st-sample",
    view: 16.0,
    body: r#"<path d="M2.5 5.5L8 2.5L13.5 5.5L8 8.5L2.5 5.5Z" stroke="currentColor" stroke-linejoin="round"/><path d="M2.5 8L8 11L13.5 8" stroke="currentColor" stroke-linejoin="round"/><path d="M2.5 10.5L8 13.5L13.5 10.5" stroke="currentColor" stroke-linejoin="round"/>"#,
};

pub static USERS: Icon = Icon {
    name: "st-users",
    view: 16.0,
    body: r#"<circle cx="6" cy="5" r="2.5" stroke="currentColor"/><path d="M1.75 13.25C1.75 11.18 3.66 9.5 6 9.5C8.34 9.5 10.25 11.18 10.25 13.25" stroke="currentColor" stroke-linecap="round"/><path d="M10.75 3.1A2.5 2.5 0 0 1 10.75 7.9M11.7 9.8C13.0 10.3 14.25 11.5 14.25 13.25" stroke="currentColor" stroke-linecap="round"/>"#,
};

pub static BELL: Icon = Icon {
    name: "st-bell",
    view: 16.0,
    body: r#"<path d="M4 6.5C4 4.29 5.79 2.5 8 2.5C10.21 2.5 12 4.29 12 6.5V9L13.25 11.5H2.75L4 9V6.5Z" stroke="currentColor" stroke-linejoin="round"/><path d="M6.5 13.5C6.75 14.1 7.3 14.5 8 14.5C8.7 14.5 9.25 14.1 9.5 13.5" stroke="currentColor" stroke-linecap="round"/>"#,
};

pub static PLUS: Icon = Icon {
    name: "st-plus",
    view: 16.0,
    body: r#"<path d="M8 3.25V12.75M3.25 8H12.75" stroke="currentColor" stroke-linecap="round"/>"#,
};

pub static SUN: Icon = Icon {
    name: "st-sun",
    view: 16.0,
    body: r#"<circle cx="8" cy="8" r="3" stroke="currentColor"/><path d="M8 1.5V3M8 13V14.5M1.5 8H3M13 8H14.5M3.4 3.4L4.45 4.45M11.55 11.55L12.6 12.6M12.6 3.4L11.55 4.45M4.45 11.55L3.4 12.6" stroke="currentColor" stroke-linecap="round"/>"#,
};

pub static MOON: Icon = Icon {
    name: "st-moon",
    view: 16.0,
    body: r#"<path d="M13.25 9.7A5.5 5.5 0 0 1 6.3 2.75A5.6 5.6 0 1 0 13.25 9.7Z" stroke="currentColor" stroke-linejoin="round"/>"#,
};

pub static SEND: Icon = Icon {
    name: "st-send",
    view: 16.0,
    body: r#"<path d="M14 2L7.5 8.5M14 2L9.75 14L7.5 8.5M14 2L2 6.25L7.5 8.5" stroke="currentColor" stroke-linejoin="round" stroke-linecap="round"/>"#,
};

pub static CLOCK: Icon = Icon {
    name: "st-clock",
    view: 16.0,
    body: r#"<circle cx="8" cy="8" r="5.75" stroke="currentColor"/><path d="M8 4.75V8L10 9.5" stroke="currentColor" stroke-linecap="round"/>"#,
};

pub static CHECK: Icon = Icon {
    name: "st-check",
    view: 16.0,
    body: r#"<path d="M3 8.25L6.25 11.5L13 4.5" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>"#,
};

pub static SHIELD: Icon = Icon {
    name: "st-shield",
    view: 16.0,
    body: r#"<path d="M8 2L13 4V7.5C13 10.5 11 12.7 8 14C5 12.7 3 10.5 3 7.5V4L8 2Z" stroke="currentColor" stroke-linejoin="round"/><path d="M5.75 7.75L7.25 9.25L10.25 6.25" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>"#,
};

pub static EMPTY: Icon = Icon {
    name: "st-empty",
    view: 16.0,
    body: r#"<path d="M2.5 9L4 3.5H12L13.5 9V12.5H2.5V9Z" stroke="currentColor" stroke-linejoin="round"/><path d="M2.5 9H6L6.75 10.5H9.25L10 9H13.5" stroke="currentColor" stroke-linejoin="round"/>"#,
};

pub static CHEVRON_RIGHT: Icon = Icon {
    name: "st-chevron-right",
    view: 16.0,
    body: r#"<path d="M6.5 4.5L10 8L6.5 11.5" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"/>"#,
};
