# Weather data colour scales

**Version 0.1.0, draft.** Scope approved by the owner on 2026-11-14 (#39); not published until it
merges and is released. Licence: CC BY 4.0 for the text, CC0 for `scales.json`. The adopted scales
remain their stewards' work and are cited, not relicensed.

## Why this exists

People read weather maps by convention: cold is blue and hot is red, green air is clean air, violet UV
means cover up. When an app paints those readings in its brand colours, or invents a new scale, it
misleads the reader, and every app that does so makes the ecosystem less consistent. The rule in
`AGENTS.md` applies: where a published scale exists it is adopted unchanged; where none exists we write
one here, so that every app built on the platform shows the same reading in the same colour.

## What is in it

`scales.json` holds every scale as data, so apps and Workers import it rather than copy it. Each band
is `[min, max)` in the scale's UCUM unit, with a label and an sRGB colour.

| Scale                 | Unit      | Origin                                                                     |
| --------------------- | --------- | -------------------------------------------------------------------------- |
| Air quality (US AQI)  | `{index}` | **Adopted.** US EPA, Technical Assistance Document, May 2026               |
| UV index              | `{index}` | **Adopted.** WHO, WMO, UNEP, ICNIRP: Global Solar UV Index, 2002           |
| Air temperature       | `Cel`     | **Ours.** Breakpoints ours; ColorBrewer RdYlBu ramp                        |
| Wind speed (Beaufort) | `km/h`    | **Ours.** Beaufort breakpoints (WMO); ColorBrewer YlGnBu ramp              |
| Rainfall rate         | `mm/h`    | **Ours.** WMO rainfall intensity classes; ColorBrewer Blues ramp           |
| Cloud cover           | `{okta}`  | **Ours.** Okta classes as in METAR (FEW, SCT, BKN, OVC); ColorBrewer Greys |

Each scale records its source, steward, edition, the date it was checked and whether each claim was
**measured** at source or is **inferred**. Inferred claims are listed below and must be confirmed
before 1.0.

## Rules

1. The band label or its value range is always printed beside the colour. Colour alone never carries
   the meaning, which is what makes the scales usable for colour-blind readers and in print.
2. An adopted scale is used unchanged: its breakpoints, labels and colours.
3. Brand colours (the Mzizi minerals) are never used for data. The tests enforce this.
4. Values are converted to the scale's UCUM unit at the edge, before a band is chosen.
5. A swatch with less than 3:1 contrast against the surface it sits on takes a 1px border in the
   legend text colour. `results.md` says which swatches need it on light and dark surfaces.

## Accessibility, measured

`results.md` is generated from `scales.json` and checked in CI. For every band it gives the legend
text colour (black or white) and its contrast, which must reach 4.5:1 (WCAG 1.4.3 AA; the tests fail
otherwise), and the swatch's contrast on light and dark surfaces. For every scale it gives the smallest
colour difference (CIE76 delta E) between neighbouring bands, for typical vision and simulated
protanopia, deuteranopia and tritanopia (Machado, Oliveira and Fernandes, 2009).

Two scales have neighbours closer than delta E 10 under protanopia: the adopted UV colours and our wind
ramp. Rule 1 covers both. The EPA publishes its own colour-blind palette for the AQI, adopted here as
`assist`.

## Inferred, to confirm before 1.0

- **UV hex values.** The WHO guide gives Pantone codes (375, 102, 151, 032, 265), not RGB. The hex
  values are Pantone's sRGB equivalents as we understand them and must be checked against Pantone.
- **ColorBrewer licence.** The ramps are ColorBrewer's published values; the licence (Apache-2.0
  style) must be confirmed at source.
- **Beaufort, rainfall and okta limits.** The km/h Beaufort equivalents, the WMO rainfall intensity
  limits (2.5, 10, 50 mm/h) and the okta grouping must be checked against WMO-No. 306 and WMO-No. 8.

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
