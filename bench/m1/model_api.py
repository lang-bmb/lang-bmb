"""Model endpoints for the real-model agents (ADR-0066). Standard library only.

Agent kinds, the same strings in every harness (`--agent`):

  openai:MODEL      OpenAI-compatible Chat Completions (default model of this
                    environment: `qwen3.8-27b`, ADR-0066 §2)
  anthropic:MODEL   Anthropic Messages API (cross-comparison with a frontier
                    model: `claude-sonnet-5-5`, ADR-0066 §3)

Base URLs default to the environment's gateway and can be overridden:
BMB_OPENAI_BASE_URL, BMB_ANTHROPIC_BASE_URL. Keys are optional (the gateway's
credentials are supplied by the environment): OPENAI_API_KEY, ANTHROPIC_API_KEY.

A transport failure (connection error, HTTP 429/5xx) is retried with backoff;
the request is the same, so this is not a task retry (ADR-0021 §3). The count is
recorded in the agent's usage as `http_retries`.
"""
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
RETRIES = 4


def post_json(url, body, headers, stats=None, timeout=TIMEOUT, retries=RETRIES):
    data = json.dumps(body, default=vars).encode()
    delay = 2
    for attempt in range(retries + 1):
        req = urllib.request.Request(url, data=data, headers={"content-type": "application/json", **headers})
        try:
            with urllib.request.urlopen(req, timeout=timeout) as r:
                return json.loads(r.read())
        except urllib.error.HTTPError as e:
            if e.code != 429 and e.code < 500 or attempt == retries:
                raise RuntimeError(f"{url}: HTTP {e.code}: {e.read()[:500]!r}") from None
        except (urllib.error.URLError, TimeoutError, ConnectionError) as e:
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
        return namespace(post_json(self.base + "/messages", kw, headers, self.stats))


def to_openai_tools(mcp_tools):
    return [{"type": "function", "function": {"name": t["name"], "description": t["description"],
                                              "parameters": t["inputSchema"]}} for t in mcp_tools]


class OpenAIAgent:
    """The harness agent protocol (`set_tools`, `next_calls`, `usage`, `stop`,
    `settings`) over Chat Completions. Reasoning text the server returns
    (`reasoning_content`) is counted in the output tokens and not sent back,
    which is how the reasoning models of this API expect their history."""

    MAX_TOKENS = 16000

    def __init__(self, model, system_prompt, task_prompt, post=None, base=None):
        self.model = model
        self.url = (base or OPENAI_BASE) + "/chat/completions"
        self.post = post or post_json
        key = os.environ.get("OPENAI_API_KEY")
        self.headers = {"authorization": f"Bearer {key}"} if key else {}
        self.messages = [{"role": "system", "content": system_prompt}, {"role": "user", "content": task_prompt}]
        self.tools = None
        self.pending = []
        self.usage = {"input_tokens": 0, "output_tokens": 0, "requests": 0}
        self.stop = None
        self.settings = {"model": model, "api": "openai-chat", "max_tokens": self.MAX_TOKENS,
                         "reasoning": "model default"}

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
        body = {"model": self.model, "max_tokens": self.MAX_TOKENS, "messages": self.messages}
        if self.tools:
            body["tools"] = self.tools
        r = self.post(self.url, body, self.headers, self.usage)
        self.usage["requests"] += 1
        u = r.get("usage") or {}
        self.usage["input_tokens"] += u.get("prompt_tokens", 0)
        self.usage["output_tokens"] += u.get("completion_tokens", 0)
        choice = r["choices"][0]
        msg = choice["message"]
        calls = msg.get("tool_calls") or []
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
