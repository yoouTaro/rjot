//! What happens on double-tap Shift: save the selection in the front app as
//! a note, or, with nothing selected, bring rjot forward to type one.
//! Runs on its own thread so the event tap never waits on other apps.

use crate::macos;
use eframe::egui;
use rjot_core::{Kind, Store, time};
use std::sync::mpsc::{Receiver, channel};
use std::sync::{Arc, Mutex};

pub enum Signal {
    /// A selection was saved as this note.
    Saved(u64),
    /// Nothing was selected: focus the composer, and hide again after saving.
    QuickNote,
    Error(String),
}

pub fn start(store: Arc<Mutex<Store>>, ctx: egui::Context) -> Receiver<Signal> {
    let (signal_tx, signal_rx) = channel();
    let (tap_tx, tap_rx) = channel::<()>();
    macos::listen(move || {
        let _ = tap_tx.send(());
    });
    std::thread::Builder::new()
        .name("capture".into())
        .spawn(move || {
            while tap_rx.recv().is_ok() {
                // A burst of taps is one capture.
                while tap_rx.try_recv().is_ok() {}
                let signal = capture(&store);
                if matches!(signal, Signal::QuickNote) {
                    macos::activate();
                }
                if signal_tx.send(signal).is_err() {
                    break;
                }
                ctx.request_repaint();
            }
        })
        .expect("spawn the capture thread");
    signal_rx
}

fn capture(store: &Mutex<Store>) -> Signal {
    let front = macos::frontmost();
    if front.as_ref().is_some_and(|f| f.pid == std::process::id() as i32) {
        return Signal::QuickNote;
    }
    let text = match macos::selected_text().filter(|t| !t.trim().is_empty()) {
        Some(t) => t,
        None => match macos::copy_selection() {
            macos::Copied::Text(t) => t,
            macos::Copied::Concealed => return Signal::Error("Not saved: the app marked that text as a password".into()),
            macos::Copied::Nothing => return Signal::QuickNote,
        },
    };
    let mut store = store.lock().unwrap();
    let Some(id) = store.add(Kind::Note, &text, front.and_then(|f| f.name), time::now()) else {
        return Signal::QuickNote;
    };
    if let Err(e) = store.save() {
        return Signal::Error(format!("Couldn't save: {e}"));
    }
    if store.settings().sound {
        macos::play("Pop");
    }
    Signal::Saved(id)
}
