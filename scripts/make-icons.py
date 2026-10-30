"""Draws Overnight's menu bar icons and the app icon source.

    python3 scripts/make-icons.py && npx tauri icon app-icon.png

Needs Pillow. Shapes are drawn 8x larger and scaled down for clean edges.
"""

import math
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw, ImageFilter

ROOT = Path(__file__).resolve().parent.parent
TRAY = ROOT / "src-tauri" / "icons" / "tray"
SS = 8  # supersampling factor


def disc(size, cx, cy, r):
    m = Image.new("L", (size, size), 0)
    ImageDraw.Draw(m).ellipse((cx - r, cy - r, cx + r, cy + r), fill=255)
    return m


def crescent(size, s, inset=0.0):
    """Moon disc minus an offset disc, on a size x size mask; s scales a 36-unit design."""
    moon = disc(size, 17 * s, 19 * s, (13 - inset) * s)
    bite = disc(size, 24 * s, 13 * s, (11 + inset) * s)
    return ImageChops.subtract(moon, bite)


def sparkle(size, cx, cy, outer, inner):
    m = Image.new("L", (size, size), 0)
    pts = []
    for i in range(8):
        r = outer if i % 2 == 0 else inner
        a = math.pi / 4 * i - math.pi / 2
        pts.append((cx + r * math.cos(a), cy + r * math.sin(a)))
    ImageDraw.Draw(m).polygon(pts, fill=255)
    return m


def tray_icon(mask, px=36):
    """Black template image: macOS only uses the alpha channel."""
    alpha = mask.resize((px, px), Image.LANCZOS)
    img = Image.new("RGBA", (px, px), (0, 0, 0, 0))
    img.putalpha(alpha)
    return img


def make_tray():
    TRAY.mkdir(parents=True, exist_ok=True)
    size, s = 36 * SS, SS
    full = crescent(size, s)
    outline = ImageChops.subtract(full, crescent(size, s, inset=2.2))
    star = sparkle(size, 28.5 * s, 7.5 * s, 5 * s, 1.5 * s)
    tray_icon(outline).save(TRAY / "off.png")
    tray_icon(full).save(TRAY / "standby.png")
    tray_icon(ImageChops.lighter(full, star)).save(TRAY / "on.png")


def make_app_icon():
    px = 1024
    size = px * 2
    k = size / 1024
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))

    # Night-sky squircle on Apple's 824px icon grid.
    top, bottom = (27, 30, 66), (58, 49, 112)
    grad = Image.new("RGBA", (size, size))
    gd = ImageDraw.Draw(grad)
    for y in range(size):
        t = y / size
        gd.line([(0, y), (size, y)], fill=tuple(int(a + (b - a) * t) for a, b in zip(top, bottom)) + (255,))
    shape = Image.new("L", (size, size), 0)
    inset = 100 * k
    ImageDraw.Draw(shape).rounded_rectangle((inset, inset, size - inset, size - inset), radius=185 * k, fill=255)
    canvas.paste(grad, (0, 0), shape)

    # Moon, with a soft glow behind it.
    s = 20 * k  # the 36-unit crescent design, scaled to ~720px
    ox, oy = 152 * k, 150 * k
    moon = Image.new("L", (size, size), 0)
    moon.paste(crescent(int(36 * s), s), (int(ox), int(oy)))
    glow = moon.filter(ImageFilter.GaussianBlur(40 * k))
    canvas.paste(Image.new("RGBA", (size, size), (255, 214, 120, 90)), (0, 0), ImageChops.multiply(glow, shape))
    canvas.paste(Image.new("RGBA", (size, size), (247, 214, 128, 255)), (0, 0), moon)

    # A few stars.
    stars = Image.new("L", (size, size), 0)
    for x, y, r in [(700, 300, 26), (790, 470, 14), (610, 210, 10), (330, 760, 9), (760, 690, 12)]:
        stars = ImageChops.lighter(stars, sparkle(size, x * k, y * k, r * 2.4 * k, r * 0.7 * k))
    canvas.paste(Image.new("RGBA", (size, size), (255, 255, 255, 235)), (0, 0), ImageChops.multiply(stars, shape))

    canvas.resize((px, px), Image.LANCZOS).save(ROOT / "app-icon.png")


if __name__ == "__main__":
    make_tray()
    make_app_icon()
    print("wrote", TRAY, "and", ROOT / "app-icon.png")
