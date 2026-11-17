# Bundu Cell (.cell)

**Version 0.1.0, draft.** Proposed, not adopted. Scoped on the owner's instruction
(2026-11-17, [#107](https://github.com/bundu-labs/bundu-standards/issues/107)). Nothing here is
final until the owner approves it. Licence: CC BY 4.0 for the text, CC0 for any schema and
context.

A cell is one record that a person owns, such as a health record, a contact, a receipt or a
note. It is JSON-LD with Markdown inside. It is stored as the same bytes in a `.cell` file or in
a database row. A pod is the set of cells that belong to one person. The Nyuchi Honeycomb is the
platform that holds pods; it is in development.

## Why this exists

The rule in `AGENTS.md`: adopt the global standard; where none exists, write one. Every part of
a cell already has a standard. What none of them defines is one self-contained record that:

- keeps its facts and its prose from drifting apart;
- works offline, with no server needed to read or check it;
- moves between a file and a database without conversion.

That envelope is the proposed standard. Everything inside it is adopted from others:

| Need                       | Standard                                                      | Steward       |
| -------------------------- | ------------------------------------------------------------- | ------------- |
| Linked-data structure      | [JSON-LD 1.1](https://www.w3.org/TR/json-ld11/)               | W3C           |
| Shared vocabulary          | [schema.org](https://schema.org)                              | schema.org CG |
| Prose                      | [CommonMark](https://spec.commonmark.org)                     | CommonMark    |
| Canonical form for hashing | [RFC 8785 (JCS)](https://www.rfc-editor.org/rfc/rfc8785)      | IETF          |
| Signatures                 | [VC Data Integrity](https://www.w3.org/TR/vc-data-integrity/) | W3C           |
| Pods and access control    | [Solid Protocol](https://solidproject.org/TR/protocol)        | W3C Solid CG  |
| Encryption at rest (open)  | [JWE (RFC 7516)](https://www.rfc-editor.org/rfc/rfc7516)      | IETF          |

It serves two of the seven engineering constraints directly: personal-data-sovereignty and
connectivity-gradient-design.

## A cell

```json
{
  "@context": ["https://schema.org", "https://standards.bundu.org/cell/v0.1"],
  "@type": "MedicalCondition",
  "@id": "urn:cell:01JD3Q7Z8K2V5M",
  "name": "Asthma",
  "possibleTreatment": { "@type": "Drug", "name": "Salbutamol inhaler" },
  "dateCreated": "2024-03-11",
  "text": "Diagnosed in {{dateCreated}}. Carry the {{possibleTreatment.name}} when walking far.",
  "cell:version": 3,
  "cell:previous": "sha256-…",
  "cell:hash": "sha256-…",
  "proof": { "type": "DataIntegrityProof", "cryptosuite": "eddsa-jcs-2022", "proofValue": "z…" }
}
```

| Field                           | Purpose                                                               |
| ------------------------------- | --------------------------------------------------------------------- |
| `@context`, `@type`             | schema.org meaning, so any system can read the cell                   |
| `@id`                           | The stable identity, and the database key                             |
| Typed fields                    | The facts, each stated once                                           |
| `text`                          | CommonMark prose; it may name a fact only as `{{field}}`              |
| `cell:version`, `cell:previous` | The version chain: each version names the hash of the one before      |
| `cell:hash`                     | SHA-256 of the cell in RFC 8785 form, without `cell:hash` and `proof` |
| `proof`                         | A W3C Data Integrity signature by the owner's key                     |

The `cell:` names and the context URL are draft. The context is not published yet.

## One shape, two homes

| Home     | Stored as                                                                        | Key                    |
| -------- | -------------------------------------------------------------------------------- | ---------------------- |
| File     | `name.cell`, UTF-8 JSON, media type `application/vnd.bundu.cell+json` (proposed) | The path; `@id` inside |
| Database | One row: an `@id` column and the cell in a JSON or JSONB column                  | `@id`                  |

Exporting writes the JSON column to a file unchanged; importing stores the file under its
`@id`. The hash is the same in both homes, so a round trip can be checked.

## Why it can't diverge

A validator refuses a cell that breaks any of these rules:

1. Each fact is stated once, in a typed field. The `text` restates none of them.
2. The `text` names a fact only as `{{field}}`, and every reference resolves to a field.
3. Readers render each `{{field}}` from the field, so changing a fact changes every sentence
   that uses it.
4. `cell:hash` covers the fields and the text together; editing either without re-hashing makes
   the cell invalid.
5. A new version names the hash of the one before. History can grow; rewriting it breaks the
   chain.

Rule 1 is the hard one to check by machine. The spec will define a lint for dates, numbers and
names that appear in `text` but not as references.

## Open questions

- Size and bundling: is a large record (a scan, a photo) one cell, or a cell that points at a
  separate file?
- Encryption at rest: JWE around the whole cell, or only around sensitive fields?
- Media type: register `application/vnd.bundu.cell+json` with IANA now, or after adoption?
- Which repository holds the Honeycomb code that reads and writes pods?
