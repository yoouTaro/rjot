---
name: rjot site
description: "Shift, Twice": the rjot website as a keycap set catalog, colorway "rjot".
colors:
  case: "#C7CBCF"
  case-hi: "#D3D6D9"
  ink: "#1C1E21"
  ink-2: "#3F444A"
  ink-3: "#50565D"
  copper-top: "#E3874D"
  copper-skirt: "#B65F2B"
  copper-legend: "#2B1205"
  alpha-top: "#EFEFEB"
  alpha-skirt: "#CBCDC9"
  alpha-legend: "#2A2C2F"
  slate-top: "#58606A"
  slate-skirt: "#3D444C"
  slate-legend: "#F6F7F8"
  plate: "#24272B"
  plate-ink: "#ECEEF0"
  plate-ink-2: "#A9B0B7"
  plate-line: "rgba(255, 255, 255, 0.10)"
  plate-code: "#33373C"
  label-red: "#F06A62"
  label-orange: "#F0883E"
  label-yellow: "#E3B341"
  label-green: "#57AB5A"
  label-blue: "#539BF5"
  label-purple: "#B083F0"
  window-bg: "#F7F4F0"
  window-card: "#FFFFFF"
  window-line: "#E4DDD5"
  window-ink: "#25211D"
  window-faint: "#6B635A"
  window-raise: "#EFEAE4"
  window-accent: "#B0643A"
typography:
  display:
    fontFamily: "Archivo, Helvetica Neue, sans-serif"
    fontSize: "clamp(52px, 7.4vw, 96px)"
    fontWeight: 820
    lineHeight: 0.98
    letterSpacing: "-0.03em"
    fontVariation: "'wdth' 112"
  headline:
    fontFamily: "Archivo, Helvetica Neue, sans-serif"
    fontSize: "clamp(34px, 4.4vw, 56px)"
    fontWeight: 780
    lineHeight: 1.04
    letterSpacing: "-0.025em"
    fontVariation: "'wdth' 112"
  title:
    fontFamily: "Archivo, Helvetica Neue, sans-serif"
    fontSize: "26px"
    fontWeight: 720
    lineHeight: 1.15
    letterSpacing: "-0.015em"
    fontVariation: "'wdth' 108"
  lead:
    fontFamily: "Archivo, Helvetica Neue, sans-serif"
    fontSize: "clamp(18px, 1.6vw, 20px)"
    fontWeight: 400
    lineHeight: 1.6
  body:
    fontFamily: "Archivo, Helvetica Neue, sans-serif"
    fontSize: "17px"
    fontWeight: 400
    lineHeight: 1.6
  legend:
    fontFamily: "Archivo, Helvetica Neue, sans-serif"
    fontSize: "calc(var(--u) * 0.24)"
    fontWeight: 600
    lineHeight: 1.1
    letterSpacing: "-0.005em"
    fontVariation: "'wdth' 104"
  label:
    fontFamily: "Archivo, Helvetica Neue, sans-serif"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: 1.4
  measure:
    fontFamily: "Geist Mono, SF Mono, ui-monospace, monospace"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: 1.5
    fontFeature: "tnum"
rounded:
  code: "4px"
  swatch: "8px"
  card: "10px"
  tray: "12px"
  window: "14px"
  shot: "16px"
  key: "calc(var(--u) * 0.13)"
spacing:
  gutter: "32px"
  gutter-narrow: "20px"
  container: "1200px"
  key-u-chip: "40px"
  key-u-tray: "50px"
  key-u-base: "56px"
  key-u-action: "62px"
  key-u-step: "64px"
  key-u-enter: "92px"
  key-gap: "clamp(4px, 0.5vw, 7px)"
  key-pitch: "clamp(42px, calc((100vw - 64px) / 15.4), 84px)"
  stack-sm: "12px"
  stack-md: "28px"
  stack-lg: "64px"
  section: "120px"
components:
  key-alpha:
    backgroundColor: "{colors.alpha-top}"
    textColor: "{colors.alpha-legend}"
    rounded: "{rounded.key}"
    height: "{spacing.key-u-base}"
    width: "{spacing.key-u-base}"
    typography: "{typography.legend}"
  key-accent:
    backgroundColor: "{colors.copper-top}"
    textColor: "{colors.copper-legend}"
    rounded: "{rounded.key}"
    typography: "{typography.legend}"
  key-mod:
    backgroundColor: "{colors.slate-top}"
    textColor: "{colors.slate-legend}"
    rounded: "{rounded.key}"
    typography: "{typography.legend}"
  key-chip:
    backgroundColor: "{colors.slate-top}"
    textColor: "{colors.slate-legend}"
    rounded: "{rounded.key}"
    height: "{spacing.key-u-chip}"
    padding: "4px 15px 9px"
  key-action:
    backgroundColor: "{colors.copper-top}"
    textColor: "{colors.copper-legend}"
    rounded: "{rounded.key}"
    height: "{spacing.key-u-action}"
    padding: "7px 22px 15px"
  key-enter:
    backgroundColor: "{colors.copper-top}"
    textColor: "{colors.copper-legend}"
    rounded: "{rounded.key}"
    height: "{spacing.key-u-enter}"
    padding: "11px 30px 22px"
  key-novelty:
    backgroundColor: "{colors.label-blue}"
    textColor: "{colors.ink}"
    rounded: "{rounded.key}"
    height: "{spacing.key-u-action}"
  kbd:
    backgroundColor: "{colors.alpha-top}"
    textColor: "{colors.alpha-legend}"
    height: "1.85em"
    padding: "0 0.45em 0.32em"
  kbd-dark:
    backgroundColor: "{colors.slate-top}"
    textColor: "{colors.slate-legend}"
    height: "1.85em"
    padding: "0 0.45em 0.32em"
  plate-band:
    backgroundColor: "{colors.plate}"
    textColor: "{colors.plate-ink}"
  tray:
    backgroundColor: "{colors.plate}"
    textColor: "{colors.plate-ink-2}"
    rounded: "{rounded.tray}"
    padding: "18px 20px 14px"
  window:
    backgroundColor: "{colors.window-bg}"
    textColor: "{colors.window-ink}"
    rounded: "{rounded.window}"
    padding: "12px 14px 10px"
  window-note:
    backgroundColor: "{colors.window-card}"
    textColor: "{colors.window-ink}"
    rounded: "{rounded.card}"
    padding: "9px 12px 7px 14px"
  shot:
    backgroundColor: "{colors.case-hi}"
    rounded: "{rounded.shot}"
    padding: "36px"
  spec-row:
    textColor: "{colors.plate-ink-2}"
    padding: "16px 0"
  code-inline:
    backgroundColor: "{colors.case-hi}"
    textColor: "{colors.ink}"
    rounded: "{rounded.code}"
    padding: "0.1em 0.35em"
---

# Design System: rjot site

Scope: the public website in `site/` only. The native egui app has its own themes and look; the page's window replica borrows the app's light Copper theme on purpose, and nothing else here describes the app.

## Overview

**Creative North Star: "Shift, Twice"**

The site is a keycap set catalog for one colorway, "rjot". The product's mechanism is a key, so the page is built from keys: every button, nav link, shortcut chip, numbered step and color swatch is a keycap, drawn top-down with a lit skirt, a dished top face and the legend set top-left. The two Shift keys are copper; they are the product, and they press under the visitor's own Shift.

The ground is a cool grey case with light grey alpha caps, slate modifiers and copper accents. A charcoal plate appears as full-width bands (the hero keyboard row, the specification sheet) and as small trays that hold each kit's caps at real size. Real app captures sit on lighter mats, and the one live element is a working copy of the rjot window in the hero. Density is calm and catalog-like: one entry per kit, generous section spacing, short copy.

Type is Archivo, run wide and heavy for catalog headings and medium for legends. Geist Mono appears only for measurements and counts (version, size, timings, cap counts). Motion is physical and tied to keys: a fast press, a sprung return with one small overshoot.

**Key Characteristics:**
- Keycaps are the single component family; they are raster art (9-slice PNGs), not CSS gradients.
- Copper is the accent and marks only Shift, Enter-like primary actions and the final step.
- Charcoal plate bands and trays carry the keyboard material; the case grey carries reading.
- Mono is for numbers you could measure, nothing else.
- Every screenshot is a real app capture with a provenance sidecar.

## Colors

A cool, low-chroma grey keyboard with one warm metal accent; color comes from the caps, not from the page.

### Primary
- **Copper Cap** (copper-top): the top face of accent caps: both Shift keys, Download keys, the fourth assembly step. The only saturated color on the case ground.
- **Copper Skirt** (copper-skirt): the cap's shaded walls in the raster; reused in CSS as the text caret. The focus outline uses a deeper copper, #8A4519, for contrast on the case ground.
- **Burnt Legend** (copper-legend): legends on copper caps (6.6:1 on the top face).

### Secondary
- **Slate Mod** (slate-top, slate-skirt): modifier caps: nav links, View source, dark inline chips in the spec sheet. Legends are **Mod White** (slate-legend).
- **Alpha Grey** (alpha-top, alpha-skirt): letter caps in the keyboard row, tray shortcut caps, inline kbd chips, numbered steps. Legends are **Alpha Ink** (alpha-legend).

### Tertiary
- **Label Novelties** (label-red through label-purple): rjot's six note label colors, used as the note-card stripe in the window replica and as the novelty caps in the color labels entry. These are product colors; they never decorate the page.
- **Window Copper set** (window-*): the light Copper theme of the app, scoped to the hero window replica only.

### Neutral
- **Case Grey** (case): page ground, sticky bar (88% mix with backdrop blur), browser theme color.
- **Case Highlight** (case-hi): screenshot mats and inline code on the case.
- **Ink / Ink 2 / Ink 3** (ink, ink-2, ink-3): headings, body copy and leads, mono measurements. Ink 3 is the lightest text allowed on the case (4.55:1).
- **Charcoal Plate** (plate): bands and trays, always with the plate grain raster laid over it. Text on it is **Plate Ink** and **Plate Ink 2**; rules are **Plate Line**; code is **Plate Code**.

### Named Rules
**The Copper Is Shift Rule.** Copper marks the gesture and the download, nothing else. If a third kind of thing turns copper, the accent stops meaning Shift.

**The Product Colors Stay Product Rule.** The six label colors and the six theme swatches appear only where they show rjot's own data or settings.

## Typography

**Display Font:** Archivo (variable, weights 100 to 900, widths 62% to 125%, self-hosted), with Helvetica Neue
**Body Font:** Archivo
**Label/Mono Font:** Geist Mono (self-hosted), with SF Mono and ui-monospace

**Character:** A wide, heavy grotesque reads like the printed legends and headings of a keyboard catalog; the mono is a ruler, used only for numbers.

### Hierarchy
- **Display** (820, clamp 52 to 96px, 0.98, width 112%): the hero headline only.
- **Headline** (780, clamp 34 to 56px, 1.04, width 112%): section titles ("The kits", "Specifications", "Assembly").
- **Title** (720, 26px, 1.15, width 108%): catalog entry names; 21px in assembly steps.
- **Lead** (400, 18 to 20px, 1.6, max 34em): hero and section leads in Ink 2.
- **Body** (400, 17px, 1.6, max 32em in entries): running copy.
- **Legend** (600, 0.24 of the cap height, width 104%): text on keycaps. Fixed sizes on sized caps: 14.5px chips, 16.5px/640 action keys, 17px tray caps, 20px step caps, 24px/680 on the closing Enter key.
- **Label** (400, 13px): tray captions under combos, footer at 14px.
- **Measure** (Geist Mono 400, 13 to 13.5px, tabular figures): the facts line, cap counts under entry titles, inline numbers in the spec sheet.

### Named Rules
**The Ruler Rule.** Geist Mono sets only values someone could measure or count (ms, MB, versions, cap counts). Words stay in Archivo.

**The Wide Heading Rule.** Headings run at 108 to 112% width with negative tracking; body never changes width.

## Layout

A centered 1200px container with 32px gutters (20px under 720px). The hero is a two-column grid (1.25fr / 0.95fr, 56px gap): headline, lead and actions left, the live window right. Below it the keyboard row runs edge to edge on a plate band.

The keyboard row is sized by one unit: the pitch is `(100vw - 64px) / 15.4`, clamped 42 to 84px, with a 4 to 7px gap; each cap's width is pitch times its unit width (Shift 2.25u left, 2.75u right) minus the gap. Under 720px the alpha caps hide and only the two Shift keys remain.

The catalog is a 12-column grid with 88px row and 40px column gaps. Entries take three shapes: a full-width base entry (5fr capture / 7fr text), half-width pairs, and full-width wide entries (7fr text / 5fr capture). The spec section is 0.8fr / 1.2fr with a sticky title column. Assembly is four columns of steps, then a centered closing Enter key.

Rhythm: sections pad 120 to 140px vertically; entries use 28px internal gaps and 64px column gaps; actions sit 12px apart. Under 960px every two-column grid stacks to one column and wide entries put the capture first; under 720px the steps go single-column and the spec sheet stacks term above value.

## Elevation & Depth

Depth is physical and mostly baked into the cap rasters: the skirt walls are lit and shaded in the PNG, so CSS adds only a short cast shadow beneath each cap. The plate reads as recessed: bands and trays use inset top shadows. The window replica and screenshots float slightly above the case with soft, long, negative-spread shadows. There is no glow and no colored shadow.

### Shadow Vocabulary
- **Cap at rest** (`0 calc(u*0.07) calc(u*0.14) -2px rgba(18,22,26,0.34), 0 1px 1px rgba(18,22,26,0.2)`): every keycap.
- **Cap pressed** (`0 calc(u*0.02) calc(u*0.05) -1px rgba(18,22,26,0.36), 0 1px 0 rgba(18,22,26,0.2)`): with a 0.06u downward travel.
- **Window lift** (`0 1px 1px rgba(18,22,26,0.12), 0 22px 48px -18px rgba(18,22,26,0.45)`): the hero window.
- **Capture lift** (`0 1px 1px rgba(18,22,26,0.1), 0 26px 44px -22px rgba(18,22,26,0.5)`): screenshots on their mats.
- **Plate recess** (`inset 0 10px 18px -12px rgba(0,0,0,0.8), inset 0 -1px 0 rgba(255,255,255,0.06)`): the keyboard band; trays use `inset 0 6px 12px -8px rgba(0,0,0,0.8)`.

### Named Rules
**The Baked Skirt Rule.** A keycap's form lives in its raster. CSS never redraws the skirt or dish; it only casts the shadow and moves the cap.

## Shapes

Corners are soft and small, scaled to the object. Caps round at 0.13 of their height (the raster's own outer radius is 26px on a 200px tile). Containers step up from 10px note cards to 12px trays, 14px window and 16px capture mats. Inline code is 4px. Window traffic lights are 12px circles; label stripes are 3px bars with 2px ends.

The keycap raster system: `scripts/render-caps.py` renders each colorway as a 200 x 200 px 9-slice PNG (built for a 100px cap at 2x, 3x supersampled): a skirt with left/right walls shaded 0.86/0.80, a 1.06 back wall and a front wall that darkens toward its foot; a top face inset 18/10/18/34 px with an 18px radius and a cylindrical dish; and a fine PBT grain whose period matches the 120px repeating middle slice so wide caps tile without seams. CSS applies it with `border-image-slice: 40 40 70 40 fill`, widths 0.2u/0.2u/0.35u/0.2u, repeat `round stretch`. Colorways: alpha, copper, slate and the six label colors. The plate is a 256px charcoal tile (#26292D base) with a bead-blast grain.

## Components

### Keycap (the one family)
Tactile and literal: it looks like a cap and presses like one.
- **Geometry:** height is `--u`, width is `--u` times `--w`. Padding is 0.12u top, 0.18u sides, 0.24u bottom, so the legend sits on the top face, top-left.
- **Variants:** alpha (light grey, dark legend), accent (copper, burnt legend), mod (slate, white legend), novelty (label color, ink legend, 62px at 1.25u).
- **Sizes:** chip 40px (nav), tray 50px, base 56px, action 62px (hero actions, auto width, 22px side padding, legend centered), step 64px, Enter 92px (76px narrow, min 420px wide, legend left and return glyph right).
- **Press:** `.down` or `:active` moves the cap down 0.06u and tightens the shadow in 60ms ease-out; release springs back over 220ms on the release curve. Linked caps brighten 4% on hover.
- **Focus:** a 2px deep copper (#8A4519) outline at 3px offset, 6px radius; 4.6:1 on the case ground, above the 3:1 focus minimum.
- **Legend icons:** inline SVG symbols (Shift arrow, Return, Command, Option), 1.05em, stroked at 2.

### Inline kbd chips
A small alpha cap set in running text: 1.85em tall, 0.8em/600 legend, same raster at 0.37em/0.65em slice widths, dropped 0.12em to sit on the baseline. A slate variant is used on the dark plate.

### Window replica
A working copy of rjot's Notes list in the app's light Copper theme: 14px corners, traffic lights, a segmented tab strip (Notes with a count, Queue, Scratch), note cards (10px, 1px window-line border, 3px label stripe inside the left edge, source and time in 12px faint text, clamped to four lines) and a footer hint. A quick-note textarea appears when nothing is selected. New notes land with a 520ms drop-and-settle and a 1.6s copper glow fade, and a short synthesized pop plays.

### Keyboard row and plate band
A full-width charcoal band holding the bottom row: copper Shift (2.25u), Z to M, three two-legend punctuation caps, copper Shift (2.75u). Only the Shift caps are interactive; they follow the physical keys and accept pointer presses without clearing the selection. A centered status line above it changes to Ink when a capture succeeds.

### Tray
A piece of plate (12px, inset shadow, grain) under each kit's text, holding shortcut combos as 50px alpha caps with a 13px Plate Ink 2 caption under each combo; the novelty tray holds the six label caps.

### Catalog entry
A capture on a Case Highlight mat (16px, 36px padding, 24px narrow) beside a title, a mono cap count, a short paragraph and a tray. Captures are capped at 360px (320px in half and wide entries).

### Colorway swatches
For each of the six themes, a dark and a light mini row: three caps and a 1.6x wide accent cap on an 8px base, caps 20px tall with 4px corners and an inset bottom edge, theme name below in 15px/600.

### Spec sheet
A definition list on the charcoal band: 9.5em term column (640, width 108%) and Plate Ink 2 values, separated by Plate Line rules, 16px row padding; terms stack above values under 720px.

### Navigation
A sticky 72px bar on translucent case with 10px backdrop blur: wordmark (icon plus lowercase "rjot", 800, 21px) left; chip-size mod caps (Kits, Specs, GitHub) and a copper Download chip right. Kits and Specs hide under 720px.

## Do's and Don'ts

### Do:
- **Do** build every control, chip, step marker and swatch as a keycap using the rendered rasters in `site/img/caps/`; add a colorway by adding it to `render-caps.py` and re-rendering.
- **Do** size caps from `--u` and `--w`, and keep legends top-left on the face except on wide action and Enter caps.
- **Do** press caps with the 0.06u travel, 60ms down, 220ms sprung return, and drop the transitions under reduced motion.
- **Do** use real app captures only, each with its `.webp.json` provenance sidecar.
- **Do** keep Geist Mono for measurements and counts, with tabular figures.

### Don't:
- **Don't** turn anything copper except the Shift keys, Download actions and the final assembly step; copper means the gesture.
- **Don't** draw cap skirts or dishes with CSS gradients or borders; the raster owns the form.
- **Don't** use the label or theme colors as page decoration.
- **Don't** set prose or headings in Geist Mono.
- **Don't** put text lighter than Ink 3 on the case ground.
