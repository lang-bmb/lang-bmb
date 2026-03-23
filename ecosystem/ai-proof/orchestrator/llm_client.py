import re


class LlmClient:
    def __init__(self, model: str = "claude-opus-4-6", temperature: float = 0.0):
        import anthropic
        self.client = anthropic.Anthropic()
        self.model = model
        self.temperature = temperature

    def generate(self, system: str, messages: list[dict]) -> str:
        resp = self.client.messages.create(
            model=self.model,
            max_tokens=8192,
            temperature=self.temperature,
            system=system,
            messages=messages
        )
        return resp.content[0].text

    @staticmethod
    def extract_code(response: str, lang: str) -> str:
        """Extract code block from LLM response."""
        pattern = rf"```(?:{lang}|)\n(.*?)```"
        match = re.search(pattern, response, re.DOTALL)
        if match:
            return match.group(1).strip()
        return response.strip()
