from scripts.strip_contracts import strip_contracts


def test_strip_pre():
    source = """fn safe_get(arr: &[i64; 10], idx: i64) -> i64
    pre idx >= 0 and idx < 10
= arr[idx];"""
    result = strip_contracts(source)
    assert "pre" not in result
    assert "fn safe_get" in result
    assert "arr[idx]" in result


def test_strip_post():
    source = """fn sort(arr: &mut [i64]) -> &[i64]
    post is_sorted(ret)
{ /* body */ }"""
    result = strip_contracts(source)
    assert "post" not in result
    assert "fn sort" in result


def test_strip_multi_line_contract():
    source = """fn foo(x: i64, y: i64) -> i64
    pre x > 0
    and y > 0
    post ret > 0
= x + y;"""
    result = strip_contracts(source)
    assert "pre" not in result
    assert "post" not in result
    assert "and y" not in result
    assert "fn foo" in result
    assert "x + y" in result


def test_strip_preserves_non_contract():
    source = "fn add(a: i64, b: i64) -> i64 = a + b;"
    result = strip_contracts(source)
    assert result.strip() == source.strip()


def test_no_false_positive_preprocess():
    source = """fn preprocess(data: &[i64]) -> i64 = data[0];
fn postfix(s: &str) -> &str = s;"""
    result = strip_contracts(source)
    assert "preprocess" in result
    assert "postfix" in result


def test_strip_both_pre_and_post():
    source = """fn bounded_add(a: i64, b: i64) -> i64
    pre a >= 0
    pre b >= 0
    post ret >= 0
= a + b;"""
    result = strip_contracts(source)
    assert "pre" not in result
    assert "post" not in result
    assert "fn bounded_add" in result
    assert "a + b" in result
