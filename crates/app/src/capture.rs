//! What happens on the keyboard triggers. Double-tap Shift saves the
//! selection in the front app as a note (text or image, with the page it
//! came from in a browser), or, with nothing selected, brings rjot forward
//! to type one. ⌥⇧V opens the paste picker for the app in front.
//! Runs on its own thread so the event tap never waits on other apps.

use crate::exporter::Exporter;
use crate::macos::{self, Front, Key};
use eframe::egui;
use rjot_core::{Kind, Meta, Store, time};
use std::sync::mpsc::{Receiver, channel};
use std::sync::{Arc, Mutex};

pub enum Signal {
    /// A selection was saved as this note.
    Saved(u64),
    /// Nothing was selected: focus the composer, and hide again after saving.
    QuickNote,
    /// ⌥⇧V: pick a queued prompt to paste into this app.
    PastePicker(Front),
    Error(String),
}

pub fn start(store: Arc<Mutex<Store>>, exporter: Exporter, ctx: egui::Context) -> Receiver<Signal> {
    let (signal_tx, signal_rx) = channel();
    let (key_tx, key_rx) = channel::<Key>();
    macos::listen(move |key| {
        let _ = key_tx.send(key);
    });
    std::thread::Builder::new()
        .name("capture".into())
        .spawn(move || {
            while let Ok(key) = key_rx.recv() {
                let signal = match key {
                    Key::DoubleShift => {
                        // A burst of taps is one capture.
                        while key_rx.try_recv().is_ok() {}
                        if store.lock().unwrap().settings().paused {
                            continue;
                        }
                        capture(&store, &exporter)
                    }
                    Key::PastePicker => match macos::frontmost() {
                        Some(front) if front.pid != std::process::id() as i32 => Signal::PastePicker(front),
                        _ => continue,
                    },
                };
                if matches!(signal, Signal::QuickNote | Signal::PastePicker(_)) {
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

fn capture(store: &Mutex<Store>, exporter: &Exporter) -> Signal {
    let front = macos::frontmost();
    if front.as_ref().is_some_and(|f| f.pid == std::process::id() as i32) {
        return Signal::QuickNote;
    }
    // Ask the browser for its page while the selection is read.
    let page = front.as_ref().and_then(|f| f.bundle.clone()).map(|bundle| std::thread::spawn(move || macos::browser_page(&bundle)));

    let mut image = None;
    let text = match macos::selected_text().filter(|t| !t.trim().is_empty()) {
        Some(t) => t,
        None => match macos::copy_selection() {
            macos::Copied::Text(t) => t,
            macos::Copied::Image(png) => {
                image = Some(png);
                String::new()
            }
            macos::Copied::Concealed => return Signal::Error("Not saved: the app marked that text as a password".into()),
            macos::Copied::Nothing => return Signal::QuickNote,
        },
    };
    let (url, title) = match page.and_then(|h| h.join().ok()).flatten() {
        Some((url, title)) => (Some(url), Some(title).filter(|t| !t.is_empty())),
        None => (None, None),
    };

    let mut store = store.lock().unwrap();
    let image_name = match image {
        Some(png) => {
            let name = format!("{}.png", store.peek_id());
            let dir = store.images_dir();
            if let Err(e) = std::fs::create_dir_all(&dir).and_then(|_| std::fs::write(dir.join(&name), png)) {
                return Signal::Error(format!("Couldn't save the image: {e}"));
            }
            Some(name)
        }
        None => None,
    };
    let meta = Meta { source: front.and_then(|f| f.name), url, title, image: image_name };
    let Some(id) = store.add_with(Kind::Note, &text, meta, time::now()) else {
        return Signal::QuickNote;
    };
    if let Err(e) = store.save() {
        return Signal::Error(format!("Couldn't save: {e}"));
    }
    exporter.request(store.export_job());
    if store.settings().sound {
        macos::play("Pop");
    }
    Signal::Saved(id)
}
