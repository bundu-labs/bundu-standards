# Harness conformance checklist

**Status:** complete as of 2026-11-17, on the owner's request "finish the harness checklist". It is
a checklist, not a standard. It has no schema, no vocabulary terms and no register entry of its own,
and it is not published to standards.bundu.org. Tracked in
[#50](https://github.com/bundu-labs/bundu-standards/issues/50).

A _harness_ is the loop around a model: it builds the context, calls the model, runs tools, asks
for permission, keeps a log and may run code in a sandbox. Following `AGENTS.md` ("adopt the global
standard; where none exists, write one"), this checklist does not define a harness. It says what a
conforming Bundu harness must expose, names the existing standard to use for each part, and states
the gaps. Claims are labelled **measured** (read at source on the date and URL given) or
**inferred** (my reading; verify before relying on it).

## 1. Parts that already have a standard: use it

Every row below is a standard in the register (`register/register.yaml`). The version and licence are
the register's, each read at source on the date it records.

| Part                 | A conforming harness must…                                       | Use (register id, status)                                           | Version / licence                                                                                                     |
| -------------------- | ---------------------------------------------------------------- | ------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| Tools and context    | call tools and read context through the Model Context Protocol   | MCP (`mcp`, adopted)                                                | revision 2026-07-28                                                                                                   |
| Instructions         | read project instructions from `AGENTS.md`                       | AGENTS.md (`agents_md`, adopted)                                    | unversioned; MIT                                                                                                      |
| Skills               | load reusable skills as `SKILL.md` folders                       | Agent Skills (`agent_skills`, adopted)                              | unversioned (no tags); Apache-2.0 code, CC-BY-4.0 docs                                                                |
| Model wire shape     | exchange one request and response as Open Responses items        | Open Responses (`open_responses`, adopted)                          | pin the dated OpenAPI file (2026-04-24), not `openapi.json`; Apache-2.0                                               |
| Agent to agent       | delegate to other agents through A2A                             | A2A (`a2a`, adopted)                                                | 1.0.0                                                                                                                 |
| Sandbox lifecycle    | create, run in and destroy a sandbox through a standard protocol | OpenSandbox (`opensandbox`, **proposed** 2026-11-17)                | Lifecycle API 0.1.0 (pre-v1), Execd API 1.0.0; Apache-2.0; an AAIF project. **Measured** 2026-11-17 at commit f1e4ac0 |
| Editor to agent      | if it talks to an editor, use ACP                                | Agent Client Protocol (`agent_client_protocol`, adopted 2026-11-15) | protocol v1; Apache-2.0; interim Zed and JetBrains governance, no foundation host                                     |
| Stored conversations | export and import chats, projects, saved answers                 | BCI (`bci`, adopted 2026-11-15)                                     | 0.1.0; first implementation nyuchi/nyuchi-servers-api#50; vocabulary terms pending in vocab.bundu.org                 |
| Session log          | write each session as an ordered trajectory                      | ATIF (`atif`, adopted 2026-11-14)                                   | ATIF-v1.8 (RFC 0001, Active, April 2026); Apache-2.0                                                                  |

ACP's and BCI's adoption is in #70; until it merges, the register on `staging` still shows them
proposed. OpenSandbox is the one row whose standard is not yet adopted; adopting it is the owner's
decision. Bundu runs no sandbox today, so the row applies only once a Bundu harness runs code.

## 2. Parts a second gap check found partly covered

Read at source 2026-11-11. ATIF: `harbor-framework/harbor`, `rfcs/0001-trajectory-format.md` (Apache-2.0, last push 2026-11-11, current
format `ATIF-v1.8`, no spec release other than the RFC's version history). OpenTelemetry GenAI:
`open-telemetry/semantic-conventions-genai` `docs/gen-ai/` (Apache-2.0, last push 2026-11-11, every
document status **Development**, no release cut).

| Part                   | What exists                                                                                                                                                                                                                                                                         | Evidence                                                                   |
| ---------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| **Session log**        | ATIF-v1.8 is a session-log interchange format: ordered `steps` with `source` (system, user, agent), `message`, `tool_calls`, `observation`, `metrics`, embedded subagent trajectories and audio. **Measured.** It is Harbor's own format, written for evaluation and training runs. | ATIF RFC, sections II and version history                                  |
| **Context management** | ATIF section VII is a _convention_ (`step.extra.context_management`) that records compaction, pruning and injection, with a normative `replace` boundary rule, plus `is_copied_context`. **Measured.** It records what happened; it does not say what a harness should keep.        | ATIF RFC, section VII                                                      |
| **Telemetry**          | OTel GenAI defines spans for inference, embeddings, retrieval, `execute_tool`, `create_agent`, `invoke_agent`, memory operations (`gen_ai.memory.client`) and agent-skill loading, with `gen_ai.conversation.id`. **Measured.** Development status: names may change.               | `README.md`, `gen-ai-spans.md`, `execute-tool.md`, `gen-ai-agent-spans.md` |
| **Permissions**        | ACP v1 has `session/request_permission` (coordinator, 2026-11-11). No permission field appears in the ATIF RFC or in six OTel GenAI documents searched by keyword (`permission`, `approval`). **Measured for those files only.**                                                    | keyword search of the files named                                          |

## 3. Gaps, stated plainly

1. **Context policy** (what a harness must keep, drop or summarise, and who may see it): ATIF records a
   transformation after the fact and OTel does not cover it. **Inferred** gap.
2. **Permission model** (what a person approves, remembered grants, scope): only ACP's request call
   exists, and only between an editor and an agent. **Inferred** gap for harnesses generally.
3. **Cross-harness tool-call dialects**: Open Responses gives one wire shape; harnesses still differ
   in what they emit and accept. Not verified against DeepSeek, Qwen, GLM, Codex, Gemini CLI,
   OpenHands or goose session schemas. **Not verified.**
4. **Portable session log: decided.** The owner adopted ATIF on 2026-11-14 (register id `atif`).
   BCI's ordered `Message` stream maps onto ATIF steps rather than defining its own log; the mapping
   is an open item on #48. ATIF is Harbor-hosted and evaluation-oriented, with no foundation host,
   so the adoption is of the format, not an endorsement of its steward.

Not checked: ISO/IEC SC 42 and IEEE items, LangGraph checkpoints, any importer actually losing data.

## 4. Bundu's own rules (not an external standard)

These are Bundu's rules for a conforming harness, taken from Shamwari. They are requirements here,
not part of any standard above.

- **Personal-scope data stays on the person's device and under their keys.** It is not sent to a
  third-party inference service.
- **Only `open_weight` provenance may train.** Storing a conversation is not consent to train on it.
- **No memory entry without the person having stated or confirmed it.**

## 5. How to use this

A harness passes when, for every row of section 1 that applies to it, it uses the named standard,
and it satisfies section 4. Section 3 lists what it cannot yet be checked against. Open items are in
the tracking issue (#50).
