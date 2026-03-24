def build_initial_prompt(problem_desc: str, lang: str,
                         test_preview: str, reference: str | None = None) -> str:
    """Build the initial code generation prompt. Fair across languages.

    Kept concise to minimize token usage.
    """
    parts = []
    if reference:
        parts.append(f"## {lang} Reference\n{reference}\n---")
    parts.append(f"{problem_desc}")
    parts.append(f"## Examples\n{test_preview}")
    parts.append(f"\nWrite a complete {lang} program. Output ONLY code in a ```{lang} block.")
    return "\n".join(parts)

def build_error_feedback_prompt(error_type: str, normalized_msg: str,
                                 location: str, raw_output: str) -> str:
    """Build error feedback prompt. Same format for all languages.

    Kept short — only error message and instruction.
    """
    # Truncate raw output to avoid blowing context
    raw_truncated = raw_output[:500] if len(raw_output) > 500 else raw_output
    return f"""{error_type}: {normalized_msg}
Location: {location}

{raw_truncated}

Fix the error. Output ONLY the complete corrected code in a code block."""
