# Weather data colour scales

**Version 0.1.0. Adopted by the owner on 2026-11-15** ("We should be adopting all standards
immediately before people start using our app"); scope approved 2026-11-14 (#39). Licence: CC BY 4.0
for the text, CC0 for `scales.json`. The adopted scales remain their stewards' work and are cited, not
relicensed.

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
**measured** at source or is **inferred**. Inferred claims are listed below and must be confirmed
before 1.0.

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
YlGnBu's nine classes; the 7-class scheme fell to 9.0 under protanopia. The adopted UV colours fall to
9.9 under protanopia and keep their colours, as an adopted scale must: rule 1 covers them. The EPA
publishes its own colour-blind palette for the AQI, adopted here as `assist`.

## Inferred, to confirm before 1.0

- **UV hex values.** The WHO guide gives Pantone codes (375, 102, 151, 032, 265), not RGB. The hex
  values are Pantone's sRGB equivalents as we understand them and must be checked against Pantone.
- **ColorBrewer licence.** The ramps are ColorBrewer's published values; the licence (Apache-2.0
  style) must be confirmed at source.
- **Rainfall and okta limits.** The WMO rainfall intensity limits (2.5, 10, 50 mm/h) and the okta
  grouping must be checked against WMO-No. 8 and WMO-No. 306. The Beaufort bounds were checked at
  source (WMO-No. 8, Vol. I, Table 5.1) on 2026-11-14.

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
