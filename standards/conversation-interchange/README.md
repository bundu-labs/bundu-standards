# Bundu Conversation Interchange (BCI)

**Version 0.1.0, draft.** Not adopted. Drafted on the owner's instruction (2026-11-11);
nothing here is published until the owner approves its scope. Licence: CC BY 4.0 for the text,
CC0 for the schema and context (like the vocabulary).

## Why this exists

The rule in `AGENTS.md`: adopt the global standard; where none exists, write one. A search on
2026-11-11 found standards for the _parts_ of an AI assistant and none for what a person keeps in it:

| Part                            | Standard                                                  | Covers                                                        |
| ------------------------------- | --------------------------------------------------------- | ------------------------------------------------------------- |
| Skills                          | Agent Skills (agentskills.io; Agentic AI Foundation)      | A skill: a folder with a `SKILL.md`                           |
| One exchange on the wire        | Open Responses (openresponses.org; Agentic AI Foundation) | Request, response, typed items, streaming                     |
| Project instructions            | AGENTS.md (Agentic AI Foundation)                         | Plain-Markdown instructions for an agent                      |
| Tools and context               | MCP; agent to agent: A2A (both in the register)           | Calling tools, agents calling agents                          |
| **Stored conversations**        | **none**                                                  | ChatGPT and Claude each export their own `conversations.json` |
| **Projects, artifacts, memory** | **none**                                                  | Dropped or lost when a person moves assistant                 |

So a person's history, the answers they kept, their projects and what the assistant remembers about
them belong to a product, not to the person. BCI is the record of those things, in a form any
assistant can write and read. Its test is the one in `AGENTS.md`: could someone move their
history without knowing which product it came from?

## What it is, in one paragraph

A stream of JSON records, one per line (NDJSON, media type `application/x-ndjson`; a `.bci.ndjson`
file, or zipped with the files artifacts point to). The first record is a `Header`. The rest are
`Conversation`, `Message`, `Project`, `Artifact` and `MemoryEntry` records, validated by
[`bci.schema.json`](bci.schema.json) (JSON Schema 2020-12) and readable as JSON-LD with
[`context.jsonld`](context.jsonld). [`examples/minimal.ndjson`](examples/minimal.ndjson) is a
complete stream; `test_examples.py` validates it and checks what the schema must refuse.

## Principles

1. **Adopt, do not reinvent.** A `Message` holds an **Open Responses item** unchanged in `item`: a
   `message` with its `role` and `content`, a tool call, or an extension item. BCI adds no wire shape.
   Skills are named by the Agent Skills `name` field, not embedded. Project `instructions` are plain
   Markdown in the AGENTS.md style. Citations use schema.org property names. Identifiers are UUIDs
   (RFC 9562), times RFC 3339 UTC, languages BCP 47, media types IANA, licences SPDX, provenance
   C2PA. Where one of these is replaced, BCI follows.
2. **The person owns it.** A stream names its owner only by an opaque identifier, never an email
   address or a name. A reader must not infer more about the person than the stream states.
3. **Storing is not training.** `Message.trainable` can only be `false`. Being stored, exported or
   synced never makes content training data. The platform rules that decide training are outside
   BCI: a model's output may train a model only if its `provenance.licenseClass` is `open_weight`,
   its `scope` is not `personal`, and the person did not use a channel or account that forbids it.
4. **Personal stays personal.** `scope: personal` content is never sent to third-party inference.
   BCI records the scope; routing is the product's duty. A stream with `personal` content is the
   person's own and moves only with their action.
5. **Nothing is silently dropped.** A reader keeps extension members it does not know (named
   `<implementor>:<name>`) when it passes a record on, and reports what it could not read.

## Records

| Record         | Purpose                                      | Notes                                                                                                             |
| -------------- | -------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| `Header`       | Version, export time, producer, opaque owner | First line. `bci` is `0.1.0`.                                                                                     |
| `Conversation` | A thread                                     | At most one `project`. `updated` is the last message's time.                                                      |
| `Message`      | One turn, in order                           | `sequence` is 1, 2, 3 without gaps. `scope` is required. `item` is Open Responses.                                |
| `Project`      | A named group of conversations               | `instructions`, `skills`, `pinned`.                                                                               |
| `Artifact`     | An answer or file the person kept            | `text` or `contentRef`; `derivedFrom`; `sha256`; `citations`; `provenance`; `license`. Survives its conversation. |
| `MemoryEntry`  | One thing the assistant remembers            | `consent` is `stated` or `confirmed`. There is no value for an inference the person has not seen.                 |

Rules a conforming writer holds to (the schema checks the shape; these need the whole stream):

- Every `Message.conversation` and `Conversation.project` names a record in the stream or one the
  writer says it omitted in the `Header`.
- A `Conversation` opens with a person's message; an answer follows a question.
- `messageCount` equals the number of `Message` records for it, when both are present.
- Deleting a `Project` keeps its conversations; deleting a `Conversation` keeps its artifacts.
- "Delete all" removes every record of the owner, in the store and in any copy the product made.

## Conformance

A **writer** conforms if every record it writes validates and the rules above hold. A **reader**
conforms if it accepts every valid stream, keeps unknown extensions, and refuses a stream whose
`Header.bci` major version it does not know. Run `python3 -m unittest
standards/conversation-interchange/test_examples.py` for the reference checks. An implementation's own
test suite should run the example stream through its importer and exporter and compare.

## Mapping from what exists today

A one-way importer from a vendor export is a conversion at the edge (`AGENTS.md`, rule 4): names,
ids and times are converted to BCI and only the BCI form is stored. A vendor's id becomes an
extension (`<vendor>:id`) if it must be kept. Fields BCI has no place for are not invented: they are
dropped and reported.

## Open questions (for the owner and the standards session)

1. **Name and home.** `bci` is a working name. Is the canonical URL
   `standards.bundu.org/conversation-interchange/`, and is the publisher of record the Bundu Foundation?
2. **Vocabulary.** `context.jsonld` uses `schema:Conversation`, `schema:Message` and new terms
   `bundu:Project`, `Artifact`, `MemoryEntry`, `scope`, `pinned`, `instructions`, `skill`,
   `provenance`, `consent`. Those terms need adding to `vocab.bundu.org` (`src/terms.rs`) before 0.1.0 is
   published; schema.org has `Project` only as a pending type. Prefer a schema.org type where one is
   stable.
3. **Versions and licences of the standards it builds on** were read on 2026-11-11 from their
   pages, which state no version number for Open Responses or Agent Skills; the repositories and
   licences must be read at source before adoption (see the register entries, `proposed`).
4. **Binary artifacts.** `contentRef` points at a file in an archive. Whether the archive is a ZIP
   or an OCI artifact, and how the digest is checked, is not decided.
5. **Branches and edits.** ChatGPT-style conversation branching and message edits are not modelled.
   A branch could be a new `Conversation` with a `bci:forkOf` extension; decide once a second
   implementation needs it.
6. **A harness profile.** The harness (the loop around a model: context policy, tool-call dialect,
   permissions, session log, sandbox) has no published standard either. That is a separate
   standard, to be scoped after this one; BCI's append-only, ordered `Message` stream is its natural
   session log.

## First implementation

Shamwari (shamwari.ai) stores its web and Lite chats in the `shamwari` MongoDB database through the
Servers API (`nyuchi/nyuchi-servers-api`, `docs/messages-api.md`, PR #37). Its field names and ids
are provisional and move to BCI: UUID v7 ids, RFC 3339 times, an `item` per message. The mobile app
keeps chats on the device and offers BCI export and import.
