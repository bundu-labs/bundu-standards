# Research note 0003 — getting analog stations into the commons

**Status:** open research brief, started 2026-11-14. No findings yet.

**Question:** across Africa, rain gauges and manual weather stations at
schools, farms, missions, clinics and district offices are read by hand and
written in logbooks. How can the people who keep them send those readings to
the Bundu Weather Commons as easily as possible, in one standard form, with
enough metadata that the commons can check them and, over time, publish them?

**Why it matters:** the commons is short of ground truth. The national
synoptic feed has 19 stations registered and 4 reporting (late October 2026),
and the community network has one station. Hand-read gauges and logbooks are
ground truth that already exists, and much of it, past and present, never
reaches a database.

---

## Principles

- **The observer's effort comes first.** If sending a reading takes longer
  than writing it in the logbook, it will not be sent.
- **One standard form, many ways in.** However a reading arrives, it is
  converted at the edge into one record, named by register id, as the rule in
  AGENTS.md requires. Nothing about the channel it came through leaks into the
  stored shape.
- **Provenance travels with the number.** Who read it, from which gauge, when,
  and how it was sent, so the commons can weigh it and the observer can be
  credited.
- **Checked, not trusted.** A hand-read value enters as an observation to
  verify against neighbours, models and satellite rainfall, never as fact.

## Standards that apply, by register id

`wmo_station_ids` (every station gets a WIGOS identifier, including a
volunteer one), `wmdr` (station metadata: where, what instrument, how it is
sited), `sosa_ssn` (the observation, the sensor or person, the procedure),
`csvw` (a spreadsheet upload that describes its own columns), `ucum` (units),
`iso_8601` (the time of reading and the period it covers), `ocha_cod` (the
administrative area), `ogc_sensorthings` (how the result is served).

## Ways in to test

| Channel                                                         | Suits                                                                   | To find out                                                                          |
| --------------------------------------------------------------- | ----------------------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| A spreadsheet template (CSVW)                                   | Schools, offices, anyone with a computer and months of logbook to enter | Can one template cover rainfall-only gauges and full manual stations?                |
| A short form in Mukoko Weather                                  | Smartphone users reading one gauge daily                                | How few taps does a daily rainfall reading need?                                     |
| SMS, USSD or WhatsApp with a fixed message shape                | Feature phones, low data, the most remote observers                     | What message shape survives typing errors, and how is the sender bound to a station? |
| A photo of the logbook page, read and confirmed by the observer | Historical records and observers who will not type                      | How accurate is reading handwritten tables, and what must a person confirm?          |
| Bulk transfer from a partner's records                          | Extension services, research farms, NGOs with archives                  | What licence and attribution terms let the data be published?                        |

## Questions the research must answer

1. **Time:** what reading time do observers actually use, and which period
   does a daily rainfall total cover? Records must say so explicitly rather
   than assume a convention.
2. **Units and instruments:** which gauge types are in use, and how is a
   reading in another unit or a "trace" recorded?
3. **Identity:** how does a volunteer station get a WIGOS identifier and WMDR
   metadata without a visit?
4. **Quality:** which checks (range, neighbours, satellite rainfall, the
   observer's own history) catch the common errors, and how is a value
   flagged rather than dropped?
5. **Credit and consent:** how is the observer credited, and how is their
   personal data kept out of the published record?
6. **Return:** what does the observer get back, such as a season summary for
   their place, so that sending a reading is worth their while?

## What would close this note

An upload profile, each channel mapped to one standard record, with a CSVW
template, a message grammar for SMS, USSD and WhatsApp, the quality checks and
their flags, and a pilot with a named set of analog stations. If no existing
standard covers the profile, it is proposed as a Bundu standard in
`standards/`, with its terms in the vocabulary.
