// The page's Shift keys follow yours, and two quick taps save the current
// selection into the page's rjot window, recognised the same way the app
// does it: two presses of Shift alone, each under 300 ms, under 350 ms apart.

const MAX_HOLD = 300;
const MAX_GAP = 350;
const reduced = matchMedia("(prefers-reduced-motion: reduce)").matches;

const shifts = [...document.querySelectorAll("[data-shift]")];
const notes = document.querySelector("[data-notes]");
const count = document.querySelector("[data-count]");
const composer = document.querySelector("[data-composer]");
const quick = composer.querySelector("textarea");
const tryLine = document.querySelector("[data-try]");
const windowEl = document.querySelector(".window");

let downAt = null;
let lastUp = null;

function tap(kind, t) {
  if (kind === "down") {
    if (lastUp !== null && t - lastUp > MAX_GAP) lastUp = null;
    downAt = t;
    return false;
  }
  if (kind === "up") {
    const d = downAt;
    downAt = null;
    if (d === null || t - d > MAX_HOLD) { lastUp = null; return false; }
    if (lastUp !== null) { lastUp = null; return true; }
    lastUp = t;
    return false;
  }
  downAt = lastUp = null;
  return false;
}

function press(side, down) {
  const keys = side ? shifts.filter((k) => k.dataset.shift === String(side)) : shifts;
  keys.forEach((k) => k.classList.toggle("down", down));
}

function source() {
  const ua = navigator.userAgent;
  if (/Edg\//.test(ua)) return "Edge";
  if (/Firefox\//.test(ua)) return "Firefox";
  if (/Chrome\//.test(ua)) return "Chrome";
  if (/Safari\//.test(ua)) return "Safari";
  return "Your browser";
}

function addNote(text, from) {
  const li = document.createElement("li");
  const p = document.createElement("p");
  const small = document.createElement("small");
  p.textContent = text;
  small.textContent = `${from} · just now`;
  li.append(p, small);
  li.className = "fresh";
  notes.prepend(li);
  count.textContent = notes.children.length;
  if (!reduced) {
    // Lands with a little mass: a short overshoot, then settles.
    li.animate(
      [
        { transform: "translateY(-16px)", opacity: 0 },
        { transform: "translateY(3px)", opacity: 1, offset: 0.62 },
        { transform: "translateY(-1px)", offset: 0.84 },
        { transform: "none" },
      ],
      { duration: 520, easing: "cubic-bezier(0.2, 0.7, 0.3, 1)" },
    );
  }
  pop();
}

let audio;
function pop() {
  // A quiet pop, like the app's: a short sine drop. Only ever after a key press.
  try {
    audio ??= new AudioContext();
    const t = audio.currentTime;
    const osc = audio.createOscillator();
    const gain = audio.createGain();
    osc.frequency.setValueAtTime(880, t);
    osc.frequency.exponentialRampToValueAtTime(320, t + 0.09);
    gain.gain.setValueAtTime(0.0001, t);
    gain.gain.exponentialRampToValueAtTime(0.06, t + 0.008);
    gain.gain.exponentialRampToValueAtTime(0.0001, t + 0.12);
    osc.connect(gain).connect(audio.destination);
    osc.start(t);
    osc.stop(t + 0.13);
  } catch {}
}

function capture() {
  const sel = window.getSelection();
  const text = sel ? sel.toString().replace(/\s+\n/g, "\n").trim() : "";
  const inside = sel && sel.anchorNode && windowEl.contains(sel.anchorNode);
  if (text && !inside) {
    addNote(text.length > 600 ? text.slice(0, 600) + "…" : text, source());
    sel.removeAllRanges();
    tryLine.textContent = "Kept. That's the whole thing, and in the app it works in every app.";
    tryLine.classList.add("done");
  } else {
    // As in the app: nothing selected opens a quick note instead.
    composer.hidden = false;
    quick.focus({ preventScroll: true });
    tryLine.textContent = "Nothing was selected, so rjot opened a quick note. Type one, then press ⌘↩.";
  }
}

// Your keyboard.
addEventListener("keydown", (e) => {
  if (e.key !== "Shift") { tap("other"); return; }
  if (e.repeat) return;
  if (e.ctrlKey || e.metaKey || e.altKey) { tap("other"); return; }
  press(e.location === 2 ? 2 : 1, true);
  tap("down", e.timeStamp);
});
addEventListener("keyup", (e) => {
  if (e.key !== "Shift") return;
  press(e.location === 2 ? 2 : 1, false);
  if (e.target === quick) return;
  if (tap("up", e.timeStamp)) capture();
});
addEventListener("blur", () => { press(0, false); tap("other"); });

// The caps on the page, for touch and mouse. Pressing them must not clear
// the selection, so the default mousedown is held back.
shifts.forEach((key) => {
  key.addEventListener("pointerdown", (e) => {
    e.preventDefault();
    key.classList.add("down");
    tap("down", e.timeStamp);
  });
  const up = (e) => {
    if (!key.classList.contains("down")) return;
    key.classList.remove("down");
    if (e.type === "pointerup" && tap("up", e.timeStamp)) capture();
  };
  key.addEventListener("pointerup", up);
  key.addEventListener("pointerleave", up);
  key.addEventListener("click", (e) => e.preventDefault());
});

// The quick note: ⌘↩ (or Ctrl+Enter) saves, Esc closes.
quick.addEventListener("keydown", (e) => {
  if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
    e.preventDefault();
    const text = quick.value.trim();
    if (text) addNote(text, "Quick note");
    quick.value = "";
    composer.hidden = true;
    quick.blur();
  } else if (e.key === "Escape") {
    composer.hidden = true;
    quick.blur();
  }
});
