"""Model endpoints for the real-model agents (ADR-0066). Standard library only.

Agent kinds, the same strings in every harness (`--agent`):

  openai:MODEL      OpenAI-compatible Chat Completions (default model of this
                    environment: `qwen3.8-27b`, ADR-0066 §2)
  anthropic:MODEL   Anthropic Messages API (cross-comparison with a frontier
                    model: `claude-sonnet-5-5`, ADR-0066 §3)

Base URLs default to the environment's gateway and can be overridden:
BMB_OPENAI_BASE_URL, BMB_ANTHROPIC_BASE_URL. Keys are optional (the gateway's
credentials are supplied by the environment): OPENAI_API_KEY, ANTHROPIC_API_KEY.

Requests stream (server-sent events) and the events are folded back into the
response the non-streaming API returns. The gateway ends a non-streaming request
after about 30 seconds with HTTP 502, and a model turn that thinks longer than
that would fail the same way on every retry.

A transport failure (connection error, HTTP 429/5xx, a stream cut off before its
end) is retried with backoff; the request is the same, so this is not a task
retry (ADR-0021 §3). The count is recorded in the agent's usage as `http_retries`.
An error that retrying does not fix (an HTTP 4xx such as a prompt over the
model's context window, or a transport failure past the retries) ends that
session with `stop: "api_error"` and the message in `usage["api_error"]`; the
harness judges the state reached, as at the tool-call limit, and the report can
tell these sessions apart.

The gateway's spend cap (HTTP 402 `cap_reached`) is not an answer about the
session: it raises `CapReached`, which no agent catches, so the run stops
without writing that session's row and resumes from it once the cap resets
(backlog INF-23, ADR-0088).
"""
import http.client
import json
import os
import time
import types
import urllib.error
import urllib.request

GATEWAY = "https://token.iyulab.com"
OPENAI_BASE = os.environ.get("BMB_OPENAI_BASE_URL", GATEWAY + "/openai/v1")
ANTHROPIC_BASE = os.environ.get("BMB_ANTHROPIC_BASE_URL", GATEWAY + "/anthropic/v1")
TIMEOUT = 900
# A whole streamed request (ADR-0076 sessions reach 65,536 output tokens,
# about half an hour at the gateway's rate). `TIMEOUT` bounds each read only,
# so a stream kept open with keep-alive lines and no end stalled a session
# for hours (T0-13, 2026-11-02); past this it is cut and retried.
REQUEST_DEADLINE = 3600
RETRIES = 6  # backoff 2..64 s, about two minutes of a gateway outage


class StreamCut(Exception):
    pass


# A response that reaches its output limit is continued once in the same
# turn (ADR-0076, backlog T0-27): the cut text stays, any tool call in it is
# dropped (its arguments may be cut), and this note follows. A second cut in
# the same turn ends the session as before (`stop: "length"`).
CONTINUE_NOTE = ("(harness) Your previous reply reached the output limit and was cut off; any tool call "
                 "in it was dropped. Continue from where you stopped and make your next tool calls.")


def visible_tokens(text, calls):
    """The visible output of one response as ceil(bytes / 4) (ADR-0076): its
    text and each tool call's name and arguments, reasoning excluded. The
    same rule for every provider, whatever the API reports as reasoning."""
    n = len(text.encode()) + sum(len(name.encode()) + len(args.encode()) for name, args in calls)
    return -(-n // 4)


def context_log():
    """`BMB_CONTEXT_LOG=1`: also record each request's tokens and each tool
    call's result size (backlog 2D-25), to decompose every turn's context.
    Off by default, so the result rows of a run stay as they were; what the
    model sees is the same either way."""
    return os.environ.get("BMB_CONTEXT_LOG") == "1"


def sse_events(lines, deadline=None):
    """The JSON payloads of a server-sent event stream, up to `[DONE]`;
    `StreamCut` once `time.monotonic()` passes `deadline`."""
    for raw in lines:
        if deadline is not None and time.monotonic() > deadline:
            raise StreamCut("the request passed its deadline")
        line = raw.decode() if isinstance(raw, bytes) else raw
        line = line.strip()
        if not line.startswith("data:"):
            continue
        data = line[5:].strip()
        if data == "[DONE]":
            return
        yield json.loads(data)


def fold_openai(events):
    """Chat Completions chunks -> the non-streaming response (content, tool
    calls by index, finish reason, usage). Reasoning deltas are dropped."""
    content, calls, finish, usage, done = [], {}, None, None, False
    for e in events:
        usage = e.get("usage") or usage
        for c in e.get("choices", []):
            d = c.get("delta") or {}
            if d.get("content"):
                content.append(d["content"])
            for tc in d.get("tool_calls") or []:
                slot = calls.setdefault(tc.get("index", 0), {"id": None, "type": "function",
                                                             "function": {"name": "", "arguments": ""}})
                slot["id"] = tc.get("id") or slot["id"]
                f = tc.get("function") or {}
                slot["function"]["name"] += f.get("name") or ""
                slot["function"]["arguments"] += f.get("arguments") or ""
            if c.get("finish_reason"):
                finish, done = c["finish_reason"], True
    if not done:
        raise StreamCut("stream ended without a finish reason")
    msg = {"role": "assistant", "content": "".join(content)}
    if calls:
        msg["tool_calls"] = [calls[i] for i in sorted(calls)]
    return {"choices": [{"finish_reason": finish, "message": msg}], "usage": usage or {}}


def fold_anthropic(events):
    """Messages API events -> the non-streaming message. Thinking blocks keep
    their text and signature, so they can be sent back unchanged."""
    msg, blocks, partial, done = None, {}, {}, False
    for e in events:
        t = e.get("type")
        if t == "message_start":
            msg = e["message"]
        elif t == "content_block_start":
            blocks[e["index"]] = dict(e["content_block"])
        elif t == "content_block_delta":
            b, d = blocks[e["index"]], e["delta"]
            kind = d.get("type")
            if kind == "text_delta":
                b["text"] = b.get("text", "") + d["text"]
            elif kind == "input_json_delta":
                partial[e["index"]] = partial.get(e["index"], "") + d["partial_json"]
            elif kind == "thinking_delta":
                b["thinking"] = b.get("thinking", "") + d["thinking"]
            elif kind == "signature_delta":
                b["signature"] = d["signature"]
        elif t == "content_block_stop":
            if e["index"] in partial:
                blocks[e["index"]]["input"] = json.loads(partial[e["index"]] or "{}")
        elif t == "message_delta":
            msg.update({k: v for k, v in e.get("delta", {}).items()})
            msg["usage"] = {**msg.get("usage", {}), **{k: v for k, v in e.get("usage", {}).items() if v is not None}}
        elif t == "message_stop":
            done = True
        elif t == "error":
            raise StreamCut(f"stream error: {e.get('error')}")
    if not done or msg is None:
        raise StreamCut("stream ended before message_stop")
    msg["content"] = [blocks[i] for i in sorted(blocks)]
    return msg


class CapReached(Exception):
    """The gateway's spend cap: the run stops, the session is not recorded
    (not a `RuntimeError`, which ends a session as `api_error`)."""


def post_json(url, body, headers, stats=None, timeout=TIMEOUT, retries=RETRIES, fold=None):
    """POST `body`; with `fold`, the request streams and `fold` turns its events
    into the response."""
    data = json.dumps(body, default=vars).encode()
    delay = 2
    for attempt in range(retries + 1):
        req = urllib.request.Request(url, data=data, headers={"content-type": "application/json", **headers})
        try:
            with urllib.request.urlopen(req, timeout=timeout) as r:
                if fold:
                    return fold(sse_events(r, time.monotonic() + REQUEST_DEADLINE))
                return json.loads(r.read())
        except urllib.error.HTTPError as e:
            text = e.read()[:500]
            if e.code == 402 and b"cap_reached" in text:
                raise CapReached(f"{url}: HTTP 402: {text!r}") from None
            if e.code != 429 and e.code < 500 or attempt == retries:
                raise RuntimeError(f"{url}: HTTP {e.code}: {text!r}") from None
        except (OSError, http.client.HTTPException, StreamCut) as e:  # connection, TLS, cut stream
            if attempt == retries:
                raise RuntimeError(f"{url}: {e}") from None
        if stats is not None:
            stats["http_retries"] = stats.get("http_retries", 0) + 1
        time.sleep(delay)
        delay *= 2


def namespace(x):
    """Attribute objects like the SDK's, except that a tool call's `input`
    stays a dict (the SDK's type for it)."""
    if isinstance(x, dict):
        return types.SimpleNamespace(**{k: v if k == "input" else namespace(v) for k, v in x.items()})
    if isinstance(x, list):
        return [namespace(v) for v in x]
    return x


class HttpAnthropic:
    """The part of the `anthropic` SDK client that AnthropicAgent uses:
    `client.messages.create(**kw)` returning attribute objects. Content blocks
    sent back in later requests serialize through `vars`."""

    def __init__(self, base=None, key=None):
        self.base = base or ANTHROPIC_BASE
        self.key = key if key is not None else os.environ.get("ANTHROPIC_API_KEY")
        self.messages = self
        self.stats = {}

    def create(self, **kw):
        headers = {"anthropic-version": "2023-06-01"}
        if self.key:
            headers["x-api-key"] = self.key
        return namespace(post_json(self.base + "/messages", {**kw, "stream": True}, headers, self.stats,
                                   fold=fold_anthropic))


def to_openai_tools(mcp_tools):
    return [{"type": "function", "function": {"name": t["name"], "description": t["description"],
                                              "parameters": t["inputSchema"]}} for t in mcp_tools]


class OpenAIAgent:
    """The harness agent protocol (`set_tools`, `next_calls`, `usage`, `stop`,
    `settings`) over Chat Completions. Reasoning text the server returns
    (`reasoning_content`) is counted in the output tokens and not sent back,
    which is how the reasoning models of this API expect their history."""

    # Qwen's recommended output length for thinking mode. With 16,000 a first
    # turn could spend the whole budget reasoning (seen on m1 w05). The prompt
    # limit of this environment's qwen3.8-27b is its 64K context.
    MAX_TOKENS = 32768

    def __init__(self, model, system_prompt, task_prompt, post=None, base=None):
        self.model = model
        self.url = (base or OPENAI_BASE) + "/chat/completions"
        self.post = post or post_json
        key = os.environ.get("OPENAI_API_KEY")
        self.headers = {"authorization": f"Bearer {key}"} if key else {}
        self.messages = [{"role": "system", "content": system_prompt}, {"role": "user", "content": task_prompt}]
        self.tools = None
        self.pending = []
        self.usage = {"input_tokens": 0, "output_tokens": 0, "requests": 0, "visible_output_tokens": 0}
        # Per request: stop reason, tokens and visible text, for the session
        # record (backlog INF-17). Not part of the result row.
        self.requests = []
        self.stop = None
        self.continued = False  # this turn was already continued once (ADR-0076)
        self.settings = {"model": model, "api": "openai-chat", "max_tokens": self.MAX_TOKENS,
                         "reasoning": "model default", "length": "continue once per turn"}

    def set_tools(self, mcp_tools):
        self.tools = to_openai_tools(mcp_tools)

    def next_calls(self, last_results):
        if self.stop:
            return []
        if self.pending:
            for i, call_id in enumerate(self.pending):
                if i < len(last_results):
                    _, _, text, err = last_results[i]
                    content = ("error: " if err else "") + text
                else:
                    content = "error: not run: tool call limit reached"
                self.messages.append({"role": "tool", "tool_call_id": call_id, "content": content})
            self.pending = []
        body = {"model": self.model, "max_tokens": self.MAX_TOKENS, "messages": self.messages,
                "stream": True, "stream_options": {"include_usage": True}}
        if self.tools:
            body["tools"] = self.tools
        try:
            r = self.post(self.url, body, self.headers, self.usage, fold=fold_openai)
        except RuntimeError as e:
            self.stop, self.usage["api_error"] = "api_error", str(e)[:500]
            self.requests.append({"error": str(e)[:500]})
            return []
        self.usage["requests"] += 1
        u = r.get("usage") or {}
        self.usage["input_tokens"] += u.get("prompt_tokens", 0)
        self.usage["output_tokens"] += u.get("completion_tokens", 0)
        if context_log():
            self.usage.setdefault("request_input_tokens", []).append(u.get("prompt_tokens", 0))
            self.usage.setdefault("request_output_tokens", []).append(u.get("completion_tokens", 0))
        choice = r["choices"][0]
        msg = choice["message"]
        calls = msg.get("tool_calls") or []
        visible = visible_tokens(msg.get("content") or "", [(c["function"].get("name") or "",
                                                              c["function"].get("arguments") or "") for c in calls])
        self.usage["visible_output_tokens"] += visible
        reasoning = (u.get("completion_tokens_details") or {}).get("reasoning_tokens")
        if reasoning is not None:
            self.usage["reasoning_tokens"] = self.usage.get("reasoning_tokens", 0) + reasoning
        self.requests.append({"stop": choice.get("finish_reason"), "input_tokens": u.get("prompt_tokens", 0),
                              "output_tokens": u.get("completion_tokens", 0), "visible_output_tokens": visible,
                              "text": msg.get("content") or ""})
        if choice.get("finish_reason") == "length" and not self.continued:
            # Cut at the output limit: continue once in this turn (ADR-0076).
            self.continued = True
            self.messages.append({"role": "assistant", "content": msg.get("content") or ""})
            self.messages.append({"role": "user", "content": CONTINUE_NOTE})
            return self.next_calls([])
        self.continued = False
        self.messages.append({"role": "assistant", "content": msg.get("content") or "",
                              **({"tool_calls": calls} if calls else {})})
        if not calls:
            self.stop = choice.get("finish_reason") or "stop"
            return []
        self.pending = [c["id"] for c in calls]
        out = []
        for c in calls:
            raw = c["function"].get("arguments") or "{}"
            try:
                args = json.loads(raw)
            except json.JSONDecodeError:
                args = None
            # Arguments that are not a JSON object reach the tool as they are,
            # so the model sees the tool's own error.
            out.append((c["function"]["name"], args if isinstance(args, dict) else {"arguments": raw}))
        return out


def make_model_agent(kind, system_prompt, task_prompt):
    """The agent for a real-model kind, or None when `kind` is not one."""
    provider, _, model = kind.partition(":")
    if provider == "openai" and model:
        return OpenAIAgent(model, system_prompt, task_prompt)
    if provider == "anthropic" and model:
        from anthropic_agent import AnthropicAgent
        return AnthropicAgent(model, system_prompt, task_prompt)
    return None


def is_model_kind(kind):
    provider, _, model = kind.partition(":")
    return provider in ("openai", "anthropic") and bool(model)
