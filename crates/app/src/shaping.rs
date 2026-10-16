//! Shaped Khmer.
//!
//! egui lays text out one codepoint at a time with no OpenType shaping,
//! which breaks Khmer: subscript consonants (coeng) don't stack and pre-base
//! vowels land on the wrong side. So Khmer is shaped with rustybuzz (a
//! HarfBuzz port), rasterized with ab_glyph and drawn over the laid-out
//! text, one syllable cluster at a time: egui keeps doing the layout (wrapping,
//! cursor, selection) with the Khmer glyphs made invisible and spaced to the
//! shaped width, and the shaped clusters are painted where it put them.
//! Adapted from rgrip, which shapes whole runs on one line; clusters let
//! long Khmer text wrap.

use ab_glyph::{Font, FontRef, GlyphId, PxScale, point};
use eframe::egui::{self, Color32, ColorImage, Painter, Pos2, Rect, TextureHandle, TextureOptions, Vec2};
use eframe::egui::text::LayoutJob;
use std::collections::HashMap;
use std::ops::Range;
use std::sync::{Arc, Mutex, OnceLock};

pub fn is_khmer(c: char) -> bool {
    matches!(c, '\u{1780}'..='\u{17FF}' | '\u{19E0}'..='\u{19FF}')
}

pub fn has_khmer(text: &str) -> bool {
    text.chars().any(is_khmer)
}

/// Vowel signs, diacritics and the coeng: they attach to the character
/// before them.
fn is_khmer_mark(c: char) -> bool {
    matches!(c, '\u{17B4}'..='\u{17D3}' | '\u{17DD}')
}

const COENG: char = '\u{17D2}';

fn is_joiner(c: char) -> bool {
    matches!(c, '\u{200B}'..='\u{200D}')
}

/// Byte ranges of Khmer syllable clusters: a base with the marks after it,
/// where a coeng pulls the next consonant into the same cluster. A zero-width
/// space or joiner after Khmer joins the cluster before it, so it's hidden
/// too (egui would draw it as a box) and its width can be taken back.
pub fn clusters(text: &str) -> Vec<Range<usize>> {
    let mut out: Vec<Range<usize>> = Vec::new();
    let mut prev: Option<char> = None;
    for (i, c) in text.char_indices() {
        let end = i + c.len_utf8();
        let joins = (is_khmer_mark(c) || is_joiner(c) || prev == Some(COENG)) && prev.is_some_and(|p| is_khmer(p) || is_joiner(p));
        if is_khmer(c) || (is_joiner(c) && prev.is_some_and(is_khmer)) {
            match out.last_mut() {
                Some(r) if joins && r.end == i => r.end = end,
                _ => out.push(i..end),
            }
        }
        prev = Some(c);
    }
    out
}

/// Cut `text` to at most `max` characters without splitting a cluster.
pub fn truncate(text: &str, max: usize) -> &str {
    let Some((cut, _)) = text.char_indices().nth(max) else { return text };
    let mut cut = cut;
    for r in clusters(text) {
        if r.start < cut && cut < r.end {
            cut = r.start;
        }
    }
    &text[..cut]
}

fn shape_run(face: &rustybuzz::Face, run: &str) -> rustybuzz::GlyphBuffer {
    let mut buffer = rustybuzz::UnicodeBuffer::new();
    buffer.push_str(run);
    buffer.guess_segment_properties();
    buffer.set_flags(rustybuzz::BufferFlags::REMOVE_DEFAULT_IGNORABLES | rustybuzz::BufferFlags::DO_NOT_INSERT_DOTTED_CIRCLE);
    rustybuzz::shape(face, &[], buffer)
}

struct Shaper {
    bytes: &'static [u8],
    cache: Mutex<HashMap<(String, u32), Arc<Shaped>>>,
}

pub struct Shaped {
    texture: TextureHandle,
    /// Size in points.
    pub size: Vec2,
    /// Width of the glyphs alone (without padding), in points.
    pub advance: f32,
}

static SHAPER: OnceLock<Option<Shaper>> = OnceLock::new();
const PAD: f32 = 2.0;

/// Load the Khmer font once, the same one the theme adds as a fallback.
pub fn init(font: Vec<u8>) {
    let _ = SHAPER.set(Some(Shaper { bytes: Box::leak(font.into_boxed_slice()), cache: Mutex::default() }));
}

fn shaper() -> Option<&'static Shaper> {
    SHAPER.get_or_init(|| None).as_ref()
}

/// Shape and rasterize `text` at `size` points: white glyphs with alpha,
/// tinted when drawn. `None` without a Khmer font.
pub fn shape(ctx: &egui::Context, text: &str, size: f32) -> Option<Arc<Shaped>> {
    let shaper = shaper()?;
    let ppp = ctx.pixels_per_point();
    let px = size * ppp;
    let key = (text.to_string(), (px * 10.0) as u32);
    if let Some(hit) = shaper.cache.lock().unwrap().get(&key) {
        return Some(hit.clone());
    }
    let face = rustybuzz::Face::from_slice(shaper.bytes, 0)?;
    let font = FontRef::try_from_slice(shaper.bytes).ok()?;
    let scale = px / face.units_per_em() as f32;
    let ascent = face.ascender() as f32 * scale;
    let descent = -face.descender() as f32 * scale;
    let glyph_scale = PxScale::from(px * font.height_unscaled() / font.units_per_em()?);
    let mut pen = 0.0;
    let mut placed = Vec::new();
    let glyphs = shape_run(&face, text);
    for (info, pos) in glyphs.glyph_infos().iter().zip(glyphs.glyph_positions()) {
        let x = pen + pos.x_offset as f32 * scale;
        let y = ascent + PAD - pos.y_offset as f32 * scale;
        if info.glyph_id != 0 {
            placed.push(GlyphId(info.glyph_id as u16).with_scale_and_position(glyph_scale, point(x + PAD, y)));
        }
        pen += pos.x_advance as f32 * scale;
    }
    let width = (pen + 2.0 * PAD).ceil().max(1.0) as usize;
    let height = (ascent + descent + 2.0 * PAD).ceil().max(1.0) as usize;
    let mut alpha = vec![0.0f32; width * height];
    for glyph in placed {
        let Some(outline) = font.outline_glyph(glyph) else { continue };
        let b = outline.px_bounds();
        outline.draw(|gx, gy, cov| {
            let x = b.min.x as i32 + gx as i32;
            let y = b.min.y as i32 + gy as i32;
            if (0..width as i32).contains(&x) && (0..height as i32).contains(&y) {
                let a = &mut alpha[y as usize * width + x as usize];
                *a = (*a + cov).min(1.0);
            }
        });
    }
    let pixels = alpha.iter().map(|a| Color32::from_white_alpha((a * 255.0).round() as u8)).collect();
    let image = ColorImage { size: [width, height], pixels, source_size: Vec2::new(width as f32, height as f32) };
    let texture = ctx.load_texture(format!("khmer:{text}"), image, TextureOptions::LINEAR);
    let shaped = Arc::new(Shaped { texture, size: Vec2::new(width as f32, height as f32) / ppp, advance: pen / ppp });
    let mut cache = shaper.cache.lock().unwrap();
    if cache.len() > 6000 {
        cache.clear();
    }
    cache.insert(key, shaped.clone());
    Some(shaped)
}

/// Khmer clusters hidden in a job, to be painted shaped afterwards.
pub type Runs = Vec<(Range<usize>, Color32, f32)>;

/// Hide the Khmer clusters in `job` and space their (invisible) glyphs so
/// each takes its shaped width; wrapping, cursor and selection then line up
/// with what [`paint_runs`] draws.
pub fn hide_runs(ui: &egui::Ui, job: &mut LayoutJob) -> Runs {
    if shaper().is_none() {
        return Vec::new();
    }
    let ranges = clusters(&job.text);
    if ranges.is_empty() {
        return Vec::new();
    }
    let mut runs = Vec::new();
    let mut spacing: Vec<(Range<usize>, f32)> = Vec::new();
    for r in &ranges {
        let Some(section) = job.sections.iter().find(|s| s.byte_range.contains(&r.start)) else { continue };
        let (font, color) = (section.format.font_id.clone(), section.format.color);
        let text = &job.text[r.clone()];
        let Some(shaped) = shape(ui.ctx(), text, font.size) else { continue };
        let plain = ui.fonts_mut(|f| f.layout_no_wrap(text.to_string(), font.clone(), color)).size().x;
        // egui adds letter spacing between the glyphs of a section, not
        // after the last, so n characters get n - 1 spacings.
        let gaps = text.chars().count().saturating_sub(1).max(1) as f32;
        spacing.push((r.clone(), (shaped.advance - plain) / gaps));
        runs.push((r.clone(), color, font.size));
    }
    let mut sections = Vec::new();
    for s in job.sections.drain(..) {
        let mut at = s.byte_range.start;
        while at < s.byte_range.end {
            let inside = spacing.iter().find(|(r, _)| r.contains(&at));
            let end = match inside {
                Some((r, _)) => r.end.min(s.byte_range.end),
                None => spacing.iter().map(|(r, _)| r.start).filter(|&x| x > at).min().unwrap_or(s.byte_range.end).min(s.byte_range.end),
            };
            let mut part = s.clone();
            part.byte_range = at..end;
            if let Some((_, extra)) = inside {
                part.format.color = Color32::TRANSPARENT;
                part.format.extra_letter_spacing = *extra;
            }
            sections.push(part);
            at = end;
        }
    }
    job.sections = sections;
    runs
}

/// Draw the clusters [`hide_runs`] hid, shaped, where the galley put them.
/// `blur` (0-1) draws them out of focus, like the rest of a blurred card.
pub fn paint_runs(painter: &Painter, galley: &egui::Galley, galley_pos: Pos2, runs: &Runs, tint: Option<Color32>, blur: f32) {
    let text = galley.text();
    for (r, color, size) in runs {
        let Some(run) = text.get(r.clone()) else { continue };
        let Some(shaped) = shape(painter.ctx(), run, *size) else { continue };
        let index = text[..r.start].chars().count();
        // A cluster that starts a wrapped row belongs to that row, not the
        // end of the one before.
        let at = galley.pos_from_cursor(egui::text::CCursor { index, prefer_next_row: true });
        let rect = Rect::from_min_size(
            Pos2::new(galley_pos.x + at.min.x - PAD / painter.ctx().pixels_per_point(), galley_pos.y + at.center().y - shaped.size.y / 2.0),
            shaped.size,
        );
        let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
        let color = tint.unwrap_or(*color);
        if blur > 0.0 {
            for offset in blur_offsets(blur) {
                painter.image(shaped.texture.id(), rect.translate(offset), uv, color.gamma_multiply(egui::lerp(1.0..=0.075, blur)));
            }
        } else {
            painter.image(shaped.texture.id(), rect, uv, color);
        }
    }
}

/// The sample offsets of the blur: the centre plus three staggered rings.
pub fn blur_offsets(amount: f32) -> Vec<Vec2> {
    const RINGS: [(f32, usize); 3] = [(2.0, 8), (4.0, 12), (6.5, 16)];
    let mut out = vec![Vec2::ZERO];
    for (ring, (radius, count)) in RINGS.into_iter().enumerate() {
        let phase = ring as f32 * 0.5;
        for k in 0..count {
            let a = std::f32::consts::TAU * (k as f32 + phase) / count as f32;
            out.push(Vec2::new(a.cos(), a.sin()) * radius * amount);
        }
    }
    out
}

/// Lay `job` out at the cursor and draw it with Khmer shaped (blurred when
/// `blur` > 0). For display text; editors use [`hide_runs`] in a layouter.
pub fn show(ui: &mut egui::Ui, mut job: LayoutJob, sense: egui::Sense, color: Color32, blur: f32) -> (egui::Response, Arc<egui::Galley>) {
    let runs = hide_runs(ui, &mut job);
    let galley = ui.fonts_mut(|f| f.layout_job(job));
    let (rect, response) = ui.allocate_exact_size(galley.size(), sense);
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        if blur > 0.0 {
            let ink = color.gamma_multiply(egui::lerp(1.0..=0.075, blur));
            for offset in blur_offsets(blur) {
                painter.galley_with_override_text_color(rect.min + offset, galley.clone(), ink);
            }
            paint_runs(painter, &galley, rect.min, &runs, Some(color), blur);
        } else {
            painter.galley(rect.min, galley.clone(), color);
            paint_runs(painter, &galley, rect.min, &runs, None, 0.0);
        }
    }
    (response, galley)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split(t: &str) -> Vec<&str> {
        clusters(t).into_iter().map(|r| &t[r]).collect()
    }

    #[test]
    fn syllable_clusters() {
        // ខ្ញុំ = ខ + coeng + ញ + u + nikahit: one cluster.
        assert_eq!(split("ខ្ញុំ"), ["ខ្ញុំ"]);
        assert_eq!(split("កូនខ្ញុំ"), ["កូ", "ន", "ខ្ញុំ"]);
        // A pre-base vowel stays with its consonant; Latin is left alone.
        assert_eq!(split("ok សេវា 2"), ["សេ", "វា"]);
        // Zero-width spaces between Khmer are hidden too.
        assert_eq!(split("ភ្នំ\u{200B}ពេញ"), ["ភ្នំ\u{200B}", "ពេ", "ញ"]);
        assert!(split("plain text").is_empty());
    }

    #[test]
    fn truncates_on_cluster_edges() {
        let t = "ខ្ញុំស្រលាញ់";
        let cut = truncate(t, 3);
        assert!(t.starts_with(cut));
        assert_eq!(cut, "");
        assert_eq!(truncate(t, 5), "ខ្ញុំ");
        assert_eq!(truncate("abc", 10), "abc");
    }

    #[test]
    fn fonts_shape_without_boxes() {
        let home = std::env::var("HOME").unwrap_or_default();
        let fonts = [
            format!("{home}/Library/Fonts/KantumruyPro-VariableFont_wght.ttf"),
            "/Library/Fonts/KantumruyPro-VariableFont_wght.ttf".to_string(),
            "/System/Library/Fonts/Supplemental/Khmer Sangam MN.ttf".to_string(),
        ];
        for bytes in fonts.iter().filter_map(|p| std::fs::read(p).ok()) {
            let face = rustybuzz::Face::from_slice(&bytes, 0).unwrap();
            let dotted = face.glyph_index('\u{25CC}').map_or(u32::MAX, |g| g.0 as u32);
            for text in ["ខ្ញុំស្រលាញ់ភាសាខ្មែរ", "សួស្តី\u{200B}ពិភពលោក", "ការ៉េ"] {
                for r in clusters(text) {
                    let glyphs = shape_run(&face, &text[r]);
                    assert!(glyphs.glyph_infos().iter().all(|g| g.glyph_id != 0 && g.glyph_id != dotted), "{text:?}");
                }
            }
        }
    }
}
