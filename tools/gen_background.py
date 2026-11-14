"""Generate the neon-green smoke background (app/assets/background.jpg)."""

import sys

import numpy as np
from PIL import Image, ImageFilter

W, H = 1920, 1080
rng = np.random.default_rng(int(sys.argv[2]) if len(sys.argv) > 2 else 7)


def value_noise(scale):
    gw, gh = max(2, int(W / scale)) + 2, max(2, int(H / scale)) + 2
    g = (rng.random((gh, gw)) * 255).astype(np.uint8)
    img = Image.fromarray(g).resize((int(gw * scale), int(gh * scale)), Image.BICUBIC)
    return np.asarray(img).astype(np.float32)[:H, :W] / 255


def fbm(base, octaves=5):
    out, amp, tot = np.zeros((H, W), np.float32), 1.0, 0.0
    s = base
    for _ in range(octaves):
        out += value_noise(s) * amp
        tot += amp
        amp *= 0.5
        s /= 2
    return out / tot


def sample(arr, x, y):
    x = np.clip(x, 0, W - 1.001)
    y = np.clip(y, 0, H - 1.001)
    x0, y0 = x.astype(int), y.astype(int)
    fx, fy = x - x0, y - y0
    a = arr[y0, x0] * (1 - fx) + arr[y0, x0 + 1] * fx
    b = arr[y0 + 1, x0] * (1 - fx) + arr[y0 + 1, x0 + 1] * fx
    return a * (1 - fy) + b * fy


yy, xx = np.mgrid[0:H, 0:W].astype(np.float32)

# domain-warped ridged noise -> thin smoke filaments
wx, wy = fbm(420), fbm(420)
n = fbm(260)
warped = sample(n, xx + (wx - 0.5) * 900, yy + (wy - 0.5) * 900)
ridge = 1 - np.abs(warped - 0.5) * 2  # peaks along the 0.5 iso-line
filaments = ridge ** 60
soft = ridge ** 10

# diagonal flow streaks (bottom-left -> top-right), as in the mockup
d = (xx / W) * 0.55 - (yy / H) * 0.85
streak_n = sample(fbm(300), xx + (wx - 0.5) * 500, yy + (wy - 0.5) * 500)
streak = np.exp(-((np.sin(d * 9 + streak_n * 6) - 0.97) ** 2) / 0.0006) * 0.35

# where the light lives (normalised coords)
def blob(cx, cy, rx, ry, w=1.0):
    return w * np.exp(-(((xx / W - cx) / rx) ** 2 + ((yy / H - cy) / ry) ** 2))

mask = (
    blob(0.05, 0.95, 0.30, 0.35, 1.0)
    + blob(0.22, 0.42, 0.25, 0.30, 0.7)
    + blob(0.60, 0.62, 0.22, 0.30, 0.8)
    + blob(0.78, 0.18, 0.25, 0.25, 0.6)
    + blob(0.98, 0.70, 0.12, 0.40, 0.7)
    + blob(0.45, 0.05, 0.25, 0.12, 0.4)
)
mask = np.clip(mask, 0, 1.3)

light = (filaments * 0.9 + soft * 0.05 + streak) * mask * 0.75
light = np.asarray(Image.fromarray(np.clip(light * 255, 0, 255).astype(np.uint8)).filter(
    ImageFilter.GaussianBlur(0.8))).astype(np.float32) / 255
glow = np.asarray(Image.fromarray(np.clip(light * 255, 0, 255).astype(np.uint8)).filter(
    ImageFilter.GaussianBlur(18))).astype(np.float32) / 255
v = np.clip(light * 0.8 + glow * 0.9, 0, 1.2)

# colour ramp: black -> deep green -> neon -> pale
base = np.array([3, 7, 4], np.float32)
deep = np.array([10, 60, 14], np.float32)
neon = np.array([68, 214, 44], np.float32)
pale = np.array([200, 255, 170], np.float32)
t = v[..., None]
col = base + deep * np.clip(t * 3, 0, 1) + (neon - deep) * np.clip(t * 1.6 - 0.25, 0, 1) \
    + (pale - neon) * np.clip(t * 1.4 - 1.1, 0, 1)

# vignette
vig = 1 - 0.55 * (((xx / W - 0.5) * 1.6) ** 2 + ((yy / H - 0.5) * 1.6) ** 2)
col *= np.clip(vig, 0.35, 1)[..., None]
col += rng.normal(0, 1.2, col.shape)  # dither against banding

Image.fromarray(np.clip(col, 0, 255).astype(np.uint8)).save(sys.argv[1], quality=92)
print("saved", sys.argv[1])
