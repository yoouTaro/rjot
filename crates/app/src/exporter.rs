//! Writes the Markdown copy on a thread of its own, so a slow or synced
//! folder never holds up typing or capture. Requests that arrive close
//! together are written once, with the newest snapshot.

use eframe::egui;
use rjot_core::export::Job;
use std::sync::mpsc::{RecvTimeoutError, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Clone)]
pub struct Exporter {
    tx: Sender<Job>,
    error: Arc<Mutex<Option<String>>>,
}

impl Exporter {
    pub fn new(ctx: egui::Context) -> Exporter {
        let (tx, rx) = channel::<Job>();
        let error = Arc::new(Mutex::new(None));
        let report = error.clone();
        std::thread::Builder::new()
            .name("markdown-copy".into())
            .spawn(move || {
                while let Ok(mut job) = rx.recv() {
                    let mut last = false;
                    loop {
                        match rx.recv_timeout(Duration::from_millis(800)) {
                            Ok(newer) => job = newer,
                            Err(RecvTimeoutError::Timeout) => break,
                            Err(RecvTimeoutError::Disconnected) => {
                                last = true;
                                break;
                            }
                        }
                    }
                    let result = job.run();
                    *report.lock().unwrap() =
                        result.err().map(|e| format!("Markdown copy to {} failed: {e}", job.dir.display()));
                    ctx.request_repaint();
                    if last {
                        break;
                    }
                }
            })
            .expect("spawn the markdown copy thread");
        Exporter { tx, error }
    }

    pub fn request(&self, job: Option<Job>) {
        if let Some(job) = job {
            let _ = self.tx.send(job);
        }
    }

    /// The last failure, once.
    pub fn take_error(&self) -> Option<String> {
        self.error.lock().unwrap().take()
    }
}
