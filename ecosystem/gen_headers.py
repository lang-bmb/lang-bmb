#!/usr/bin/env python3
"""
Generate C header files (.h) from BMB @export function signatures.

Usage:
    python gen_headers.py                    # Generate all headers
    python gen_headers.py bmb-algo           # Generate one header
    python gen_headers.py --output-dir ./out # Custom output directory
"""

import re
import os
import sys
import argparse
from datetime import date

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))

# BMB type -> C type mapping
TYPE_MAP = {
    "i64": "int64_t",
    "i32": "int32_t",
    "i8": "int8_t",
    "bool": "int64_t",
    "String": "void*",  # BMB string handle (opaque)
}

LIBRARIES = {
    "bmb-algo": {
        "src": "bmb-algo/src/lib.bmb",
        "guard": "BMB_ALGO_H",
        "description": "High-performance algorithms",
    },
    "bmb-compute": {
        "src": "bmb-compute/src/lib.bmb",
        "guard": "BMB_COMPUTE_H",
        "description": "Numeric computation",
    },
    "bmb-crypto": {
        "src": "bmb-crypto/src/lib.bmb",
        "guard": "BMB_CRYPTO_H",
        "description": "Cryptographic functions",
    },
    "bmb-text": {
        "src": "bmb-text/src/lib.bmb",
        "guard": "BMB_TEXT_H",
        "description": "String processing",
    },
    "bmb-json": {
        "src": "bmb-json/src/lib.bmb",
        "guard": "BMB_JSON_H",
        "description": "JSON parsing",
    },
}


def parse_exports(bmb_path):
    """Parse @export function signatures from a .bmb file."""
    with open(bmb_path, "r", encoding="utf-8") as f:
        content = f.read()

    # Find @export followed by pub fn
    pattern = r"@export\s*\n\s*pub\s+fn\s+(\w+)\(([^)]*)\)\s*->\s*(\w+)"
    functions = []

    for match in re.finditer(pattern, content):
        name = match.group(1)
        params_str = match.group(2)
        ret_type = match.group(3)

        # Parse parameters
        params = []
        if params_str.strip():
            for param in params_str.split(","):
                param = param.strip()
                if ":" in param:
                    pname, ptype = param.split(":", 1)
                    pname = pname.strip()
                    ptype = ptype.strip()
                    c_type = TYPE_MAP.get(ptype, "void*")
                    params.append((pname, c_type))

        c_ret = TYPE_MAP.get(ret_type, "void*")
        functions.append((name, params, c_ret))

    # Also find preceding comments for each function
    comment_pattern = r"// ([^\n]+)\n@export\s*\n\s*pub\s+fn\s+(\w+)"
    comments = {}
    for match in re.finditer(comment_pattern, content):
        comments[match.group(2)] = match.group(1)

    return functions, comments


def generate_header(lib_name, config, output_dir):
    """Generate a C header file for a library."""
    src_path = os.path.join(SCRIPT_DIR, config["src"])
    functions, comments = parse_exports(src_path)

    if not functions:
        print(f"  WARNING: No @export functions found in {src_path}")
        return None

    module = lib_name.replace("-", "_")
    header_name = f"{module}.h"
    header_path = os.path.join(output_dir, header_name)

    guard = config["guard"]
    desc = config["description"]

    lines = [
        f"/**",
        f" * {module}.h — {desc}",
        f" *",
        f" * Auto-generated from BMB source. Do not edit manually.",
        f" * Generated: {date.today().isoformat()}",
        f" *",
        f" * Usage:",
        f" *   #include \"{header_name}\"",
        f" *   // Link with {module}.dll / lib{module}.so / lib{module}.dylib",
        f" */",
        f"",
        f"#ifndef {guard}",
        f"#define {guard}",
        f"",
        f"#include <stdint.h>",
        f"",
        f"#ifdef __cplusplus",
        f'extern "C" {{',
        f"#endif",
        f"",
        f"/* FFI Safety API */",
        f"int bmb_ffi_begin(void);",
        f"void bmb_ffi_end(void);",
        f"int bmb_ffi_has_error(void);",
        f"const char* bmb_ffi_error_message(void);",
        f"",
        f"/* String FFI API */",
        f"void* bmb_ffi_cstr_to_string(const char* s);",
        f"const char* bmb_ffi_string_data(void* s);",
        f"int64_t bmb_ffi_string_len(void* s);",
        f"void bmb_ffi_free_string(void* s);",
        f"",
        f"/* {desc} — {len(functions)} functions */",
        f"",
    ]

    # Group by section (based on comment headers)
    current_section = None
    for name, params, ret_type in functions:
        # Add comment if available
        comment = comments.get(name, "")
        if comment:
            lines.append(f"/** {comment} */")

        # Build parameter list
        if params:
            param_str = ", ".join(f"{ctype} {pname}" for pname, ctype in params)
        else:
            param_str = "void"

        lines.append(f"{ret_type} {name}({param_str});")
        lines.append("")

    lines.extend([
        f"#ifdef __cplusplus",
        f"}}",
        f"#endif",
        f"",
        f"#endif /* {guard} */",
    ])

    with open(header_path, "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(lines))

    return len(functions)


def main():
    parser = argparse.ArgumentParser(description="Generate C headers from BMB @export functions")
    parser.add_argument("libraries", nargs="*", help="Libraries to generate (default: all)")
    parser.add_argument("--output-dir", default=None, help="Output directory (default: each lib's include/)")
    args = parser.parse_args()

    targets = args.libraries if args.libraries else list(LIBRARIES.keys())

    print("=== BMB C Header Generation ===")
    print()

    total = 0
    for lib_name in targets:
        if lib_name not in LIBRARIES:
            print(f"Unknown library: {lib_name}")
            continue

        config = LIBRARIES[lib_name]

        if args.output_dir:
            out_dir = args.output_dir
        else:
            out_dir = os.path.join(SCRIPT_DIR, lib_name, "include")

        os.makedirs(out_dir, exist_ok=True)

        print(f"[{lib_name}]", end=" ")
        count = generate_header(lib_name, config, out_dir)
        if count:
            module = lib_name.replace("-", "_")
            print(f"{count} functions -> {out_dir}/{module}.h")
            total += count
        else:
            print("SKIP (no exports)")

    print()
    print(f"Total: {total} function declarations generated")


if __name__ == "__main__":
    main()
