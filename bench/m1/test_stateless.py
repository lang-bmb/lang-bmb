"""The stateless repair condition (ADR-0110, backlog T0-36) over both
provider agents, with stub transports (no network): every request is the
system prompt and one user message (task, state now, one line per attempt,
the last results); nothing of the earlier turns is sent again."""
import json
import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from anthropic_agent import AnthropicAgent  # noqa: E402
from model_api import OpenAIAgent  # noqa: E402
from stateless import ATTEMPT_LINE, Stateless, attempt_line  # noqa: E402
from test_anthropic_agent import StubClient, block  # noqa: E402
from test_anthropic_agent import resp as aresp  # noqa: E402
from test_model_api import StubPost, resp  # noqa: E402

TOOLS = [{"name": "write", "description": "d", "inputSchema": {"type": "object"}}]


class OpenAI(unittest.TestCase):
    def test_each_request_is_a_new_conversation(self):
        post = StubPost([
            resp("tool_calls", calls=[("c1", "write", '{"unit": "u"}')]),
            resp("tool_calls", calls=[("c2", "write", '{"unit": "u"}'), ("c3", "verify", "{}")]),
            resp("stop", "done"),
        ])
        states = iter(["(empty)", "mod u { v1 }", "mod u { v2 }"])
        a = Stateless(OpenAIAgent("qwen", "SYS", "TASK", post=post), "TASK", lambda: next(states))
        a.set_tools(TOOLS)
        self.assertEqual(a.next_calls([]), [("write", {"unit": "u"})])
        self.assertEqual(a.next_calls([("write", {}, "{\"committed\":true}\nverified", False)]),
                         [("write", {"unit": "u"}), ("verify", {})])
        self.assertEqual(a.next_calls([("write", {}, "E0802 at 3:5\nmore", True), ("verify", {}, "ok", False)]), [])
        for r in post.requests:
            self.assertEqual([m["role"] for m in r["messages"]], ["system", "user"])
            self.assertEqual(r["messages"][0]["content"], "SYS")
        last = post.requests[2]["messages"][1]["content"]
        self.assertTrue(last.startswith("TASK\n\n## State now\nmod u { v2 }\n\n## Attempts so far\n"), last)
        self.assertIn("1. write: ok: {\"committed\":true}\n2. write: error: E0802 at 3:5\n3. verify: ok: ok", last)
        self.assertIn("## Results of your last calls\n### write\nerror: E0802 at 3:5\nmore\n\n### verify\nok", last)
        first = post.requests[0]["messages"][1]["content"]
        self.assertIn("## Attempts so far\n(none)", first)
        self.assertEqual(a.stop, "stop")
        self.assertEqual(a.usage["requests"], 3)
        self.assertEqual(a.settings["history"], "stateless")


class Anthropic(unittest.TestCase):
    def test_no_tool_results_and_system_apart(self):
        client = StubClient([
            aresp("tool_use", block(type="tool_use", id="t1", name="write", input={"unit": "u"})),
            aresp("end_turn", block(type="text", text="done")),
        ])
        a = Stateless(AnthropicAgent("claude-sonnet-5-5", "SYS", "TASK", client=client), "TASK", lambda: "S")
        a.set_tools(TOOLS)
        self.assertEqual(a.next_calls([]), [("write", {"unit": "u"})])
        self.assertEqual(a.next_calls([("write", {}, "rejected", True)]), [])
        for r in client.requests:
            self.assertEqual(r["system"], "SYS")
            self.assertEqual(len(r["messages"]), 1)
            self.assertIsInstance(r["messages"][0]["content"], str)
        self.assertIn("1. write: error: rejected", client.requests[1]["messages"][0]["content"])
        self.assertEqual(a.stop, "end_turn")


class Lines(unittest.TestCase):
    def test_attempt_lines_are_one_short_line(self):
        self.assertEqual(attempt_line(4, "verify", "", False), "4. verify: ok: ")
        long = attempt_line(1, "write", "x" * 500 + "\nsecond", True)
        self.assertLessEqual(len(long.encode()), ATTEMPT_LINE + 3)
        self.assertNotIn("second", long)
        json.dumps(long)


if __name__ == "__main__":
    unittest.main()
