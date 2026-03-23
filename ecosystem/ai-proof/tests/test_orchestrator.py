from protocol.prompt_templates import build_initial_prompt, build_error_feedback_prompt
from orchestrator.llm_client import LlmClient

def test_initial_prompt_contains_problem():
    prompt = build_initial_prompt(
        problem_desc="Implement binary search",
        lang="bmb",
        test_preview="Input: [1,2,3], target=2 → Output: 1",
        reference=None
    )
    assert "binary search" in prompt.lower()
    assert "bmb" in prompt.lower()

def test_initial_prompt_with_reference():
    prompt = build_initial_prompt(
        problem_desc="Sort an array", lang="bmb",
        test_preview="...", reference="fn foo() -> i64 = 42;"
    )
    assert "fn foo" in prompt

def test_initial_prompt_no_reference_for_rust():
    prompt = build_initial_prompt(
        problem_desc="Sort", lang="rust",
        test_preview="...", reference=None
    )
    assert "Reference" not in prompt

def test_error_feedback_prompt():
    prompt = build_error_feedback_prompt(
        error_type="compile_error",
        normalized_msg="pre-condition violated",
        location="solution.bmb:5:3",
        raw_output="full output"
    )
    assert "compile_error" in prompt
    assert "pre-condition" in prompt

def test_extract_code_from_markdown():
    response = "Here is the code:\n```bmb\nfn main() -> i64 = 42;\n```\nDone."
    code = LlmClient.extract_code(response, "bmb")
    assert code == "fn main() -> i64 = 42;"

def test_extract_code_no_block():
    response = "fn main() -> i64 = 42;"
    code = LlmClient.extract_code(response, "bmb")
    assert "fn main" in code

def test_extract_code_generic_block():
    response = "```\nfn main() -> i64 = 42;\n```"
    code = LlmClient.extract_code(response, "bmb")
    assert code == "fn main() -> i64 = 42;"
