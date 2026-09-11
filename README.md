# Bundu vocabulary

> RDF terms for concepts schema.org has no word for.

**Repository:** `bundu-labs/vocab`
**Live at:** <https://vocab.bundu.org>
**Licence:** Apache 2.0 (code) · CC0 1.0 (the vocabulary itself)
**Owner:** Bundu Foundation (Zimbabwe CLG)

---

## What this is

A vocabulary namespace. When something dereferences
`https://vocab.bundu.org/Family`, it gets back a definition — JSON-LD for a
machine, a readable page for a person, from the same URI by content
negotiation.

That sounds trivial and mostly isn't. Plenty of published vocabularies serve
404s at their own term URIs, and consumers quietly stop trusting them. This
repository exists so ours resolves, forever, from a binary with no database
and no runtime dependencies to rot.

## Scope

This namespace serves terms for **every Bundu standard, across every domain**.
Identity and trust happen to be first because that is what was built first; a
standard about health records, agricultural supply or education delivery would
publish its terms here too.

Terms carry a subject area for grouping, and **URIs stay flat** —
`vocab.bundu.org/Family`, never `vocab.bundu.org/identity/Family`. A term URI
becomes a permanent contract the moment someone embeds it in their data, and a
path segment encodes a taxonomy that will eventually be wrong. schema.org keeps
800+ terms flat for the same reason.

## Why it exists

The Bundu identity model classifies entities as `family`, `organization`,
`community` or `place_owner`. Only one of those has a schema.org type.

Rather than mislabel families as businesses or invent a private type that
breaks JSON-LD for everyone else, entities serialise as the nearest valid
schema.org type plus an `additionalType` pointing here:

```json
{
  "@type": "Organization",
  "additionalType": "https://vocab.bundu.org/Family",
  "name": "Fawcett Family"
}
```

Consumers that have never heard of Bundu read a valid `Organization`.
Consumers that have read a family. Nothing is lied to.

## The bar for adding a term

**If schema.org can express it, use schema.org and add nothing.**

A vocabulary earns trust by being small. Every term here carries a
`rationale` field recording what was tried first and why it did not fit, so
that anyone auditing the model later can judge whether the term is still
needed instead of guessing at intent. A term without a defensible rationale
does not belong.

## Terms

### Identity and kinship

| Term         | Kind  | Extends               |
| ------------ | ----- | --------------------- |
| `Family`     | class | `schema:Organization` |
| `Community`  | class | `schema:Organization` |
| `PlaceOwner` | class | `schema:Organization` |

### Trust and recognition

| Term               | Kind     | Value                                                                            |
| ------------------ | -------- | -------------------------------------------------------------------------------- |
| `recognitionBasis` | property | `customary` · `statutory` · `registry` · `member_consent` · `contested` · `none` |
| `verificationTier` | property | integer, 0–3                                                                     |
| `ubuntuScore`      | property | number                                                                           |
| `communityVouches` | property | integer                                                                          |

### Informal economy

| Term         | Kind     | Value   |
| ------------ | -------- | ------- |
| `isInformal` | property | boolean |

## Using it

Fetch any term for its machine-readable definition:

```sh
curl -H 'Accept: application/ld+json' https://vocab.bundu.org/Family
```

Or embed the context to refer to terms by short name:

```json
{
  "@context": [
    "https://schema.org/",
    "https://vocab.bundu.org/context.jsonld"
  ],
  "@type": "Organization",
  "additionalType": "Family",
  "verificationTier": 2
}
```

## Endpoints

| Path              | Purpose                                     |
| ----------------- | ------------------------------------------- |
| `/`               | Index of every term                         |
| `/{Term}`         | A single term                               |
| `/context.jsonld` | JSON-LD context mapping short names to URIs |
| `/health`         | Liveness probe                              |

Everything except `/health` negotiates on `Accept`. HTML when the caller asks
for it, JSON-LD otherwise — so `curl` gets data rather than markup.

## Development

```sh
cargo run                                   # http://0.0.0.0:8080
LISTEN_ADDR=127.0.0.1:8099 cargo run        # somewhere else
cargo clippy --all-targets -- -D warnings
cargo fmt
```

Terms live in [`src/terms.rs`](src/terms.rs) as a compiled-in constant. There
is no database — the deployed binary cannot serve a term the source does not
define, which is the property that makes the namespace trustworthy.

## Deployment

```sh
fly deploy
```

Fly.io, Johannesburg, one always-warm machine. The DNS record for
`vocab.bundu.org` must point at this app before any published `additionalType`
URI will resolve.

## History

This repository previously held a scaffold for a Bundu identity gateway. That
work was superseded by Mukoko Account, and the repository was repurposed
rather than archived — the vocabulary is the durable artefact that outlives
whichever service happens to be issuing tokens.

## Licence

**Code: Apache 2.0** — see [LICENSE](LICENSE). The server is software, and
Apache's patent grant is the right instrument for it.

**The vocabulary: CC0 1.0** — see [LICENSE-VOCAB](LICENSE-VOCAB). The terms and
their definitions are content, not code, and a vocabulary's entire value is
adoption. Attribution obligations inside machine-readable JSON-LD are
impractical to discharge — there is nowhere in an embedded `additionalType` URI
to put credit — and share-alike clauses make lawyers nervous about any metadata
that touches them. schema.org's CC-BY-SA is a live example of that friction.

CC0 says: take these terms, use them anywhere, no legal review needed. For
terms describing African family and informal-economy structures that we want
other systems to adopt, that is the point.
