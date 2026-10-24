//! Proof cache (backlog 2D-01). An obligation is cached only once proved, and
//! its key is the SHA-256 of the exact text the solver received (options,
//! prelude, revealed definitions, declarations, query) plus the solver
//! version. The same text gives the same answer from the pinned solver, so a
//! hit is as good as running it. Anything that changes the query (a callee's
//! contract, a revealed definition, the encoding, `rlimit`, the seed) changes
//! the key; unproved obligations always run again.
//!
//! The file is trusted like the source tree: whoever can write it can mark an
//! obligation proved. It lives in the workspace (`.bmb-cache/proofs`).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const FILE: &str = "proofs";
const MAGIC: &str = "bmb proof cache v1";

#[derive(Default)]
pub struct ProofCache {
    /// key -> (contribution to the definition's `rlimit` total, to its max).
    known: BTreeMap<String, (u64, u64)>,
    added: usize,
    pub hits: usize,
    pub misses: usize,
}

impl ProofCache {
    pub fn key(query: &str) -> String {
        let mut text = String::from(crate::toolchain::z3_version());
        text.push('\n');
        text.push_str(query);
        crate::sha256::hex(text.as_bytes())
    }

    pub fn get(&mut self, key: &str) -> Option<(u64, u64)> {
        let hit = self.known.get(key).copied();
        if hit.is_some() {
            self.hits += 1;
        } else {
            self.misses += 1;
        }
        hit
    }

    pub fn insert(&mut self, key: String, total: u64, max: u64) {
        if self.known.insert(key, (total, max)).is_none() {
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
            if let (Some(k), Some(t), Some(m)) = (it.next(), it.next(), it.next())
                && let (Ok(t), Ok(m)) = (t.parse(), m.parse())
                && k.starts_with("sha256:")
            {
                c.known.insert(k.to_string(), (t, m));
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
        for (k, (t, m)) in &self.known {
            out.push_str(&format!("{k} {t} {m}\n"));
        }
        let tmp = dir.join(format!("{FILE}.tmp"));
        std::fs::write(&tmp, out)?;
        std::fs::rename(tmp, Self::path(dir))
    }
}
