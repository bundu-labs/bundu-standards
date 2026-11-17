# The ecosystem record

The one place the Bundu ecosystem's identity, four pillars, shared foundations, terms and
products are written down (#105, #106). Websites, documentation, APIs and agents read it;
none of them keeps its own version.

| File                                           | What it is                                   |
| ---------------------------------------------- | -------------------------------------------- |
| `ecosystem.yaml`                               | The source. The only file edited by hand.    |
| `ECOSYSTEM.md`                                 | Generated: the record as tables, for people. |
| `ecosystem.jsonld`                             | Generated: the record as schema.org JSON-LD. |
| `../workers/standards/public/ecosystem.jsonld` | Generated: served at standards.bundu.org.    |

```sh
deno run --allow-read --allow-write tools/ecosystem.ts write   # after editing ecosystem.yaml
deno run --allow-read tools/ecosystem.ts check                 # CI: valid and up to date
deno test --allow-read tools/ecosystem_test.ts
```

## Rules

- **The owner approves every change**, through a pull request. A change bumps the record's
  `version` and `last_reviewed`.
- **The Ubuntu doctrine is not restated here.** The five pillars, five principles and the
  seven engineering constraints live in Mzizi (`get_doctrine`); this record names the
  constraints by id only, and the check fails unless all seven are listed in order.
- **A product's status changes here first**, with `status_source` saying where it was read.
  Nothing planned or in development is described as live.
- **Banned terms** (listed in `ecosystem.yaml`) fail the check if they appear in any name,
  role, audience or description.
