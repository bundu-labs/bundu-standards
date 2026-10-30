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

### Fixed

- The seed no longer calls a proposed standard "Adopted": its adoption rationale
  starts "Proposed:" when the register says proposed.
- Removed a note on GeoJSON that said it was the one document in
  `integrations.standards`, which stopped being true when the register was loaded.

### Added

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
- A `Standards endpoint` CI job, and a deploy workflow that fails loudly when
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
