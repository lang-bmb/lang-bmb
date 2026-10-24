"""Loop logic of the real-model adapter, with a stub client (no network)."""
import os
import sys
import types
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from anthropic_agent import AnthropicAgent  # noqa: E402


def block(**kw):
    return types.SimpleNamespace(**kw)


class StubClient:
    def __init__(self, responses):
        self.responses = list(responses)
        self.requests = []
        self.messages = self

    def create(self, **kw):
        self.requests.append({**kw, "messages": list(kw["messages"])})
        return self.responses.pop(0)


def resp(stop, *content):
    return block(stop_reason=stop, content=list(content), usage=block(input_tokens=10, output_tokens=5))


class T(unittest.TestCase):
    def test_round_trip(self):
        client = StubClient([
            resp("tool_use", block(type="tool_use", id="t1", name="query", input={"name": "units"}),
                 block(type="tool_use", id="t2", name="verify", input={})),
            resp("end_turn", block(type="text", text="done")),
        ])
        a = AnthropicAgent("claude-opus-5-5", "sys", "task", client=client)
        a.set_tools([{"name": "query", "description": "d", "inputSchema": {"type": "object"}}])
        calls = a.next_calls([])
        self.assertEqual(calls, [("query", {"name": "units"}), ("verify", {})])
        self.assertEqual(a.next_calls([("query", {}, "rows", False), ("verify", {}, "bad", True)]), [])
        last = client.requests[1]["messages"][-1]
        self.assertEqual([r["tool_use_id"] for r in last["content"]], ["t1", "t2"])
        self.assertTrue(last["content"][1]["is_error"])
        self.assertEqual(client.requests[0]["tools"][0]["input_schema"], {"type": "object"})
        self.assertEqual(client.requests[0]["output_config"], {"effort": "high"})
        self.assertEqual(a.usage, {"input_tokens": 20, "output_tokens": 10, "requests": 2})
        self.assertEqual(a.stop, "end_turn")

    def test_unrun_calls_get_error_results(self):
        client = StubClient([
            resp("tool_use", block(type="tool_use", id="a", name="verify", input={}),
                 block(type="tool_use", id="b", name="verify", input={})),
            resp("end_turn"),
        ])
        a = AnthropicAgent("m", "s", "t", client=client)
        a.set_tools([])
        a.next_calls([])
        a.next_calls([("verify", {}, "ok", False)])
        results = client.requests[1]["messages"][-1]["content"]
        self.assertEqual(results[1]["content"], "not run: tool call limit reached")

    def test_refusal_stops(self):
        a = AnthropicAgent("m", "s", "t", client=StubClient([resp("refusal")]))
        a.set_tools([])
        self.assertEqual(a.next_calls([]), [])
        self.assertEqual(a.stop, "refusal")


if __name__ == "__main__":
    unittest.main()
