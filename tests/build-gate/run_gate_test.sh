#!/usr/bin/env bash
# CT-G build-gate acceptance test. Exercises `compiler.exe build
# --require-verified <manifest>` and asserts the gate fails-closed: only an
# all-`verified` manifest builds; anything else (refuted / absent / missing
# manifest) refuses the build with a nonzero exit. NOT wired into CI
# (compiler.exe is Windows-only — see HANDOFF CI-wiring). Run locally:
#   bash tests/build-gate/run_gate_test.sh
set -u
cd "$(dirname "$0")/../.." || exit 2
EXE=bootstrap/compiler.exe
F=tests/build-gate
OUT=$F/.gate_out
export BMB_ARENA_MAX_SIZE=32G
fails=0

check() { # $1=label  $2=expected_exit  $3=expected_status  $4=manifest
  rm -f "$OUT"* "$F/gate_demo" "$F/gate_demo.exe" "$F/gate_demo.ll" "$F/gate_demo_opt.ll" 2>/dev/null
  out=$("$EXE" build "$F/gate_demo.bmb" --require-verified "$4" -o "$OUT" 2>&1)
  code=$?
  line=$(echo "$out" | grep -E '"gate"' | head -1)
  if [ "$code" = "$2" ] && echo "$line" | grep -q "\"status\":\"$3\""; then
    echo "PASS  $1 (exit=$code status=$3)"
  else
    echo "FAIL  $1 (exit=$code, expected $2; line=$line)"; fails=$((fails+1))
  fi
}

check "all-verified manifest builds"        0 pass  "$F/manifest_pass.txt"
check "refuted in manifest refuses"         1 fail  "$F/manifest_fail.txt"
check "absent fn in manifest refuses"       1 fail  "$F/manifest_absent.txt"
check "missing manifest errors"             1 error "$F/does_not_exist.txt"

# In-language @verify gate (no flag — the manifest is the source itself).
# A @verify-annotated function must pass verify-vc or the build is refused.
vcheck() { # $1=label  $2=fixture  $3=expected_exit  $4=expected_status
  rm -f "$OUT"* "$F/$2" "$F/$2.exe" "$F/$2.ll" "$F/${2}_opt.ll" 2>/dev/null
  out=$("$EXE" build "$F/$2.bmb" -o "$OUT" 2>&1)
  code=$?
  line=$(echo "$out" | grep -E '"gate"' | head -1)
  if [ "$code" = "$3" ] && echo "$line" | grep -q "\"gate\":\"verify-annotation\",\"status\":\"$4\""; then
    echo "PASS  $1 (exit=$code status=$4)"
  else
    echo "FAIL  $1 (exit=$code, expected $3; line=$line)"; fails=$((fails+1))
  fi
}
vcheck "@verify verifiable fn builds"        verify_demo_pass 0 pass
vcheck "@verify refuted fn refuses"          verify_demo_fail 1 fail

# baseline: no flag => no gate, normal build, exit 0
rm -f "$OUT"* "$F/gate_demo" "$F/gate_demo.exe" "$F/gate_demo.ll" "$F/gate_demo_opt.ll" 2>/dev/null
"$EXE" build "$F/gate_demo.bmb" -o "$OUT" >/dev/null 2>&1
if [ $? = 0 ] && [ -f "$OUT" ]; then echo "PASS  no-flag baseline builds"; else echo "FAIL  no-flag baseline"; fails=$((fails+1)); fi

rm -f "$OUT"* "$F/gate_demo" "$F/gate_demo.exe" "$F/gate_demo.ll" "$F/gate_demo_opt.ll" 2>/dev/null
echo "---"
if [ "$fails" = 0 ]; then echo "ALL GATE TESTS PASS"; exit 0; else echo "$fails GATE TEST(S) FAILED"; exit 1; fi
