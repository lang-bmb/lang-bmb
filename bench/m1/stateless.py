"""The stateless repair condition (ADR-0110, backlog T0-36): every request
starts a new conversation.

A provider agent (`model_api.OpenAIAgent`, `anthropic_agent.AnthropicAgent`)
keeps the whole history: each request sends every earlier response and tool
result again, so a session's input tokens grow with the square of its turns
(AI-user feedback 2026-11-10 §3.3: `t14-dedup-sorted` went from 2,749 to
56,096 input tokens per request). In BMB the state is in the workspace,
behind the tools, so the history repeats it.

`Stateless` wraps one of them. Before each request it replaces the
conversation with the system prompt and one user message, in this order:

  1. the task prompt, as given;
  2. `## State now`: what `state()` returns (the workspace's files in
     normal form for the BMB arm, the crate's source files for the Rust arm:
     the same rule for both, ADR-0110 §2);
  3. `## Attempts so far`: one line per tool call of the session, in order
     (`N. tool: ok|error: first line of the result`, at most 160 bytes);
  4. `## Results of your last calls`: the last turn's tool results in full.

The prompt does not say that the history was dropped; the message is
self-contained. The agent's own counters (`usage`, `requests`, `stop`) are
the provider agent's, so result rows and session records keep their shape;
`settings` adds `"history": "stateless"`.

Nothing in the harnesses uses this yet: a set opts in with
`{"history": "stateless"}` in its `set.json` once T0-37 prepares the frozen
arms. Python standard library only.
"""

ATTEMPT_LINE = 160


def attempt_line(n, name, text, err):
    """One line of `## Attempts so far`."""
    first = (text or "").strip().splitlines()[0] if (text or "").strip() else ""
    line = f"{n}. {name}: {'error' if err else 'ok'}: {first}"
    raw = line.encode()
    if len(raw) > ATTEMPT_LINE:
        line = raw[:ATTEMPT_LINE].decode("utf-8", "ignore") + "..."
    return line


class Stateless:
    """A provider agent whose every request is a new conversation."""

    def __init__(self, inner, task_prompt, state):
        self.inner = inner
        self.task = task_prompt
        self.state = state
        self.attempts = []

    # The harness agent protocol, through the provider agent.
    @property
    def usage(self):
        return self.inner.usage

    @property
    def requests(self):
        return self.inner.requests

    @property
    def stop(self):
        return self.inner.stop

    @property
    def settings(self):
        return {**getattr(self.inner, "settings", {}), "history": "stateless"}

    def set_tools(self, mcp_tools):
        self.inner.set_tools(mcp_tools)

    def message(self, last_results):
        """The one user message of the next request."""
        for name, _args, text, err in last_results:
            self.attempts.append(attempt_line(len(self.attempts) + 1, name, text, err))
        last = "\n\n".join(f"### {name}\n{'error: ' if err else ''}{text}"
                           for name, _args, text, err in last_results)
        return (f"{self.task}\n\n## State now\n{self.state()}\n\n"
                f"## Attempts so far\n{chr(10).join(self.attempts) or '(none)'}\n\n"
                f"## Results of your last calls\n{last or '(none)'}")

    def next_calls(self, last_results):
        if self.inner.stop:
            return []
        user = {"role": "user", "content": self.message(last_results)}
        msgs = self.inner.messages
        # OpenAI keeps the system prompt as the first message; Anthropic
        # sends it apart (`system`).
        head = [msgs[0]] if msgs and msgs[0].get("role") == "system" else []
        self.inner.messages = head + [user]
        # The last calls' results are in the message, not tool results.
        self.inner.pending = []
        return self.inner.next_calls([])
