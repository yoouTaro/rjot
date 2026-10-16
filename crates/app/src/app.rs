//! The window: Notes (captured and typed), Queue (prompts to use later and
//! check off) and Scratch (one free-form pad).

use crate::capture::{self, Signal};
use crate::icons::{self, Svg};
use crate::{macos, theme};
use eframe::egui::{
    self, Align, CornerRadius, FontId, Frame, Id, Key, Layout, Margin, Modifiers, Rect, Response,
    RichText, Sense, Stroke, TextFormat, TextStyle, Ui, text::LayoutJob, vec2,
};
use rjot_core::store::matches;
use rjot_core::{Appearance, Color, Item, Kind, Settings, Store, time};
use std::collections::HashSet;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Notes,
    Queue,
    Scratch,
}

impl Tab {
    fn kind(self) -> Option<Kind> {
        match self {
            Tab::Notes => Some(Kind::Note),
            Tab::Queue => Some(Kind::Prompt),
            Tab::Scratch => None,
        }
    }
}

enum Undo {
    /// Put removed items back at their positions.
    Restore(Vec<(usize, Item)>),
    /// Uncheck a prompt that copying checked off.
    Uncheck(u64),
}

struct Toast {
    text: String,
    undo: Option<Undo>,
    until: f64,
}

enum Action {
    Copy(u64),
    Delete(u64),
    Toggle(u64),
    Move(u64, Kind),
    Edit(u64),
    SaveEdit,
    CancelEdit,
    Expand(u64),
    ClearDone,
    SetColor(u64, Option<Color>),
}

/// Lines a card shows before it's expanded.
const COLLAPSED_ROWS: usize = 6;
/// How long a fresh capture glows.
const FLASH_SECS: f64 = 1.6;

pub struct App {
    store: Arc<Mutex<Store>>,
    signals: Receiver<Signal>,
    tab: Tab,
    /// Composer text for Notes and Queue.
    drafts: [String; 2],
    /// Open search and its query.
    search: Option<String>,
    /// Show only items with this color label.
    color_filter: Option<Color>,
    focus_search: bool,
    focus_composer: bool,
    editing: Option<(u64, String)>,
    expanded: HashSet<u64>,
    flash: Option<(u64, f64)>,
    toast: Option<Toast>,
    trusted: bool,
    trust_checked: f64,
    /// Opened by double-tap Shift with nothing selected: hide again once
    /// the note is saved or dismissed.
    quick: bool,
    scratch: String,
    scratch_edited: Option<f64>,
    /// A copy of the stored settings.
    settings: Settings,
    /// Whether the window is currently kept out of screen capture.
    window_private: Option<bool>,
    utc_offset: i64,
    demo: Demo,
}

/// Scripted scenes for scripts/screenshots.sh, read from RJOT_DEMO_*
/// variables in debug builds only.
struct Demo {
    tab: Tab,
    /// Look as if Accessibility access were granted.
    trusted: bool,
    /// The card that looks hovered.
    hover: Option<u64>,
    /// The card whose color picker is open.
    pick: Option<u64>,
    settings: bool,
    /// Leave the window capturable in sensitive mode.
    capturable: bool,
    /// File to create once the scene has settled.
    ready: Option<std::path::PathBuf>,
    frames: u32,
}

impl Demo {
    fn from_env() -> Demo {
        let var = |k: &str| std::env::var(k).ok().filter(|_| cfg!(debug_assertions));
        Demo {
            tab: match var("RJOT_DEMO_TAB").as_deref() {
                Some("queue") => Tab::Queue,
                Some("scratch") => Tab::Scratch,
                _ => Tab::Notes,
            },
            trusted: var("RJOT_DEMO_TRUSTED").is_some(),
            hover: var("RJOT_DEMO_HOVER").and_then(|v| v.parse().ok()),
            pick: var("RJOT_DEMO_PICK").and_then(|v| v.parse().ok()),
            settings: var("RJOT_DEMO_SETTINGS").is_some(),
            capturable: var("RJOT_DEMO_CAPTURABLE").is_some(),
            ready: var("RJOT_DEMO_READY").map(Into::into),
            frames: 0,
        }
    }

    /// Popups opened by the scene stay open only for the first frames;
    /// after that they're left to egui.
    fn opening(&self) -> bool {
        self.frames < 3
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext) -> App {
        // RJOT_DATA points at another note file (for trying things out).
        let path = std::env::var_os("RJOT_DATA").map(std::path::PathBuf::from).unwrap_or_else(Store::default_path);
        let (store, error) = match Store::open(&path) {
            Ok(s) => (s, None),
            Err(e) => (Store::empty(&path), Some(e)),
        };
        let scratch = store.scratch().to_string();
        let settings = store.settings().clone();
        theme::init(&cc.egui_ctx, &settings.theme, settings.appearance);
        let store = Arc::new(Mutex::new(store));
        let signals = capture::start(store.clone(), cc.egui_ctx.clone());
        let demo = Demo::from_env();
        App {
            store,
            signals,
            tab: demo.tab,
            drafts: Default::default(),
            search: None,
            color_filter: None,
            focus_search: false,
            focus_composer: false,
            editing: None,
            expanded: HashSet::new(),
            flash: None,
            toast: error.map(|text| Toast { text, undo: None, until: f64::INFINITY }),
            trusted: demo.trusted || macos::trusted(),
            trust_checked: 0.0,
            quick: false,
            scratch,
            scratch_edited: None,
            settings,
            window_private: None,
            utc_offset: macos::utc_offset(),
            demo,
        }
    }

    /// Change the store and save it, reporting a failed save.
    fn change<R>(&mut self, f: impl FnOnce(&mut Store) -> R) -> R {
        let store = self.store.clone();
        let mut store = store.lock().unwrap();
        let out = f(&mut store);
        if let Err(e) = store.save() {
            self.notify(format!("Couldn't save {}: {e}", store.path().display()), None);
        }
        out
    }

    fn notify(&mut self, text: impl Into<String>, undo: Option<Undo>) {
        self.toast = Some(Toast { text: text.into(), undo, until: f64::NAN });
    }

    /// Change settings, save them, and re-apply the theme.
    fn set_settings(&mut self, ctx: &egui::Context, f: impl FnOnce(&mut Settings)) {
        f(&mut self.settings);
        let settings = self.settings.clone();
        theme::apply(ctx, &settings.theme, settings.appearance);
        self.change(|s| *s.settings_mut() = settings);
    }

    /// How blurred note text is, 0 (clear) to 1: sensitive mode on and ⌥
    /// not held. Animates, so peeking eases the text into focus.
    fn blur(&self, ui: &Ui) -> f32 {
        let hidden = self.settings.sensitive && !ui.input(|i| i.modifiers.alt);
        ui.ctx().animate_bool_with_time(Id::new("blur"), hidden, 0.18)
    }

    fn flush_scratch(&mut self) {
        if self.scratch_edited.take().is_some() {
            let text = self.scratch.clone();
            self.change(|s| s.set_scratch(text));
        }
    }

    fn handle_signals(&mut self, ctx: &egui::Context, now: f64) {
        while let Ok(signal) = self.signals.try_recv() {
            match signal {
                Signal::Saved(id) => {
                    self.flash = Some((id, now));
                    let source = self.store.lock().unwrap().get(id).and_then(|i| i.source.clone());
                    let text = match source {
                        Some(app) => format!("Saved from {app}"),
                        None => "Saved".to_string(),
                    };
                    self.notify(text, None);
                }
                Signal::QuickNote => {
                    if self.tab == Tab::Scratch {
                        self.tab = Tab::Notes;
                    }
                    self.quick = true;
                    self.focus_composer = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                }
                Signal::Error(e) => self.notify(e, None),
            }
        }
    }

    fn shortcuts(&mut self, ctx: &egui::Context) {
        let (n1, n2, n3, find, new, close) = ctx.input_mut(|i| {
            (
                i.consume_key(Modifiers::COMMAND, Key::Num1),
                i.consume_key(Modifiers::COMMAND, Key::Num2),
                i.consume_key(Modifiers::COMMAND, Key::Num3),
                i.consume_key(Modifiers::COMMAND, Key::F),
                i.consume_key(Modifiers::COMMAND, Key::N),
                i.consume_key(Modifiers::COMMAND, Key::W),
            )
        });
        if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::S)) {
            let on = !self.settings.sensitive;
            self.set_settings(ctx, |s| s.sensitive = on);
            self.notify(if on { "Sensitive mode on" } else { "Sensitive mode off" }, None);
        }
        for (hit, tab) in [(n1, Tab::Notes), (n2, Tab::Queue), (n3, Tab::Scratch)] {
            if hit {
                self.tab = tab;
            }
        }
        if find {
            if self.tab == Tab::Scratch {
                self.tab = Tab::Notes;
            }
            self.search.get_or_insert_with(String::new);
            self.focus_search = true;
        }
        if new {
            if self.tab == Tab::Scratch {
                self.tab = Tab::Notes;
            }
            self.focus_composer = true;
        }
        if close {
            macos::hide();
        }
        let nothing_focused = ctx.memory(|m| m.focused().is_none());
        if nothing_focused && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            if self.editing.take().is_none() && self.search.take().is_none() && self.quick {
                self.quick = false;
                macos::hide();
            }
        }
    }

    // ---- Top bar ----

    fn top_bar(&mut self, ui: &mut Ui) {
        let drag = ui.interact(ui.max_rect(), Id::new("titlebar"), Sense::click_and_drag());
        if drag.drag_started() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
        let (notes, queue) = {
            let s = self.store.lock().unwrap();
            (s.count(Kind::Note), s.count(Kind::Prompt))
        };
        ui.horizontal_centered(|ui| {
            let track = Frame::new().fill(theme::RAISE()).corner_radius(8).inner_margin(Margin::same(2));
            track.show(ui, |ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                for (tab, label, count) in [(Tab::Notes, "Notes", Some(notes)), (Tab::Queue, "Queue", Some(queue)), (Tab::Scratch, "Scratch", None)] {
                    if segment(ui, label, count, self.tab == tab).clicked() {
                        self.tab = tab;
                    }
                }
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                self.settings_button(ui);
                if self.settings.sensitive && icon_button(ui, Svg::EyeOff, "Sensitive mode is on (⌘⇧S to turn off). Hold ⌥ to peek.").clicked() {
                    self.set_settings(ui.ctx(), |s| s.sensitive = false);
                }
                if self.tab != Tab::Scratch {
                    self.color_filter_button(ui);
                    let open = self.search.is_some();
                    if icon_button(ui, if open { Svg::X } else { Svg::Search }, if open { "Close search" } else { "Search  ⌘F" }).clicked() {
                        if open {
                            self.search = None;
                        } else {
                            self.search = Some(String::new());
                            self.focus_search = true;
                        }
                    }
                }
            });
        });
    }

    fn settings_button(&mut self, ui: &mut Ui) {
        let r = icon_button(ui, Svg::Sliders, "Settings");
        if self.demo.settings && self.demo.opening() {
            egui::Popup::open_id(ui.ctx(), r.id.with("popup"));
        }
        egui::Popup::menu(&r)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .show(|ui| {
                ui.set_width(250.0);
                ui.spacing_mut().item_spacing.y = 4.0;
                section_label(ui, "Appearance");
                let mut appearance = self.settings.appearance;
                Frame::new().fill(theme::RAISE()).corner_radius(8).inner_margin(Margin::same(2)).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;
                        for (a, label) in [(Appearance::System, "System"), (Appearance::Light, "Light"), (Appearance::Dark, "Dark")] {
                            if segment(ui, label, None, appearance == a).clicked() {
                                appearance = a;
                            }
                        }
                    });
                });
                if appearance != self.settings.appearance {
                    self.set_settings(ui.ctx(), |s| s.appearance = appearance);
                }

                ui.add_space(6.0);
                section_label(ui, "Theme");
                for t in theme::THEMES {
                    if theme_row(ui, t, self.settings.theme == t.name).clicked() {
                        self.set_settings(ui.ctx(), |s| s.theme = t.name.to_string());
                    }
                }

                ui.add_space(4.0);
                ui.separator();
                let mut sensitive = self.settings.sensitive;
                if switch_row(ui, &mut sensitive, "Sensitive mode", "Blurs note text (hold ⌥ to peek) and keeps rjot out of screenshots and screen sharing.  ⌘⇧S").changed() {
                    self.set_settings(ui.ctx(), |s| s.sensitive = sensitive);
                }
                let mut sound = self.settings.sound;
                if switch_row(ui, &mut sound, "Sound on capture", "A short pop when a selection is saved.").changed() {
                    self.set_settings(ui.ctx(), |s| s.sound = sound);
                }
                ui.separator();
                let path = self.store.lock().unwrap().path().to_path_buf();
                let reveal = egui::Button::new(RichText::new("Show notes file in Finder").color(theme::MUTED())).frame(false);
                if ui.add(reveal).clicked() {
                    let _ = std::process::Command::new("open").arg("-R").arg(&path).spawn();
                }
            });
    }

    /// Filter by color label: a list icon, or the chosen color's dot.
    fn color_filter_button(&mut self, ui: &mut Ui) {
        let tip = match self.color_filter {
            Some(c) => format!("Showing {} only", c.name().to_lowercase()),
            None => "Filter by color".to_string(),
        };
        let r = match self.color_filter {
            Some(c) => {
                let (rect, r) = ui.allocate_exact_size(vec2(24.0, 22.0), Sense::click());
                if r.hovered() {
                    ui.painter().rect_filled(rect, CornerRadius::same(6), theme::RAISE());
                }
                ui.painter().circle_filled(rect.center(), 5.5, theme::label(c));
                r.on_hover_text(tip)
            }
            None => icon_button(ui, Svg::ListFilter, &tip),
        };
        let kind = self.tab.kind().unwrap_or(Kind::Note);
        let counts: Vec<usize> = {
            let s = self.store.lock().unwrap();
            Color::ALL.iter().map(|c| s.items(kind).filter(|i| i.color == Some(*c)).count()).collect()
        };
        egui::Popup::menu(&r).show(|ui| {
            ui.set_min_width(150.0);
            if ui.selectable_label(self.color_filter.is_none(), "All").clicked() {
                self.color_filter = None;
            }
            ui.separator();
            for (c, n) in Color::ALL.into_iter().zip(counts) {
                let selected = self.color_filter == Some(c);
                if color_row(ui, c, n, selected).clicked() {
                    self.color_filter = if selected { None } else { Some(c) };
                }
            }
        });
    }

    fn search_row(&mut self, ui: &mut Ui) {
        let Some(query) = &mut self.search else { return };
        let mut close = false;
        Frame::new()
            .fill(theme::INPUT())
            .stroke(Stroke::new(1.0_f32, theme::BORDER()))
            .corner_radius(8)
            .inner_margin(Margin::symmetric(10, 5))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
                    icons::paint(ui, Svg::Search, r, theme::FAINT());
                    let edit = egui::TextEdit::singleline(query).frame(false).hint_text("Search").desired_width(f32::INFINITY);
                    let r = ui.add(edit);
                    if std::mem::take(&mut self.focus_search) {
                        r.request_focus();
                    }
                    if r.has_focus() && ui.input(|i| i.key_pressed(Key::Escape)) {
                        close = true;
                    }
                });
            });
        if close {
            self.search = None;
        }
        ui.add_space(8.0);
    }

    // ---- Notes and Queue ----

    fn composer(&mut self, ui: &mut Ui, kind: Kind) {
        let idx = (kind == Kind::Prompt) as usize;
        let id = Id::new(("composer", idx));
        let focused = ui.memory(|m| m.has_focus(id));
        let (mut submit, escape) = if focused {
            ui.input_mut(|i| (i.consume_key(Modifiers::COMMAND, Key::Enter), i.consume_key(Modifiers::NONE, Key::Escape)))
        } else {
            (false, false)
        };
        let hint = match kind {
            Kind::Note => "Write a note…",
            Kind::Prompt => "Queue a prompt for later…",
        };
        let border = if focused { theme::ACCENT().gamma_multiply(0.55) } else { theme::BORDER() };
        Frame::new()
            .fill(theme::INPUT())
            .stroke(Stroke::new(1.0_f32, border))
            .corner_radius(10)
            .inner_margin(Margin::symmetric(12, 9))
            .show(ui, |ui| {
                let edit = egui::TextEdit::multiline(&mut self.drafts[idx])
                    .id(id)
                    .frame(false)
                    .hint_text(hint)
                    .margin(Margin::ZERO)
                    .desired_rows(2)
                    .desired_width(f32::INFINITY);
                let r = ui.add(edit);
                if std::mem::take(&mut self.focus_composer) {
                    r.request_focus();
                }
                if focused || !self.drafts[idx].trim().is_empty() {
                    ui.horizontal(|ui| {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let enabled = !self.drafts[idx].trim().is_empty();
                            let save = egui::Button::new(RichText::new("Save").color(theme::BG()))
                                .fill(if enabled { theme::ACCENT() } else { theme::ACCENT().gamma_multiply(0.4) })
                                .corner_radius(6);
                            if ui.add_enabled(enabled, save).clicked() {
                                submit = true;
                            }
                            ui.label(RichText::new("⌘↩").small().color(theme::FAINT()));
                        });
                    });
                }
            });
        if submit && !self.drafts[idx].trim().is_empty() {
            let text = std::mem::take(&mut self.drafts[idx]);
            if let Some(id) = self.change(|s| s.add(kind, &text, None, time::now())) {
                self.flash = Some((id, ui.input(|i| i.time)));
            }
            if std::mem::take(&mut self.quick) {
                macos::hide();
            }
        }
        if escape {
            ui.memory_mut(|m| m.surrender_focus(id));
            if std::mem::take(&mut self.quick) {
                macos::hide();
            }
        }
    }

    fn list(&mut self, ui: &mut Ui, kind: Kind, actions: &mut Vec<Action>) {
        let query = self.search.clone().unwrap_or_default();
        let (open, done): (Vec<Item>, Vec<Item>) = {
            let s = self.store.lock().unwrap();
            let color = self.color_filter;
            s.items(kind)
                .filter(|i| matches(i, &query) && color.is_none_or(|c| i.color == Some(c)))
                .cloned()
                .partition(|i| !i.done)
        };
        if open.is_empty() && done.is_empty() {
            if !query.trim().is_empty() {
                empty_state(ui, "No matches", None);
            } else if let Some(c) = self.color_filter {
                let what = if kind == Kind::Note { "notes" } else { "prompts" };
                empty_state(ui, &format!("No {} {what}", c.name().to_lowercase()), Some("Hover a card and use the tag button to label it."));
            } else if kind == Kind::Note {
                empty_state(ui, "Nothing saved yet", Some("Select text in any app and double-tap ⇧. It lands here."));
            } else {
                empty_state(ui, "No prompts queued", Some("Write prompts here, copy one when you need it, and it's checked off."));
            }
            return;
        }
        let now = time::now();
        for item in &open {
            self.card(ui, item, now, actions);
        }
        if !done.is_empty() {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("Done · {}", done.len())).small().color(theme::FAINT()));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let clear = egui::Button::new(RichText::new("Clear").small().color(theme::MUTED())).frame(false);
                    if ui.add(clear).clicked() {
                        actions.push(Action::ClearDone);
                    }
                });
            });
            for item in &done {
                self.card(ui, item, now, actions);
            }
        }
    }

    fn card(&mut self, ui: &mut Ui, item: &Item, now: i64, actions: &mut Vec<Action>) {
        let hover_id = Id::new(("card-hover", item.id));
        let color_popup = Id::new(("color-popup", item.id));
        if self.demo.pick == Some(item.id) && self.demo.opening() {
            egui::Popup::open_id(ui.ctx(), color_popup);
        }
        let picking = egui::Popup::is_id_open(ui.ctx(), color_popup);
        let hovered = picking
            || self.demo.hover == Some(item.id)
            || ui.data(|d| d.get_temp::<bool>(hover_id)).unwrap_or(false);
        let t = ui.input(|i| i.time);
        let glow = match self.flash {
            Some((id, at)) if id == item.id && t - at < FLASH_SECS => {
                ui.ctx().request_repaint();
                (1.0 - (t - at) / FLASH_SECS) as f32
            }
            _ => 0.0,
        };
        let fill = theme::tint(if hovered { theme::tint(theme::SURFACE(), theme::RAISE(), 0.6) } else { theme::SURFACE() }, theme::ACCENT(), 0.16 * glow);
        let stroke = theme::tint(theme::BORDER(), theme::ACCENT(), 0.7 * glow);
        let editing = self.editing.as_ref().is_some_and(|(id, _)| *id == item.id);

        let frame = Frame::new()
            .fill(fill)
            .stroke(Stroke::new(1.0_f32, stroke))
            .corner_radius(10)
            .inner_margin(Margin { left: 12, right: 8, top: 9, bottom: 6 })
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.spacing_mut().item_spacing.y = 2.0;
                if editing {
                    self.edit_card(ui, actions);
                    return;
                }
                ui.horizontal_top(|ui| {
                    if item.kind == Kind::Prompt && check_box(ui, item.done).clicked() {
                        actions.push(Action::Toggle(item.id));
                    }
                    let expanded = self.expanded.contains(&item.id);
                    let blur = self.blur(ui);
                    let galley = item_text(ui, item, expanded);
                    let truncated = galley.elided;
                    let r = if blur > 0.0 {
                        let (rect, r) = ui.allocate_exact_size(galley.size(), Sense::click());
                        paint_blurred(ui, rect.min, galley, theme::TEXT(), blur);
                        r
                    } else {
                        ui.add(egui::Label::new(galley).sense(Sense::click()))
                    };
                    if r.double_clicked() {
                        actions.push(Action::Edit(item.id));
                    } else if r.clicked() && (truncated || expanded) {
                        actions.push(Action::Expand(item.id));
                    }
                    if truncated {
                        r.on_hover_cursor(egui::CursorIcon::PointingHand);
                    }
                });
                ui.horizontal(|ui| {
                    let when = time::label(item.created, now, self.utc_offset);
                    let meta = match &item.source {
                        Some(app) => format!("{app} · {when}"),
                        None => when,
                    };
                    ui.label(RichText::new(meta).small().color(theme::FAINT()));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;
                        if !hovered {
                            ui.add_space(22.0);
                            return;
                        }
                        if icon_button(ui, Svg::Trash, "Delete").clicked() {
                            actions.push(Action::Delete(item.id));
                        }
                        let (to, svg, tip) = match item.kind {
                            Kind::Note => (Kind::Prompt, Svg::ListPlus, "Move to Queue"),
                            Kind::Prompt => (Kind::Note, Svg::StickyNote, "Move to Notes"),
                        };
                        if icon_button(ui, svg, tip).clicked() {
                            actions.push(Action::Move(item.id, to));
                        }
                        if icon_button(ui, Svg::Pencil, "Edit").clicked() {
                            actions.push(Action::Edit(item.id));
                        }
                        let tag = icon_button(ui, Svg::Tag, "Color");
                        egui::Popup::from_toggle_button_response(&tag).id(color_popup).show(|ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 4.0;
                                for c in Color::ALL {
                                    if swatch(ui, Some(c), item.color == Some(c)).on_hover_text(c.name()).clicked() {
                                        actions.push(Action::SetColor(item.id, Some(c)));
                                    }
                                }
                                if item.color.is_some() && swatch(ui, None, false).on_hover_text("No color").clicked() {
                                    actions.push(Action::SetColor(item.id, None));
                                }
                            });
                        });
                        let copy_tip = if item.kind == Kind::Prompt && !item.done { "Copy and check off" } else { "Copy" };
                        if icon_button(ui, Svg::Copy, copy_tip).clicked() {
                            actions.push(Action::Copy(item.id));
                        }
                    });
                });
            });
        if let Some(c) = item.color {
            let r = frame.response.rect;
            let stripe = Rect::from_min_max(egui::pos2(r.left() + 1.0, r.top() + 10.0), egui::pos2(r.left() + 4.0, r.bottom() - 10.0));
            let color = if item.done { theme::label(c).gamma_multiply(0.45) } else { theme::label(c) };
            ui.painter().rect_filled(stripe, CornerRadius::same(2), color);
        }
        let now_hovered = ui.rect_contains_pointer(frame.response.rect);
        if now_hovered != hovered {
            ui.data_mut(|d| d.insert_temp(hover_id, now_hovered));
            ui.ctx().request_repaint();
        }
        ui.add_space(2.0);
    }

    fn edit_card(&mut self, ui: &mut Ui, actions: &mut Vec<Action>) {
        let Some((_, text)) = &mut self.editing else { return };
        let id = Id::new("edit-card");
        let (save, cancel) = ui.input_mut(|i| (i.consume_key(Modifiers::COMMAND, Key::Enter), i.consume_key(Modifiers::NONE, Key::Escape)));
        let r = ui.add(egui::TextEdit::multiline(text).id(id).frame(false).margin(Margin::ZERO).desired_width(f32::INFINITY).desired_rows(3));
        if !ui.memory(|m| m.focused().is_some()) {
            r.request_focus();
        }
        ui.horizontal(|ui| {
            ui.label(RichText::new("⌘↩ to save, esc to cancel").small().color(theme::FAINT()));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let save_btn = egui::Button::new(RichText::new("Save").color(theme::BG())).fill(theme::ACCENT()).corner_radius(6);
                if ui.add(save_btn).clicked() || save {
                    actions.push(Action::SaveEdit);
                }
                if ui.add(egui::Button::new("Cancel").frame(false)).clicked() || cancel {
                    actions.push(Action::CancelEdit);
                }
            });
        });
    }

    fn apply(&mut self, ctx: &egui::Context, action: Action) {
        match action {
            Action::Copy(id) => {
                let Some(item) = self.store.lock().unwrap().get(id).cloned() else { return };
                ctx.copy_text(item.text.clone());
                if item.kind == Kind::Prompt && !item.done {
                    self.change(|s| s.set_done(id, true));
                    self.notify("Copied and checked off", Some(Undo::Uncheck(id)));
                } else {
                    self.notify("Copied", None);
                }
            }
            Action::Delete(id) => {
                if let Some(removed) = self.change(|s| s.remove(id)) {
                    self.notify("Deleted", Some(Undo::Restore(vec![removed])));
                }
            }
            Action::Toggle(id) => {
                let done = self.store.lock().unwrap().get(id).is_some_and(|i| i.done);
                self.change(|s| s.set_done(id, !done));
            }
            Action::Move(id, kind) => {
                self.change(|s| s.set_kind(id, kind));
                self.notify(if kind == Kind::Prompt { "Moved to Queue" } else { "Moved to Notes" }, None);
            }
            Action::Edit(id) => {
                if let Some(item) = self.store.lock().unwrap().get(id) {
                    self.editing = Some((id, item.text.clone()));
                }
            }
            Action::SaveEdit => {
                if let Some((id, text)) = self.editing.take() {
                    if text.trim().is_empty() {
                        self.apply(ctx, Action::Delete(id));
                    } else {
                        self.change(|s| s.set_text(id, &text));
                    }
                }
            }
            Action::CancelEdit => self.editing = None,
            Action::Expand(id) => {
                if !self.expanded.remove(&id) {
                    self.expanded.insert(id);
                }
            }
            Action::SetColor(id, color) => {
                self.change(|s| s.set_color(id, color));
                egui::Popup::close_all(ctx);
            }
            Action::ClearDone => {
                let removed = self.change(|s| s.clear_done());
                let n = removed.len();
                self.notify(format!("Cleared {n} done"), Some(Undo::Restore(removed)));
            }
        }
    }

    fn undo(&mut self, undo: Undo) {
        match undo {
            Undo::Restore(items) => self.change(|s| {
                for (pos, item) in items {
                    s.restore(pos, item);
                }
            }),
            Undo::Uncheck(id) => self.change(|s| s.set_done(id, false)),
        }
    }

    fn access_banner(&mut self, ui: &mut Ui) {
        Frame::new()
            .fill(theme::tint(theme::SURFACE(), theme::ACCENT(), 0.07))
            .stroke(Stroke::new(1.0_f32, theme::ACCENT().gamma_multiply(0.5)))
            .corner_radius(10)
            .inner_margin(Margin::same(12))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new("Allow Accessibility access").strong());
                ui.label(
                    RichText::new("rjot uses it to read the text you select and to hear double-tap ⇧. Nothing leaves your Mac.")
                        .color(theme::MUTED()),
                );
                ui.label(
                    RichText::new("Click below and switch rjot on. If it looked switched on already, that was an older build: it's been cleared, so switch it on again.")
                        .small()
                        .color(theme::FAINT()),
                );
                ui.add_space(4.0);
                let open = egui::Button::new(RichText::new("Open System Settings").color(theme::BG())).fill(theme::ACCENT()).corner_radius(6);
                if ui.add(open).clicked() {
                    macos::ask_for_access();
                }
            });
        ui.add_space(8.0);
    }

    // ---- Scratch ----

    fn scratch(&mut self, ui: &mut Ui, now: f64) {
        let blur = self.blur(ui);
        let id = Id::new("scratch");
        let editing = ui.memory(|m| m.has_focus(id));
        Frame::new()
            .fill(theme::INPUT())
            .stroke(Stroke::new(1.0_f32, theme::BORDER()))
            .corner_radius(10)
            .inner_margin(Margin::same(12))
            .show(ui, |ui| {
                egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                    // Blurred until clicked into (or peeked at with ⌥).
                    if blur > 0.0 && !editing && !self.scratch.is_empty() {
                        let font = TextStyle::Body.resolve(ui.style());
                        let job = LayoutJob::simple(self.scratch.clone(), font, theme::TEXT(), ui.available_width());
                        let galley = ui.painter().layout_job(job);
                        let size = vec2(ui.available_width(), galley.size().y.max(ui.available_height()));
                        let (rect, r) = ui.allocate_exact_size(size, Sense::click());
                        paint_blurred(ui, rect.min, galley, theme::TEXT(), blur);
                        if r.on_hover_cursor(egui::CursorIcon::Text).clicked() {
                            ui.memory_mut(|m| m.request_focus(id));
                        }
                        return;
                    }
                    let rows = (ui.available_height() / 18.0).max(4.0) as usize;
                    let edit = egui::TextEdit::multiline(&mut self.scratch)
                        .id(id)
                        .frame(false)
                        .hint_text("Scratchpad. Anything goes; it's saved as you type.")
                        .margin(Margin::ZERO)
                        .desired_width(f32::INFINITY)
                        .desired_rows(rows);
                    if ui.add(edit).changed() {
                        self.scratch_edited = Some(now);
                    }
                });
            });
    }

    fn status_bar(&mut self, ui: &mut Ui, now: f64) {
        if let Some(toast) = &mut self.toast
            && toast.until.is_nan()
        {
            toast.until = now + 5.0;
        }
        if self.toast.as_ref().is_some_and(|t| now > t.until) {
            self.toast = None;
        }
        ui.horizontal_centered(|ui| {
            if let Some(toast) = &self.toast {
                ui.label(RichText::new(&toast.text).small().color(theme::MUTED()));
                if toast.undo.is_some() {
                    let undo = egui::Button::new(RichText::new("Undo").small().color(theme::ACCENT())).frame(false);
                    if ui.add(undo).clicked()
                        && let Some(undo) = self.toast.take().and_then(|t| t.undo)
                    {
                        self.undo(undo);
                    }
                }
                ui.ctx().request_repaint_after(std::time::Duration::from_millis(500));
            } else if self.settings.sensitive {
                ui.label(RichText::new("Sensitive mode: hold ⌥ to peek").small().color(theme::FAINT()));
            } else if self.trusted {
                ui.label(RichText::new("Double-tap ⇧ to save selected text").small().color(theme::FAINT()));
            } else {
                ui.label(RichText::new("Waiting for Accessibility access").small().color(theme::FAINT()));
            }
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        theme::sync(ctx);
        if self.window_private != Some(self.settings.sensitive) {
            self.window_private = Some(self.settings.sensitive);
            if !self.demo.capturable {
                macos::set_window_private(frame, self.settings.sensitive);
            }
        }
        let now = ctx.input(|i| i.time);
        self.handle_signals(ctx, now);

        if !self.trusted && now - self.trust_checked > 1.5 {
            self.trust_checked = now;
            self.trusted = macos::trusted();
        }
        if !self.trusted {
            ctx.request_repaint_after(std::time::Duration::from_secs(2));
        }
        // Time labels age.
        ctx.request_repaint_after(std::time::Duration::from_secs(30));

        if ctx.input(|i| i.viewport().close_requested()) {
            // Closing hides; double-tap Shift keeps working. ⌘Q quits.
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            macos::hide();
        }
        self.shortcuts(ctx);

        if self.scratch_edited.is_some_and(|t| now - t > 0.6) {
            self.flush_scratch();
        } else if self.scratch_edited.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(700));
        }

        egui::TopBottomPanel::top("bar")
            .exact_height(40.0)
            .show_separator_line(false)
            .frame(Frame::new().fill(theme::BG()).inner_margin(Margin { left: 80, right: 10, top: 4, bottom: 0 }))
            .show(ctx, |ui| self.top_bar(ui));

        egui::TopBottomPanel::bottom("status")
            .exact_height(28.0)
            .show_separator_line(false)
            .frame(Frame::new().fill(theme::BG()).inner_margin(Margin::symmetric(14, 0)))
            .show(ctx, |ui| self.status_bar(ui, now));

        let mut actions = Vec::new();
        egui::CentralPanel::default()
            .frame(Frame::new().fill(theme::BG()).inner_margin(Margin { left: 12, right: 12, top: 6, bottom: 0 }))
            .show(ctx, |ui| match self.tab.kind() {
                Some(kind) => {
                    if !self.trusted && kind == Kind::Note {
                        self.access_banner(ui);
                    }
                    self.search_row(ui);
                    self.composer(ui, kind);
                    ui.add_space(10.0);
                    egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                        self.list(ui, kind, &mut actions);
                        ui.add_space(12.0);
                    });
                }
                None => self.scratch(ui, now),
            });
        for action in actions {
            self.apply(ctx, action);
        }

        self.demo.frames = self.demo.frames.saturating_add(1);
        if let Some(ready) = &self.demo.ready {
            if self.demo.frames == 12 {
                let _ = std::fs::write(ready, "");
            } else if self.demo.frames < 12 {
                ctx.request_repaint();
            }
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.flush_scratch();
    }
}

// ---- Widgets ----

/// One segment of the tab control: a label and an optional count.
fn segment(ui: &mut Ui, label: &str, count: Option<usize>, selected: bool) -> Response {
    let mut job = LayoutJob::default();
    let color = if selected { theme::TEXT() } else { theme::MUTED() };
    job.append(label, 0.0, TextFormat::simple(FontId::proportional(12.5), color));
    if let Some(n) = count.filter(|n| *n > 0) {
        job.append(&n.to_string(), 5.0, TextFormat::simple(FontId::proportional(11.5), theme::FAINT()));
    }
    let galley = ui.painter().layout_job(job);
    let size = vec2(galley.size().x + 20.0, 24.0);
    let (rect, r) = ui.allocate_exact_size(size, Sense::click());
    if selected {
        ui.painter().rect_filled(rect, 6.0, theme::tint(theme::RAISE(), theme::TEXT(), 0.10));
    } else if r.hovered() {
        ui.painter().rect_filled(rect, 6.0, theme::tint(theme::RAISE(), theme::TEXT(), 0.04));
    }
    ui.painter().galley(rect.center() - galley.size() / 2.0, galley, color);
    r
}

/// Draw `galley` as if out of focus: many faint copies spread over rings
/// around its position. `amount` (0-1) scales the spread, so at 0 it's
/// plain text.
fn paint_blurred(ui: &Ui, pos: egui::Pos2, galley: Arc<egui::Galley>, color: egui::Color32, amount: f32) {
    /// Radius and sample count per ring.
    const RINGS: [(f32, usize); 3] = [(2.0, 8), (4.0, 12), (6.5, 16)];
    // Faint enough that the overlapping copies add up to soft, unreadable ink.
    let ink = color.gamma_multiply(egui::lerp(1.0..=0.075, amount));
    let painter = ui.painter();
    painter.galley_with_override_text_color(pos, galley.clone(), ink);
    for (ring, (radius, count)) in RINGS.into_iter().enumerate() {
        // Stagger the rings so their samples don't line up.
        let phase = ring as f32 * 0.5;
        for k in 0..count {
            let a = std::f32::consts::TAU * (k as f32 + phase) / count as f32;
            let offset = vec2(a.cos(), a.sin()) * radius * amount;
            painter.galley_with_override_text_color(pos + offset, galley.clone(), ink);
        }
    }
}

fn section_label(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).small().color(theme::FAINT()));
}

/// A theme in the settings menu: a dark/light preview chip and its name.
fn theme_row(ui: &mut Ui, t: &theme::ThemeDef, selected: bool) -> Response {
    let (rect, r) = ui.allocate_exact_size(vec2(ui.available_width(), 26.0), Sense::click());
    if selected || r.hovered() {
        ui.painter().rect_filled(rect, CornerRadius::same(5), theme::RAISE());
    }
    let chip = Rect::from_min_size(rect.left_center() + vec2(8.0, -8.0), vec2(30.0, 16.0));
    let (left, right) = chip.split_left_right_at_fraction(0.5);
    let p = ui.painter();
    p.rect_filled(left, CornerRadius { nw: 4, sw: 4, ne: 0, se: 0 }, t.dark.bg);
    p.rect_filled(right, CornerRadius { nw: 0, sw: 0, ne: 4, se: 4 }, t.light.bg);
    p.circle_filled(left.center(), 3.5, t.dark.accent);
    p.circle_filled(right.center(), 3.5, t.light.accent);
    p.rect_stroke(chip, CornerRadius::same(4), Stroke::new(1.0_f32, theme::BORDER()), egui::StrokeKind::Inside);
    p.text(chip.right_center() + vec2(10.0, 0.0), egui::Align2::LEFT_CENTER, t.name, FontId::proportional(13.0), theme::TEXT());
    if selected {
        icons::paint(ui, Svg::Check, Rect::from_center_size(rect.right_center() - vec2(14.0, 0.0), vec2(13.0, 13.0)), theme::ACCENT());
    }
    r
}

/// A setting with an on/off switch and a line of explanation.
fn switch_row(ui: &mut Ui, on: &mut bool, title: &str, detail: &str) -> Response {
    let mut changed = false;
    let inner = ui.horizontal_top(|ui| {
        let (rect, r) = ui.allocate_exact_size(vec2(30.0, 18.0), Sense::click());
        if r.clicked() {
            *on = !*on;
            changed = true;
        }
        let t = ui.ctx().animate_bool(r.id, *on);
        let track = theme::tint(theme::RAISE(), theme::ACCENT(), t);
        ui.painter().rect_filled(rect, CornerRadius::same(9), track);
        let x = egui::lerp(rect.left() + 9.0..=rect.right() - 9.0, t);
        ui.painter().circle_filled(egui::pos2(x, rect.center().y), 7.0, theme::tint(theme::MUTED(), theme::BG(), t));
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            ui.label(title);
            ui.label(RichText::new(detail).small().color(theme::FAINT()));
        });
        r
    });
    let mut r = inner.inner;
    if changed {
        r.mark_changed();
    }
    r
}

/// A round color swatch; `None` is the "no color" swatch (a crossed ring).
fn swatch(ui: &mut Ui, color: Option<Color>, selected: bool) -> Response {
    let (rect, r) = ui.allocate_exact_size(vec2(22.0, 22.0), Sense::click());
    let c = rect.center();
    match color {
        Some(color) => {
            let fill = theme::label(color);
            ui.painter().circle_filled(c, if r.hovered() { 8.0 } else { 7.0 }, fill);
            if selected {
                ui.painter().circle_stroke(c, 10.0, Stroke::new(1.5_f32, fill));
            }
        }
        None => {
            let ink = if r.hovered() { theme::TEXT() } else { theme::MUTED() };
            ui.painter().circle_stroke(c, 7.0, Stroke::new(1.2_f32, ink));
            ui.painter().line_segment([c + vec2(-4.5, 4.5), c + vec2(4.5, -4.5)], Stroke::new(1.2_f32, ink));
        }
    }
    r.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A row in the color filter menu: dot, name, count.
fn color_row(ui: &mut Ui, color: Color, count: usize, selected: bool) -> Response {
    let (rect, r) = ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::click());
    if selected || r.hovered() {
        ui.painter().rect_filled(rect, CornerRadius::same(5), theme::RAISE());
    }
    let mid = rect.left_center();
    ui.painter().circle_filled(mid + vec2(12.0, 0.0), 5.0, theme::label(color));
    let font = FontId::proportional(13.0);
    ui.painter().text(mid + vec2(26.0, 0.0), egui::Align2::LEFT_CENTER, color.name(), font.clone(), theme::TEXT());
    if count > 0 {
        ui.painter().text(rect.right_center() - vec2(8.0, 0.0), egui::Align2::RIGHT_CENTER, count.to_string(), font, theme::FAINT());
    }
    r
}

fn icon_button(ui: &mut Ui, svg: Svg, tip: &str) -> Response {
    let (rect, r) = ui.allocate_exact_size(vec2(24.0, 22.0), Sense::click());
    if r.hovered() {
        ui.painter().rect_filled(rect, CornerRadius::same(6), theme::RAISE());
    }
    let tint = if r.hovered() { theme::TEXT() } else { theme::MUTED() };
    icons::paint(ui, svg, Rect::from_center_size(rect.center(), vec2(14.0, 14.0)), tint);
    r.on_hover_text(tip)
}

/// The round check-off box of a prompt.
fn check_box(ui: &mut Ui, done: bool) -> Response {
    let (rect, r) = ui.allocate_exact_size(vec2(20.0, 19.0), Sense::click());
    let center = rect.center() - vec2(1.0, 0.0);
    if done {
        ui.painter().circle_filled(center, 8.0, theme::ACCENT());
        icons::paint(ui, Svg::Check, Rect::from_center_size(center, vec2(11.0, 11.0)), theme::BG());
    } else {
        let color = if r.hovered() { theme::ACCENT() } else { theme::FAINT() };
        ui.painter().circle_stroke(center, 7.5, Stroke::new(1.4_f32, color));
    }
    r.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// The item's text, cut to a few lines unless expanded; checked-off prompts
/// are struck through.
fn item_text(ui: &Ui, item: &Item, expanded: bool) -> Arc<egui::Galley> {
    let font = TextStyle::Body.resolve(ui.style());
    let mut format = TextFormat { font_id: font, color: theme::TEXT(), line_height: Some(19.0), ..Default::default() };
    if item.done {
        format.color = theme::FAINT();
        format.strikethrough = Stroke::new(1.0_f32, theme::FAINT());
    }
    let mut job = LayoutJob::single_section(item.text.trim_end().to_string(), format);
    job.wrap.max_width = ui.available_width();
    if !expanded {
        job.wrap.max_rows = COLLAPSED_ROWS;
    }
    ui.painter().layout_job(job)
}

fn empty_state(ui: &mut Ui, title: &str, hint: Option<&str>) {
    ui.add_space(48.0);
    ui.vertical_centered(|ui| {
        ui.label(RichText::new(title).color(theme::MUTED()));
        if let Some(hint) = hint {
            ui.add_space(2.0);
            ui.label(RichText::new(hint).small().color(theme::FAINT()));
        }
    });
}
