//! Lucide line icons (ISC), rasterised once per size with resvg and drawn
//! tinted.

use eframe::egui::{self, Color32, ColorImage, Rect, TextureHandle, TextureOptions, Ui};
use resvg::{tiny_skia, usvg};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Svg {
    Copy,
    Trash,
    Pencil,
    Check,
    Search,
    X,
    /// Move a note to the queue.
    ListPlus,
    /// Move a prompt back to notes.
    StickyNote,
}

impl Svg {
    fn body(self) -> &'static str {
        match self {
            Svg::Copy => r#"<rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/>"#,
            Svg::Trash => r#"<path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/>"#,
            Svg::Pencil => r#"<path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"/><path d="m15 5 4 4"/>"#,
            Svg::Check => r#"<path d="M20 6 9 17l-5-5"/>"#,
            Svg::Search => r#"<circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>"#,
            Svg::X => r#"<path d="M18 6 6 18"/><path d="m6 6 12 12"/>"#,
            Svg::ListPlus => r#"<path d="M11 12H3"/><path d="M16 6H3"/><path d="M16 18H3"/><path d="M18 9v6"/><path d="M21 12h-6"/>"#,
            Svg::StickyNote => r#"<path d="M16 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V8Z"/><path d="M15 3v4a2 2 0 0 0 2 2h4"/>"#,
        }
    }

    fn source(self) -> String {
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{}</svg>"#,
            self.body()
        )
    }
}

/// White-on-transparent pixels of `svg` at `px` pixels square.
fn rasterize(svg: Svg, px: u32) -> Option<ColorImage> {
    let tree = usvg::Tree::from_str(&svg.source(), &usvg::Options::default()).ok()?;
    let mut pixmap = tiny_skia::Pixmap::new(px, px)?;
    let scale = px as f32 / tree.size().width().max(tree.size().height());
    resvg::render(&tree, tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
    Some(ColorImage::from_rgba_premultiplied([px as usize, px as usize], pixmap.data()))
}

/// The icon's texture at `size` points, cached per icon and pixel size.
fn texture(ctx: &egui::Context, svg: Svg, size: f32) -> Option<TextureHandle> {
    let px = (size * ctx.pixels_per_point()).round().max(1.0) as u32;
    let id = egui::Id::new(("svg-icon", svg, px));
    ctx.data_mut(|d| d.get_temp::<TextureHandle>(id)).or_else(|| {
        let image = rasterize(svg, px)?;
        let texture = ctx.load_texture(format!("icon-{svg:?}-{px}"), image, TextureOptions::LINEAR);
        ctx.data_mut(|d| d.insert_temp(id, texture.clone()));
        Some(texture)
    })
}

/// Draw `svg` in `rect`, tinted.
pub fn paint(ui: &Ui, svg: Svg, rect: Rect, tint: Color32) {
    if let Some(t) = texture(ui.ctx(), svg, rect.width().max(rect.height())) {
        let uv = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        ui.painter().image(t.id(), rect, uv, tint);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icons_rasterize() {
        for svg in [Svg::Copy, Svg::Trash, Svg::Pencil, Svg::Check, Svg::Search, Svg::X, Svg::ListPlus, Svg::StickyNote] {
            let img = rasterize(svg, 32).expect("valid svg");
            assert!(img.pixels.iter().any(|p| p.a() > 200), "{svg:?} drew something");
        }
    }
}
