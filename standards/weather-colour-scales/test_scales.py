"""Checks for scales.json: run with `python3 -m unittest discover standards/weather-colour-scales`."""

import json
import re
import unittest
from pathlib import Path

DOC = json.loads((Path(__file__).parent / "scales.json").read_text())
HEX = re.compile(r"^#[0-9A-F]{6}$")


def luminance(h):
    c = [int(h[i : i + 2], 16) / 255 for i in (1, 3, 5)]
    c = [x / 12.92 if x <= 0.03928 else ((x + 0.055) / 1.055) ** 2.4 for x in c]
    return 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]


def contrast(a, b):
    hi, lo = sorted((luminance(a), luminance(b)), reverse=True)
    return (hi + 0.05) / (lo + 0.05)


class Scales(unittest.TestCase):
    def test_every_scale_cites_a_source_read_on_a_date(self):
        for key, scale in DOC["scales"].items():
            self.assertTrue(scale["source"]["url"].startswith("https://"), key)
            self.assertRegex(scale["source"]["read"], r"^\d{4}-\d{2}-\d{2}$", key)
            self.assertTrue(scale["register"], key)

    def test_bands_are_contiguous_and_only_the_last_is_open(self):
        step = {"aqi": 1, "uv_index": 1, "beaufort": 0.1}
        for key, scale in DOC["scales"].items():
            bands = scale["bands"]
            for prev, cur in zip(bands, bands[1:]):
                self.assertIsNotNone(prev["max"], key)
                self.assertAlmostEqual(cur["min"] - prev["max"], step[key], places=6, msg=key)
            self.assertIsNone(bands[-1]["max"], key)

    def test_colours_are_hex_or_explicitly_open(self):
        for key, scale in DOC["scales"].items():
            for band in scale["bands"]:
                for c in (band.get("srgb"), band.get("standard", {}).get("srgb"), band.get("colorVisionAssist", {}).get("srgb")):
                    if c is not None:
                        self.assertRegex(c, HEX, key)

    def test_aqi_matches_epa_table_2(self):
        got = [b["standard"]["srgb"] for b in DOC["scales"]["aqi"]["bands"]]
        self.assertEqual(got, ["#00E400", "#FFFF00", "#FF7E00", "#FF0000", "#8F3F97", "#7E0023"])

    def test_aqi_labels_meet_wcag_aa(self):
        for band in DOC["scales"]["aqi"]["bands"]:
            for variant in ("standard", "colorVisionAssist"):
                v = band[variant]
                self.assertGreaterEqual(contrast(v["srgb"], v["label"]["text"]), 4.5, (band["key"], variant))

    def test_uv_index_keeps_who_pantone_references(self):
        got = [b["pantone"] for b in DOC["scales"]["uv_index"]["bands"]]
        self.assertEqual(got, ["PMS 375", "PMS 102", "PMS 151", "PMS 032", "PMS 265"])

    def test_beaufort_matches_wmo_table_5_1(self):
        bands = DOC["scales"]["beaufort"]["bands"]
        self.assertEqual([b["force"] for b in bands], list(range(13)))
        self.assertEqual(bands[12]["min"], 32.7)
        self.assertEqual(DOC["scales"]["beaufort"]["unit"], "m/s")


if __name__ == "__main__":
    unittest.main()
