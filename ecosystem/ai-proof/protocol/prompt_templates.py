def build_initial_prompt(problem_desc: str, lang: str,
                         test_preview: str, reference: str | None = None) -> str:
    """Build the initial code generation prompt. Fair across languages."""
    parts = [f"# Language: {lang}\n"]
    if reference:
        parts.append(f"## {lang} Language Reference\n\n{reference}\n\n---\n")
    parts.append(f"{problem_desc}\n")
    parts.append(f"## Test Cases\n{test_preview}\n")
    parts.append(f"\nWrite a complete, compilable {lang} program that satisfies the above requirements.")
    return "\n".join(parts)

def build_error_feedback_prompt(error_type: str, normalized_msg: str,
                                 location: str, raw_output: str) -> str:
    """Build error feedback prompt. Same format for all languages."""
    return f"""Build/test failed.

[Error type]: {error_type}
[Message]: {normalized_msg}
[Location]: {location}

Raw compiler output:
{raw_output}

Fix the error and provide the complete corrected code."""
