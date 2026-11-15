# Weather data colour scales

**Version 1.0.0, adopted.** Scope approved by the owner on 2026-11-14 (#39); every claim checked at
source on 2026-11-15. Licence: CC BY 4.0 for the text, CC0 for `scales.json`. The adopted scales
remain their stewards' work and are cited, not relicensed. The ramps of our own scales are
ColorBrewer's, under Apache-2.0: this product includes color specifications and designs developed by
Cynthia Brewer (<http://colorbrewer.org/>).

## Why this exists

People read weather maps by convention: cold is blue and hot is red, green air is clean air, violet UV
means cover up. When an app paints those readings in its brand colours, or invents a new scale, it
misleads the reader, and every app that does so makes the ecosystem less consistent. The rule in
`AGENTS.md` applies: where a published scale exists it is adopted unchanged; where none exists we write
one here, so that every app built on the platform shows the same reading in the same colour.

## What is in it

`scales.json` holds every scale as data, so apps and Workers import it rather than copy it. Each band
is `[min, max)` in the scale's UCUM unit, with a label and an sRGB colour.

| Scale                 | Unit      | Origin                                                                                                 |
| --------------------- | --------- | ------------------------------------------------------------------------------------------------------ |
| Air quality (US AQI)  | `{index}` | **Adopted.** US EPA, EPA-403/B-26-003, May 2026 (register `us_epa_aqi`)                                |
| UV index              | `{index}` | **Adopted.** WHO, WMO, UNEP, ICNIRP, 2002 (register `who_global_solar_uvi`)                            |
| Air temperature       | `Cel`     | **Ours.** Breakpoints ours; ColorBrewer RdYlBu ramp                                                    |
| Wind speed (Beaufort) | `km/h`    | **Ours.** Beaufort breakpoints, WMO-No. 8 Table 5.1 (register `wmo_beaufort`); ColorBrewer YlGnBu ramp |
| Rainfall rate         | `mm/h`    | **Ours.** WMO rainfall intensity classes; ColorBrewer Blues ramp                                       |
| Cloud cover           | `{okta}`  | **Ours.** Okta classes as in METAR (FEW, SCT, BKN, OVC); ColorBrewer Greys                             |

Each scale records its source, steward, edition, the date it was checked and whether each claim was
**measured** at source or is **inferred**. Since 1.0.0 every claim is measured; the sources are
listed below.

## Rules

1. The band label or its value range is always printed beside the colour. Colour alone never carries
   the meaning, which is what makes the scales usable for colour-blind readers and in print.
2. An adopted scale is used unchanged: its breakpoints, labels and colours.
3. Brand colours (the Mzizi minerals) are never used for data. The tests enforce this.
4. Values are converted to the scale's UCUM unit at the edge, before a band is chosen.
5. A swatch with less than 3:1 contrast against the surface it sits on takes a 1px outline in the
   legend text colour (WCAG 1.4.11). Each band's `outline` lists the surfaces where it needs one.
6. A label goes on a swatch only where black or white text on it reaches 7:1, the Bundu floor (WCAG
   1.4.6 AAA). Otherwise it sits beside the swatch, on the page, in the page's own text colour. Each
   band's `labelText` and `labelOnSwatch` say which (and `assistLabelText`, `assistLabelOnSwatch` for
   the AQI's assist palette).
7. A scale of ours keeps every pair of neighbouring bands at least delta E 10 apart under typical vision
   and simulated protanopia, deuteranopia and tritanopia. The tests enforce this.
8. A legend is readable without sight or colour: it is a list or a table whose every band has its label
   and range as text, never only a colour or an image, so screen readers and keyboard users get the same
   information.

## Accessibility, measured

`scales.py write` fills in each band's `labelText`, `labelOnSwatch` and `outline` in `scales.json` and
generates `results.md`; CI fails if either is stale. For every band `results.md` gives the label colour
(black or white), its contrast and whether it goes on the swatch or beside it, and the swatch's contrast
on light and dark surfaces. For every scale it gives the smallest colour difference (CIE76 delta E)
between neighbouring bands, for typical vision and simulated protanopia, deuteranopia and tritanopia
(Machado, Oliveira and Fernandes, 2009).

Every scale of ours clears delta E 10 under all four. The wind ramp now takes seven of ColorBrewer
YlGnBu's nine classes; the 7-class scheme fell to 9.0 under protanopia. The adopted UV colours, in
WHO's own RGB, clear it too (18.9 at their closest, under deuteranopia). The EPA publishes its own
colour-blind palette for the AQI, adopted here as `assist`.

## Checked at source for 1.0

All checked on 2026-11-15.

- **UV colours.** The WHO guide's Table 4 gives Pantone codes (375, 102, 151, 032, 265). WHO's notes on
  its UV index clipart (2002) give its own 8-bit RGB for each category and call the Pantone codes an
  approximate guide only, so 1.0 takes WHO's RGB: `#289500`, `#F7E400`, `#F85900`, `#D8001D`,
  `#6B49C8`. These replace the 0.1.0 values, which were our reading of the Pantone codes.
- **ColorBrewer licence.** Apache-2.0, from `LICENCE.txt` in github.com/axismaps/colorbrewer, with the
  attribution above. Every ramp colour was checked against ColorBrewer's own `colorbrewer.js`:
  temperature is RdYlBu 11 reversed, wind is YlGnBu 9 (classes 1, 3 to 7, 9), rainfall is Blues 5
  (classes 2 to 5), cloud is Greys 6 (classes 1 to 5).
- **Rainfall limits.** WMO-No. 8, Part I, Chapter 14, Annex: rain is light below 2.5 mm/h, moderate from
  2.5 to below 10.0, heavy from 10.0 to below 50.0 and violent from 50.0. Checked in the 2008 edition.
  The 0.1 mm/h floor is ours.
- **Okta classes.** WMO-No. 306, Vol. I.1, FM 15 METAR, regulation 15.9.1.1: few 1 to 2 oktas,
  scattered 3 to 4, broken 5 to 7, overcast 8. The clear class (0 oktas) is ours. Okta 9, sky obscured,
  is not an amount and takes no band.
- **Beaufort bounds.** WMO-No. 8, Vol. I, Table 5.1, checked on 2026-11-14.

## Not yet covered

- **Radar reflectivity (dBZ).** The NWS progression is the convention, but no published specification
  or licence statement was found. It stays out until one is.
- **National air quality indices.** Where a country publishes its own index (the AQHI, the EU EAQI),
  that index replaces the US AQI for that country. None is added yet.
- **Gradients.** These are stepped legends. A continuous map uses the same colours as stops.

## Commands

```sh
python3 standards/weather-colour-scales/scales.py write   # regenerate results.md
python3 standards/weather-colour-scales/scales.py check   # results.md is current
python3 -m unittest standards/weather-colour-scales/test_scales.py
```

## Consumers

Mukoko Weather (map legends, charts) and the Mzizi data-visualisation token family. Both should import
`scales.json` rather than copy its values.
