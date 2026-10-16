//! The menu bar icon: the last few captures (click to copy), Quick note,
//! Open rjot, Pause capture, Launch at login and Quit. Menu clicks arrive on
//! the main thread and are forwarded to the app through a channel.

use eframe::egui;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObject, Sel};
use objc2::{AnyThread, DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSControlStateValueOff, NSControlStateValueOn, NSImage, NSMenu, NSMenuItem, NSStatusBar, NSStatusItem,
    NSVariableStatusItemLength,
};
use objc2_foundation::{NSData, NSSize, NSString};
use std::sync::mpsc::{Receiver, Sender, channel};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuEvent {
    Open,
    QuickNote,
    TogglePause,
    ToggleLogin,
    Quit,
    /// Copy the recent capture at this index of the menu's list.
    CopyRecent(usize),
}

const TAG_OPEN: isize = 1;
const TAG_QUICK: isize = 2;
const TAG_PAUSE: isize = 3;
const TAG_LOGIN: isize = 4;
const TAG_QUIT: isize = 5;
const TAG_RECENT: isize = 100;

pub struct TargetIvars {
    tx: Sender<MenuEvent>,
    ctx: egui::Context,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "RjotMenuTarget"]
    #[ivars = TargetIvars]
    struct MenuTarget;

    impl MenuTarget {
        #[unsafe(method(itemClicked:))]
        fn item_clicked(&self, sender: &NSMenuItem) {
            let event = match sender.tag() {
                TAG_OPEN => MenuEvent::Open,
                TAG_QUICK => MenuEvent::QuickNote,
                TAG_PAUSE => MenuEvent::TogglePause,
                TAG_LOGIN => MenuEvent::ToggleLogin,
                TAG_QUIT => MenuEvent::Quit,
                t if t >= TAG_RECENT => MenuEvent::CopyRecent((t - TAG_RECENT) as usize),
                _ => return,
            };
            let _ = self.ivars().tx.send(event);
            self.ivars().ctx.request_repaint();
        }
    }
);

/// What the menu reflects; rebuilt when it changes.
#[derive(Clone, PartialEq)]
pub struct MenuState {
    pub recent: Vec<String>,
    pub paused: bool,
    pub login: Option<bool>,
}

pub struct StatusBar {
    item: Retained<NSStatusItem>,
    menu: Retained<NSMenu>,
    target: Retained<MenuTarget>,
    shown: Option<MenuState>,
    mtm: MainThreadMarker,
}

impl StatusBar {
    /// Create the status item. Main thread only (the app's creation runs there).
    pub fn new(ctx: &egui::Context) -> Option<(StatusBar, Receiver<MenuEvent>)> {
        let mtm = MainThreadMarker::new()?;
        let (tx, rx) = channel();
        let target = MenuTarget::alloc(mtm).set_ivars(TargetIvars { tx, ctx: ctx.clone() });
        let target: Retained<MenuTarget> = unsafe { msg_send![super(target), init] };

        let item = NSStatusBar::systemStatusBar().statusItemWithLength(NSVariableStatusItemLength);
        if let Some(button) = item.button(mtm) {
            let data = NSData::with_bytes(include_bytes!("../../../assets/menubar.png"));
            if let Some(image) = NSImage::initWithData(NSImage::alloc(), &data) {
                image.setSize(NSSize::new(18.0, 18.0));
                image.setTemplate(true);
                button.setImage(Some(&image));
            }
            button.setToolTip(Some(&NSString::from_str("rjot")));
        }
        let menu = NSMenu::new(mtm);
        menu.setAutoenablesItems(false);
        item.setMenu(Some(&menu));
        Some((StatusBar { item, menu, target, shown: None, mtm }, rx))
    }

    /// Rebuild the menu if `state` differs from what it shows.
    pub fn update(&mut self, state: MenuState) {
        if self.shown.as_ref() == Some(&state) {
            return;
        }
        let mtm = self.mtm;
        self.menu.removeAllItems();
        if state.recent.is_empty() {
            let none = self.add("No captures yet", None, 0);
            none.setEnabled(false);
        } else {
            let header = self.add("Recent: click to copy", None, 0);
            header.setEnabled(false);
            for (i, label) in state.recent.iter().enumerate() {
                self.add(label, Some(sel!(itemClicked:)), TAG_RECENT + i as isize);
            }
        }
        self.menu.addItem(&NSMenuItem::separatorItem(mtm));
        self.add("Quick note…", Some(sel!(itemClicked:)), TAG_QUICK);
        self.add("Open rjot", Some(sel!(itemClicked:)), TAG_OPEN);
        self.menu.addItem(&NSMenuItem::separatorItem(mtm));
        let pause = self.add("Pause capture", Some(sel!(itemClicked:)), TAG_PAUSE);
        pause.setState(if state.paused { NSControlStateValueOn } else { NSControlStateValueOff });
        if let Some(on) = state.login {
            let login = self.add("Launch at login", Some(sel!(itemClicked:)), TAG_LOGIN);
            login.setState(if on { NSControlStateValueOn } else { NSControlStateValueOff });
        }
        self.menu.addItem(&NSMenuItem::separatorItem(mtm));
        self.add("Quit rjot", Some(sel!(itemClicked:)), TAG_QUIT);
        if let Some(button) = self.item.button(mtm) {
            button.setAppearsDisabled(state.paused);
        }
        self.shown = Some(state);
    }

    fn add(&self, title: &str, action: Option<Sel>, tag: isize) -> Retained<NSMenuItem> {
        let item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(self.mtm),
                &NSString::from_str(title),
                action,
                &NSString::from_str(""),
            )
        };
        item.setTag(tag);
        if action.is_some() {
            let target: &AnyObject = &self.target;
            unsafe { item.setTarget(Some(target)) };
        }
        self.menu.addItem(&item);
        item
    }
}

/// A capture as one menu line: its first line, cut to a readable length.
pub fn menu_label(text: &str, image: bool) -> String {
    let line = text.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or("");
    if line.is_empty() {
        return if image { "Image".into() } else { "(empty)".into() };
    }
    let mut out: String = line.chars().take(48).collect();
    if line.chars().count() > 48 {
        out.push('…');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels() {
        assert_eq!(menu_label("  \n first line \nsecond", false), "first line");
        assert_eq!(menu_label("", true), "Image");
        let long = "x".repeat(60);
        assert_eq!(menu_label(&long, false).chars().count(), 49);
    }
}
