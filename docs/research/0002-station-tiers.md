# Research note 0002 — what each level of weather station should measure

**Status:** open research brief, started 2026-11-14. No findings yet.

**Question:** a weather station on a homestead, one on a commercial farm or a
school, one a forecaster can rely on, and one on a mine, airfield or utility
are different instruments with different jobs. What should each level carry,
how should it be sited and maintained, and what metadata must it publish, so
that its readings can enter the Bundu Weather Commons at a known quality?

**Why it matters:** the commons publishes a station's readings back to the
open data ecosystem only once the station has been verified accurate over
time. That bar is easier to meet, and to state honestly, when every station
declares what level it is built to and the commons knows what to expect of
it. Today the nyuchi StationKit comes in two builds (Basic and Full), and the
commons has one registered station.

---

## What we already have

- **StationKit Basic:** ESP32-C3, temperature and humidity (BME280),
  barometric pressure, solar power with an 18650 battery, IP65 enclosure.
- **StationKit Full:** everything in Basic, plus a tipping-bucket rain gauge,
  anemometer and wind vane, capacitive soil moisture, and an optional UV or
  light sensor.
- **Standards in the register that a station level must meet, by id:**
  `wmo_station_ids` (a WIGOS identifier for every station), `wmdr` (the
  station's metadata: siting, instruments, maintenance), `sosa_ssn` (what was
  observed, by which sensor, with which procedure), `ucum` (units),
  `ogc_sensorthings` and `wis2` (how readings are served and exchanged),
  `iso_8601` (times).

## Candidate levels to test

These are working hypotheses for the research to confirm or replace, not a
specification.

| Level                  | Who it is for                                                 | Candidate core                                                                                                | Open questions                                                                                                                      |
| ---------------------- | ------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| Home and community     | Homesteads, small plots, community gardens                    | Temperature, humidity, pressure, rainfall                                                                     | Is rainfall without wind useful enough? What siting can a household realistically meet?                                             |
| Farm and school        | Commercial farms, cooperatives, schools, clinics              | Home, plus wind speed and direction, soil moisture, solar radiation or UV                                     | Which soil depths? Is leaf wetness worth its cost for disease advice?                                                               |
| Agrometeorological     | Extension services, research farms, district forecasters      | Farm, plus a ventilated radiation shield, soil temperature, evapotranspiration inputs, a calibration schedule | How close to WMO siting guidance must it be to count as reference-grade for the commons?                                            |
| Industrial and network | Mines, airfields, utilities, large estates, national networks | Agrometeorological, plus redundancy, lightning, visibility, and particulate air quality where relevant        | Which of these belong in a station and which in a separate instrument? What uptime and maintenance commitment comes with the level? |

## How the research will answer it

1. **Ask the users.** Interview farmers, cooperatives, schools, extension
   officers, a mine safety officer and a district forecaster: which decisions
   would a reading change, and which readings do they already have.
2. **Read the guidance.** Compare each candidate level with WMO siting and
   instrument guidance and with the national weather service's own station
   classes, and record where a low-cost build can and cannot meet it.
3. **Measure the builds.** Run each StationKit build beside a reference
   station for at least one full season, and publish the bias and drift per
   variable, so "verified accurate over time" has a number behind it.
4. **Cost it.** Price each level as built, installed and maintained for three
   years, so a host or a funder knows what they are taking on.

## What would close this note

A table of levels, each with its required variables, siting and maintenance
commitments, the WMDR fields it must publish, and the measured accuracy of the
StationKit build that meets it. If no existing standard defines station
levels this way, the result is proposed as a Bundu standard in `standards/`
and its terms in the vocabulary, as the rule in AGENTS.md requires.
