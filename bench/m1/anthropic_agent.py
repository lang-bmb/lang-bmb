"""Real-model agent for the M1 harness (ADR-0021), Anthropic Messages API.
Models and endpoints are the ones ADR-0066 approves (`model_api.py`). A manual
tool loop, because the harness itself must execute every tool call (limits,
metrics). The client is the standard-library `model_api.HttpAnthropic`, which
speaks the same `messages.create` as the `anthropic` SDK.

Measurement choices, recorded with each result:
- effort is explicit (`EFFORT`), thinking is the model default (adaptive);
- no server-side model fallback: a refusal ends the session and is recorded,
  since a silent model switch would corrupt the measurement.
"""

EFFORT = "high"
MAX_TOKENS = 16000


def to_api_tools(mcp_tools):
    return [
        {"name": t["name"], "description": t["description"], "input_schema": t["inputSchema"]}
        for t in mcp_tools
    ]


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
        self.usage = {"input_tokens": 0, "output_tokens": 0, "requests": 0}
        if hasattr(client, "stats"):
            client.stats = self.usage  # transport retries land in the usage row
        self.stop = None
        self.settings = {"model": model, "effort": EFFORT, "max_tokens": MAX_TOKENS}

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
        response = self.client.messages.create(
            model=self.model,
            max_tokens=MAX_TOKENS,
            system=self.system,
            tools=self.tools,
            messages=self.messages,
            output_config={"effort": EFFORT},
        )
        self.usage["requests"] += 1
        self.usage["input_tokens"] += response.usage.input_tokens
        self.usage["output_tokens"] += response.usage.output_tokens
        if response.stop_reason == "refusal":
            self.stop = "refusal"
            return []
        self.messages.append({"role": "assistant", "content": response.content})
        uses = [b for b in response.content if b.type == "tool_use"]
        if not uses:
            self.stop = response.stop_reason
            return []
        self.pending = [b.id for b in uses]
        return [(b.name, dict(b.input)) for b in uses]
