"""Loop logic of the real-model adapter, with a stub client (no network)."""
import os
import sys
import types
import unittest
from unittest import mock

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from anthropic_agent import AnthropicAgent  # noqa: E402
from model_api import CONTINUE_NOTE  # noqa: E402


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
        self.assertEqual(a.usage, {"input_tokens": 20, "output_tokens": 10, "requests": 2,
                                   "visible_output_tokens": 9, "cache_creation_input_tokens": 0,
                                   "cache_read_input_tokens": 0})
        self.assertEqual(a.stop, "end_turn")

    def test_prompt_caching(self):
        # ADR-0093: the last tool and the conversation are cached for 5 minutes,
        # and `input_tokens` stays the whole prompt.
        cached = block(input_tokens=3, output_tokens=5, cache_creation_input_tokens=40,
                       cache_read_input_tokens=700)
        client = StubClient([block(stop_reason="end_turn", content=[block(type="text", text="done")],
                                   usage=cached)])
        a = AnthropicAgent("claude-sonnet-5-5", "sys", "task", client=client)
        a.set_tools([{"name": n, "description": "d", "inputSchema": {"type": "object"}} for n in ("a", "b")])
        a.next_calls([])
        req = client.requests[0]
        self.assertEqual(req["cache_control"], {"type": "ephemeral"})
        self.assertNotIn("cache_control", req["tools"][0])
        self.assertEqual(req["tools"][1]["cache_control"], {"type": "ephemeral"})
        self.assertEqual((a.usage["input_tokens"], a.usage["cache_creation_input_tokens"],
                          a.usage["cache_read_input_tokens"]), (743, 40, 700))
        self.assertEqual(a.requests[0]["input_tokens"], 743)

    def test_context_log_is_opt_in(self):
        client = StubClient([resp("end_turn", block(type="text", text="done"))])
        a = AnthropicAgent("claude-opus-5-5", "sys", "task", client=client)
        with mock.patch.dict(os.environ, {"BMB_CONTEXT_LOG": "1"}):
            a.next_calls([])
        self.assertEqual((a.usage["request_input_tokens"], a.usage["request_output_tokens"]), ([10], [5]))

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

    def test_max_tokens_is_continued_once_per_turn(self):
        # ADR-0076: the cut text stays, the cut tool call is dropped, the note follows.
        client = StubClient([
            resp("max_tokens", block(type="thinking", thinking="..."), block(type="text", text="half"),
                 block(type="tool_use", id="x", name="write", input={})),
            resp("tool_use", block(type="tool_use", id="t1", name="verify", input={})),
        ])
        a = AnthropicAgent("m", "s", "t", client=client)
        a.set_tools([])
        self.assertEqual(a.next_calls([]), [("verify", {})])
        msgs = client.requests[1]["messages"]
        self.assertEqual([b.type for b in msgs[-2]["content"]], ["text"])
        self.assertEqual(msgs[-1], {"role": "user", "content": CONTINUE_NOTE})

    def test_max_tokens_with_nothing_visible_joins_the_last_user_turn(self):
        client = StubClient([resp("max_tokens", block(type="thinking", thinking="...")),
                             resp("max_tokens"), resp("max_tokens")])
        a = AnthropicAgent("m", "s", "t", client=client)
        a.set_tools([])
        self.assertEqual(a.next_calls([]), [])
        msgs = client.requests[1]["messages"]
        self.assertEqual(msgs, [{"role": "user", "content": [{"type": "text", "text": "t"},
                                                             {"type": "text", "text": CONTINUE_NOTE}]}])
        # The continuation was cut too: the session ends with the stop reason.
        self.assertEqual((a.stop, len(client.requests)), ("max_tokens", 2))


if __name__ == "__main__":
    unittest.main()
