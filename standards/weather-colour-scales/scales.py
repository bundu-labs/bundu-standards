"""Checks the weather colour scales and writes their accessibility results.

    python3 standards/weather-colour-scales/scales.py write   # update scales.json's derived fields and results.md
    python3 standards/weather-colour-scales/scales.py check   # CI: both are current

Contrast is WCAG 2.x relative luminance. Colour-blind simulation uses the
Machado, Oliveira and Fernandes (2009) matrices at full severity, applied in
linear sRGB. Separation between neighbouring bands is CIE76 delta E in CIELAB
(D65). The numbers are recorded, not hidden: a scale adopted from a published
standard keeps its colours even where neighbours are hard to tell apart, which
is why the label rule exists.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
SCALES = json.loads((HERE / "scales.json").read_text())
RESULTS = HERE / "results.md"

TEXT = {"dark": "#000000", "light": "#FFFFFF"}
SURFACES = {"light": "#FFFFFF", "dark": "#121212"}
MIN_TEXT = 7.0  # the Bundu floor for text (Mzizi), WCAG 1.4.6 AAA; above WCAG 1.4.3 AA (4.5)
MIN_SWATCH = 3.0  # WCAG 1.4.11 non-text contrast
CLOSE = 10.0  # delta E below which neighbours are hard to tell apart; scales of ours may not go under it
VISIONS = (None, "protanopia", "deuteranopia", "tritanopia")

MACHADO = {
    "protanopia": (
        (0.152286, 1.052583, -0.204868),
        (0.114503, 0.786281, 0.099216),
        (-0.003882, -0.048116, 1.051998),
    ),
    "deuteranopia": (
        (0.367322, 0.860646, -0.227968),
        (0.280085, 0.672501, 0.047413),
        (-0.011820, 0.042940, 0.968881),
    ),
    "tritanopia": (
        (1.255528, -0.076749, -0.178779),
        (-0.078411, 0.930809, 0.147602),
        (0.004733, 0.691367, 0.303900),
    ),
}


def rgb(hex_: str) -> tuple[float, float, float]:
    h = hex_.lstrip("#")
    return tuple(int(h[i : i + 2], 16) / 255 for i in (0, 2, 4))  # type: ignore[return-value]


def linear(c: float) -> float:
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def gamma(c: float) -> float:
    c = min(1.0, max(0.0, c))
    return 12.92 * c if c <= 0.0031308 else 1.055 * c ** (1 / 2.4) - 0.055


def luminance(hex_: str) -> float:
    r, g, b = (linear(c) for c in rgb(hex_))
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def contrast(a: str, b: str) -> float:
    la, lb = sorted((luminance(a), luminance(b)), reverse=True)
    return (la + 0.05) / (lb + 0.05)


def text_for(hex_: str) -> tuple[str, float]:
    best = max(TEXT, key=lambda k: contrast(hex_, TEXT[k]))
    return best, contrast(hex_, TEXT[best])


def label_for(hex_: str) -> dict:
    """Where a band's label goes and in which colour.

    The label is drawn on the swatch only where black or white text on it reaches MIN_TEXT;
    otherwise it sits beside the swatch, on the page surface, in the page's own text colour.
    """
    t, c = text_for(hex_)
    return {"labelText": TEXT[t], "labelOnSwatch": c >= MIN_TEXT}


def outline_for(hex_: str) -> list[str]:
    """The surfaces on which the swatch needs a 1px outline to reach MIN_SWATCH."""
    return [k for k, v in SURFACES.items() if contrast(hex_, v) < MIN_SWATCH]


def derived(band: dict) -> dict:
    """The accessibility fields scales.py writes into each band of scales.json."""
    out = {**label_for(band["colour"]), "outline": outline_for(band["colour"])}
    if "assist" in band:
        a = label_for(band["assist"])
        out |= {"assistLabelText": a["labelText"], "assistLabelOnSwatch": a["labelOnSwatch"], "assistOutline": outline_for(band["assist"])}
    return out


def with_derived(doc: dict) -> dict:
    """scales.json with every band's derived accessibility fields brought up to date."""
    doc = json.loads(json.dumps(doc))
    for s in doc["scales"]:
        for b in s["bands"]:
            b.update(derived(b))
    return doc


def dump(doc: dict) -> str:
    """scales.json as the repository's formatter writes it (2 spaces, 80 columns): an array of
    plain values that fits on one line is printed on one line."""
    text = json.dumps(doc, indent=2, ensure_ascii=False)

    def collapse(m: re.Match) -> str:
        items = [i.strip() for i in m.group(2).split(",\n")]
        one = m.group(1) + "[" + ", ".join(items) + "]"
        return one if len(one) <= 80 else m.group(0)

    text = re.sub(r"^( *\"[^\"\n]+\": )\[\n((?: *(?:\"[^\"\n]*\"|[-0-9.]+|true|false|null),?\n)+) *\]", collapse, text, flags=re.M)
    return text + "\n"


def simulate(hex_: str, kind: str) -> tuple[float, float, float]:
    lin = [linear(c) for c in rgb(hex_)]
    m = MACHADO[kind]
    return tuple(gamma(sum(m[i][j] * lin[j] for j in range(3))) for i in range(3))  # type: ignore[return-value]


def lab(c: tuple[float, float, float]) -> tuple[float, float, float]:
    r, g, b = (linear(v) for v in c)
    x = (0.4124 * r + 0.3576 * g + 0.1805 * b) / 0.95047
    y = 0.2126 * r + 0.7152 * g + 0.0722 * b
    z = (0.0193 * r + 0.1192 * g + 0.9505 * b) / 1.08883

    def f(t: float) -> float:
        return t ** (1 / 3) if t > 216 / 24389 else (24389 / 27 * t + 16) / 116

    fx, fy, fz = f(x), f(y), f(z)
    return 116 * fy - 16, 500 * (fx - fy), 200 * (fy - fz)


def delta_e(a: tuple[float, float, float], b: tuple[float, float, float]) -> float:
    la, lb = lab(a), lab(b)
    return sum((p - q) ** 2 for p, q in zip(la, lb)) ** 0.5


def neighbour_separation(colours: list[str], kind: str | None) -> float:
    pts = [rgb(c) if kind is None else simulate(c, kind) for c in colours]
    return min(delta_e(pts[i], pts[i + 1]) for i in range(len(pts) - 1))


def fmt(v: float | None) -> str:
    return "" if v is None else f"{v:g}".replace("-", "\u2212")


def span(b: dict, unit: str) -> str:
    if b["min"] is None:
        return f"below {fmt(b['max'])} {unit}"
    if b["max"] is None:
        return f"{fmt(b['min'])} {unit} and above"
    return f"{fmt(b['min'])} to {fmt(b['max'])} {unit}"


def table(head: list[str], rows: list[list[str]]) -> list[str]:
    """A Markdown table padded the way the repository's formatter writes it."""
    widths = [max(len(r[i]) for r in [head, *rows]) for i in range(len(head))]
    widths = [max(w, 3) for w in widths]

    def line(cells: list[str]) -> str:
        return "| " + " | ".join(c.ljust(w) for c, w in zip(cells, widths)) + " |"

    return [line(head), "| " + " | ".join("-" * w for w in widths) + " |", *(line(r) for r in rows)]


def separation(colours: list[str]) -> list[str]:
    rows = []
    for kind in VISIONS:
        d = neighbour_separation(colours, kind)
        rows.append([kind or "typical", f"{d:.1f}", "hard to tell apart" if d < CLOSE else ""])
    return table(["Vision", "Smallest neighbour delta E", "Note"], rows)


def render() -> str:
    out = [
        "# Weather colour scales: accessibility results",
        "",
        "Generated by `python3 standards/weather-colour-scales/scales.py write` from `scales.json`.",
        "Do not edit by hand; CI fails if this file is stale.",
        "",
        f"Text contrast target {MIN_TEXT:g}:1, the Bundu floor (WCAG 1.4.6 AAA). A label goes on the swatch",
        "only where black or white text on it reaches that; otherwise it sits beside the swatch on the",
        "page, in the page's own text colour (`labelOnSwatch` in scales.json).",
        f"Swatch-to-surface target {MIN_SWATCH:g}:1 (WCAG 1.4.11); a swatch below it on a surface takes a",
        "1px outline in the legend text colour (`outline` in scales.json).",
        f"Neighbour separation is the smallest CIE76 delta E between adjacent bands. A scale of ours must",
        f"reach {CLOSE:g} under typical vision and simulated protanopia, deuteranopia and tritanopia; an",
        "adopted scale keeps its colours and is flagged below it, where the printed label carries the meaning.",
        "",
    ]
    for s in SCALES["scales"]:
        colours = [b["colour"] for b in s["bands"]]
        out += [f"## {s['name']} (`{s['id']}`, {s['origin']})", ""]
        rows = []
        for b in s["bands"]:
            t, c = text_for(b["colour"])
            on = {k: contrast(b["colour"], v) for k, v in SURFACES.items()}
            mark = {k: f"{v:.2f}" + ("" if v >= MIN_SWATCH else " border") for k, v in on.items()}
            place = "on swatch" if c >= MIN_TEXT else "beside"
            rows.append([b["label"], span(b, s["unit"]), f"`{b['colour']}`", t, f"{c:.2f}", place, mark["light"], mark["dark"]])
        out += table(["Band", "Range", "Colour", "Text", "Text contrast", "Label", "On light", "On dark"], rows)
        out += ["", *separation(colours)]
        if any("assist" in b for b in s["bands"]):
            out += ["", "Assist palette (`assist`):", "", *separation([b["assist"] for b in s["bands"]])]
        out.append("")
    return "\n".join(out)


def main(argv: list[str]) -> int:
    cmd = argv[1] if len(argv) > 1 else "check"
    text = render()
    current = dump(with_derived(SCALES))
    if cmd == "write":
        (HERE / "scales.json").write_text(current)
        SCALES.clear()
        SCALES.update(json.loads(current))
        text = render()
        RESULTS.write_text(text)
        print(f"wrote {RESULTS.relative_to(Path.cwd()) if RESULTS.is_relative_to(Path.cwd()) else RESULTS}")
        return 0
    if cmd == "check":
        if (HERE / "scales.json").read_text() != current:
            print("scales.json's derived accessibility fields are stale: run python3 standards/weather-colour-scales/scales.py write")
            return 1
        if not RESULTS.exists() or RESULTS.read_text() != text:
            print("results.md is stale: run python3 standards/weather-colour-scales/scales.py write")
            return 1
        print("scales.json and results.md are current")
        return 0
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))
