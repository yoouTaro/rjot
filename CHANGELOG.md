# Changelog

## 0.0.1-alpha — 2026-10-16

The first release.

- **Capture:** select text in any app and tap Shift twice; it's saved as a note with the app it came from. rjot asks the focused element through Accessibility first, then falls back to ⌘C and puts the clipboard back the way it was, every type included. Copies a password manager marks as concealed are never saved.
- **Quick note:** with nothing selected, the same gesture opens rjot with the note box focused; ⌘↩ saves and hides it again.
- **Queue** for prompts (copying one checks it off, with undo) and a **Scratch** pad.
- **Color labels** (six colors) with a filter by color; search, inline edit, undo delete.
- **Themes:** Copper, Graphite, Sage, Ocean, Plum and Mono, each dark and light; appearance follows the system or is set by hand.
- **Sensitive mode** (⌘⇧S): note text blurs (hold ⌥ to peek) and the window stays out of screenshots and screen sharing.
- Everything in one local JSON file; no account, sync or network.
- The Accessibility button clears a stale permission entry left by an older build before asking again.

Apple silicon, macOS 11 or later. Not notarized: right-click → Open the first time, or `xattr -dr com.apple.quarantine rjot.app`.
