# Weather data colour scales

**Status: 0.1.0 draft. Not published.** Nothing here goes to standards.bundu.org
or into the vocabulary until the owner approves the scope (#39).

Apps on the platform show weather with colour: air quality, UV, wind,
temperature, rain. People read those colours by convention, so a brand palette
or a scale each app invents misleads them. The rule (`AGENTS.md`) applies: where
a published scale exists we adopt it unchanged and cite it; where none exists we
write one here.

The scales are data, in [`scales.json`](scales.json), so apps and Workers import
them instead of copying values by hand.

## One rule for every scale

Colour never carries meaning alone. The band label or the value range is always
shown beside the colour, for people with colour-vision deficiencies and for
black-and-white print.

## What this draft holds

| Scale               | Breakpoints             | Colours                                             | Source (register id)                                 |
| ------------------- | ----------------------- | --------------------------------------------------- | ---------------------------------------------------- |
| Air Quality Index   | adopted                 | adopted: standard and ColorVision Assist            | US EPA, EPA-403/B-26-003, May 2026 (`us_epa_aqi`)    |
| UV Index            | adopted                 | adopted as Pantone references; screen sRGB **open** | WHO, WMO, UNEP, ICNIRP 2002 (`who_global_solar_uvi`) |
| Beaufort wind force | adopted, in m/s at 10 m | **open**: WMO defines none                          | WMO-No. 8, Table 5.1 (`wmo_beaufort`)                |

Each source was read on 2026-11-14. The values in `scales.json` are copied from
the tables named there, and `test_scales.py` pins them.

### Air Quality Index

EPA Table 2 colours are required for EPA reporting. Table 3, the ColorVision
Assist colours, may be used to help people who find red and green hard to tell
apart. Both are included. Label text is chosen for contrast and recorded per
band: every band, in both sets, reaches WCAG 2 AA (4.5:1) with black or white
text. The lowest is Unhealthy in ColorVision Assist, 4.93:1 with black.

The index breakpoints for each pollutant (EPA Table 6, for example PM2.5
"Good" 0.0–9.0 µg/m³ since the 2024 revision) are the EPA's and are not restated
here. An app computes the index by the EPA method, then reads its colour here.

### UV Index

The guide gives each category a colour name and a Pantone (PMS) reference only,
not sRGB. Screen colours are left empty on purpose. They must come from
Pantone's own published sRGB value for each reference, recorded with its source,
and are never estimated. The guide writes Low as "<2" in Table 1 and "(1,2)" in
Table 4, and the index is reported as a whole number, so this scale uses 0–2.

### Beaufort

The WMO gives force numbers and speeds (kt, m/s, km/h, mi/h) at 10 m above open
flat ground. The scale stores m/s, the UCUM unit `m/s`. WMO defines no colours,
so the ramp is ours to write, and it is open.

## Open

- Temperature (UCUM `Cel`): breakpoints ours, ramp from a published diverging
  scheme (for example ColorBrewer RdYlBu), with its licence checked.
- Rain rate (`mm/h`) and radar reflectivity (dBZ): find whether a national
  service publishes a reusable progression and under what licence.
- Cloud cover.
- UV Index sRGB from Pantone's published values.
- Beaufort colours.
- Dark-theme variants.
- Colour-vision-deficiency separation results (deuteranopia, protanopia,
  tritanopia) for every scale.

## Consumers

- Mukoko Weather: map legends, charts and display.
- Mzizi design tokens: a data-visualisation token family, where domain colour
  standards override brand styling.
