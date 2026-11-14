# Changelog

All notable changes to the Bundu vocabulary and this repository. Term URIs
are a permanent contract: a release may add terms, refine a pending term's
definition or promote it to stable, and never removes a term or changes a
stable term's meaning.

From 2026-10-04 releases follow the org versioning policy
([nyuchi/.github#80](https://github.com/nyuchi/.github/issues/80)): a release
to `main` is the next minor above the highest tag (so the next is 0.2.0),
merges into `staging` are patches, and a major is only released by hand.
Versions released before then are not renumbered.

## Unreleased

### Changed

- Adopted ten standards that were proposed: H3, IPTC ninjs, IPTC Digital
  Source Type, Apache Avro, Activity Streams 2.0, SPDX, the Common Alerting
  Protocol (CAP 1.2), OGC API - Common, SD-JWT VC and the Legal Entity Identifier
  (ISO 17442). The decision is the owner's instruction of 2026-11-11 in the
  coordinator's chat: "all the proposed standards please release them, into a
  beta. if they are ones we are adopting then we fully adopt everything now",
  then "we fully adopt". Each entry quotes it in `decided`. The Agent Client
  Protocol stays proposed, as the owner decided earlier the same day (no
  foundation host is confirmed). Adopting CAP the standard does not settle the
  separate choice to push severe warnings only from an official CAP source,
  which is still open. Adopted 88 to 98, proposed 11 to 1. REGISTER.md and the
  integrations.standards seed are regenerated with `register.py write`; applying
  the seed to the live collection is a separate step. Merging to `staging`
  releases this to the live beta as the next patch.
- Re-read UCUM at its source on 2026-11-11 (still version 2.2, 2024-06-17) and
  recorded its licence in the entry: Regenstrief Institute's own UCUM Copyright
  Notice and License, version 1.1, which is neither open-source nor Creative
  Commons and bars modified copies and derivative works. UCUM was the eighth
  standard on nyuchi/data-infra#94; it was already adopted (2026-10-30), so it
  gets no new entry and stays adopted.
- Adopted three standards from the Agentic AI Foundation: Agent Skills, Open
  Responses and AGENTS.md. The decision recorded is the owner's instruction of
  2026-11-11 in the coordinator's chat ("Adopt the three AAIF projects, ACP
  stays proposed"). Each was read at its source on 2026-11-11. Agent Skills and
  AGENTS.md have no version (no tags, none stated on their pages), so a consumer
  pins the commit; Open Responses is versioned by immutable dated releases, the
  latest 2026-04-24, and a consumer pins that dated file, not the "latest" one,
  which differs from it. The licences are recorded as the repositories state
  them: Agent Skills code Apache-2.0 and documentation CC-BY-4.0, Open Responses
  code Apache-2.0 and documentation CC BY 4.0, AGENTS.md MIT. Applying the seed
  to the live `integrations.standards` collection is an owner step and was not
  done.
- Adopted three device-domain standards: the OAuth 2.0 Device Authorization
  Grant (RFC 8628), the OAuth 2.0 Authorization Framework (RFC 6749) and
  Schema.org Product. The decision recorded is the owner's instruction of
  2026-11-11 in the coordinator's chat ("Go ahead and adapt the standards, push
  and release to production across the repos"), which the coordinator read as
  adopting these three; "adapt" is the owner's word, and the reading is
  recorded as a reading. Each was read at its source on 2026-11-11 (RFC 8628 and
  RFC 6749 are Proposed Standards, neither obsoleted; RFC 6749 is updated by
  RFC 8252, RFC 8996 and RFC 9700). Applying the seed to the live
  `integrations.standards` collection is an owner step and was not done.
- Adopted eight standards (owner, 2026-10-30): UCUM (2.2), SOSA/SSN (W3C
  Recommendation 2017), OGC SensorThings (Part 1 1.1, Part 2 1.0), WMDR
  (WMO-No. 1192, 2019 edition), WIS2 (Manual on WIS, Volume II), OGC API -
  Features (Part 1 corrigendum 1.0.1, ISO 19168-1:2020), SDMX (3.1, May 2025)
  and OCHA COD-AB. Each version was read at its source on that day. `sd_jwt_vc`
  stays proposed.
- Corrected the links for WMDR and WIS2, which pointed at WMO pages that return
  404, and for SDMX and OCHA COD, which now point at the standard itself.

### Fixed

- `standards.bundu.org/register/latest` answered 502 ("The repository answered
  403") for a share of requests: it asked GitHub's REST API for the head of
  `main` without a token, and Cloudflare's shared egress addresses run out of
  that API's 60 requests an hour. When the API refuses, the endpoint now reads
  the head of `main` from git's own ref advertisement (`info/refs`), which is
  not held to that limit. Two tests cover the fallback and the case where
  neither names `main`.
- Removed an approval date (2022-03-24) from the OGC Tile Matrix Set entry: it
  could not be confirmed at the source (the Nyuchi Maps tile contract reads a
  different one), so the entry now states the status (Approved) and no date.
- The seed no longer calls a proposed standard "Adopted": its adoption rationale
  starts "Proposed:" when the register says proposed.
- Removed a note on GeoJSON that said it was the one document in
  `integrations.standards`, which stopped being true when the register was loaded.

### Added

- Research notes 0002 and 0003, both open briefs with no findings yet:
  what each level of weather station should measure, from a homestead to an
  industrial or network station (`docs/research/0002-station-tiers.md`), and
  how hand-read rain gauges and manual stations across Africa can send their
  readings into the Bundu Weather Commons in one standard form
  (`docs/research/0003-analog-station-uploads.md`). Owner's direction,
  2026-11-14: "we should be researching what should be included in a
  different levels of stationkits, home to industrial to much larger. We
  should also be working on ways to support analog stations across Africa to
  upload data into the platform and how we standardize the uploads and find
  easier ways for people to provide data easily." Both name the standards that
  apply by register id and say what would close them.
- Weather data colour scales, 0.1.0 draft (`standards/weather-colour-scales/`,
  #39): six scales as data in `scales.json`. The US EPA AQI colours (May 2026
  Technical Assistance Document, with the EPA's own ColorVision Assist palette)
  and the WHO Global Solar UV Index colours are adopted unchanged; temperature,
  wind (Beaufort breakpoints), rainfall rate (WMO intensity classes) and cloud
  cover (oktas) are ours, with ColorBrewer ramps. `scales.py` generates
  `results.md`: legend text contrast (AA enforced), swatch contrast on light and
  dark surfaces, and neighbour separation under simulated colour-blindness.
  Register entry `weather_colour_scales`, proposed. Adopted the three external
  standards it uses: `us_epa_aqi` (EPA-403/B-26-003), `who_global_solar_uvi`
  (2002 guide) and `wmo_beaufort` (WMO-No. 8, Table 5.1). The owner approved
  the scope on 2026-11-14.
- Adopted the Agent Trajectory Interchange Format (ATIF-v1.8, register id
  `atif`) as the session log of an agent harness, on the owner's decision of
  2026-11-14 ("adopt ATIF"). The harness conformance checklist now names it in
  section 1 and records the decision in section 3; the BCI message stream maps
  onto ATIF steps instead of defining its own log (#48, #50).

- Proposed seven standards that the October 2026 standards audit found in use
  on the data platform and not in the register (nyuchi/data-infra#94, tracked in
  #179): H3 (places, release 4.5.0), IPTC ninjs (news, version 3.2), IPTC
  Digital Source Type (news, released 2024-10-23), Apache Avro (streaming,
  release 1.12.2), Activity Streams 2.0 (communication, W3C Recommendations of
  2017-05-23), SPDX (open data, specification 3.0.1 and License List 3.29.0) and
  the Legal Entity Identifier (identity, ISO 17442-1:2020 and -2:2020). Each was
  read at its source on 2026-11-11, and its entry records the steward, the
  version, the licence of the specification and of any reference code, and where
  the platform uses it. They stay `proposed`, not adopted: #94 lists them and
  says that adding a standard is an owner decision. Two findings are recorded in
  the entries: ISO/IEC 5962:2021 is SPDX 2.2.1 and SPDX 3.0 is in ISO's final
  approval stage (50.20) and not yet published; IPTC's ninjs page still names 3.0
  as current while its repository and schema say 3.2. Applying the seed to the
  live `integrations.standards` collection is an owner step and was not done.
- Draft of Bundu Conversation Interchange 0.1.0 (`standards/conversation-interchange/`): a JSON
  Schema, JSON-LD context, an example stream and tests for how conversations, messages, projects,
  saved answers and memory are written down so a person can move them between assistants. A thin
  profile: a message holds an Open Responses item unchanged. Registered as `bci`, `proposed`. Not
  published until the owner approves its scope (#48).

- Proposed the Agent Client Protocol (ACP; protocol version 1 is the stable
  version and version 2 is in alpha), read at its source on 2026-11-11. It
  standardises how a code editor drives a coding agent. Its governance is
  interim, jointly Zed and JetBrains, and no foundation host is confirmed, so it
  stays `proposed` (the owner left it so on 2026-11-11). It is not IBM's Agent
  Communication Protocol, which merged into A2A.
- Proposed three standards for the device domain (nyuchi/data-infra#96, the
  Mukoko Home pairing work, mukoko-dev/mukoko-home#24) under a new `devices`
  domain, "Devices and pairing": the OAuth 2.0 Device Authorization Grant (RFC
  8628), the OAuth 2.0 Authorization Framework (RFC 6749, for the paired
  device's rotating refresh token) and Schema.org Product (what a device is in
  JSON-LD). They were proposed on 2026-11-11 and adopted the same day (see
  Changed).
- Proposed the Common Alerting Protocol (CAP, version 1.2, OASIS Standard, 01
  July 2010), read at its source on 2026-11-04. The severe-weather alert rules
  in mukoko-weather (#289) read it, and a warning is pushed only from an
  official CAP source. It stays `proposed` until the owner adopts it.
- Proposed OGC API - Common, Part 1 Core (OGC 19-072, version 1.0), which OGC
  API - Tiles depends on (found in the Nyuchi Maps tile contract,
  nyuchi/maps#5). It stays `proposed` until the owner adopts it.
- Five map-tile standards for Nyuchi Maps (nyuchi/maps#4): OGC API - Tiles
  (Part 1 Core 1.0, OGC 20-057), the OGC Two Dimensional Tile Matrix Set and
  Tile Set Metadata (2.0, OGC 17-083r4), the Mapbox Vector Tile specification
  (2.1), TileJSON (3.0.0) and PMTiles (version 3). Each was read at its source
  on 2026-11-03, proposed that day and adopted by the owner on 2026-11-04. The
  base-map schema (OpenMapTiles or another) is not listed: it is not decided.
- `register/`: the register of external standards the platform adopts
  (`register.yaml`), moved here from `nyuchi/data-infra` `standards/` so that
  standards have one home. It generates `REGISTER.md` and
  `seed/integrations.standards.json`, and `register/tools/register.py live`
  compares the MongoDB `integrations.standards` collection with it.
- `docs/where-things-live.md`: what lives where, and the rules that keep copies
  from drifting.
- `AGENTS.md`: the rule that standards are adopted or built and published, never
  invented per provider, and how to apply it.
- Eight proposed standards the weather and agriculture work needed and the
  register lacked: UCUM, SOSA/SSN, OGC SensorThings, WMDR, WIS2, OGC API -
  Features, SDMX and OCHA COD P-codes. Listed as proposed (seeded inactive) and
  not yet checked at source.
- `workers/standards/`: `standards.bundu.org`, built on Astro for Cloudflare
  Workers. It publishes the register as pages (one per standard, with
  schema.org `DefinedTerm` JSON-LD) and `/register/index.json`, and serves
  four files of `register/` at a commit (nothing else), with RFC 9457 errors
  and immutable caching for a commit. The register is read at build time from
  `register/register.yaml`, so the pages cannot drift from it.
- `register.py lock`: prints the sha256 of each published file, for a consumer's
  lock. A test pins the Worker's file list to the tool's.
- A `Standards endpoint` CI job, and a deploy workflow (on `main` and `staging`, when the site or register changes) that fails loudly when
  its Cloudflare secrets are missing.

### Found

- Drift measured on 2026-10-29, the first time the live collection was compared
  with the register: the register had 70 standards, the database 66. Missing
  from the database: `bags`, `csvw`, `nist_sp_800_63_4` and `sd_jwt_vc`
  (proposed, so seeded inactive). The register's own page said the database held
  one document. The database still named the old `bundu-labs/vocab` repository.
  The old register page described vocab.bundu.org as version 0.1.2; this
  repository is at 0.2.0.

### CI

- Production deploys move to `.github/workflows/fly-deploy.yml`, which runs
  when `CI` passes on a push to `main` and deploys the commit CI tested to the
  Fly app `bundu-vocab`, then smoke-tests vocab.bundu.org. Without the
  `FLY_API_TOKEN` secret it skips with a notice. The tag-and-release job stays
  on its own as `.github/workflows/release.yml` (the Release workflow), which
  no longer deploys.
- standards.bundu.org is deployed by Cloudflare Workers Builds, connected to
  this repository in the Cloudflare dashboard, instead of
  `.github/workflows/standards-endpoint-deploy.yml`, which is removed. That
  workflow failed at its Deploy step on every run from 2026-11-10, most likely
  because the `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` secrets were
  never set, so the live site stayed at 78 standards. Only `main` deploys to
  the live site now; other branches, `staging` included, get preview versions.
  The post-deploy smoke test went with the workflow.
- The site builds and is tested on Node 24, in CI and on Workers Builds
  (`NODE_VERSION` 24): Node 22 is no longer supported. `workers/standards`
  requires Node 24 or later (`engines`).

## 0.2.0 (2026-10-05)

The first release of `staging` to `main` under the org versioning policy.

### Changed

- **`verificationTier` is the platform's 0-4 ladder** (#24). The term said
  0-3 (unclaimed, claimed, verified, fully verified). Every system that
  writes it uses 0-4: 0 = unverified, 1 = community, 2 = contact verified (a
  one-time passcode), 3 = government, 4 = licensed (api-gateway
  `gateway/lib/verification.py`, Kweli). Kweli's ActivityPub actors now
  emit it (owner decision Q6, 2026-10-04, mukoko-dev/kweli#171), so the
  published definition has to match.

### CI

- Merges into `staging` are tagged as patches and CI runs on `staging`
  (#22). The Fly beta (`bundu-vocab-beta`) that #22 added was removed again
  (#23).

## 0.1.4 (2026-10-04)

### Added

- **`notificationPreference`** (pending, published as `testing`). A person's
  choice about optional notifications, one boolean per channel (today
  `email`), off until the person turns it on. Security and account notices
  are always sent and no preference covers them. schema.org has no consent
  or notification-preference property. Asked for by the canonical schema
  (nyuchi/data-infra) and the Nyuchi API (nyuchi/api-gateway#186).

## 0.1.3 (2026-10-04)

### Changed

- **The AI guardrail terms are stable.** The Bundu AI Guardrails Standard
  (BAGS) 1.0.0 was published on bundu.org on 2026-10-04
  ([bundu.org/research/standards/ai-guardrails](https://www.bundu.org/research/standards/ai-guardrails)), with its versioned
  snapshot in `standards/ai-guardrails/bags-1.0.0.json`. Every term in the AI
  guardrails group (`AIGuardrail`, `PlatformGuardrail`, `EntityGuardrail`,
  `GuardrailCategory`, `guardrailCategory`, `guardrailSeverity`,
  `appliesToSurface`, `enforcementLayer`, `isCoreGuardrail`,
  `standardVersion`, `BAGS` and `BAGS-01` to `BAGS-12`) now publishes
  `vs:term_status` `stable`, and its meaning is fixed.

### Added

- **Deploy on merge.** `.github/workflows/deploy.yml` deploys `main` to the
  Fly app `bundu-vocab` after format, clippy and the tests pass, then
  smoke-tests vocab.bundu.org. It uses the repository secret `FLY_API_TOKEN`
  (a deploy token scoped to the app), and skips with a notice if it is not
  set.

## 0.1.2 (2026-10-04)

### Added

- **BAGS 1.0.0 rule terms.** The rule codes are final (owner-approved
  2026-10-04). `BAGS` is the standard's set of rules (`schema:DefinedTermSet`).
  `BAGS-01` to `BAGS-12` are its twelve core rules, each a
  `schema:DefinedTerm` and a `PlatformGuardrail` with its `schema:termCode`,
  set, category, severity and standard version. A rule's wording stays in the
  standard (CC BY 4.0) and is not copied into this CC0 vocabulary.
- `guardrailCategory`, whose values are the twelve BAGS 1.0.0 category keys.
- A new kind of term, the **individual**: a named instance such as a rule,
  published with its own types.

### Changed

- `AIGuardrail` (pending) is now a subclass of `schema:DefinedTerm`, not
  `schema:CreativeWork`. That is how the canonical schema (nyuchi/data-infra
  `shamwari.yaml`) models a guardrail: a coded term in a published set. The
  standard document itself is the CreativeWork.
- `GuardrailCategory` now says that its `termCode` is a `guardrailCategory`
  key.
- The standard is called the Bundu AI Guardrails Standard (BAGS) throughout.
- A term id may contain single hyphens between alphanumerics, so that a rule
  keeps its published code as its URI.

## 0.1.1 (2026-10-04)

The repository is renamed from `bundu-labs/vocab` to
`bundu-labs/bundu-standards`, the home for every Bundu standard. Term URIs
under `https://vocab.bundu.org/` are unchanged.

### Added

- **Term status.** Every term publishes `vs:term_status` (W3C vocabulary
  status): `stable`, or `testing` for a pending term whose standard is still
  being written. The term page and the index show it.
- **Canonical-schema terms** that the Nyuchi canonical schema
  (nyuchi/data-infra) and the Nyuchi API use and schema.org lacks, all stable:
  - `PublishingHouse` and `NewsDesk`, the `additionalType` terms for
    publishing groups and the desks inside a masthead (owner decision,
    2026-10-02).
  - `defaultFamily`, the family a person belongs to first (the family-first
    invariant).
  - `assuranceLevel` (0–4, with NIST SP 800-63-4 IAL as a reference mapping)
    and `attestation` (`community`, `business`, `domain`).
  - `kAnonymity` and `maxReidentificationRisk`, for the open research data
    given back when a person deletes their account. `kAnonymity` was already
    emitted by the Nyuchi API's open-data catalogue and did not resolve.
- **AI guardrail terms** for the Bundu AI Guardrails Standard, all pending:
  `AIGuardrail`, `PlatformGuardrail`, `EntityGuardrail`, `GuardrailCategory`,
  `guardrailSeverity`, `appliesToSurface`, `enforcementLayer`,
  `isCoreGuardrail` and `standardVersion`. Per-rule terms wait until the
  standard fixes stable rule identifiers.
- New subject areas: News and media, Open data, AI guardrails.
- Tests that every term resolves through the real handler as valid JSON-LD
  and as a rendered HTML page, that content negotiation holds, that every
  class's parent chain reaches schema.org, and that no definition points at
  a `bundu:` term that does not exist. The CI contract job now parses each
  term's JSON-LD and checks its HTML page names the term.

### Changed

- README, Cargo metadata and the CI badge point at
  `bundu-labs/bundu-standards`.
- The image build copies `Cargo.lock` and builds `--locked` on a Rust 1.90
  bookworm builder. Without the lockfile, each deploy resolved the newest
  crates, and one that needs edition 2024 broke the build on Rust 1.83.

## 0.1.0

The first release: `Family`, `Community`, `PlaceOwner`, `recognitionBasis`,
`verificationTier`, `ubuntuScore`, `communityVouches` and `isInformal`.
