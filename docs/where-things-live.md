# Where things live

This repository is the one home for standards. A standard has exactly one
source here, and everything else is generated from it or checked against it.
If a fact about a standard is written anywhere else by hand, that is drift.

## The rule

| What                                                   | Lives in                                                    | Generated from it / checked against it                                                                               |
| ------------------------------------------------------ | ----------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| Standards we **adopt** (external: WMO, CF, GeoJSON, …) | `register/register.yaml`                                    | `register/REGISTER.md`, `register/seed/integrations.standards.json`, the MongoDB `integrations.standards` collection |
| Standards we **publish** (BAGS, the Bundu vocabulary)  | `standards/`, `src/terms.rs`                                | `vocab.bundu.org`, docs.bundu.org                                                                                    |
| How the data is **structured** (the canonical model)   | `nyuchi/data-infra` `schemas/` today; moves here in phase 2 | validators, indexes, JSON-LD context, Doris/Postgres/Cassandra DDL                                                   |

## Why a database copy exists, and why it drifts

`integrations.standards` holds the register as records so services can read it.
It is a **copy**. A copy drifts unless something compares it with its source, so
the comparison is part of the register tooling:

```sh
python3 register/tools/register.py check          # CI: the generated files match the YAML
python3 register/tools/register.py live export.json   # the database matches the register
```

`check` runs in CI on every pull request. `live` needs an export of the
collection, so it runs wherever a read-only connection exists (see "Owner steps"
in the tracking issue). On 2026-10-29 the first run of this comparison found the
drift recorded in the CHANGELOG; none of the earlier checks read the database.

## Rules that stop drift coming back

1. **One edit point.** Add or change a standard in `register/register.yaml`
   only. Never edit `REGISTER.md` tables or the seed by hand: `register.py write`
   regenerates them and CI fails if they are stale.
2. **No counts in prose.** Write "adopted standards" and let the generated tables
   count them. "The database has one document today" went stale the day it was
   written.
3. **No second list.** A playbook, README or model file that needs a standard
   links to its entry here by `id`. It does not restate the version or the
   steward.
4. **The model cites ids.** A field in the canonical model names the standard it
   follows by register `id`; the data-infra check fails on an id that is not in
   the register.
5. **A new external feed is converted at the edge.** Its names and units are
   mapped to registered standards (CF names, UCUM units, ISO codes) before
   anything is stored. A provider's own schema is never adopted.
6. **Standards the platform uses but the register lacks are proposed here
   first.** The weather and agriculture work added UCUM, SOSA/SSN, OGC
   SensorThings, WMDR, WIS2, OGC API - Features, SDMX and OCHA COD; the owner
   adopted them on 2026-10-30, with versions checked at source that day.
7. **No standard exists? Build one and publish it here.** Never keep a private
   shape because nothing was found (see `AGENTS.md`).

## Phases

1. **Now (this change):** the register, its generator, tests and the live
   comparison move here from `nyuchi/data-infra`.
2. **Next:** the canonical model (`schemas/model`, `projections.yaml`,
   `domains.yaml`, the generators and the generated validators) moves here.
   `nyuchi/data-infra` keeps the stores themselves (Cassandra, Redpanda, Doris,
   the ledger, monitoring).
3. **Then:** the live check runs on a schedule against a read-only connection and
   opens an issue when it finds drift.
