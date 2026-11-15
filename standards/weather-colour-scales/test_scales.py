"""The weather colour scales are well formed, readable, and their results are current.

    python3 -m unittest standards/weather-colour-scales/test_scales.py
"""

import re
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import scales  # noqa: E402

HEX = re.compile(r"^#[0-9A-F]{6}$")
UNITS = {"{index}", "Cel", "km/h", "mm/h", "{okta}"}


class Scales(unittest.TestCase):
    def test_every_scale_names_its_source_and_origin(self):
        for s in scales.SCALES["scales"]:
            self.assertIn(s["origin"], {"adopted", "ours"}, s["id"])
            for key in ("title", "steward", "url", "checked", "claim"):
                self.assertTrue(s["source"].get(key), f"{s['id']} source.{key}")
            self.assertIn(s["unit"], UNITS, s["id"])

    def test_bands_are_contiguous_and_in_order(self):
        for s in scales.SCALES["scales"]:
            bands = s["bands"]
            self.assertGreaterEqual(len(bands), 3, s["id"])
            for a, b in zip(bands, bands[1:]):
                self.assertEqual(a["max"], b["min"], f"{s['id']}: gap or overlap at {a['label']}")
            for b in bands:
                if b["min"] is not None and b["max"] is not None:
                    self.assertLess(b["min"], b["max"], f"{s['id']} {b['label']}")
            self.assertIsNone(bands[-1]["max"], f"{s['id']}: the top band is open-ended")

    def test_every_band_has_a_label_and_a_valid_colour(self):
        for s in scales.SCALES["scales"]:
            for b in s["bands"]:
                self.assertTrue(b["label"].strip(), s["id"])
                self.assertRegex(b["colour"], HEX)
                if "assist" in b:
                    self.assertRegex(b["assist"], HEX)

    def test_label_text_on_every_swatch_meets_aa(self):
        for s in scales.SCALES["scales"]:
            for b in s["bands"]:
                _, c = scales.text_for(b["colour"])
                self.assertGreaterEqual(c, scales.MIN_TEXT, f"{s['id']} {b['label']} {b['colour']}")

    def test_no_brand_mineral_is_used_as_a_data_colour(self):
        # The seven Mzizi minerals, every variant (mzizi_get_tokens family=minerals, 2026-11-14).
        # Data scales must not borrow brand colours.
        minerals = {
            "#0047AB", "#00B0FF", "#B388FF", "#4B0082", "#64FFDA", "#004D40",
            "#FFD740", "#5D4037", "#A0522D", "#E1B07E", "#283593", "#3D5AFE",
            "#BF5A36", "#FF8A65",
        }
        for s in scales.SCALES["scales"]:
            for b in s["bands"]:
                for c in (b["colour"], b.get("assist", "")):
                    self.assertNotIn(c.upper(), minerals, s["id"])

    def test_results_file_is_current(self):
        self.assertEqual(scales.RESULTS.read_text(), scales.render(), "run scales.py write")


class Maths(unittest.TestCase):
    def test_contrast_matches_wcag_reference_values(self):
        self.assertAlmostEqual(scales.contrast("#000000", "#FFFFFF"), 21.0, places=2)
        self.assertAlmostEqual(scales.contrast("#777777", "#FFFFFF"), 4.48, places=2)

    def test_identical_colours_have_no_separation(self):
        self.assertAlmostEqual(scales.delta_e(scales.rgb("#336699"), scales.rgb("#336699")), 0.0)


if __name__ == "__main__":
    unittest.main()
