#!/bin/sh
# The ends of `verify` that are no verdict, on the file nb.bmb beside this
# script: a directory for the scripts that cannot be made, a script and a
# legend that cannot be written, a statistics file that cannot be written,
# a proof cache that cannot be written (which is said and does not change
# the verdict), definitions that are not in the program, a file that cannot
# be read. The resource limit is lowered: no run here is about a proof.
#   errors.sh BMB
bmb=$1
here=$(cd "$(dirname "$0")" && pwd)
nb=$here/nb.bmb
BMB_Z3_RLIMIT=200000
export BMB_Z3_RLIMIT
run() {
  "$bmb" verify "$@"
  echo "exit $?"
}
run --emit-smt /dev/null/smt "$nb"
mkdir -p script/nb.f.smt2 legend/nb.g.smt2.legend
run --emit-smt script "$nb"
run --emit-smt legend "$nb"
run --stats /dev/null/stats "$nb"
run --cache /dev/null/cache --emit-smt a "$nb"
mkdir -p tmp/proofs.tmp held/proofs
run --cache tmp --emit-smt b "$nb"
run --cache held --emit-smt c "$nb"
run --def nb::nope --def nb::f --def other::g --def nb::f "$nb"
run --def nb::f --def nb::in_rows --emit-smt d "$nb"
run "$here/absent.bmb"
run --audit --bounded-search --def nb::g "$nb"
run --cache
run --emit-smt e
run --def trust::above --def trust::ignores --def trust::outside --emit-smt f "$here/trust.bmb"
run --def trust::bad --bounded-search "$here/trust.bmb"
