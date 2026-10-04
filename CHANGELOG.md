# Changelog

All notable changes to the Bundu vocabulary and this repository. Term URIs
are a permanent contract: a release may add terms, refine a pending term's
definition or promote it to stable, and never removes a term or changes a
stable term's meaning.

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
- **AI guardrail terms** for the Bundu AI Guardrail Standard, all pending:
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

## 0.1.0

The first release: `Family`, `Community`, `PlaceOwner`, `recognitionBasis`,
`verificationTier`, `ubuntuScore`, `communityVouches` and `isInformal`.
