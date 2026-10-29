# AGENTS.md: bundu-standards

Instructions for any AI agent working in this repository, and the rule every
other Bundu, Mukoko and Nyuchi repository follows. Where this file and the code
disagree, the code is the fact.

## The rule (owner, 2026-10-29)

**If a standard does not exist, we build one and publish it here. Otherwise we
adopt the global standard.** Everything we build is accessible because of it.

- We create nothing proprietary. A model, field name, unit, code or API shape
  that a published standard already covers is taken from that standard, not
  invented.
- When we pull in data from another source that does not match a standard, we
  do the work: we transform it at the edge into the standard, and only the
  standard form is stored and served. A provider's schema, names and units
  never become ours.
- When no standard covers it, we write one, publish it in `standards/` and its
  terms in the vocabulary, and say so in the register.
- The test for any change: could someone integrate with this without knowing
  who our providers were? If not, it is not finished.

This is the mission: open infrastructure and architecture. Building around a
single provider, or in our own shape, breaks it.

## How to apply it

1. **Look before you name.** Before adding a field, unit, code list or record
   shape, search the register (`register/register.yaml`) and then the world
   (schema.org, W3C, OGC, ISO, WMO, IETF, SDMX, DCAT, CF). If a standard fits,
   propose it in the register and use it.
2. **Name the standard by register id** wherever a model or playbook uses it.
   Never restate its version or steward by hand.
3. **A standard that is missing is a gap to fill, not a reason to invent.**
   Add it to the register as `proposed`, and ask for the owner's adoption.
4. **Convert at the edge.** Write the transformation (names, units, ids, times,
   provenance, licence) and test it. Keep a bridging adapter only where an older
   store needs it, label it temporary and name what removes it.
5. **Do not mirror a provider's table.** Fields named after a provider
   (`zimdata…`, a vendor's `Kph` suffix) are a defect in our model.

## Where things live

See `docs/where-things-live.md`. The register of adopted standards is
`register/register.yaml` and nothing else keeps a list. After editing it, run
`python3 register/tools/register.py write`; CI runs `check`.

## Commands

```sh
python3 register/tools/register.py write    # regenerate REGISTER.md and the seed
python3 register/tools/register.py check    # lint; generated files must be current
python3 register/tools/register.py live FILE # compare a database export with the register
python3 -m unittest discover register/tools
cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings && cargo test
```

## Branching

Pull requests go to `staging`. Never push to `main`. A merge into `staging` is a
patch, a release to `main` is a minor. Add a `CHANGELOG.md` entry for every
change.
