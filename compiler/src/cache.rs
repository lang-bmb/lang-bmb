//! Proof cache (backlog 2D-01). An obligation is cached only once proved, and
//! its key is the SHA-256 of the exact text the solver received (options,
//! prelude, revealed definitions, declarations, query; with canonical
//! symbols, ADR-0063, `smt::canonical`) plus the solver version and the
//! memory limit the query is asked under (`smt::query_memory_mb`, ADR-0129).
//! The same text gives the same answer from the pinned solver under the same
//! limits, so a hit is as good as running it. Anything that changes the query
//! (a callee's contract, a revealed definition, the encoding, `rlimit`, the
//! seed) changes the key, and so does the memory limit, which is not in the
//! text: a proof found under the gate's limit does not answer the lint's
//! advisory query of the same text, which runs under a smaller one (backlog
//! T2-17). An advisory query's `rlimit` is its own too (ADR-0131), and that
//! one is in the text (`verify::query_header`), so the two differ there as
//! well. Unproved obligations always run again.
//!
//! The file is trusted like the source tree: whoever can write it can mark an
//! obligation proved. It lives in the workspace (`.bmb-cache/proofs`).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const FILE: &str = "proofs";
/// v5: the key covers the memory limit (ADR-0129); a v4 file is not read.
const MAGIC: &str = "bmb proof cache v5";

/// Stage bits of an entry: the ground-instance retry ran at the probe's
/// share (backlog 1B-46), and after the full query (it proved).
pub const EARLY_RETRY: u8 = 1;
pub const LATE_RETRY: u8 = 2;

#[derive(Default)]
pub struct ProofCache {
    /// key -> (contribution to the definition's `rlimit` total, to its max,
    /// the retries that ran: `EARLY_RETRY`, `LATE_RETRY`). The stage bits
    /// let a hit print the same script as the run that stored it (backlog
    /// 2V-03).
    known: BTreeMap<String, (u64, u64, u8)>,
    added: usize,
    pub hits: usize,
    pub misses: usize,
}

impl ProofCache {
    /// The key of `query` as the encoder writes it, asked now: the text the
    /// solver receives is its canonical form (ADR-0063), so a renamed
    /// program's query has the same key; the memory limit is the one a query
    /// asked now is decided under (the gate's, or the advisory one).
    pub fn key(query: &str) -> String {
        let mut text = String::from(crate::toolchain::z3_version());
        text.push_str(&format!("\nmemory {}", crate::smt::query_memory_mb()));
        text.push('\n');
        text.push_str(&crate::smt::canonical(query));
        crate::sha256::hex(text.as_bytes())
    }

    pub fn get(&mut self, key: &str) -> Option<(u64, u64, u8)> {
        let hit = self.known.get(key).copied();
        if hit.is_some() {
            self.hits += 1;
        } else {
            self.misses += 1;
        }
        hit
    }

    pub fn insert(&mut self, key: String, total: u64, max: u64, stages: u8) {
        if self.known.insert(key, (total, max, stages)).is_none() {
            self.added += 1;
        }
    }

    pub fn path(dir: &Path) -> PathBuf {
        dir.join(FILE)
    }

    /// Loads `dir/proofs`; a missing or foreign file is an empty cache.
    pub fn load(dir: &Path) -> ProofCache {
        let mut c = ProofCache::default();
        let Ok(text) = std::fs::read_to_string(Self::path(dir)) else {
            return c;
        };
        let mut lines = text.lines();
        if lines.next() != Some(MAGIC) {
            return c;
        }
        for line in lines {
            let mut it = line.split(' ');
            if let (Some(k), Some(t), Some(m), Some(r)) =
                (it.next(), it.next(), it.next(), it.next())
                && let (Ok(t), Ok(m)) = (t.parse(), m.parse())
                && k.starts_with("sha256:")
                && let Ok(r) = r.parse::<u8>()
                && r <= EARLY_RETRY | LATE_RETRY
            {
                c.known.insert(k.to_string(), (t, m, r));
            }
        }
        c
    }

    /// Writes the cache back when something was added. Sorted, so the file
    /// is deterministic.
    pub fn save(&self, dir: &Path) -> std::io::Result<()> {
        if self.added == 0 {
            return Ok(());
        }
        std::fs::create_dir_all(dir)?;
        let mut out = format!("{MAGIC}\n");
        for (k, (t, m, r)) in &self.known {
            out.push_str(&format!("{k} {t} {m} {r}\n"));
        }
        let tmp = dir.join(format!("{FILE}.tmp"));
        crate::workspace::write_new(&tmp, &out)?;
        std::fs::rename(tmp, Self::path(dir))
    }
}
