"""Turn the OpenBlade logo (neon on black) into transparent app assets.

Outputs into app/assets/:
  logo.png       – serpent only (header plate)
  logo-full.png  – serpent + OPENBLADE wordmark (settings / about)
  wordmark.png   – OPENBLADE text only (header, next to the window buttons)
  icon.png       – 256 px app/tray icon (serpent on dark chamfered tile)
"""

import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

src, out_dir = Path(sys.argv[1]), Path(sys.argv[2])
im = np.asarray(Image.open(src).convert("RGB")).astype(np.float32)

# black background -> alpha (keeps the glow); un-premultiply the colour
peak = im.max(axis=2)
alpha = np.clip((peak - 6) / 249, 0, 1) ** 0.85
rgb = np.where(alpha[..., None] > 0.004, im / np.maximum(alpha[..., None], 1e-3), 0)
rgba = np.dstack([np.clip(rgb, 0, 255), alpha * 255]).astype(np.uint8)
full = Image.fromarray(rgba, "RGBA")

# find the serpent vs. the wordmark by the empty row band between them (on the crisp strokes)
solid = peak > 120
rows = np.where(solid.any(axis=1))[0]
gaps = np.where(np.diff(rows) > 8)[0]
split = int((rows[gaps[-1]] + rows[gaps[-1] + 1]) / 2) if len(gaps) else rows[-1]


def crop_to(mask_rows_from, mask_rows_to, pad):
    sub = solid[mask_rows_from:mask_rows_to]
    ys, xs = np.where(sub)
    x0, x1 = xs.min() - pad, xs.max() + pad
    y0, y1 = ys.min() + mask_rows_from - pad, ys.max() + mask_rows_from + pad
    return max(0, x0), max(0, y0), min(full.width, x1), min(full.height, y1)


def square(img):
    s = max(img.size)
    canvas = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    canvas.paste(img, ((s - img.width) // 2, (s - img.height) // 2))
    return canvas


x0, y0, x1, y1 = crop_to(0, split, 30)
serpent = full.crop((x0, y0, x1, min(y1, split - 12)))  # stop above the wordmark's glow
square(serpent).resize((512, 512), Image.LANCZOS).save(out_dir / "logo.png")
full.crop(crop_to(0, full.height, 40)).save(out_dir / "logo-full.png")
wx0, wy0, wx1, wy1 = crop_to(split, full.height, 24)
full.crop((wx0, max(wy0, split + 12), wx1, wy1)).save(out_dir / "wordmark.png")  # OPENBLADE text only

# icon: dark chamfered tile with green border + serpent
S = 256
tile = Image.new("RGBA", (S, S), (0, 0, 0, 0))
d = ImageDraw.Draw(tile)
c = 36
poly = [(c, 4), (S - c, 4), (S - 4, c), (S - 4, S - c), (S - c, S - 4), (c, S - 4), (4, S - c), (4, c)]
d.polygon(poly, fill=(6, 12, 6, 255), outline=(68, 214, 44, 255), width=6)
glyph = square(serpent).resize((210, 210), Image.LANCZOS)
tile.alpha_composite(glyph, ((S - 210) // 2, (S - 210) // 2 + 4))
tile.save(out_dir / "icon.png")
print("split row", split, "serpent", serpent.size)
