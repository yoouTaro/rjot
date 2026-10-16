#!/usr/bin/env python3
"""Render the website's keycap and plate rasters into site/img/caps/.

Each keycap is drawn once per colour as a 9-slice image (200x200 px, made
for a 100 px cap at 2x): a skirt with lit and shaded walls, a dished top face
(a cylindrical dip, lighter at the back edge and the front lip) and a fine
PBT grain. CSS stretches it to any width with border-image. The plate is an
anodized charcoal tile with a fine bead-blast grain.

Usage: python3 -I scripts/render-caps.py   (needs numpy and Pillow)
"""

from pathlib import Path

import numpy as np
from PIL import Image

OUT = Path(__file__).resolve().parent.parent / "site" / "img" / "caps"
SIZE = 200          # output pixels
SS = 3              # supersampling
RNG = np.random.default_rng(7)

# Top face inset inside the skirt (output px): left, top, right, bottom.
FACE = (18, 10, 18, 34)
OUTER_R = 26
FACE_R = 18


def hex_rgb(h):
    h = h.lstrip("#")
    return np.array([int(h[i:i + 2], 16) for i in (0, 2, 4)], dtype=np.float64) / 255.0


def rounded_mask(x, y, x0, y0, x1, y1, r):
    """Inside test for a rounded rectangle, on a pixel grid."""
    cx = np.clip(x, x0 + r, x1 - r)
    cy = np.clip(y, y0 + r, y1 - r)
    return (x - cx) ** 2 + (y - cy) ** 2 <= r * r


def grain(shape, amount, blur=1):
    n = RNG.normal(0.0, 1.0, shape)
    if blur:
        # A light box blur so the grain reads as plastic, not pixel noise.
        k = 2 * blur + 1
        pad = np.pad(n, blur, mode="wrap")
        acc = np.zeros(shape)
        for dy in range(k):
            for dx in range(k):
                acc += pad[dy:dy + shape[0], dx:dx + shape[1]]
        n = acc / (k * k)
        n /= n.std() + 1e-9
    return n * amount


def render_cap(top_hex, skirt_hex, name):
    n = SIZE * SS
    y, x = np.mgrid[0:n, 0:n].astype(np.float64) + 0.5
    s = SS
    outer = rounded_mask(x, y, 0, 0, n, n, OUTER_R * s)
    fx0, fy0, fx1, fy1 = FACE[0] * s, FACE[1] * s, n - FACE[2] * s, n - FACE[3] * s
    face = rounded_mask(x, y, fx0, fy0, fx1, fy1, FACE_R * s)

    top = hex_rgb(top_hex)
    skirt = hex_rgb(skirt_hex)
    img = np.zeros((n, n, 3))

    # Skirt walls: which wall a pixel belongs to follows its nearest outer
    # edge, weighted by that wall's depth (the front wall is the deepest).
    d_left = x / FACE[0]
    d_right = (n - x) / FACE[2]
    d_top = y / FACE[1]
    d_front = (n - y) / FACE[3]
    walls = np.stack([d_left, d_right, d_top, d_front])
    wall = walls.argmin(axis=0)
    shade = np.choose(wall, [0.86, 0.80, 1.06, 1.0])
    # The front wall: lit at the top where it meets the face, darker at the foot.
    tf = np.clip((y - fy1) / (n - fy1), 0, 1)
    front = 1.0 - 0.16 * tf
    shade = np.where(wall == 3, front, shade)
    img[:] = skirt * shade[..., None]

    # Top face: a cylindrical dish across its depth, a lighter rim, grain.
    v = np.clip((y - fy0) / (fy1 - fy0), 0, 1)
    dish = 1.035 - 0.075 * np.sin(np.pi * v) ** 1.4 + 0.03 * (v > 0.9) * (v - 0.9) / 0.1
    edge = np.minimum.reduce([x - fx0, fx1 - x, y - fy0, fy1 - y]) / s
    rim = 1.0 + 0.06 * np.exp(-edge / 1.4)
    face_rgb = top * (dish * rim)[..., None]
    img = np.where(face[..., None], face_rgb, img)

    # PBT grain on everything, finer on the face. CSS repeats the middle
    # 120 px column across wide keys, so the grain repeats with that period
    # (aligned to the slice) and the tiles meet without a seam.
    period = (SIZE - 80) * SS
    tile = grain((n, period), 0.012 if top.mean() > 0.5 else 0.018, blur=2)
    cols = (np.arange(n) - 40 * SS) % period
    g = tile[:, cols]
    img = img * (1.0 + g[..., None])
    img = np.clip(img, 0, 1)

    alpha = outer.astype(np.float64)
    rgba = np.dstack([img, alpha])
    big = Image.fromarray((rgba * 255).round().astype(np.uint8), "RGBA")
    small = big.resize((SIZE, SIZE), Image.LANCZOS)
    path = OUT / f"{name}.png"
    small.save(path, optimize=True)
    return path


def render_plate(name="plate", size=256):
    base = hex_rgb("#26292D")
    g = grain((size, size), 0.035, blur=1)
    shade = np.clip(base[None, None, :] * (1.0 + g[..., None]), 0, 1)
    img = Image.fromarray((shade * 255).round().astype(np.uint8), "RGB")
    path = OUT / f"{name}.png"
    img.save(path, optimize=True)
    return path


CAPS = {
    "alpha": ("#EFEFEB", "#CBCDC9"),
    "copper": ("#E3874D", "#B65F2B"),
    "slate": ("#58606A", "#3D444C"),
    "red": ("#F06A62", "#BA4A44"),
    "orange": ("#F0883E", "#BB662C"),
    "yellow": ("#E3B341", "#AE862C"),
    "green": ("#57AB5A", "#418043"),
    "blue": ("#539BF5", "#3E74B8"),
    "purple": ("#B083F0", "#8462B4"),
}

if __name__ == "__main__":
    OUT.mkdir(parents=True, exist_ok=True)
    for name, (top, skirt) in CAPS.items():
        print(render_cap(top, skirt, name))
    print(render_plate())
