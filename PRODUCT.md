# Product

<!-- impeccable:product-schema 1 -->

## Platform

web (the landing site in `site/`; the product itself is a native macOS app built with Rust and egui)

## Users

AI-heavy builders: developers and makers who work all day across ChatGPT, Claude, Cursor, a browser and an editor. They keep running into answers, snippets, links and ideas worth keeping, and they write prompts ahead of time to use later. They want to keep things without switching apps or breaking their flow.

## Product Purpose

rjot keeps the text you highlight. Select text in any app and double-tap Shift: it's saved as a note with the source app and time, and you stay where you were. With nothing selected, the same gesture opens a quick note box. A Queue holds prompts to use later (copying one checks it off), and a Scratch pad holds anything else. Success: someone installs it, grants Accessibility once, and saves their first selection within a minute.

## Positioning

Free, open source (MIT), local-only and a small native binary. Capture is one gesture (⇧⇧) with no window to switch to. Never name or compare against competitors.

## Operating Context

Runs in the background on a Mac next to AI chats, editors and browsers. Closing the window hides it; capture keeps working. Distributed as an Apple silicon zip on GitHub Releases (`yoouTaro/rjot`), signed with a self-signed certificate and not notarized, so the first launch needs right-click → Open or `xattr -dr com.apple.quarantine /Applications/rjot.app`. Requires Accessibility permission.

## Capabilities and Constraints

- Capture: double-tap Shift; Accessibility `AXSelectedText` first, then a ⌘C fallback that restores the clipboard with every type it held. Concealed (password-manager) copies are never saved.
- Notes, Queue (check off, copy-and-check-off, clear done with undo) and Scratch; search; inline edit; undo delete.
- Color labels (red, orange, yellow, green, blue, purple) with a filter by color.
- Six themes (Copper, Graphite, Sage, Ocean, Plum, Mono), each in dark and light; appearance can follow the system or be set to light or dark.
- Sensitive mode (⌘⇧S): blurs note and scratch text, ⌥ to peek, and keeps the window out of screenshots and screen sharing.
- Data: one JSON file at `~/Library/Application Support/dev.rjot.rjot/notes.json`, images in `images/` beside it, and an optional Markdown copy in a folder the user picks. No account, sync or analytics. The only network request is "Check for updates…" (GitHub), and only when the user clicks it.
- Menu bar icon: recent captures (click to copy), Quick note, Open, Pause capture, Launch at login (macOS 13+), Quit.
- ⌥⇧V paste picker: a filterable list of queued prompts; the chosen one is pasted into the app that was in front and checked off.
- Browser captures (Safari, Chrome, Arc, Brave, Edge, Vivaldi) also keep the page URL and title (macOS asks once per browser). Firefox has no AppleScript, so it gets the app name only.
- Pin (top of the list) and archive (out of the list, kept). Image notes: select an image and tap Shift twice.
- macOS 11+, Apple silicon only for now. Release v0.0.3-alpha, zip of about 5.2 MB.
- Known limits: editors that copy the whole line on ⌘C with nothing selected (VS Code) save that line; macOS 14+ may not let rjot bring itself to the front.

## Brand Commitments

- Name: rjot, always lowercase. App icon: `assets/logo.svg` (a highlighted line between two plain lines, copper on warm dark).
- Voice: modest and plain. Say what it does in short sentences. Never salesy, no hype words.
- No emoji. Author: Taro (GitHub yoouTaro). Sibling project: rgrip.

## Evidence on Hand

Real app captures from scripted demo scenes (`scripts/screenshots.sh`): `docs/screenshots/*.png` (dark and light), `site/img/*.webp` (light Copper). Measured size: 5.2 MB zip (0.0.3-alpha). No users, testimonials, download counts or benchmarks exist; don't invent any.

## Product Principles

1. The gesture is the product: one move, no context switch.
2. Local and honest: nothing leaves the Mac unless the user asks (the update check), and the site says only what's true.
3. Small over feature lists.
4. Plain words; the real app shows what it does.
