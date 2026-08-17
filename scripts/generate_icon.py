#!/usr/bin/env python3
"""Generate flag-free China Travel Companion brand icon assets.

The mark uses a destination pin and journey route in a warm red and amber
palette. It writes the canonical 1024px source PNG, a matching favicon, and
Android launcher density images used by scripts/install.sh.
"""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
CANVAS = 1024
DENSITIES = {
    "mdpi": 48,
    "hdpi": 72,
    "xhdpi": 96,
    "xxhdpi": 144,
    "xxxhdpi": 192,
}

CRIMSON_TOP = (229, 49, 27)
CRIMSON_BOTTOM = (168, 17, 28)
CRIMSON_DARK = (121, 14, 25)
AMBER = (255, 222, 0)
AMBER_HIGHLIGHT = (255, 239, 112)
CREAM = (255, 249, 218)


def cubic_bezier(
    start: tuple[float, float],
    control_one: tuple[float, float],
    control_two: tuple[float, float],
    end: tuple[float, float],
    steps: int = 128,
) -> list[tuple[int, int]]:
    points: list[tuple[int, int]] = []
    for index in range(steps + 1):
        t = index / steps
        inverse = 1 - t
        x = (
            inverse**3 * start[0]
            + 3 * inverse**2 * t * control_one[0]
            + 3 * inverse * t**2 * control_two[0]
            + t**3 * end[0]
        )
        y = (
            inverse**3 * start[1]
            + 3 * inverse**2 * t * control_one[1]
            + 3 * inverse * t**2 * control_two[1]
            + t**3 * end[1]
        )
        points.append((round(x), round(y)))
    return points


def dashed_line(
    draw: ImageDraw.ImageDraw,
    points: list[tuple[int, int]],
    fill: tuple[int, int, int, int],
    width: int,
    dash_length: int,
    gap_length: int,
) -> None:
    """Draw a rounded-feeling dashed curve using sampled curve points."""
    step = dash_length + gap_length
    for start in range(0, len(points) - 1, step):
        segment = points[start : min(start + dash_length, len(points))]
        if len(segment) > 1:
            draw.line(segment, fill=fill, width=width, joint="curve")


def draw_pin(
    image: Image.Image,
    center: tuple[int, int],
    radius: int,
) -> None:
    """Draw a high-contrast location pin with a subtle dimensional shadow."""
    x, y = center
    layer = Image.new("RGBA", image.size, (0, 0, 0, 0))
    draw = ImageDraw.Draw(layer)

    shadow_center = (x + 10, y + 15)
    shadow_box = (
        shadow_center[0] - radius,
        shadow_center[1] - radius,
        shadow_center[0] + radius,
        shadow_center[1] + radius,
    )
    shadow_tip = (shadow_center[0], shadow_center[1] + radius + 118)
    draw.polygon(
        [
            (shadow_center[0] - radius + 10, shadow_center[1] + 14),
            (shadow_tip[0], shadow_tip[1]),
            (shadow_center[0] + radius - 10, shadow_center[1] + 14),
        ],
        fill=(89, 10, 19, 110),
    )
    draw.ellipse(shadow_box, fill=(89, 10, 19, 110))

    pin_box = (x - radius, y - radius, x + radius, y + radius)
    pin_tip = (x, y + radius + 118)
    draw.polygon(
        [(x - radius + 10, y + 14), pin_tip, (x + radius - 10, y + 14)],
        fill=AMBER,
    )
    draw.ellipse(pin_box, fill=AMBER)
    draw.ellipse(
        (x - radius + 9, y - radius + 9, x + radius - 9, y + radius - 9),
        outline=AMBER_HIGHLIGHT,
        width=8,
    )
    draw.ellipse((x - 55, y - 55, x + 55, y + 55), fill=CRIMSON_DARK)
    draw.ellipse((x - 35, y - 35, x + 35, y + 35), fill=CREAM)

    image.alpha_composite(layer)


def build_icon() -> Image.Image:
    """Create a launcher-safe rounded-square icon at source resolution."""
    image = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    pixels = image.load()
    assert pixels is not None

    # A warm diagonal crimson field keeps the original palette while the
    # star-free foreground remains clearly a travel mark at launcher sizes.
    for y in range(CANVAS):
        for x in range(CANVAS):
            blend = min(1.0, max(0.0, 0.62 * x / (CANVAS - 1) + 0.38 * y / (CANVAS - 1)))
            pixels[x, y] = tuple(
                round(top + (bottom - top) * blend)
                for top, bottom in zip(CRIMSON_TOP, CRIMSON_BOTTOM, strict=True)
            ) + (255,)

    mask = Image.new("L", (CANVAS, CANVAS), 0)
    mask_draw = ImageDraw.Draw(mask)
    mask_draw.rounded_rectangle((42, 42, 982, 982), radius=258, fill=255)
    image.putalpha(mask)

    overlay = Image.new("RGBA", image.size, (0, 0, 0, 0))
    draw = ImageDraw.Draw(overlay)
    draw.rounded_rectangle(
        (48, 48, 976, 976),
        radius=252,
        outline=(255, 222, 0, 58),
        width=6,
    )

    # The route-and-pin composition is the only foreground mark. Its bounds
    # sit around the canvas midpoint instead of occupying the lower edge.
    route = cubic_bezier((168, 615), (300, 572), (407, 758), (585, 623))
    draw.line(route, fill=(121, 14, 25, 145), width=38, joint="curve")
    dashed_line(draw, route, (255, 238, 100, 245), 20, 13, 8)
    draw.ellipse((142, 589, 194, 641), fill=AMBER)
    draw.ellipse((155, 602, 181, 628), fill=CRIMSON_DARK)

    image.alpha_composite(overlay)
    draw_pin(image, (710, 440), 124)
    return image


def write_assets(icon: Image.Image) -> None:
    icon_path = ROOT / "assets" / "icon.png"
    icon_path.parent.mkdir(parents=True, exist_ok=True)
    icon.save(icon_path, format="PNG", optimize=True)

    icon.save(
        ROOT / "assets" / "favicon.ico",
        format="ICO",
        sizes=[(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)],
    )

    for density, size in DENSITIES.items():
        output = ROOT / "assets" / "android-launcher" / f"mipmap-{density}" / "ic_launcher.png"
        output.parent.mkdir(parents=True, exist_ok=True)
        icon.resize((size, size), Image.Resampling.LANCZOS).save(output, format="PNG", optimize=True)


def main() -> None:
    write_assets(build_icon())


if __name__ == "__main__":
    main()
