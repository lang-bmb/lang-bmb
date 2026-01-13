#!/bin/bash
# Build unified BMB compiler from Bootstrap sources
# v0.46 Independence Phase (v0.32 syntax)

OUTPUT="bootstrap/bmb_unified.bmb"
echo "// BMB Unified Compiler (v0.46 Independence)" > $OUTPUT
echo "// Auto-generated from Bootstrap sources" >> $OUTPUT
echo "// Generated: $(date -Iseconds)" >> $OUTPUT
echo "// Syntax: v0.32 (braced if-else, // comments)" >> $OUTPUT
echo "" >> $OUTPUT

# Order matters: dependencies first
FILES=(
    "bootstrap/utils.bmb"
    "bootstrap/lexer.bmb"
    "bootstrap/parser.bmb"
    "bootstrap/parser_ast.bmb"
    "bootstrap/types.bmb"
    "bootstrap/mir.bmb"
    "bootstrap/lowering.bmb"
    "bootstrap/llvm_ir.bmb"
    "bootstrap/optimize.bmb"
    "bootstrap/compiler.bmb"
    "bootstrap/pipeline.bmb"
)

for file in "${FILES[@]}"; do
    echo "// ========================================" >> $OUTPUT
    echo "// Source: $file" >> $OUTPUT
    echo "// ========================================" >> $OUTPUT
    # Skip existing main() functions to avoid duplicates
    grep -v "^fn main()" "$file" | grep -v "^// Test:" >> $OUTPUT
    echo "" >> $OUTPUT
done

# Add CLI main function with v0.32 syntax
cat >> $OUTPUT << 'MAIN'

// ========================================
// Unified CLI Entry Point
// ========================================

fn print_str_nl(s: String) -> i64 =
    let x = print_str(s);
    let y = print_str("
");
    0;

fn unified_main() -> i64 =
    let argc = arg_count();
    if argc < 2 {
        let x = print_str_nl("BMB Unified Compiler (v0.46)");
        let y = print_str_nl("Usage: bmb_unified <input.bmb> [output.ll]");
        1
    } else {
        let input_file = get_arg(1);
        let source = read_file(input_file);
        if source == "" {
            let x = print_str_nl("Error: Cannot read file");
            2
        } else {
            // Full compilation pipeline
            let result = compile_pipeline(source);
            if argc >= 3 {
                let output_file = get_arg(2);
                let w = write_file(output_file, result);
                0
            } else {
                let x = print_str(result);
                0
            }
        }
    };

fn main() -> i64 = unified_main();
MAIN

echo "Generated: $OUTPUT"
wc -l $OUTPUT
