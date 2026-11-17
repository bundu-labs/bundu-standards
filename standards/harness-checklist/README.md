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
| Sandbox lifecycle    | create, run in and destroy a sandbox through a standard protocol | OpenSandbox (`opensandbox`, adopted 2026-11-17)                     | Lifecycle API 0.1.0 (pre-v1), Execd API 1.0.0; Apache-2.0; an AAIF project. **Measured** 2026-11-17 at commit f1e4ac0 |
| Editor to agent      | if it talks to an editor, use ACP                                | Agent Client Protocol (`agent_client_protocol`, adopted 2026-11-15) | protocol v1; Apache-2.0; interim Zed and JetBrains governance, no foundation host                                     |
| Stored conversations | export and import chats, projects, saved answers                 | BCI (`bci`, adopted 2026-11-15)                                     | 0.1.0; first implementation nyuchi/nyuchi-servers-api#50; vocabulary terms stable in vocab.bundu.org                  |
| Session log          | write each session as an ordered trajectory                      | ATIF (`atif`, adopted 2026-11-14)                                   | ATIF-v1.8 (RFC 0001, Active, April 2026); Apache-2.0                                                                  |

ACP and BCI were adopted in #70 (merged 2026-11-17), and OpenSandbox by the owner on 2026-11-17.
Every row's standard is now adopted. OpenSandbox's Lifecycle API is pre-v1, so a harness pins the
OpenAPI files it builds against. Bundu runs no sandbox today, so that row applies only once a Bundu
harness runs code.

### The Agentic AI Foundation's projects

The AAIF lists ten projects (aaif.io/projects, read 2026-11-17). **Measured.**

- **In the table above (6):** MCP, AGENTS.md, Agent Skills, Open Responses, A2A and OpenSandbox.
- **OpenEnv: adopted** (`openenv`, owner 2026-11-17). These are environments for training agents
  with reinforcement learning, from Hugging Face, under BSD-3-Clause. Its environment spec (RFC 002)
  is still In Review, so a trainer pins the release. It becomes a harness row when Bundu trains
  agents.
- **Implementations, not standards (3), adopted into the build** (owner, 2026-11-17: "the software
  we should be adopting as well … The agent router will [be] our routing"):
  - Agent Router, a gateway on Envoy for models and MCP tools (v1.2.1, formerly Envoy AI Gateway):
    Shamwari's model routing;
  - agentgateway, a gateway for agents, models and tools (v1.7.0): MCP and A2A traffic;
  - goose, an agent (see the dialect table in section 3): a client that can use Shamwari.

  All three are Apache-2.0. The register lists standards, so they are noted here and not there.
  Where each sits in Shamwari, always behind its own Rule 1 scope gate, is in
  `shamwari-ai/shamwari` `docs/aaif-runtime.md` (shamwari-ai/shamwari#325).

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

## 3. Gaps, checked at source 2026-11-17

Each gap below was checked at source on 2026-11-17. Repositories were read at these commits: Codex
05ae1b0, Gemini CLI 4fa64e7, OpenHands software-agent-sdk 06f643c, goose eb4f412, MCP 05aeb07, A2A
b30ab16, OpenTelemetry semantic-conventions-genai e6658f1, Harbor 110c71b, Open Responses dce9ddb and
LangGraph 02f2c2e.

1. **Context policy: partly covered.** This is what a harness must keep, drop or summarise, and who
   may see it.
   - Open Responses 2026-04-24 has `truncation` ("auto": the server "MAY truncate earlier parts" and
     "SHOULD apply sensible heuristics"), `previous_response_id` (the server "MUST preserve the
     semantic order") and a `/responses/compact` endpoint. That is provider-side policy, not
     harness policy.
   - ATIF section VII reconstructs the context after the fact.
   - OTel GenAI says instrumentations "SHOULD NOT capture" inputs and outputs by default, which
     covers telemetry only.
   - No rule for what a harness keeps, or for who may see it, was found in MCP. **Measured.**
2. **Permission model: partly covered.**
   - MCP 2026-07-28 (`server/tools.mdx`): "there **SHOULD** always be a human in the loop with the
     ability to deny tool invocations".
   - MCP elicitation (`client/elicitation.mdx`): clients **MUST** "provide clear decline and cancel
     options".
   - MCP tool annotations (`destructiveHint` and the others) are hints that clients **MUST** treat
     as untrusted unless the server is trusted.
   - A2A 1.0.0 section 7.6: an agent that needs human approval before a destructive action
     **MUST** move to `TASK_STATE_AUTH_REQUIRED`. Section 7.6.4 leaves "scope, representation,
     validity, or revocation" undefined.
   - ACP has `session/request_permission`, between an editor and an agent only.
   - **Measured.** No standard defines remembered grants, grant scope or how an approval is
     recorded.
3. **Cross-harness tool-call dialects: still a gap.** Four wire dialects are in use: Chat
   Completions `tool_calls`, the Responses API's `function_call` items, Anthropic `tool_use` and
   Gemini `functionCall`. None of the products below claims Open Responses conformance. The Codex
   CLI accepts only the Responses wire, which Open Responses is based on. **Measured** for the
   repositories and vendor pages named. The Qwen Responses page was read from search excerpts only.

   | Product    | Tool-call wire                                                    | Sessions on disk                         | MCP | AGENTS.md     | Skills | ACP                |
   | ---------- | ----------------------------------------------------------------- | ---------------------------------------- | --- | ------------- | ------ | ------------------ |
   | DeepSeek   | Chat Completions; Anthropic Messages; Responses (no `store`)      | n/a                                      | n/a | n/a           | n/a    | n/a                |
   | Qwen       | Chat Completions; Responses (OpenAI-compatible); DashScope native | n/a                                      | n/a | n/a           | n/a    | n/a                |
   | GLM (z.ai) | Chat Completions; Anthropic Messages                              | n/a                                      | n/a | n/a           | n/a    | n/a                |
   | Codex CLI  | Responses only (`wire_api = "chat"` is refused)                   | JSONL rollout files                      | yes | yes           | yes    | through an adapter |
   | Gemini CLI | Gemini `functionCall`                                             | JSONL chat files                         | yes | if configured | yes    | yes, as an agent   |
   | OpenHands  | Chat Completions or Responses, through LiteLLM                    | a state file and one JSON file per event | yes | yes           | yes    | yes, as a client   |
   | goose      | its own message type, translated per provider                     | SQLite                                   | yes | yes           | yes    | yes, as an agent   |

   So the harnesses agree on MCP, AGENTS.md, Agent Skills and, increasingly, ACP. They disagree on
   the model wire and on how a session is stored. BCI (stored conversations) and ATIF (session log)
   are the register's answer to the second disagreement. Nothing in the register answers the first
   beyond adopting Open Responses.

4. **Portable session log: decided.** The owner adopted ATIF on 2026-11-14 (register id `atif`).
   BCI's ordered `Message` stream maps onto ATIF steps rather than defining its own log; the mapping
   is an open item on #48. ATIF is Harbor-hosted and evaluation-oriented, with no foundation host,
   so the adoption is of the format, not an endorsement of its steward.
   - ISO/IEC 24970 "AI system logging" (JTC 1/SC 42) is at stage 60.00, under publication. It is
     the nearest formal standard and has not been read here. **Measured** status only.

Checked and found not to be harness standards:

- IEEE P3119 is about AI procurement. **Measured.**
- IEEE P3394 "Large Language Model Agent Interface" is an active PAR from 2023-09-21, not
  published, and its page does not mention tool calls, logs or context. **Measured.**
- ISO/IEC 42105 (human oversight) and ISO/IEC 42001 (a management system) are not harness formats.
  **Measured** for 42105's stage, **inferred** for the rest.
- ISO/IEC AWI TR 27258 (multi-agent interactions) is an early technical report. **Measured.**
- LangGraph checkpoints (`v: 1`) are a versioned internal persistence format, serialised as Python
  types, not an interchange format. **Inferred** from the source.

## 4. Bundu's own rules (not an external standard)

These are Bundu's rules for a conforming harness, taken from Shamwari. They are requirements here,
not part of any standard above.

- **Personal-scope data stays on the person's device and under their keys.** It is not sent to a
  third-party inference service.
- **Only `open_weight` provenance may train.** Storing a conversation is not consent to train on it.
- **No memory entry without the person having stated or confirmed it.**

## 5. How to use this

A harness passes when, for every row of section 1 that applies to it, it uses the named standard,
and it satisfies section 4. Section 3 lists what it cannot yet be checked against. Where section 3
names a partial standard (MCP's human-in-the-loop rules, A2A's `AUTH_REQUIRED`), a harness follows
it.

What is left is the owner's, not the checklist's:

1. Whether to write a Bundu profile for the two partly covered gaps: context policy, and permission
   grants. `AGENTS.md`'s rule says to write one where none exists. The checklist does not start one.
2. Running this checklist against Shamwari's harness (`shamwari-ai/shamwari`). The data-plane audit
   is nyuchi/nyuchi-servers-api#48.
