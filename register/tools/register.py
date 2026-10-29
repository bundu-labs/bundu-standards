"""The standards register: one source, three generated outputs, one live check.

    python3 register/tools/register.py write        # regenerate REGISTER.md tables + the seed
    python3 register/tools/register.py check        # lint, and fail if a generated file is stale (CI)
    python3 register/tools/register.py live FILE    # compare a live export to the register

`register/register.yaml` is the only place the list of adopted standards is
edited. Everything else is generated from it:

- the tables in `register/REGISTER.md` (between the GENERATED markers);
- `register/seed/integrations.standards.json`, one document per standard,
  shaped like the `integrations.standards` collection (MongoDB Extended JSON).

`live` is the check that was missing: it compares the documents actually in
`integrations.standards` with the seed, so a database that was never loaded, or
was loaded and then edited by hand, shows up as drift. FILE is a JSON array or
newline-delimited JSON from `mongoexport` / the Atlas or Nyuchi Data tools.
Nothing here connects to a database or writes to one.
"""

from __future__ import annotations

import json
import re
import sys
import unicodedata
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[2]
REGISTER_YAML = ROOT / "register" / "register.yaml"
REGISTER_MD = ROOT / "register" / "REGISTER.md"
SEED_JSON = ROOT / "register" / "seed" / "integrations.standards.json"
BEGIN = "<!-- BEGIN GENERATED: tables from register/register.yaml by register/tools/register.py -->"
END = "<!-- END GENERATED -->"
SEED_DATE = "2026-10-02T00:00:00Z"  # the owner's adoption decision; fixed so the seed is reproducible
FIELD = re.compile(r"^([a-z]+\.[A-Za-z]+(?:\.[A-Za-z_]+)?)(\s.*)?$")
CATEGORIES = {"data_format", "protocol", "vocabulary", "identifier_scheme", "encoding", "specification", "rfc"}
# Fields a live document is compared on. Dates are left out: a re-load changes them without changing meaning.
COMPARED = (
    "displayName", "category", "description", "versionCommitment", "specificationUrl",
    "documentationUrl", "stewardOrganisation", "usageNotes", "isActive", "bundu",
)


def load() -> dict:
    return yaml.safe_load(REGISTER_YAML.read_text())


# --- markdown, padded the way prettier pads tables --------------------------


def _width(s: str) -> int:
    return sum(2 if unicodedata.east_asian_width(ch) in "WF" else 1 for ch in s)


def table(header: list[str], rows: list[list[str]]) -> str:
    cells = [header] + rows
    widths = [max(3, max(_width(r[i]) for r in cells)) for i in range(len(header))]

    def line(r):
        return "| " + " | ".join(c + " " * (w - _width(c)) for c, w in zip(r, widths)) + " |"

    out = [line(header), "| " + " | ".join("-" * w for w in widths) + " |"]
    out += [line(r) for r in rows]
    return "\n".join(out)


def _cell(s) -> str:
    return " ".join(str(s).split()).replace("|", "\\|")


def evidence_cell(e: dict) -> str:
    parts = [_cell(e["decided"])]
    fields = [f"`{_cell(x)}`" if FIELD.match(x) and not x.startswith("and ") else _cell(x) for x in e.get("evidence") or []]
    if fields:
        parts.append("cited by " + ", ".join(fields))
    return "; ".join(parts)


def render_tables(reg: dict) -> str:
    out = []
    for key, title in reg["domains"].items():
        rows = [
            [
                f"[{_cell(e['name'])}]({e['url']})",
                _cell(e["commitment"]),
                _cell(e["current"]),
                _cell(e["governs"]),
                "**Adopted**" if e["status"] == "adopted" else "Proposed",
                evidence_cell(e),
            ]
            for e in reg["standards"]
            if e["domain"] == key
        ]
        if not rows:
            continue
        out.append(f"### {title}\n")
        out.append(table(["Standard", "Commitment", "Current at source", "Governs", "Status", "Evidence"], rows))
        out.append("")
    adopted = [e["name"] for e in reg["standards"] if e["status"] == "adopted"]
    proposed = [e["name"] for e in reg["standards"] if e["status"] != "adopted"]
    out.append(f"**Adopted ({len(adopted)}):** " + "; ".join(adopted) + ".\n")
    out.append(f"**Proposed ({len(proposed)}):** " + ("; ".join(proposed) + "." if proposed else "none.") + "\n")
    return "\n".join(out).rstrip() + "\n"


def splice(md: str, tables: str) -> str:
    if BEGIN not in md or END not in md:
        raise ValueError(f"{REGISTER_MD} has no generated-table markers")
    head, rest = md.split(BEGIN, 1)
    _, tail = rest.split(END, 1)
    return f"{head}{BEGIN}\n\n{tables}\n{END}{tail}"


# --- seed documents ------------------------------------------------------------


def seed_docs(reg: dict) -> list[dict]:
    docs = []
    for e in reg["standards"]:
        notes = [_cell(e["notes"])] if e.get("notes") else []
        fields = [x for x in e.get("evidence") or [] if FIELD.match(x)]
        if fields:
            notes.append("Cited by: " + ", ".join(fields) + ".")
        docs.append(
            {
                "_id": f"standard_{e['id']}",
                "_schemaVersion": "v3.1",
                "displayName": e["name"],
                "category": e["category"],
                "description": _cell(e["governs"]),
                "versionCommitment": str(e["commitment"]),
                "specificationUrl": e.get("url"),
                "documentationUrl": e.get("docs"),
                "stewardOrganisation": e.get("steward"),
                "usageNotes": " ".join(notes) or None,
                "isActive": e["status"] == "adopted",
                "bundu": {
                    "isFoundational": bool(e.get("foundational", False)),
                    "adoptionRationale": _cell(e.get("rationale") or f"Adopted: {e['decided']}"),
                },
                "createdAt": {"$date": SEED_DATE},
                "updatedAt": {"$date": SEED_DATE},
            }
        )
    return docs


def seed_json(reg: dict) -> str:
    return json.dumps(seed_docs(reg), indent=2, ensure_ascii=False) + "\n"


# --- lint ------------------------------------------------------------------------


def lint(reg: dict) -> list[str]:
    errors = []
    seen = set()
    for e in reg["standards"]:
        for k in ("id", "name", "domain", "category", "commitment", "current", "governs", "status", "decided", "url"):
            if not e.get(k):
                errors.append(f"register {e.get('id')}: missing {k}")
        if e.get("id") in seen:
            errors.append(f"register {e['id']}: duplicate id")
        seen.add(e.get("id"))
        if e.get("domain") not in reg["domains"]:
            errors.append(f"register {e.get('id')}: unknown domain {e.get('domain')}")
        if e.get("category") not in CATEGORIES:
            errors.append(f"register {e.get('id')}: category {e.get('category')} not in the integrations.standards enum")
        if e.get("status") not in ("adopted", "proposed"):
            errors.append(f"register {e.get('id')}: status {e.get('status')} is neither adopted nor proposed")
    return errors


# --- live comparison -------------------------------------------------------------


def read_live(path: Path) -> list[dict]:
    text = path.read_text().strip()
    if text.startswith("["):
        return json.loads(text)
    return [json.loads(line) for line in text.splitlines() if line.strip()]


def compare(seed: list[dict], live: list[dict]) -> list[str]:
    """Drift between the register's seed and the documents actually stored."""
    want = {d["_id"]: d for d in seed}
    have = {d["_id"]: d for d in live}
    out = [f"missing in the database: {i}" for i in sorted(set(want) - set(have))]
    out += [f"in the database, not in the register: {i}" for i in sorted(set(have) - set(want))]
    for i in sorted(set(want) & set(have)):
        for f in COMPARED:
            if want[i].get(f) != have[i].get(f):
                out.append(f"{i}.{f} differs")
    return out


def main(argv: list[str]) -> int:
    cmd = argv[1] if len(argv) > 1 else "check"
    reg = load()
    errors = lint(reg)
    if cmd == "write":
        if errors:
            print("\n".join(errors))
            return 1
        REGISTER_MD.write_text(splice(REGISTER_MD.read_text(), render_tables(reg)))
        SEED_JSON.parent.mkdir(parents=True, exist_ok=True)
        SEED_JSON.write_text(seed_json(reg))
        print(f"wrote REGISTER.md tables and the seed ({len(reg['standards'])} standards)")
        return 0
    if cmd == "check":
        md = REGISTER_MD.read_text()
        if splice(md, render_tables(reg)) != md:
            errors.append("register/REGISTER.md tables are stale (run: python3 register/tools/register.py write)")
        if not SEED_JSON.exists() or SEED_JSON.read_text() != seed_json(reg):
            errors.append("register/seed/integrations.standards.json is stale (run: python3 register/tools/register.py write)")
        print("\n".join(errors) if errors else f"standards register: {len(reg['standards'])} standards, in step")
        return 1 if errors else 0
    if cmd == "live":
        if len(argv) < 3:
            print("usage: register.py live FILE")
            return 2
        drift = compare(seed_docs(reg), read_live(Path(argv[2])))
        print("\n".join(drift) if drift else "the database matches the register")
        return 1 if drift else 0
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))
