"""Run: python3 -m unittest discover register/tools"""

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import register  # noqa: E402


class RegisterTests(unittest.TestCase):
    REG = register.load()

    def test_the_register_lints_clean(self):
        self.assertEqual(register.lint(self.REG), [])

    def test_generated_files_are_current(self):
        md = register.REGISTER_MD.read_text()
        self.assertEqual(register.splice(md, register.render_tables(self.REG)), md)
        self.assertEqual(register.SEED_JSON.read_text(), register.seed_json(self.REG))

    def test_proposed_standards_seed_inactive(self):
        docs = {d["_id"]: d for d in register.seed_docs(self.REG)}
        for e in self.REG["standards"]:
            self.assertEqual(docs[f"standard_{e['id']}"]["isActive"], e["status"] == "adopted")

    def test_a_matching_database_has_no_drift(self):
        seed = register.seed_docs(self.REG)
        self.assertEqual(register.compare(seed, seed), [])

    def test_missing_extra_and_changed_documents_are_all_reported(self):
        seed = register.seed_docs(self.REG)
        live = [dict(d) for d in seed[1:]]  # first one missing
        live.append({"_id": "standard_invented"})  # one not in the register
        live[0] = dict(live[0], versionCommitment="old")  # one changed
        out = register.compare(seed, live)
        self.assertIn(f"missing in the database: {seed[0]['_id']}", out)
        self.assertIn("in the database, not in the register: standard_invented", out)
        self.assertIn(f"{live[0]['_id']}.versionCommitment differs", out)

    def test_dates_do_not_count_as_drift(self):
        seed = register.seed_docs(self.REG)
        live = [dict(d, updatedAt={"$date": "2030-01-01T00:00:00Z"}) for d in seed]
        self.assertEqual(register.compare(seed, live), [])


if __name__ == "__main__":
    unittest.main()
