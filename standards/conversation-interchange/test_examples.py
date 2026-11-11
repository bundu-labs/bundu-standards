"""The BCI example stream validates, and the schema refuses what the standard forbids.

    python3 -m unittest standards/conversation-interchange/test_examples.py
"""

import json
import unittest
from pathlib import Path

import jsonschema

HERE = Path(__file__).resolve().parent
SCHEMA = json.loads((HERE / "bci.schema.json").read_text())
VALIDATOR = jsonschema.Draft202012Validator(SCHEMA, format_checker=jsonschema.FormatChecker())


def records():
    return [json.loads(line) for line in (HERE / "examples" / "minimal.ndjson").read_text().splitlines() if line]


def errors(record):
    return [e.message for e in VALIDATOR.iter_errors(record)]


class Examples(unittest.TestCase):
    def test_every_record_of_the_example_stream_is_valid(self):
        for r in records():
            self.assertEqual(errors(r), [], r["type"])

    def test_the_stream_opens_with_a_header_and_covers_every_type(self):
        rs = records()
        self.assertEqual(rs[0]["type"], "Header")
        self.assertEqual({r["type"] for r in rs}, {"Header", "Conversation", "Message", "Project", "Artifact", "MemoryEntry"})

    def test_sequences_are_gapless_from_one(self):
        seqs = [r["sequence"] for r in records() if r["type"] == "Message"]
        self.assertEqual(seqs, list(range(1, len(seqs) + 1)))

    def test_context_is_valid_json_and_names_every_type(self):
        ctx = json.loads((HERE / "context.jsonld").read_text())["@context"]
        for t in ("Conversation", "Message", "Project", "Artifact", "MemoryEntry"):
            self.assertIn(t, ctx)


class Refusals(unittest.TestCase):
    def message(self):
        return next(r for r in records() if r["type"] == "Message")

    def test_a_message_cannot_claim_to_be_trainable(self):
        m = {**self.message(), "trainable": True}
        self.assertNotEqual(errors(m), [])

    def test_a_message_needs_a_scope(self):
        m = {k: v for k, v in self.message().items() if k != "scope"}
        self.assertNotEqual(errors(m), [])

    def test_a_sequence_starts_at_one(self):
        self.assertNotEqual(errors({**self.message(), "sequence": 0}), [])

    def test_an_id_must_be_a_uuid(self):
        self.assertNotEqual(errors({**self.message(), "id": "msg_000001"}), [])

    def test_a_timestamp_must_be_utc_rfc3339(self):
        self.assertNotEqual(errors({**self.message(), "created": "2026-11-11 11:59:00"}), [])
        self.assertNotEqual(errors({**self.message(), "created": 1762862340000}), [])

    def test_an_unknown_member_is_refused_but_an_extension_is_kept(self):
        self.assertNotEqual(errors({**self.message(), "vendorThing": 1}), [])
        self.assertEqual(errors({**self.message(), "shamwari:tier": "economy"}), [])

    def test_an_artifact_has_text_or_a_reference_not_neither(self):
        a = next(r for r in records() if r["type"] == "Artifact")
        self.assertNotEqual(errors({k: v for k, v in a.items() if k != "text"}), [])

    def test_a_memory_entry_cannot_be_an_unseen_inference(self):
        m = next(r for r in records() if r["type"] == "MemoryEntry")
        self.assertNotEqual(errors({**m, "consent": "inferred"}), [])
        self.assertNotEqual(errors({k: v for k, v in m.items() if k != "consent"}), [])


if __name__ == "__main__":
    unittest.main()
