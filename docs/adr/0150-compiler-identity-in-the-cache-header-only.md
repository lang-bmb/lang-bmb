# ADR-0150: The compiler identity guards the incremental cache in its header only

- Status: Accepted (autonomous)
- Date: 2026-11-25
- Related: ADR-0032 (incremental build key; this record changes one row of its key), ADR-0043

## Context

- ADR-0032 put the compiler identity in two places of the incremental cache: the file's header, and every proof key and IR key. A file whose header names another build is not read at all, so the identity inside the keys never decided anything.
- The compiler is being ported to BMB, and the port is compared with the Rust compiler byte for byte. Two builds have two identities. With the identity inside every key, no line of the two cache files could be the same, and the port's cache could not be compared with the original's.

## Decision

1. The compiler identity is in the header of the cache file and nowhere else. The proof key and the IR key no longer contain it.
2. A file whose header names another build is still not read, and is rewritten. This is the only guard between builds, and each build checks it: the Rust compiler names itself by its version and the digest of its sources, the port by the digest of its own executable. A port that cannot read its executable reads no cache file.
3. Everything else of ADR-0032 stays: the format version, the pinned Z3 in the proof key, the digest on every line.

## Rationale

- Nothing that was rejected before is accepted now: the header already rejected every file of another build before any key was looked at. A test shows both halves: two builds write the same entry lines under their headers, and neither reuses anything of the other's file.
- What is given up is a second check behind the first one. It would matter only for a file whose header was forged or whose lines were copied under another header by hand. The cache is not a trust boundary: a forged file could as well carry forged keys.
- Alternative: keep the identity in the keys and compare the two compilers through a dump that leaves it out. Rejected because the file itself would then never be compared, and the file is what a later run reads.
