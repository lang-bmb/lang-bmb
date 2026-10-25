#!/usr/bin/env bash
# Prepares the pinned toolchain (INF-02, INF-03, INF-04). Idempotent.
set -euo pipefail
cd "$(dirname "$0")/.."
# shellcheck disable=SC1091
source scripts/toolchain.env

if ! command -v z3 >/dev/null || [[ "$(z3 --version)" != *"version ${Z3_VERSION} "* ]]; then
  echo "installing z3 ${Z3_VERSION}"
  python3 -m pip install --quiet --break-system-packages "z3-solver==${Z3_VERSION}.0" 2>/dev/null \
    || python3 -m pip install --quiet "z3-solver==${Z3_VERSION}.0"
fi

llvm_ok() { command -v llc >/dev/null && llc --version | grep -q "LLVM version ${LLVM_MAJOR}\."; }
if ! llvm_ok; then
  bin="/usr/lib/llvm-${LLVM_MAJOR}/bin"
  if [ ! -x "$bin/llc" ] && command -v apt-get >/dev/null; then
    echo "installing LLVM ${LLVM_MAJOR}"
    apt-get install -y -qq "llvm-${LLVM_MAJOR}" "clang-${LLVM_MAJOR}" >/dev/null
  fi
  # Distribution packages install versioned tools off PATH; expose them.
  if [ -x "$bin/llc" ]; then
    for t in llc opt clang llvm-as; do
      [ -x "$bin/$t" ] && ln -sf "$bin/$t" "/usr/local/bin/$t"
    done
  fi
  llvm_ok || { echo "LLVM ${LLVM_MAJOR} is required (llc, opt, clang)" >&2; exit 1; }
fi

# The Vec soundness fuzzer's free check needs valgrind or clang's
# AddressSanitizer runtime (ADR-0031 adoption note 9).
asan_ok() {
  local d
  d="$(mktemp -d)"
  printf 'int main(void) { return 0; }\n' > "$d/p.c"
  clang -fsanitize=address -o "$d/p" "$d/p.c" >/dev/null 2>&1 && "$d/p"
  local r=$?
  rm -rf "$d"
  return $r
}
if ! command -v valgrind >/dev/null && ! asan_ok && command -v apt-get >/dev/null; then
  echo "installing valgrind"
  apt-get install -y -qq valgrind >/dev/null || { apt-get update -qq && apt-get install -y -qq valgrind >/dev/null; }
fi
command -v valgrind >/dev/null || asan_ok || { echo "valgrind or AddressSanitizer is required" >&2; exit 1; }

z3 --version
llc --version | grep "LLVM version"
