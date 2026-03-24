import re
import json
import urllib.request
import urllib.error
import time


class LlmClient:
    def __init__(self, model: str = "claude-code", temperature: float = 0.0,
                 base_url: str = "http://172.30.1.62:6190/v1",
                 max_tokens: int = 4096, timeout: int = 600):
        self.model = model
        self.temperature = temperature
        self.base_url = base_url.rstrip("/")
        self.max_tokens = max_tokens
        self.timeout = timeout

    def generate(self, system: str, messages: list[dict],
                 retries: int = 2) -> str:
        """Send messages via OpenAI-compatible API, return text response.

        Retries on timeout/connection errors. Detects truncated responses
        (finish_reason == 'length') and returns what we got with a warning marker.
        """
        payload = {
            "model": self.model,
            "temperature": self.temperature,
            "max_tokens": self.max_tokens,
            "messages": ([{"role": "system", "content": system}] if system else [])
                        + messages,
        }
        data = json.dumps(payload).encode("utf-8")

        last_error = None
        for attempt in range(1, retries + 2):
            try:
                req = urllib.request.Request(
                    f"{self.base_url}/chat/completions",
                    data=data,
                    headers={"Content-Type": "application/json"},
                    method="POST",
                )
                with urllib.request.urlopen(req, timeout=self.timeout) as resp:
                    result = json.loads(resp.read().decode("utf-8"))

                choice = result["choices"][0]
                content = choice["message"]["content"]
                finish = choice.get("finish_reason", "")

                if finish == "length":
                    # Response truncated — append marker so caller can detect
                    content += "\n// [TRUNCATED]"

                return content

            except (urllib.error.URLError, TimeoutError, OSError) as e:
                last_error = e
                if attempt <= retries:
                    wait = min(30, 5 * attempt)
                    print(f"    LLM retry {attempt}/{retries} after {wait}s: {e}")
                    time.sleep(wait)
            except urllib.error.HTTPError as e:
                body = e.read().decode("utf-8", errors="replace")
                raise RuntimeError(f"LLM API error {e.code}: {body}") from e

        raise RuntimeError(f"LLM API failed after {retries + 1} attempts: {last_error}")

    @staticmethod
    def extract_code(response: str, lang: str) -> str:
        """Extract code block from LLM response.

        Handles truncated responses by looking for unclosed code blocks.
        """
        # Try standard closed block first
        pattern = rf"```(?:{lang}|)\n(.*?)```"
        match = re.search(pattern, response, re.DOTALL)
        if match:
            return match.group(1).strip()

        # Try unclosed block (truncated response)
        pattern_open = rf"```(?:{lang}|)\n(.*)"
        match_open = re.search(pattern_open, response, re.DOTALL)
        if match_open:
            code = match_open.group(1).strip()
            # Remove truncation marker if present
            code = code.replace("// [TRUNCATED]", "").strip()
            return code

        return response.strip()
