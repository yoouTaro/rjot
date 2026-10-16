//! The macOS side: a listen-only event tap for double-tap Shift, reading the
//! selected text (Accessibility first, then a ⌘C round trip that puts the
//! clipboard back), and small AppKit calls.

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
use std::sync::atomic::{AtomicPtr, Ordering};
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

/// Whether rjot may read other apps' selections and listen for Shift.
pub fn trusted() -> bool {
    unsafe { AXIsProcessTrusted() }
}

/// Show the system's Accessibility prompt and open the settings pane.
pub fn ask_for_access() {
    let key = CFString::from_static_string("AXTrustedCheckOptionPrompt");
    let options = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), CFBoolean::true_value().as_CFType())]);
    unsafe { AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) };
    let _ = std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn();
}

// ---- Double-tap Shift ----

static TAP_PORT: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

const KEY_LEFT_SHIFT: i64 = 56;
const KEY_RIGHT_SHIFT: i64 = 60;

/// Listen for double-tap Shift on a thread of its own and call `on_tap`
/// each time. Waits for Accessibility access first.
pub fn listen(on_tap: impl Fn() + Send + 'static) {
    std::thread::Builder::new()
        .name("shift-tap".into())
        .spawn(move || {
            while !trusted() {
                std::thread::sleep(Duration::from_secs(1));
            }
            let start = Instant::now();
            let state = RefCell::new(DoubleTap::default());
            let tap = CGEventTap::new(
                CGEventTapLocation::Session,
                CGEventTapPlacement::TailAppendEventTap,
                CGEventTapOptions::ListenOnly,
                vec![CGEventType::FlagsChanged, CGEventType::KeyDown],
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
                        _ => Input::Other,
                    };
                    if state.borrow_mut().feed(input) {
                        on_tap();
                    }
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

// ---- Reading the selection ----

pub struct Front {
    pub name: Option<String>,
    pub pid: i32,
}

/// The app in front, which is where the selection is.
pub fn frontmost() -> Option<Front> {
    let app = NSWorkspace::sharedWorkspace().frontmostApplication()?;
    Some(Front { name: app.localizedName().map(|n| n.to_string()), pid: app.processIdentifier() })
}

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
    /// A password manager marked the copy as secret; it isn't kept.
    Concealed,
    Nothing,
}

/// Pasteboard types apps use to say "don't record this" (nspasteboard.org).
const CONCEALED_TYPES: [&str; 2] = ["org.nspasteboard.ConcealedType", "com.agilebits.onepassword"];

/// Whether the clipboard holds something marked as secret.
fn concealed(pb: &NSPasteboard) -> bool {
    pb.types().is_some_and(|types| types.iter().any(|t| CONCEALED_TYPES.contains(&t.to_string().as_str())))
}

/// Press ⌘C in the front app and read what it copied, then put the
/// clipboard back the way it was.
pub fn copy_selection() -> Copied {
    let pb = NSPasteboard::generalPasteboard();
    let before = pb.changeCount();
    let saved = snapshot(&pb);
    if press_cmd_c().is_none() {
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
    restore(&pb, &saved);
    match text {
        _ if secret => Copied::Concealed,
        Some(t) if !t.trim().is_empty() => Copied::Text(t),
        _ => Copied::Nothing,
    }
}

fn press_cmd_c() -> Option<()> {
    const KEY_C: u16 = 8;
    let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState).ok()?;
    for down in [true, false] {
        let event = CGEvent::new_keyboard_event(source.clone(), KEY_C, down).ok()?;
        event.set_flags(CGEventFlags::CGEventFlagCommand);
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
