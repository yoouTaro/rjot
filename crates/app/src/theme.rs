//! Warm, quiet palettes with one copper accent. Dark and light follow the
//! macOS appearance.

#![allow(non_snake_case)]

use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, TextStyle,
    Theme, ThemePreference, Visuals, vec2,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

struct Palette {
    bg: Color32,
    /// Cards.
    surface: Color32,
    /// Hovered cards, segmented control track.
    raise: Color32,
    border: Color32,
    text: Color32,
    muted: Color32,
    faint: Color32,
    accent: Color32,
    error: Color32,
    input: Color32,
    /// Button fills: inactive, hovered, pressed.
    widget: [Color32; 3],
    strong: Color32,
}

const fn hex(v: u32) -> Color32 {
    Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

const DARK: Palette = Palette {
    bg: hex(0x151311),
    surface: hex(0x1C1A17),
    raise: hex(0x24211D),
    border: hex(0x2D2925),
    text: hex(0xEAE4DD),
    muted: hex(0xA39A90),
    faint: hex(0x766E66),
    accent: hex(0xD38D5C),
    error: hex(0xE5786D),
    input: hex(0x1F1C19),
    widget: [hex(0x26231F), hex(0x302C27), hex(0x3A352F)],
    strong: hex(0xFFFFFF),
};

const LIGHT: Palette = Palette {
    bg: hex(0xF7F4F0),
    surface: hex(0xFFFFFF),
    raise: hex(0xEFEAE4),
    border: hex(0xE4DDD5),
    text: hex(0x25211D),
    muted: hex(0x6E655C),
    faint: hex(0x9C9389),
    accent: hex(0xB0643A),
    error: hex(0xC4473A),
    input: hex(0xFFFFFF),
    widget: [hex(0xEDE8E2), hex(0xE5DFD8), hex(0xDBD4CC)],
    strong: hex(0x000000),
};

static IS_DARK: AtomicBool = AtomicBool::new(true);

fn p() -> &'static Palette {
    if IS_DARK.load(Ordering::Relaxed) { &DARK } else { &LIGHT }
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
    ctx.set_fonts(fonts);
}

pub fn apply(ctx: &egui::Context) {
    install_fonts(ctx);
    ctx.style_mut_of(Theme::Dark, |s| style_for(s, &DARK, Visuals::dark()));
    ctx.style_mut_of(Theme::Light, |s| style_for(s, &LIGHT, Visuals::light()));
    ctx.set_theme(ThemePreference::System);
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
