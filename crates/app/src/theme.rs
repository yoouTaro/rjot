//! Themes: each a quiet dark and light palette with one accent. Which of
//! the two shows follows the appearance setting (or macOS).

#![allow(non_snake_case)]

use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, TextStyle,
    Theme, ThemePreference, Visuals, vec2,
};
use rjot_core::Appearance;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub struct Palette {
    pub bg: Color32,
    /// Cards.
    surface: Color32,
    /// Hovered cards, segmented control track.
    raise: Color32,
    border: Color32,
    text: Color32,
    muted: Color32,
    faint: Color32,
    pub accent: Color32,
    error: Color32,
    input: Color32,
    /// Button fills: inactive, hovered, pressed.
    widget: [Color32; 3],
    strong: Color32,
}

pub struct ThemeDef {
    pub name: &'static str,
    pub dark: Palette,
    pub light: Palette,
}

const fn hex(v: u32) -> Color32 {
    Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

/// bg, surface, raise, border, text, muted, faint, accent, input, then the
/// three button fills.
const fn dark(c: [u32; 12]) -> Palette {
    Palette {
        bg: hex(c[0]), surface: hex(c[1]), raise: hex(c[2]), border: hex(c[3]),
        text: hex(c[4]), muted: hex(c[5]), faint: hex(c[6]), accent: hex(c[7]), input: hex(c[8]),
        widget: [hex(c[9]), hex(c[10]), hex(c[11])],
        error: hex(0xE5786D), strong: hex(0xFFFFFF),
    }
}

const fn light(c: [u32; 12]) -> Palette {
    Palette {
        bg: hex(c[0]), surface: hex(c[1]), raise: hex(c[2]), border: hex(c[3]),
        text: hex(c[4]), muted: hex(c[5]), faint: hex(c[6]), accent: hex(c[7]), input: hex(c[8]),
        widget: [hex(c[9]), hex(c[10]), hex(c[11])],
        error: hex(0xC4473A), strong: hex(0x000000),
    }
}

/// The first is the default.
pub const THEMES: &[ThemeDef] = &[
    ThemeDef {
        name: "Copper",
        dark: dark([0x151311, 0x1C1A17, 0x24211D, 0x2D2925, 0xEAE4DD, 0xA39A90, 0x766E66, 0xD38D5C, 0x1F1C19, 0x26231F, 0x302C27, 0x3A352F]),
        light: light([0xF7F4F0, 0xFFFFFF, 0xEFEAE4, 0xE4DDD5, 0x25211D, 0x6E655C, 0x9C9389, 0xB0643A, 0xFFFFFF, 0xEDE8E2, 0xE5DFD8, 0xDBD4CC]),
    },
    ThemeDef {
        name: "Graphite",
        dark: dark([0x141518, 0x1B1C20, 0x23252A, 0x2B2D33, 0xE6E7EA, 0x9A9DA6, 0x6B6E77, 0x8AA4F8, 0x1E1F23, 0x25272C, 0x2E3036, 0x383A41]),
        light: light([0xF5F5F7, 0xFFFFFF, 0xECECEF, 0xE1E1E5, 0x1E1F22, 0x62656D, 0x9A9CA3, 0x3B6FE0, 0xFFFFFF, 0xEAEAED, 0xE1E1E5, 0xD6D6DB]),
    },
    ThemeDef {
        name: "Sage",
        dark: dark([0x121513, 0x191D1A, 0x212622, 0x29302A, 0xE3E8E3, 0x97A398, 0x667066, 0x8FC28F, 0x1C201D, 0x222724, 0x2B312D, 0x343B36]),
        light: light([0xF4F6F2, 0xFFFFFF, 0xE9EEE6, 0xDCE3D8, 0x1F2620, 0x5D6A5E, 0x939E92, 0x3F7F4A, 0xFFFFFF, 0xE6EBE3, 0xDCE3D8, 0xD0D8CB]),
    },
    ThemeDef {
        name: "Ocean",
        dark: dark([0x0F1418, 0x151C21, 0x1C252B, 0x243038, 0xDFE8EE, 0x8FA2AE, 0x5F717C, 0x5FC3D6, 0x182026, 0x1D262D, 0x253139, 0x2E3B44]),
        light: light([0xF2F6F8, 0xFFFFFF, 0xE5EDF1, 0xD6E1E7, 0x172229, 0x566A75, 0x8DA0AA, 0x1F7F99, 0xFFFFFF, 0xE2EBF0, 0xD6E1E7, 0xC9D6DD]),
    },
    ThemeDef {
        name: "Plum",
        dark: dark([0x151217, 0x1C181F, 0x252028, 0x2E2832, 0xEAE3EE, 0xA096A8, 0x6F6676, 0xC49BF0, 0x1F1A22, 0x262129, 0x2F2933, 0x39313E]),
        light: light([0xF7F4F8, 0xFFFFFF, 0xEEE8F1, 0xE2DAE6, 0x241E28, 0x6A5F70, 0x9D93A3, 0x7B4BC4, 0xFFFFFF, 0xECE6EF, 0xE2DAE6, 0xD7CDDC]),
    },
    ThemeDef {
        name: "Mono",
        dark: dark([0x111111, 0x181818, 0x202020, 0x2A2A2A, 0xEDEDED, 0x9E9E9E, 0x6B6B6B, 0xF2F2F2, 0x1B1B1B, 0x222222, 0x2C2C2C, 0x363636]),
        light: light([0xF6F6F6, 0xFFFFFF, 0xEDEDED, 0xE0E0E0, 0x161616, 0x626262, 0x9A9A9A, 0x161616, 0xFFFFFF, 0xEBEBEB, 0xE0E0E0, 0xD4D4D4]),
    },
];

static IS_DARK: AtomicBool = AtomicBool::new(true);
static THEME_IDX: AtomicUsize = AtomicUsize::new(0);

fn p() -> &'static Palette {
    let t = &THEMES[THEME_IDX.load(Ordering::Relaxed)];
    if IS_DARK.load(Ordering::Relaxed) { &t.dark } else { &t.light }
}

pub fn is_dark() -> bool {
    IS_DARK.load(Ordering::Relaxed)
}

pub fn BG() -> Color32 { p().bg }
pub fn SURFACE() -> Color32 { p().surface }
pub fn RAISE() -> Color32 { p().raise }
pub fn BORDER() -> Color32 { p().border }
pub fn TEXT() -> Color32 { p().text }
pub fn MUTED() -> Color32 { p().muted }
pub fn FAINT() -> Color32 { p().faint }
pub fn ACCENT() -> Color32 { p().accent }
pub fn INPUT() -> Color32 { p().input }

/// The swatch for a color label, tuned per appearance.
pub fn label(c: rjot_core::Color) -> Color32 {
    use rjot_core::Color::*;
    match (c, is_dark()) {
        (Red, true) => hex(0xF7768E),
        (Red, false) => hex(0xD6334F),
        (Orange, true) => hex(0xFF9E64),
        (Orange, false) => hex(0xD9701F),
        (Yellow, true) => hex(0xE0AF68),
        (Yellow, false) => hex(0xB8860B),
        (Green, true) => hex(0x9ECE6A),
        (Green, false) => hex(0x3E8E41),
        (Blue, true) => hex(0x7AA2F7),
        (Blue, false) => hex(0x2F6FDE),
        (Purple, true) => hex(0xBB9AF7),
        (Purple, false) => hex(0x8250DF),
    }
}

/// `color` laid over `base` at strength `t`.
pub fn tint(base: Color32, color: Color32, t: f32) -> Color32 {
    let mix = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t).round() as u8;
    Color32::from_rgb(mix(base.r(), color.r()), mix(base.g(), color.g()), mix(base.b(), color.b()))
}

/// Call once per frame: picks the palette for the appearance egui shows.
pub fn sync(ctx: &egui::Context) {
    IS_DARK.store(ctx.theme() == Theme::Dark, Ordering::Relaxed);
}

/// SF Pro and SF Mono, keeping egui's bundled fonts as fallbacks.
fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    for (family, path, name) in [
        (FontFamily::Proportional, "/System/Library/Fonts/SFNS.ttf", "ui"),
        (FontFamily::Monospace, "/System/Library/Fonts/SFNSMono.ttf", "mono"),
    ] {
        if let Ok(bytes) = std::fs::read(path) {
            fonts.font_data.insert(name.into(), Arc::new(FontData::from_owned(bytes)));
            fonts.families.entry(family).or_default().insert(0, name.into());
        }
    }
    // Khmer: SF has no Khmer glyphs. Kantumruy Pro when installed, else the
    // system's Khmer Sangam MN. The same font shapes Khmer (see shaping.rs).
    let home = std::env::var("HOME").unwrap_or_default();
    let khmer = [
        format!("{home}/Library/Fonts/KantumruyPro-VariableFont_wght.ttf"),
        "/Library/Fonts/KantumruyPro-VariableFont_wght.ttf".to_string(),
        "/System/Library/Fonts/Supplemental/Khmer Sangam MN.ttf".to_string(),
    ];
    if let Some(bytes) = khmer.iter().find_map(|p| std::fs::read(p).ok()) {
        crate::shaping::init(bytes.clone());
        fonts.font_data.insert("khmer".into(), Arc::new(FontData::from_owned(bytes)));
        for family in [FontFamily::Proportional, FontFamily::Monospace] {
            let list = fonts.families.entry(family).or_default();
            list.insert(1.min(list.len()), "khmer".into());
        }
    }
    ctx.set_fonts(fonts);
}

/// Fonts, then the theme. Call once at startup.
pub fn init(ctx: &egui::Context, theme: &str, appearance: Appearance) {
    install_fonts(ctx);
    apply(ctx, theme, appearance);
}

/// Switch theme and appearance.
pub fn apply(ctx: &egui::Context, theme: &str, appearance: Appearance) {
    let idx = THEMES.iter().position(|t| t.name == theme).unwrap_or(0);
    THEME_IDX.store(idx, Ordering::Relaxed);
    let t = &THEMES[idx];
    ctx.style_mut_of(Theme::Dark, |s| style_for(s, &t.dark, Visuals::dark()));
    ctx.style_mut_of(Theme::Light, |s| style_for(s, &t.light, Visuals::light()));
    ctx.set_theme(match appearance {
        Appearance::System => ThemePreference::System,
        Appearance::Light => ThemePreference::Light,
        Appearance::Dark => ThemePreference::Dark,
    });
    sync(ctx);
}

fn style_for(style: &mut egui::Style, c: &Palette, base: Visuals) {
    let mut v = base;
    v.override_text_color = Some(c.text);
    v.weak_text_color = Some(c.faint);
    v.panel_fill = c.bg;
    v.window_fill = c.surface;
    v.extreme_bg_color = c.input;
    v.code_bg_color = c.raise;
    v.window_stroke = Stroke::new(1.0_f32, c.border);
    v.window_corner_radius = CornerRadius::same(10);
    v.menu_corner_radius = CornerRadius::same(8);
    v.hyperlink_color = c.accent;
    v.error_fg_color = c.error;
    v.selection.bg_fill = c.accent.gamma_multiply(0.32);
    v.selection.stroke = Stroke::new(1.0_f32, c.accent);
    v.text_cursor.stroke = Stroke::new(1.5_f32, c.accent);

    let r = CornerRadius::same(6);
    let w = &mut v.widgets;
    w.noninteractive.bg_stroke = Stroke::new(1.0_f32, c.border);
    w.noninteractive.fg_stroke = Stroke::new(1.0_f32, c.text);
    w.noninteractive.corner_radius = r;
    for (state, fill) in [&mut w.inactive, &mut w.hovered, &mut w.active].into_iter().zip(c.widget) {
        state.bg_fill = fill;
        state.weak_bg_fill = fill;
        state.bg_stroke = Stroke::NONE;
        state.corner_radius = r;
        state.expansion = 0.0;
    }
    w.open = w.active;
    w.inactive.fg_stroke = Stroke::new(1.0_f32, c.text);
    w.hovered.fg_stroke = Stroke::new(1.0_f32, c.strong);
    w.active.fg_stroke = Stroke::new(1.0_f32, c.strong);
    style.visuals = v;

    style.spacing.item_spacing = vec2(8.0, 6.0);
    style.spacing.button_padding = vec2(10.0, 4.0);
    style.spacing.interact_size.y = 26.0;
    style.spacing.scroll.bar_width = 6.0;
    style.spacing.scroll.floating = true;
    style.text_styles = [
        (TextStyle::Heading, FontId::proportional(15.0)),
        (TextStyle::Body, FontId::proportional(13.5)),
        (TextStyle::Button, FontId::proportional(13.0)),
        (TextStyle::Small, FontId::proportional(11.5)),
        (TextStyle::Monospace, FontId::monospace(12.5)),
    ]
    .into();
}
