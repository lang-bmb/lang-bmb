#!/bin/bash
# BMB CI Results Reporter
# Part of the Bootstrap + Benchmark Cycle System
#
# Generates formatted reports for CI output (GitHub Actions, etc.)
#
# Usage:
#   ./scripts/ci/report-results.sh <results-dir> [options]
#
# Options:
#   --format FORMAT    Output format: markdown, text, json (default: markdown)
#   --output FILE      Write report to file
#   --summary          Also write to GITHUB_STEP_SUMMARY

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Parse arguments
RESULTS_DIR=""
FORMAT="markdown"
OUTPUT_FILE=""
SUMMARY=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --format)
            FORMAT="$2"
            shift 2
            ;;
        --output)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        --summary)
            SUMMARY=true
            shift
            ;;
        -*)
            echo "Unknown option: $1"
            exit 1
            ;;
        *)
            RESULTS_DIR="$1"
            shift
            ;;
    esac
done

if [ -z "$RESULTS_DIR" ] || [ ! -d "$RESULTS_DIR" ]; then
    echo "Usage: $0 <results-dir> [options]"
    exit 1
fi

# Generate report based on format
generate_markdown_report() {
    local dir="$1"

    echo "# BMB CI Report"
    echo ""
    echo "Generated: $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
    echo ""

    # Bootstrap Results
    if [ -f "$dir/bootstrap.json" ]; then
        echo "## Bootstrap Verification"
        echo ""

        STAGE1_SUCCESS=$(python3 -c "import json; print(json.load(open('$dir/bootstrap.json'))['bootstrap']['stage1']['success'])" 2>/dev/null || echo "unknown")
        STAGE2_SUCCESS=$(python3 -c "import json; print(json.load(open('$dir/bootstrap.json'))['bootstrap']['stage2']['success'])" 2>/dev/null || echo "unknown")
        STAGE3_SUCCESS=$(python3 -c "import json; print(json.load(open('$dir/bootstrap.json'))['bootstrap']['stage3']['success'])" 2>/dev/null || echo "unknown")
        FIXED_POINT=$(python3 -c "import json; print(json.load(open('$dir/bootstrap.json'))['bootstrap']['fixed_point'])" 2>/dev/null || echo "unknown")
        STAGE1_TIME=$(python3 -c "import json; print(json.load(open('$dir/bootstrap.json'))['bootstrap']['stage1']['time_ms'])" 2>/dev/null || echo "0")

        echo "| Stage | Status | Time |"
        echo "|-------|--------|------|"
        echo "| Stage 0→1 | $STAGE1_SUCCESS | ${STAGE1_TIME}ms |"
        echo "| Stage 1→2 | $STAGE2_SUCCESS | - |"
        echo "| Stage 2→3 | $STAGE3_SUCCESS | - |"
        echo "| Fixed Point | $FIXED_POINT | - |"
        echo ""
    fi

    # Benchmark Results
    if [ -f "$dir/benchmarks.json" ]; then
        echo "## Benchmark Results"
        echo ""

        # Parse and display tier 1 results
        python3 <<EOF
import json
data = json.load(open('$dir/benchmarks.json'))

tier1 = [r for r in data.get('results', []) if r.get('tier') == 1]

if tier1:
    print("### Tier 1: Core Compute")
    print("")
    print("| Benchmark | BMB (ms) | C (ms) | Ratio |")
    print("|-----------|----------|--------|-------|")

    for r in tier1:
        name = r.get('name', 'unknown')
        bmb = r.get('bmb_ms', 'N/A')
        c = r.get('c_ms', 'N/A')
        ratio = r.get('ratio_c', 'N/A')

        if bmb != 'null' and bmb is not None:
            bmb = f"{bmb}"
        else:
            bmb = "FAIL"

        if c != 'null' and c is not None:
            c = f"{c}"
        else:
            c = "N/A"

        if ratio != 'null' and ratio is not None:
            ratio = f"{ratio}x"
        else:
            ratio = "-"

        print(f"| {name} | {bmb} | {c} | {ratio} |")

    print("")
EOF
    fi

    # Comparison Results
    if [ -f "$dir/comparison_report.txt" ]; then
        echo "## Comparison Report"
        echo ""
        echo "\`\`\`"
        cat "$dir/comparison_report.txt"
        echo "\`\`\`"
        echo ""
    fi

    # Performance Gate Status
    echo "## Performance Gates"
    echo ""
    echo "| Gate | Threshold | Status |"
    echo "|------|-----------|--------|"

    if [ -f "$dir/bootstrap.json" ]; then
        if [ "$FIXED_POINT" = "true" ] || [ "$FIXED_POINT" = "True" ]; then
            echo "| Bootstrap Fixed Point | - | PASSED |"
        else
            echo "| Bootstrap Fixed Point | - | PENDING |"
        fi
    fi

    if [ -f "$dir/comparison_report.txt" ]; then
        if grep -q "FAILED" "$dir/comparison_report.txt"; then
            echo "| Tier 1 Regression | 2% | FAILED |"
        else
            echo "| Tier 1 Regression | 2% | PASSED |"
        fi
    fi
    echo ""
}

generate_text_report() {
    local dir="$1"

    echo "BMB CI Report"
    echo "============="
    echo ""
    echo "Generated: $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
    echo ""

    if [ -f "$dir/bootstrap.json" ]; then
        echo "Bootstrap Verification"
        echo "----------------------"
        cat "$dir/bootstrap.json" | python3 -m json.tool 2>/dev/null || cat "$dir/bootstrap.json"
        echo ""
    fi

    if [ -f "$dir/benchmarks.json" ]; then
        echo "Benchmark Results"
        echo "-----------------"
        cat "$dir/benchmarks.json" | python3 -m json.tool 2>/dev/null || cat "$dir/benchmarks.json"
        echo ""
    fi

    if [ -f "$dir/comparison_report.txt" ]; then
        echo "Comparison Report"
        echo "-----------------"
        cat "$dir/comparison_report.txt"
        echo ""
    fi
}

generate_json_report() {
    local dir="$1"

    echo "{"
    echo "  \"generated\": \"$(date -u '+%Y-%m-%dT%H:%M:%SZ')\","

    if [ -f "$dir/bootstrap.json" ]; then
        echo "  \"bootstrap\": $(cat "$dir/bootstrap.json"),"
    fi

    if [ -f "$dir/benchmarks.json" ]; then
        echo "  \"benchmarks\": $(cat "$dir/benchmarks.json"),"
    fi

    echo "  \"status\": \"complete\""
    echo "}"
}

# Generate report
REPORT=""
case $FORMAT in
    markdown)
        REPORT=$(generate_markdown_report "$RESULTS_DIR")
        ;;
    text)
        REPORT=$(generate_text_report "$RESULTS_DIR")
        ;;
    json)
        REPORT=$(generate_json_report "$RESULTS_DIR")
        ;;
    *)
        echo "Unknown format: $FORMAT"
        exit 1
        ;;
esac

# Output
if [ -n "$OUTPUT_FILE" ]; then
    echo "$REPORT" > "$OUTPUT_FILE"
    echo "Report written to: $OUTPUT_FILE"
else
    echo "$REPORT"
fi

# GitHub Actions summary
if [ "$SUMMARY" = true ] && [ -n "$GITHUB_STEP_SUMMARY" ]; then
    echo "$REPORT" >> "$GITHUB_STEP_SUMMARY"
fi
