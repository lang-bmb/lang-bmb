"""The OpenAI-compatible agent and the agent-kind dispatch (ADR-0066), with a
stub transport (no network)."""
import json
import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import model_api  # noqa: E402
from anthropic_agent import AnthropicAgent  # noqa: E402
from model_api import OpenAIAgent, is_model_kind, make_model_agent  # noqa: E402


class StubPost:
    def __init__(self, responses):
        self.responses = list(responses)
        self.requests = []

    def __call__(self, url, body, headers, stats=None):
        self.requests.append(json.loads(json.dumps(body)))
        return self.responses.pop(0)


def resp(finish, content="", calls=(), usage=(10, 5)):
    msg = {"role": "assistant", "content": content, "reasoning_content": "thinking"}
    if calls:
        msg["tool_calls"] = [{"id": i, "type": "function", "function": {"name": n, "arguments": a}}
                             for i, n, a in calls]
    return {"choices": [{"finish_reason": finish, "message": msg}],
            "usage": {"prompt_tokens": usage[0], "completion_tokens": usage[1]}}


class OpenAI(unittest.TestCase):
    def test_round_trip(self):
        post = StubPost([
            resp("tool_calls", calls=[("c1", "query", '{"name": "units"}'), ("c2", "verify", "{}")]),
            resp("stop", "done"),
        ])
        a = OpenAIAgent("qwen3.8-27b", "sys", "task", post=post, base="http://x")
        a.set_tools([{"name": "query", "description": "d", "inputSchema": {"type": "object"}}])
        self.assertEqual(a.next_calls([]), [("query", {"name": "units"}), ("verify", {})])
        self.assertEqual(a.next_calls([("query", {}, "rows", False), ("verify", {}, "bad", True)]), [])
        first, second = post.requests
        self.assertEqual(first["messages"][0], {"role": "system", "content": "sys"})
        self.assertEqual(first["tools"][0]["function"]["parameters"], {"type": "object"})
        tail = second["messages"][-3:]
        self.assertEqual(tail[0]["role"], "assistant")
        self.assertNotIn("reasoning_content", tail[0])  # reasoning is not sent back
        self.assertEqual([(m["tool_call_id"], m["content"]) for m in tail[1:]], [("c1", "rows"), ("c2", "error: bad")])
        self.assertEqual(a.usage, {"input_tokens": 20, "output_tokens": 10, "requests": 2})
        self.assertEqual(a.stop, "stop")

    def test_unrun_calls_and_bad_arguments(self):
        post = StubPost([resp("tool_calls", calls=[("a", "verify", "{}"), ("b", "query", "not json")]),
                         resp("stop")])
        a = OpenAIAgent("m", "s", "t", post=post, base="http://x")
        a.set_tools([])
        self.assertEqual(a.next_calls([]), [("verify", {}), ("query", {"arguments": "not json"})])
        a.next_calls([("verify", {}, "ok", False)])
        self.assertEqual(post.requests[1]["messages"][-1]["content"], "error: not run: tool call limit reached")
        self.assertNotIn("tools", post.requests[0])

    def test_length_without_calls_stops(self):
        a = OpenAIAgent("m", "s", "t", post=StubPost([resp("length")]), base="http://x")
        a.set_tools([])
        self.assertEqual(a.next_calls([]), [])
        self.assertEqual(a.stop, "length")


class Kinds(unittest.TestCase):
    def test_dispatch(self):
        self.assertIsInstance(make_model_agent("openai:qwen3.8-27b", "s", "t"), OpenAIAgent)
        self.assertIsInstance(make_model_agent("anthropic:claude-sonnet-5-5", "s", "t"), AnthropicAgent)
        for kind in ("oracle", "weak", "openai:", "anthropic", "other:m"):
            self.assertIsNone(make_model_agent(kind, "s", "t"), kind)
            self.assertFalse(is_model_kind(kind), kind)
        self.assertTrue(is_model_kind("openai:qwen3.8-27b"))

    def test_namespace_round_trip(self):
        # Response blocks become attribute objects and serialize back unchanged.
        block = {"type": "tool_use", "id": "t", "input": {"a": [1, {"b": 2}]}}
        ns = model_api.namespace({"content": [block]})
        self.assertEqual(ns.content[0].input, {"a": [1, {"b": 2}]})
        ns = ns.content[0]
        self.assertEqual(json.loads(json.dumps(ns, default=vars)), block)


if __name__ == "__main__":
    unittest.main()
