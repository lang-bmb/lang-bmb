"""Real-model agent for the M1 harness (ADR-0021), Anthropic Messages API.
Models and endpoints are the ones ADR-0066 approves (`model_api.py`). A manual
tool loop, because the harness itself must execute every tool call (limits,
metrics). The client is the standard-library `model_api.HttpAnthropic`, which
speaks the same `messages.create` as the `anthropic` SDK.

Measurement choices, recorded with each result:
- effort is explicit (`EFFORT`), thinking is the model default (adaptive);
- no server-side model fallback: a refusal ends the session and is recorded,
  since a silent model switch would corrupt the measurement;
- a response cut at `max_tokens` is continued once in the same turn, and the
  visible output (text and tool calls, not thinking) is counted apart
  (ADR-0076, `model_api.CONTINUE_NOTE`, `model_api.visible_tokens`);
- prompt caching (ADR-0093): a 5-minute breakpoint on the last tool, which the
  sessions of a set share, and automatic caching of the growing conversation.
  It changes the price of a request, not what the model is shown, and
  `input_tokens` stays the whole prompt (uncached + written + read), with the
  written and read parts counted apart.
"""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from model_api import CONTINUE_NOTE, context_log, visible_tokens  # noqa: E402

EFFORT = "high"
MAX_TOKENS = 16000
CACHE = {"type": "ephemeral"}  # 5 minutes: a turn's output is 460 tokens at the median (ADR-0093)
CACHE_FIELDS = ("cache_creation_input_tokens", "cache_read_input_tokens")


def to_api_tools(mcp_tools):
    tools = [
        {"name": t["name"], "description": t["description"], "input_schema": t["inputSchema"]}
        for t in mcp_tools
    ]
    if tools:  # tools render first: one entry for every session with these tools
        tools[-1]["cache_control"] = CACHE
    return tools


def prompt_tokens(u):
    """Uncached, written and read input tokens of one response's usage."""
    return (u.input_tokens,) + tuple(getattr(u, k, 0) or 0 for k in CACHE_FIELDS)


class AnthropicAgent:
    def __init__(self, model, system_prompt, task_prompt, client=None):
        if client is None:
            from model_api import HttpAnthropic

            client = HttpAnthropic()
        self.client = client
        self.model = model
        self.system = system_prompt
        self.messages = [{"role": "user", "content": task_prompt}]
        self.tools = None
        self.pending = []  # tool_use ids of the last response, in order
        self.usage = {"input_tokens": 0, "output_tokens": 0, "requests": 0, "visible_output_tokens": 0,
                      "cache_creation_input_tokens": 0, "cache_read_input_tokens": 0}
        self.continued = False  # this turn was already continued once (ADR-0076)
        self.requests = []  # per request, for the session record (backlog INF-17)
        if hasattr(client, "stats"):
            client.stats = self.usage  # transport retries land in the usage row
        self.stop = None
        self.settings = {"model": model, "effort": EFFORT, "max_tokens": MAX_TOKENS,
                         "length": "continue once per turn",
                         "cache": "5m: last tool + automatic"}

    def set_tools(self, mcp_tools):
        self.tools = to_api_tools(mcp_tools)

    def next_calls(self, last_results):
        if self.stop:
            return []
        if self.pending:
            results = []
            for tool_id, (_, _, text, err) in zip(self.pending, last_results):
                results.append({"type": "tool_result", "tool_use_id": tool_id, "content": text, "is_error": err})
            # A call the harness did not run (limit reached) still needs a result.
            for tool_id in self.pending[len(last_results):]:
                results.append({"type": "tool_result", "tool_use_id": tool_id,
                                "content": "not run: tool call limit reached", "is_error": True})
            self.messages.append({"role": "user", "content": results})
            self.pending = []
        try:
            response = self.client.messages.create(
                model=self.model,
                max_tokens=MAX_TOKENS,
                system=self.system,
                tools=self.tools,
                messages=self.messages,
                output_config={"effort": EFFORT},
                cache_control=CACHE,
            )
        except RuntimeError as e:  # model_api: not fixed by retrying (ADR-0066)
            self.stop, self.usage["api_error"] = "api_error", str(e)[:500]
            self.requests.append({"error": str(e)[:500]})
            return []
        self.usage["requests"] += 1
        uncached, written, read = prompt_tokens(response.usage)
        prompt = uncached + written + read
        self.usage["input_tokens"] += prompt
        self.usage["cache_creation_input_tokens"] += written
        self.usage["cache_read_input_tokens"] += read
        self.usage["output_tokens"] += response.usage.output_tokens
        if context_log():  # model_api: opt-in per-request tokens (backlog 2D-25)
            self.usage.setdefault("request_input_tokens", []).append(prompt)
            self.usage.setdefault("request_output_tokens", []).append(response.usage.output_tokens)
        text = "".join(getattr(b, "text", "") for b in response.content if getattr(b, "type", None) == "text")
        visible = visible_tokens(text, [(b.name, json.dumps(b.input, ensure_ascii=False))
                                        for b in response.content if getattr(b, "type", None) == "tool_use"])
        self.usage["visible_output_tokens"] += visible
        self.requests.append({"stop": response.stop_reason, "input_tokens": prompt,
                              "cache_creation_input_tokens": written, "cache_read_input_tokens": read,
                              "output_tokens": response.usage.output_tokens, "visible_output_tokens": visible,
                              "text": text})
        if response.stop_reason == "refusal":
            self.stop = "refusal"
            return []
        if response.stop_reason == "max_tokens" and not self.continued:
            # Cut at the output limit: continue once in this turn (ADR-0076).
            # Only the text stays: a cut tool call would need a result.
            self.continued = True
            kept = [b for b in response.content if getattr(b, "type", None) == "text"]
            if kept:
                self.messages.append({"role": "assistant", "content": kept})
                self.messages.append({"role": "user", "content": CONTINUE_NOTE})
            else:
                # Nothing visible to keep: the note joins the last user turn
                # (roles must alternate).
                last = self.messages[-1]
                content = last["content"] if isinstance(last["content"], list) else [
                    {"type": "text", "text": last["content"]}]
                last["content"] = content + [{"type": "text", "text": CONTINUE_NOTE}]
            return self.next_calls([])
        self.continued = False
        self.messages.append({"role": "assistant", "content": response.content})
        uses = [b for b in response.content if b.type == "tool_use"]
        if not uses:
            self.stop = response.stop_reason
            return []
        self.pending = [b.id for b in uses]
        return [(b.name, dict(b.input)) for b in uses]
