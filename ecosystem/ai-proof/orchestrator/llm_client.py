import re
import json
import urllib.request
import urllib.error


class LlmClient:
    def __init__(self, model: str = "claude-code", temperature: float = 0.0,
                 base_url: str = "http://172.30.1.62:6190/v1"):
        self.model = model
        self.temperature = temperature
        self.base_url = base_url.rstrip("/")

    def generate(self, system: str, messages: list[dict]) -> str:
        """Send messages via OpenAI-compatible API, return text response."""
        payload = {
            "model": self.model,
            "temperature": self.temperature,
            "max_tokens": 8192,
            "messages": [{"role": "system", "content": system}] + messages,
        }
        data = json.dumps(payload).encode("utf-8")
        req = urllib.request.Request(
            f"{self.base_url}/chat/completions",
            data=data,
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        try:
            with urllib.request.urlopen(req, timeout=600) as resp:
                result = json.loads(resp.read().decode("utf-8"))
                return result["choices"][0]["message"]["content"]
        except urllib.error.HTTPError as e:
            body = e.read().decode("utf-8", errors="replace")
            raise RuntimeError(f"LLM API error {e.code}: {body}") from e
        except urllib.error.URLError as e:
            raise RuntimeError(f"LLM API connection error: {e.reason}") from e

    @staticmethod
    def extract_code(response: str, lang: str) -> str:
        """Extract code block from LLM response."""
        pattern = rf"```(?:{lang}|)\n(.*?)```"
        match = re.search(pattern, response, re.DOTALL)
        if match:
            return match.group(1).strip()
        return response.strip()
