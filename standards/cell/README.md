# Bundu Cell (.cell)

**Version 0.1.0, draft.** Proposed, not adopted. Nothing in it is final until the owner approves
the scope ([#107](https://github.com/bundu-labs/bundu-standards/issues/107)).

A cell is one record that an entity owns (a person, a household or a business): JSON-LD with
Markdown inside, the same in a `.cell` file or a database row, with its facts and its words kept
from drifting apart.

## Where it lives

The spec, the prototype validator and its stress tests live in
**[siafuDB/cell](https://github.com/siafuDB/cell)** (owner, 2026-11-17). The .cell maps directly
to SiafuDB: on the device and at the edge a cell is a SiafuDB node keyed by its `@id`, and the
SiafuDB Graph Sync Protocol carries cells from device to cloud to pod without reshaping them.

- Spec: [SPEC.md](https://github.com/siafuDB/cell/blob/main/SPEC.md)
- Register entry: `cell` in [register/register.yaml](../../register/register.yaml)
- How the Foundation proposes a standard:
  [Open standards playbook](https://docs.bundu.org/bundu/playbooks/open-standards/)

This page stays so that links to standards.bundu.org/cell/ keep working.
