# Changelog

## Unreleased

- **Scratch** is ready to type the moment you open it (⌘3 or the tab), with the cursor at the end. In sensitive mode it stays blurred until you click.
- **Khmer:** notes, prompts, the scratchpad, search and page titles are drawn shaped (subscript consonants stack, vowels sit where they belong) and wrap at word breaks. rjot uses Kantumruy Pro when it's installed, otherwise the Khmer Sangam MN font that comes with macOS. Markdown file names keep Khmer intact.

## 0.0.2-alpha — 2026-10-16

- **Menu bar icon:** the last five captures (click to copy), Quick note, Open rjot, Pause capture, Launch at login (macOS 13+) and Quit.
- **⌥⇧V paste picker:** a list of queued prompts, filtered as you type. ↩ pastes the chosen one into the app you were in (your clipboard is put back) and checks it off.
- **Page links:** captures from Safari, Chrome, Arc, Brave, Edge and Vivaldi keep the page's URL and title; the card links back to it. macOS asks once per browser.
- **Markdown copy:** pick a folder (an Obsidian vault, say) and rjot keeps an `rjot` folder there up to date, with Notes, Queue, Archive, images and Scratch.md. It only ever deletes files it wrote.
- **Pin and archive:** pinned cards stay at the top; archived cards leave the list but stay one click away.
- **Image notes:** select an image and tap Shift twice.
- **Check for updates…** in settings: downloads the next release from GitHub, checks its SHA-256 and restarts into it. rjot never checks on its own.
- Pause capture from settings or the menu bar.

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
