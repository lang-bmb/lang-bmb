"""LLM client — OpenAI-compatible API for code generation."""

import json
import re
import time
import urllib.error
import urllib.request


class LlmClient:
    def __init__(self, model: str, base_url: str, api_key: str = "",
                 temperature: float = 0.0, max_tokens: int = 4096,
                 timeout: int = 120):
        self.model = model
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key
        self.temperature = temperature
        self.max_tokens = max_tokens
        self.timeout = timeout

    def generate(self, system: str, messages: list[dict],
                 retries: int = 2) -> str:
        """Send messages via OpenAI-compatible API, return text response."""
        payload = {
            "model": self.model,
            "temperature": self.temperature,
            "max_tokens": self.max_tokens,
            "messages": ([{"role": "system", "content": system}] if system else [])
                        + messages,
        }
        data = json.dumps(payload).encode("utf-8")
        headers = {"Content-Type": "application/json"}
        if self.api_key:
            headers["Authorization"] = f"Bearer {self.api_key}"

        last_error = None
        for attempt in range(1, retries + 2):
            try:
                req = urllib.request.Request(
                    f"{self.base_url}/chat/completions",
                    data=data, headers=headers, method="POST",
                )
                with urllib.request.urlopen(req, timeout=self.timeout) as resp:
                    result = json.loads(resp.read().decode("utf-8", errors="replace"))

                choice = result["choices"][0]
                content = choice["message"]["content"]
                finish = choice.get("finish_reason", "")
                if finish == "length":
                    content += "\n// [TRUNCATED]"
                return content

            except (urllib.error.URLError, TimeoutError, OSError) as e:
                last_error = e
                if attempt <= retries:
                    wait = min(30, 5 * attempt)
                    time.sleep(wait)
            except urllib.error.HTTPError as e:
                body = e.read().decode("utf-8", errors="replace")
                raise RuntimeError(f"LLM API error {e.code}: {body}") from e

        raise RuntimeError(f"LLM API failed after {retries + 1} attempts: {last_error}")

    @staticmethod
    def extract_code(response: str, lang: str = "bmb") -> str:
        """Extract code block from LLM response."""
        pattern = rf"```(?:{lang}|)\n(.*?)```"
        match = re.search(pattern, response, re.DOTALL)
        if match:
            return match.group(1).strip()
        pattern_open = rf"```(?:{lang}|)\n(.*)"
        match_open = re.search(pattern_open, response, re.DOTALL)
        if match_open:
            return match_open.group(1).replace("// [TRUNCATED]", "").strip()
        return response.strip()
