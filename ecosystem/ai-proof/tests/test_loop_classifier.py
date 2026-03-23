from orchestrator.loop_classifier import classify_loop, LoopType


def test_classify_contract_violation():
    error = {"type": "compile_error", "normalized": "pre-condition violated: idx < len", "raw": "..."}
    assert classify_loop(error, lang="bmb") == LoopType.A_CONTRACT


def test_classify_syntax_error():
    error = {"type": "compile_error", "normalized": "unexpected token '}'", "raw": "..."}
    assert classify_loop(error, lang="bmb") == LoopType.B_SYNTAX


def test_classify_semantic_error():
    error = {"type": "compile_error", "normalized": "type mismatch: expected i64, found bool", "raw": "..."}
    assert classify_loop(error, lang="bmb") == LoopType.C_SEMANTIC


def test_classify_test_failure():
    error = {"type": "test_failure", "normalized": "expected '42', got '0'", "raw": "..."}
    assert classify_loop(error, lang="bmb") == LoopType.D_TEST
