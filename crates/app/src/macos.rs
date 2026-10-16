//! The macOS side: a keyboard event tap for double-tap Shift and the ⌥⇧V
//! paste picker, reading the selected text (Accessibility first, then a ⌘C
//! round trip that puts the clipboard back), the front browser's page,
//! pasting into another app, launch at login, and small AppKit calls.

use crate::doubletap::{DoubleTap, Input};
use core_foundation::base::{CFType, CFTypeRef, TCFType};
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
use core_foundation::mach_port::CFMachPortRef;
use core_foundation::runloop::{CFRunLoop, kCFRunLoopCommonModes};
use core_foundation::string::{CFString, CFStringRef};
use core_graphics::event::{
    CGEvent, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventType, EventField,
};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_app_kit::{
    NSApplicationActivationOptions, NSPasteboard, NSPasteboardItem, NSPasteboardTypeString,
    NSPasteboardWriting, NSRunningApplication, NSSound, NSWorkspace,
};
use objc2_foundation::{NSArray, NSData, NSString};
use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::time::{Duration, Instant};

type AXUIElementRef = *const c_void;

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> bool;
    fn AXUIElementCreateSystemWide() -> AXUIElementRef;
    fn AXUIElementCopyAttributeValue(el: AXUIElementRef, attr: CFStringRef, value: *mut CFTypeRef) -> i32;
    fn AXUIElementSetMessagingTimeout(el: AXUIElementRef, seconds: f32) -> i32;
    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
}

// Linked so SMAppService (macOS 13+) is available at runtime.
#[link(name = "ServiceManagement", kind = "framework")]
unsafe extern "C" {}

/// Whether rjot may read other apps' selections and listen for Shift.
pub fn trusted() -> bool {
    unsafe { AXIsProcessTrusted() }
}

/// Show the system's Accessibility prompt and open the settings pane.
///
/// An entry left over from an earlier build (signed differently) still
/// shows rjot as switched on but no longer applies, and switching it on
/// again does nothing. So rjot's entry is cleared first, and the prompt
/// then adds one for this build.
pub fn ask_for_access() {
    if let Some(id) = NSRunningApplication::currentApplication().bundleIdentifier() {
        let _ = std::process::Command::new("/usr/bin/tccutil")
            .args(["reset", "Accessibility", &id.to_string()])
            .status();
    }
    let key = CFString::from_static_string("AXTrustedCheckOptionPrompt");
    let options = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), CFBoolean::true_value().as_CFType())]);
    unsafe { AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) };
    let _ = std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn();
}

// ---- The keyboard: double-tap Shift and ⌥⇧V ----

static TAP_PORT: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
/// Set when a ⌥⇧V key-down was swallowed, so its key-up is swallowed too.
static SWALLOWED_V: AtomicBool = AtomicBool::new(false);

const KEY_LEFT_SHIFT: i64 = 56;
const KEY_RIGHT_SHIFT: i64 = 60;
const KEY_V: i64 = 9;

pub enum Key {
    /// Shift was tapped twice.
    DoubleShift,
    /// ⌥⇧V: open the paste picker.
    PastePicker,
}

/// Watch the keyboard on a thread of its own and call `on_key` for
/// double-tap Shift and ⌥⇧V. The tap is active so it can swallow ⌥⇧V
/// (which would otherwise type "◊"); every other event passes through
/// untouched. Waits for Accessibility access first.
pub fn listen(on_key: impl Fn(Key) + Send + 'static) {
    std::thread::Builder::new()
        .name("keyboard-tap".into())
        .spawn(move || {
            while !trusted() {
                std::thread::sleep(Duration::from_secs(1));
            }
            let start = Instant::now();
            let state = RefCell::new(DoubleTap::default());
            let tap = CGEventTap::new(
                CGEventTapLocation::Session,
                CGEventTapPlacement::HeadInsertEventTap,
                CGEventTapOptions::Default,
                vec![CGEventType::FlagsChanged, CGEventType::KeyDown, CGEventType::KeyUp],
                |_proxy, kind, event| {
                    let input = match kind {
                        CGEventType::TapDisabledByTimeout | CGEventType::TapDisabledByUserInput => {
                            let port = TAP_PORT.load(Ordering::Relaxed);
                            if !port.is_null() {
                                unsafe { CGEventTapEnable(port as CFMachPortRef, true) };
                            }
                            return None;
                        }
                        CGEventType::FlagsChanged => shift_input(event, start.elapsed().as_millis() as u64),
                        CGEventType::KeyDown => {
                            if is_paste_chord(event) {
                                // Swallow it, repeats included: a null event
                                // goes nowhere. Open the picker once.
                                event.set_type(CGEventType::Null);
                                SWALLOWED_V.store(true, Ordering::Relaxed);
                                if event.get_integer_value_field(EventField::KEYBOARD_EVENT_AUTOREPEAT) == 0 {
                                    on_key(Key::PastePicker);
                                }
                                return None;
                            }
                            Input::Other
                        }
                        CGEventType::KeyUp => {
                            let code = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
                            if code == KEY_V && SWALLOWED_V.swap(false, Ordering::Relaxed) {
                                event.set_type(CGEventType::Null);
                            }
                            // Key-ups don't interrupt a double tap.
                            return None;
                        }
                        _ => Input::Other,
                    };
                    if state.borrow_mut().feed(input) {
                        on_key(Key::DoubleShift);
                    }
                    // None passes the event on unchanged.
                    None
                },
            );
            let Ok(tap) = tap else {
                eprintln!("rjot: could not create the keyboard event tap");
                return;
            };
            TAP_PORT.store(tap.mach_port.as_concrete_TypeRef() as *mut c_void, Ordering::Relaxed);
            let Ok(source) = tap.mach_port.create_runloop_source(0) else { return };
            unsafe { CFRunLoop::get_current().add_source(&source, kCFRunLoopCommonModes) };
            tap.enable();
            CFRunLoop::run_current();
        })
        .expect("spawn the event tap thread");
}

fn shift_input(event: &CGEvent, now_ms: u64) -> Input {
    let code = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
    let flags = event.get_flags();
    let others = CGEventFlags::CGEventFlagCommand | CGEventFlags::CGEventFlagControl | CGEventFlags::CGEventFlagAlternate;
    if !(code == KEY_LEFT_SHIFT || code == KEY_RIGHT_SHIFT) || flags.intersects(others) {
        return Input::Other;
    }
    if flags.contains(CGEventFlags::CGEventFlagShift) { Input::ShiftDown(now_ms) } else { Input::ShiftUp(now_ms) }
}

/// ⌥⇧V exactly: Option and Shift, no Command or Control.
fn is_paste_chord(event: &CGEvent) -> bool {
    let flags = event.get_flags();
    event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) == KEY_V
        && flags.contains(CGEventFlags::CGEventFlagAlternate | CGEventFlags::CGEventFlagShift)
        && !flags.intersects(CGEventFlags::CGEventFlagCommand | CGEventFlags::CGEventFlagControl)
}

// ---- The front app and its page ----

#[derive(Clone)]
pub struct Front {
    pub name: Option<String>,
    pub bundle: Option<String>,
    pub pid: i32,
}

/// The app in front, which is where the selection is.
pub fn frontmost() -> Option<Front> {
    let app = NSWorkspace::sharedWorkspace().frontmostApplication()?;
    Some(Front {
        name: app.localizedName().map(|n| n.to_string()),
        bundle: app.bundleIdentifier().map(|b| b.to_string()),
        pid: app.processIdentifier(),
    })
}

/// Browsers that answer AppleScript, and how to ask each one.
fn page_script(bundle: &str) -> Option<String> {
    // Chromium-based browsers share Chrome's AppleScript dictionary.
    let chromium = [
        "com.google.Chrome",
        "com.google.Chrome.beta",
        "com.google.Chrome.dev",
        "com.google.Chrome.canary",
        "company.thebrowser.Browser",
        "company.thebrowser.dia",
        "com.brave.Browser",
        "com.brave.Browser.beta",
        "com.microsoft.edgemac",
        "org.chromium.Chromium",
        "com.vivaldi.Vivaldi",
        "com.operasoftware.Opera",
        "net.imput.helium",
        "ai.perplexity.comet",
    ];
    if bundle == "com.apple.Safari" || bundle == "com.apple.SafariTechnologyPreview" {
        Some(format!(
            "tell application id \"{bundle}\" to return (URL of front document) & linefeed & (name of front document)"
        ))
    } else if chromium.contains(&bundle) {
        Some(format!(
            "tell application id \"{bundle}\" to tell active tab of front window to return (URL) & linefeed & (title)"
        ))
    } else {
        None
    }
}

/// The URL and title of the page in front, when the front app is a browser
/// rjot knows. macOS asks once per browser before allowing it.
pub fn browser_page(bundle: &str) -> Option<(String, String)> {
    let script = page_script(bundle)?;
    let out = run_with_timeout(
        std::process::Command::new("/usr/bin/osascript").arg("-e").arg(script),
        Duration::from_millis(1500),
    )?;
    let mut lines = out.lines();
    let url = lines.next()?.trim().to_string();
    let title = lines.next().unwrap_or("").trim().to_string();
    (url.starts_with("http://") || url.starts_with("https://")).then_some((url, title))
}

/// Run a command and return its stdout, giving up after `limit`.
fn run_with_timeout(cmd: &mut std::process::Command, limit: Duration) -> Option<String> {
    use std::io::Read;
    let mut child = cmd.stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::null()).spawn().ok()?;
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => {
                let mut out = String::new();
                child.stdout.take()?.read_to_string(&mut out).ok()?;
                return Some(out);
            }
            Ok(Some(_)) => return None,
            Ok(None) if started.elapsed() < limit => std::thread::sleep(Duration::from_millis(20)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    }
}

// ---- Reading the selection ----

/// The focused element's selected text, through Accessibility. Many native
/// apps answer; browsers and Electron apps often don't.
pub fn selected_text() -> Option<String> {
    unsafe {
        let system = AXUIElementCreateSystemWide();
        if system.is_null() {
            return None;
        }
        AXUIElementSetMessagingTimeout(system, 0.4);
        let focused = copy_attribute(system, "AXFocusedUIElement");
        core_foundation::base::CFRelease(system);
        let focused = focused?;
        let text = copy_attribute(focused.as_CFTypeRef(), "AXSelectedText")?;
        text.downcast::<CFString>().map(|s| s.to_string())
    }
}

unsafe fn copy_attribute(el: AXUIElementRef, name: &'static str) -> Option<CFType> {
    let attr = CFString::from_static_string(name);
    let mut value: CFTypeRef = std::ptr::null();
    let err = unsafe { AXUIElementCopyAttributeValue(el, attr.as_concrete_TypeRef(), &mut value) };
    (err == 0 && !value.is_null()).then(|| unsafe { CFType::wrap_under_create_rule(value) })
}

pub enum Copied {
    Text(String),
    /// A copied image, as PNG bytes.
    Image(Vec<u8>),
    /// A password manager marked the copy as secret; it isn't kept.
    Concealed,
    Nothing,
}

/// Pasteboard types apps use to say "don't record this" (nspasteboard.org).
const CONCEALED_TYPES: [&str; 2] = ["org.nspasteboard.ConcealedType", "com.agilebits.onepassword"];
/// Marks rjot's own short-lived clipboard writes, so clipboard managers
/// skip them.
const TRANSIENT_TYPE: &str = "org.nspasteboard.TransientType";

/// Whether the clipboard holds something marked as secret.
fn concealed(pb: &NSPasteboard) -> bool {
    pb.types().is_some_and(|types| types.iter().any(|t| CONCEALED_TYPES.contains(&t.to_string().as_str())))
}

/// Press ⌘C in the front app and read what it copied (text first, else an
/// image), then put the clipboard back the way it was.
pub fn copy_selection() -> Copied {
    let pb = NSPasteboard::generalPasteboard();
    let before = pb.changeCount();
    let saved = snapshot(&pb);
    if press(KEY_C, CGEventFlags::CGEventFlagCommand).is_none() {
        return Copied::Nothing;
    }
    let started = Instant::now();
    while pb.changeCount() == before && started.elapsed() < Duration::from_millis(350) {
        std::thread::sleep(Duration::from_millis(15));
    }
    if pb.changeCount() == before {
        return Copied::Nothing;
    }
    let secret = concealed(&pb);
    let text = pb.stringForType(unsafe { NSPasteboardTypeString }).map(|s| s.to_string());
    let image = if text.as_deref().is_none_or(|t| t.trim().is_empty()) { clipboard_image(&pb) } else { None };
    restore(&pb, &saved);
    match (text, image) {
        _ if secret => Copied::Concealed,
        (Some(t), _) if !t.trim().is_empty() => Copied::Text(t),
        (_, Some(png)) => Copied::Image(png),
        _ => Copied::Nothing,
    }
}

/// The clipboard's image as PNG, converting TIFF when that's all there is.
fn clipboard_image(pb: &NSPasteboard) -> Option<Vec<u8>> {
    let data = |t: &str| pb.dataForType(&NSString::from_str(t)).map(|d| d.to_vec());
    if let Some(png) = data("public.png") {
        return Some(png);
    }
    let tiff = data("public.tiff")?;
    let img = image::load_from_memory_with_format(&tiff, image::ImageFormat::Tiff).ok()?;
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png).ok()?;
    Some(out.into_inner())
}

const KEY_C: u16 = 8;

fn press(key: u16, flags: CGEventFlags) -> Option<()> {
    let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState).ok()?;
    for down in [true, false] {
        let event = CGEvent::new_keyboard_event(source.clone(), key, down).ok()?;
        event.set_flags(flags);
        event.post(CGEventTapLocation::HID);
    }
    Some(())
}

/// Every item on the clipboard, with the data for each of its types.
type Snapshot = Vec<Vec<(Retained<NSString>, Retained<NSData>)>>;

fn snapshot(pb: &NSPasteboard) -> Snapshot {
    let Some(items) = pb.pasteboardItems() else { return Vec::new() };
    items
        .iter()
        .map(|item| item.types().iter().filter_map(|t| Some((t.clone(), item.dataForType(&t)?))).collect())
        .collect()
}

fn restore(pb: &NSPasteboard, saved: &Snapshot) {
    pb.clearContents();
    if saved.is_empty() {
        return;
    }
    let items: Vec<Retained<ProtocolObject<dyn NSPasteboardWriting>>> = saved
        .iter()
        .map(|types| {
            let item = NSPasteboardItem::new();
            for (t, data) in types {
                item.setData_forType(data, t);
            }
            ProtocolObject::from_retained(item)
        })
        .collect();
    pb.writeObjects(&NSArray::from_retained_slice(&items));
}

// ---- Writing to other apps ----

/// Paste `text` into the app with `pid`: bring it forward, put the text on
/// the clipboard (marked transient), press ⌘V, then put the clipboard back,
/// unless something else was copied meanwhile. Refuses (before touching
/// the clipboard) if that app never comes to the front. Blocking for about
/// a second; call it off the UI thread.
pub fn paste_into(pid: i32, text: &str) -> Result<(), String> {
    let app = NSRunningApplication::runningApplicationWithProcessIdentifier(pid).ok_or("That app has quit")?;
    app.activateWithOptions(NSApplicationActivationOptions::empty());
    let started = Instant::now();
    while frontmost().is_none_or(|f| f.pid != pid) {
        if started.elapsed() > Duration::from_millis(1200) {
            return Err("Couldn't bring that app back to the front, so nothing was pasted".into());
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    std::thread::sleep(Duration::from_millis(80));
    let pb = NSPasteboard::generalPasteboard();
    let saved = snapshot(&pb);
    pb.clearContents();
    pb.setString_forType(&NSString::from_str(text), unsafe { NSPasteboardTypeString });
    pb.setData_forType(Some(&NSData::new()), &NSString::from_str(TRANSIENT_TYPE));
    let mine = pb.changeCount();
    press(KEY_V as u16, CGEventFlags::CGEventFlagCommand).ok_or("Couldn't press ⌘V")?;
    // Give slow apps time to read the clipboard before it changes back.
    std::thread::sleep(Duration::from_millis(700));
    if pb.changeCount() == mine {
        restore(&pb, &saved);
    }
    Ok(())
}

/// Put a PNG on the clipboard.
pub fn copy_image(png: &[u8]) {
    let pb = NSPasteboard::generalPasteboard();
    pb.clearContents();
    pb.setData_forType(Some(&NSData::with_bytes(png)), &NSString::from_str("public.png"));
}

/// Put text on the clipboard (from any thread; the menu bar uses it).
pub fn copy_text(text: &str) {
    let pb = NSPasteboard::generalPasteboard();
    pb.clearContents();
    pb.setString_forType(&NSString::from_str(text), unsafe { NSPasteboardTypeString });
}

// ---- Launch at login (SMAppService, macOS 13+) ----

/// The main-app login item, when this macOS has SMAppService and rjot runs
/// from an app bundle.
fn login_service() -> Option<Retained<objc2::runtime::AnyObject>> {
    use objc2::msg_send;
    use objc2::runtime::{AnyClass, AnyObject};
    NSRunningApplication::currentApplication().bundleIdentifier()?;
    let class = AnyClass::get(c"SMAppService")?;
    let service: Option<Retained<AnyObject>> = unsafe { msg_send![class, mainAppService] };
    service
}

pub fn login_supported() -> bool {
    login_service().is_some()
}

pub fn login_enabled() -> bool {
    use objc2::msg_send;
    // SMAppServiceStatusEnabled = 1.
    login_service().is_some_and(|s| {
        let status: isize = unsafe { msg_send![&*s, status] };
        status == 1
    })
}

pub fn set_login(on: bool) -> Result<(), String> {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;
    let service = login_service().ok_or("Launch at login needs macOS 13 or later")?;
    let mut error: *mut AnyObject = std::ptr::null_mut();
    let ok: bool = unsafe {
        if on {
            msg_send![&*service, registerAndReturnError: &mut error]
        } else {
            msg_send![&*service, unregisterAndReturnError: &mut error]
        }
    };
    if ok {
        Ok(())
    } else {
        let what = if on { "turn on" } else { "turn off" };
        let detail = if error.is_null() {
            String::new()
        } else {
            let desc: Option<Retained<NSString>> = unsafe { msg_send![error, localizedDescription] };
            desc.map(|d| format!(": {d}")).unwrap_or_default()
        };
        Err(format!("Couldn't {what} launch at login{detail}"))
    }
}

// ---- App ----

/// A short system sound to confirm a capture without taking focus.
pub fn play(sound: &str) {
    if let Some(s) = NSSound::soundNamed(&NSString::from_str(sound)) {
        s.play();
    }
}

/// Bring rjot to the front.
pub fn activate() {
    NSRunningApplication::currentApplication().activateWithOptions(NSApplicationActivationOptions::ActivateAllWindows);
}

/// Hide rjot (like ⌘H); the previous app comes back to the front.
pub fn hide() {
    NSRunningApplication::currentApplication().hide();
}

/// Open a URL in the default browser.
pub fn open_url(url: &str) {
    if url.starts_with("http://") || url.starts_with("https://") {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
}

/// Keep the window out of screenshots, recordings and screen sharing (or
/// let it back in). Main thread only.
pub fn set_window_private(frame: &eframe::Frame, private: bool) {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let Ok(handle) = frame.window_handle() else { return };
    let RawWindowHandle::AppKit(h) = handle.as_raw() else { return };
    let view = h.ns_view.as_ptr() as *mut AnyObject;
    unsafe {
        let window: *mut AnyObject = msg_send![view, window];
        if !window.is_null() {
            // NSWindowSharingNone = 0, NSWindowSharingReadOnly = 1.
            let sharing: usize = if private { 0 } else { 1 };
            let _: () = msg_send![window, setSharingType: sharing];
        }
    }
}

/// Local UTC offset in seconds.
pub fn utc_offset() -> i64 {
    unsafe {
        let now = libc::time(std::ptr::null_mut());
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&now, &mut tm);
        tm.tm_gmtoff
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_scripts() {
        assert!(page_script("com.apple.Safari").unwrap().contains("front document"));
        assert!(page_script("company.thebrowser.Browser").unwrap().contains("active tab"));
        assert!(page_script("net.imput.helium").unwrap().contains("active tab"));
        assert!(page_script("org.mozilla.firefox").is_none());
        assert!(page_script("com.apple.Notes").is_none());
    }
}
