// Prints the CoreGraphics window number of the main window owned by a PID,
// plus its bounds ("<id> <x> <y> <w> <h>"), for `screencapture -l` / `-R`
// (scripts/screenshots.sh).
import CoreGraphics
import Foundation

// `winid --park x y`: move the mouse pointer to x,y (out of a recording).
if CommandLine.arguments.count == 4, CommandLine.arguments[1] == "--park",
   let x = Double(CommandLine.arguments[2]), let y = Double(CommandLine.arguments[3]) {
    CGWarpMouseCursorPosition(CGPoint(x: x, y: y))
    exit(0)
}

let pid = Int32(CommandLine.arguments[1]) ?? 0
let list = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
for w in list where (w[kCGWindowOwnerPID as String] as? Int32) == pid && (w[kCGWindowLayer as String] as? Int) == 0 {
    let b = w[kCGWindowBounds as String] as? [String: Any] ?? [:]
    print(w[kCGWindowNumber as String]!, b["X"] ?? 0, b["Y"] ?? 0, b["Width"] ?? 0, b["Height"] ?? 0)
    exit(0)
}
exit(1)
