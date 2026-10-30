"""The OpenAI-compatible agent and the agent-kind dispatch (ADR-0066), with a
stub transport (no network)."""
import json
import os
import sys
import types
import unittest
from unittest import mock

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import model_api  # noqa: E402
from anthropic_agent import AnthropicAgent  # noqa: E402
from model_api import OpenAIAgent, is_model_kind, make_model_agent  # noqa: E402


class StubPost:
    def __init__(self, responses):
        self.responses = list(responses)
        self.requests = []

    def __call__(self, url, body, headers, stats=None, fold=None):
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

    def test_context_log_is_opt_in(self):
        # BMB_CONTEXT_LOG=1 adds per-request tokens (backlog 2D-25); without it
        # the usage is the same as before (previous test).
        post = StubPost([resp("tool_calls", calls=[("c1", "verify", "{}")], usage=(7, 3)),
                         resp("stop", usage=(9, 4))])
        a = OpenAIAgent("qwen3.8-27b", "sys", "task", post=post, base="http://x")
        with mock.patch.dict(os.environ, {"BMB_CONTEXT_LOG": "1"}):
            a.next_calls([])
            a.next_calls([("verify", {}, "ok", False)])
        self.assertEqual((a.usage["request_input_tokens"], a.usage["request_output_tokens"]), ([7, 9], [3, 4]))

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


class ApiErrors(unittest.TestCase):
    def test_unrecoverable_error_ends_the_session(self):
        def fail(*a, **k):
            raise RuntimeError("http://x: HTTP 400: context")
        for agent in (OpenAIAgent("m", "s", "t", post=fail, base="http://x"),
                      AnthropicAgent("m", "s", "t", client=types.SimpleNamespace(
                          messages=types.SimpleNamespace(create=fail)))):
            agent.set_tools([])
            self.assertEqual(agent.next_calls([]), [])
            self.assertEqual(agent.stop, "api_error")
            self.assertIn("HTTP 400", agent.usage["api_error"])
            self.assertEqual(agent.next_calls([]), [])


class Streams(unittest.TestCase):
    def test_fold_openai(self):
        lines = [b": keep-alive", b""] + [b"data: " + json.dumps(e).encode() for e in [
            {"choices": [{"delta": {"reasoning_content": "hmm"}}]},
            {"choices": [{"delta": {"content": "do"}}]},
            {"choices": [{"delta": {"tool_calls": [{"index": 0, "id": "c1", "function": {"name": "que",
                                                                                          "arguments": '{"na'}}]}}]},
            {"choices": [{"delta": {"tool_calls": [{"index": 0, "function": {"name": "ry", "arguments": 'me": 1}'}},
                                                   {"index": 1, "id": "c2", "function": {"name": "verify",
                                                                                         "arguments": "{}"}}]}}]},
            {"choices": [{"delta": {}, "finish_reason": "tool_calls"}]},
            {"choices": [], "usage": {"prompt_tokens": 7, "completion_tokens": 3}},
        ]] + [b"data: [DONE]"]
        r = model_api.fold_openai(model_api.sse_events(lines))
        msg = r["choices"][0]["message"]
        self.assertEqual(msg["content"], "do")
        self.assertEqual([(c["id"], c["function"]["name"], c["function"]["arguments"]) for c in msg["tool_calls"]],
                         [("c1", "query", '{"name": 1}'), ("c2", "verify", "{}")])
        self.assertEqual((r["choices"][0]["finish_reason"], r["usage"]["completion_tokens"]), ("tool_calls", 3))

    def test_fold_anthropic(self):
        events = [
            {"type": "message_start", "message": {"id": "m", "role": "assistant", "content": [],
                                                  "stop_reason": None, "usage": {"input_tokens": 9,
                                                                                 "output_tokens": 1}}},
            {"type": "content_block_start", "index": 0, "content_block": {"type": "thinking", "thinking": ""}},
            {"type": "content_block_delta", "index": 0, "delta": {"type": "thinking_delta", "thinking": "a"}},
            {"type": "content_block_delta", "index": 0, "delta": {"type": "signature_delta", "signature": "s"}},
            {"type": "content_block_stop", "index": 0},
            {"type": "content_block_start", "index": 1, "content_block": {"type": "tool_use", "id": "t",
                                                                          "name": "verify", "input": {}}},
            {"type": "content_block_delta", "index": 1, "delta": {"type": "input_json_delta",
                                                                  "partial_json": '{"x": '}},
            {"type": "content_block_delta", "index": 1, "delta": {"type": "input_json_delta", "partial_json": "2}"}},
            {"type": "content_block_stop", "index": 1},
            {"type": "message_delta", "delta": {"stop_reason": "tool_use"}, "usage": {"output_tokens": 12}},
            {"type": "message_stop"},
        ]
        m = model_api.fold_anthropic(events)
        self.assertEqual(m["content"], [{"type": "thinking", "thinking": "a", "signature": "s"},
                                        {"type": "tool_use", "id": "t", "name": "verify", "input": {"x": 2}}])
        self.assertEqual((m["stop_reason"], m["usage"]), ("tool_use", {"input_tokens": 9, "output_tokens": 12}))

    def test_cut_streams_are_transport_failures(self):
        with self.assertRaises(model_api.StreamCut):
            model_api.fold_openai([{"choices": [{"delta": {"content": "x"}}]}])
        with self.assertRaises(model_api.StreamCut):
            model_api.fold_anthropic([{"type": "message_start", "message": {"content": []}}])


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
