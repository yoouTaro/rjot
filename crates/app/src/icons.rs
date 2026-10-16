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
    /// Set a color label.
    Tag,
    /// Filter by color label.
    ListFilter,
    /// Settings.
    Sliders,
    /// Sensitive mode is on.
    EyeOff,
    /// Keep at the top.
    Pin,
    /// Move out of the way.
    Archive,
    /// Take an archived item back.
    ArchiveRestore,
    /// The page a capture came from.
    Link,
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
            Svg::Tag => r#"<path d="M12.586 2.586A2 2 0 0 0 11.172 2H4a2 2 0 0 0-2 2v7.172a2 2 0 0 0 .586 1.414l8.704 8.704a2.426 2.426 0 0 0 3.42 0l6.58-6.58a2.426 2.426 0 0 0 0-3.42z"/><circle cx="7.5" cy="7.5" r=".5" fill="white"/>"#,
            Svg::ListFilter => r#"<path d="M3 6h18"/><path d="M7 12h10"/><path d="M10 18h4"/>"#,
            Svg::Sliders => r#"<line x1="21" x2="14" y1="4" y2="4"/><line x1="10" x2="3" y1="4" y2="4"/><line x1="21" x2="12" y1="12" y2="12"/><line x1="8" x2="3" y1="12" y2="12"/><line x1="21" x2="16" y1="20" y2="20"/><line x1="12" x2="3" y1="20" y2="20"/><line x1="14" x2="14" y1="2" y2="6"/><line x1="8" x2="8" y1="10" y2="14"/><line x1="16" x2="16" y1="18" y2="22"/>"#,
            Svg::EyeOff => r#"<path d="M10.733 5.076a10.744 10.744 0 0 1 11.205 6.575 1 1 0 0 1 0 .696 10.747 10.747 0 0 1-1.444 2.49"/><path d="M14.084 14.158a3 3 0 0 1-4.242-4.242"/><path d="M17.479 17.499a10.75 10.75 0 0 1-15.417-5.151 1 1 0 0 1 0-.696 10.75 10.75 0 0 1 4.446-5.143"/><path d="m2 2 20 20"/>"#,
            Svg::Pin => r#"<path d="M12 17v5"/><path d="M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H8a2 2 0 0 0 0 4 1 1 0 0 1 1 1z"/>"#,
            Svg::Archive => r#"<rect width="20" height="5" x="2" y="3" rx="1"/><path d="M4 8v11a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8"/><path d="M10 12h4"/>"#,
            Svg::ArchiveRestore => r#"<rect width="20" height="5" x="2" y="3" rx="1"/><path d="M4 8v11a2 2 0 0 0 2 2h2"/><path d="M20 8v11a2 2 0 0 1-2 2h-2"/><path d="m9 15 3-3 3 3"/><path d="M12 12v9"/>"#,
            Svg::Link => r#"<path d="M15 3h6v6"/><path d="M10 14 21 3"/><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>"#,
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
        for svg in [Svg::Copy, Svg::Trash, Svg::Pencil, Svg::Check, Svg::Search, Svg::X, Svg::ListPlus, Svg::StickyNote, Svg::Tag, Svg::ListFilter, Svg::Sliders, Svg::EyeOff, Svg::Pin, Svg::Archive, Svg::ArchiveRestore, Svg::Link] {
            let img = rasterize(svg, 32).expect("valid svg");
            assert!(img.pixels.iter().any(|p| p.a() > 200), "{svg:?} drew something");
        }
    }
}
