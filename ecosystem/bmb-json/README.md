# bmb-json — Fast JSON Parser

Zero-copy JSON parser and serializer compiled from [BMB](https://github.com/iyulab/lang-bmb).

## Installation

```bash
pip install bmb-json
```

## Quick Start

```python
import bmb_json

bmb_json.validate('{"name": "BMB"}')                  # True
bmb_json.get_string('{"name": "BMB"}', "name")        # "BMB"
bmb_json.get_number('{"version": 97}', "version")     # 97
bmb_json.array_len('[1, 2, 3]')                        # 3
bmb_json.array_get('[10, 20, 30]', 1)                  # '20'
bmb_json.stringify('{ "a" : 1 , "b" : [2,3] }')       # '{"a":1,"b":[2,3]}'
bmb_json.get_type('{"a":1}')                           # 'object'
```

## Full API (12 functions)

| Function | Description | Return |
|----------|-------------|--------|
| `validate(json_str)` | Check valid JSON | `bool` |
| `stringify(json_str)` | Roundtrip normalization (minified) | `str` |
| `get_type(json_str)` | Root value type | `"null"` / `"bool"` / `"number"` / `"string"` / `"array"` / `"object"` |
| `get(json_str, key)` | Get value as raw JSON | `str` (empty if missing) |
| `get_string(json_str, key)` | Get string value (unquoted) | `str` |
| `get_number(json_str, key)` | Get number value | `int` (0 if missing) |
| `array_len(json_str)` | Array length | `int` |
| `array_get(json_str, idx)` | Array element as JSON | `str` (empty if out of bounds) |
| `has_key(json_str, key)` | Check if object has key | `bool` |
| `object_len(json_str)` | Number of keys in object | `int` (-1 for non-objects) |
| `get_bool(json_str, key)` | Get boolean value | `int` (1/0/-1) |
| `count(json_str)` | Count elements (shallow) | `int` |

All outputs cross-validated against Python's `json.loads` / `json.dumps`.

## How?

Written in [BMB](https://github.com/iyulab/lang-bmb) — compile-time contracts prove correctness, then generate code faster than hand-tuned C.

## License

MIT
