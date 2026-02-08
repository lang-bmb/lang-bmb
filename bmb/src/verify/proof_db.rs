//! Proof Database
//!
//! Phase 1.1: Storage and caching infrastructure for verification results.
//! Enables incremental compilation and proof reuse.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::cir::Proposition;

/// Proof database for storing and caching verification results
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProofDatabase {
    /// Function-level verification results
    function_proofs: HashMap<String, FunctionProofResult>,

    /// File hashes for incremental compilation
    file_hashes: HashMap<String, u64>,

    /// Statistics
    stats: ProofDbStats,
}

/// Function identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FunctionId {
    /// Module path
    pub module: String,
    /// Function name
    pub name: String,
    /// Hash of function signature (for detecting signature changes)
    pub signature_hash: u64,
}

impl FunctionId {
    pub fn new(module: &str, name: &str, signature_hash: u64) -> Self {
        Self {
            module: module.to_string(),
            name: name.to_string(),
            signature_hash,
        }
    }

    /// Create from function name only (module defaults to "main")
    pub fn simple(name: &str) -> Self {
        Self {
            module: "main".to_string(),
            name: name.to_string(),
            signature_hash: 0,
        }
    }

    /// Get string key for HashMap storage
    pub fn key(&self) -> String {
        format!("{}::{}", self.module, self.name)
    }
}

/// Verification result for a function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionProofResult {
    /// Verification status
    pub status: VerificationStatus,

    /// Proven facts about this function
    pub proven_facts: Vec<ProofFact>,

    /// Verification time
    #[serde(with = "duration_serde")]
    pub verification_time: Duration,

    /// Number of SMT queries used
    pub smt_queries: usize,

    /// Timestamp when verified
    pub verified_at: u64,
}

impl Default for FunctionProofResult {
    fn default() -> Self {
        Self {
            status: VerificationStatus::Unknown,
            proven_facts: Vec::new(),
            verification_time: Duration::ZERO,
            smt_queries: 0,
            verified_at: 0,
        }
    }
}

/// Verification status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    /// Successfully verified
    Verified,
    /// Verification failed with reason
    Failed(String),
    /// Verification was skipped (no contracts)
    Skipped,
    /// Verification timed out
    Timeout,
    /// Verification status unknown
    Unknown,
    /// Trusted (marked with @trust)
    Trusted(String),
}

impl VerificationStatus {
    pub fn is_verified(&self) -> bool {
        matches!(self, Self::Verified | Self::Trusted(_))
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed(_))
    }
}

/// A proven fact about the program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofFact {
    /// The proven proposition
    pub proposition: Proposition,

    /// Where this fact is valid
    pub scope: ProofScope,

    /// How this fact was proven
    pub evidence: ProofEvidence,
}

/// Scope where a proof fact is valid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofScope {
    /// Valid throughout the entire function
    Function(FunctionId),

    /// Valid only at a specific program point
    ProgramPoint {
        function: FunctionId,
        /// Line number or instruction index
        location: u32,
    },

    /// Valid only under a condition
    Conditional {
        /// The condition under which this holds
        condition: Box<Proposition>,
        /// Inner scope
        inner: Box<ProofScope>,
    },
}

/// Evidence for how a fact was proven
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofEvidence {
    /// Proven by SMT solver
    SmtProof {
        /// Query hash for reproducibility
        query_hash: u64,
        /// Solver used
        solver: String,
    },

    /// Derived from precondition
    Precondition,

    /// Derived from type invariant
    TypeInvariant(String),

    /// Propagated from another function call
    FunctionCall {
        callee: FunctionId,
        /// Which postcondition
        postcondition_index: usize,
    },

    /// Assumed (trusted code)
    Trusted(String),

    /// Derived from control flow
    ControlFlow,
}

/// Database statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ProofDbStats {
    /// Total functions stored
    pub functions_stored: usize,
    /// Cache hits
    pub cache_hits: usize,
    /// Cache misses
    pub cache_misses: usize,
    /// Total SMT queries made
    pub total_smt_queries: usize,
    /// Total verification time
    #[serde(with = "duration_serde")]
    pub total_verification_time: Duration,
}

impl ProofDatabase {
    /// Create a new empty proof database
    pub fn new() -> Self {
        Self::default()
    }

    /// Store verification result for a function
    pub fn store_function_proof(&mut self, id: &FunctionId, result: FunctionProofResult) {
        self.stats.total_smt_queries += result.smt_queries;
        self.stats.total_verification_time += result.verification_time;
        self.function_proofs.insert(id.key(), result);
        self.stats.functions_stored = self.function_proofs.len();
    }

    /// Get verification result for a function
    pub fn get_function_proof(&mut self, id: &FunctionId) -> Option<&FunctionProofResult> {
        let key = id.key();
        if self.function_proofs.contains_key(&key) {
            self.stats.cache_hits += 1;
            self.function_proofs.get(&key)
        } else {
            self.stats.cache_misses += 1;
            None
        }
    }

    /// Check if a function has been verified
    pub fn is_verified(&self, id: &FunctionId) -> bool {
        self.function_proofs
            .get(&id.key())
            .map(|r| r.status.is_verified())
            .unwrap_or(false)
    }

    /// Get all proven facts for a function
    pub fn get_proven_facts(&self, id: &FunctionId) -> &[ProofFact] {
        self.function_proofs
            .get(&id.key())
            .map(|r| r.proven_facts.as_slice())
            .unwrap_or(&[])
    }

    /// Invalidate proofs for a file (for incremental compilation)
    pub fn invalidate_file(&mut self, path: &Path) {
        let path_str = path.display().to_string();
        // Remove all functions from this file
        self.function_proofs.retain(|key, _| {
            !key.starts_with(&path_str)
        });
        self.file_hashes.remove(&path_str);
        self.stats.functions_stored = self.function_proofs.len();
    }

    /// Update file hash
    pub fn update_file_hash(&mut self, path: &Path, hash: u64) {
        self.file_hashes.insert(path.display().to_string(), hash);
    }

    /// Check if file hash matches (for incremental compilation)
    pub fn file_hash_matches(&self, path: &Path, hash: u64) -> bool {
        self.file_hashes.get(&path.display().to_string()) == Some(&hash)
    }

    /// Get statistics
    pub fn stats(&self) -> &ProofDbStats {
        &self.stats
    }

    /// Clear all cached proofs
    pub fn clear(&mut self) {
        self.function_proofs.clear();
        self.file_hashes.clear();
        self.stats = ProofDbStats::default();
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Number of stored proofs
    pub fn len(&self) -> usize {
        self.function_proofs.len()
    }

    /// Check if database is empty
    pub fn is_empty(&self) -> bool {
        self.function_proofs.is_empty()
    }

    /// Save database to a file
    pub fn save_to_file(&self, path: &std::path::Path) -> std::io::Result<()> {
        let json = self.to_json()
            .map_err(std::io::Error::other)?;
        std::fs::write(path, json)
    }

    /// Load database from a file
    pub fn load_from_file(path: &std::path::Path) -> std::io::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        Self::from_json(&json)
            .map_err(std::io::Error::other)
    }

    /// Get the default cache file path for a source file
    pub fn cache_path_for(source_path: &std::path::Path) -> PathBuf {
        source_path.with_extension("bmb.proofcache")
    }
}

/// Helper module for Duration serialization
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_millis().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_database_crud() {
        let mut db = ProofDatabase::new();

        let id = FunctionId::simple("foo");
        let result = FunctionProofResult {
            status: VerificationStatus::Verified,
            proven_facts: vec![],
            verification_time: Duration::from_millis(100),
            smt_queries: 5,
            verified_at: 12345,
        };

        // Store
        db.store_function_proof(&id, result);
        assert_eq!(db.len(), 1);

        // Get
        let retrieved = db.get_function_proof(&id);
        assert!(retrieved.is_some());
        assert!(retrieved.unwrap().status.is_verified());

        // is_verified
        assert!(db.is_verified(&id));
        assert!(!db.is_verified(&FunctionId::simple("bar")));

        // Stats
        assert_eq!(db.stats().functions_stored, 1);
        assert_eq!(db.stats().cache_hits, 1);
        assert_eq!(db.stats().total_smt_queries, 5);
    }

    #[test]
    fn test_proof_database_serialization() {
        let mut db = ProofDatabase::new();
        let id = FunctionId::simple("test");
        db.store_function_proof(
            &id,
            FunctionProofResult {
                status: VerificationStatus::Verified,
                proven_facts: vec![],
                verification_time: Duration::from_millis(50),
                smt_queries: 3,
                verified_at: 0,
            },
        );

        // Serialize
        let json = db.to_json().unwrap();
        assert!(json.contains("main::test")); // key format

        // Deserialize
        let db2 = ProofDatabase::from_json(&json).unwrap();
        assert_eq!(db2.len(), 1);
        assert!(db2.is_verified(&FunctionId::simple("test")));
    }

    #[test]
    fn test_file_hash_invalidation() {
        let mut db = ProofDatabase::new();
        let path = PathBuf::from("test.bmb");

        db.update_file_hash(&path, 12345);
        assert!(db.file_hash_matches(&path, 12345));
        assert!(!db.file_hash_matches(&path, 99999));

        db.invalidate_file(&path);
        assert!(!db.file_hash_matches(&path, 12345));
    }

    #[test]
    fn test_verification_status() {
        assert!(VerificationStatus::Verified.is_verified());
        assert!(VerificationStatus::Trusted("reason".to_string()).is_verified());
        assert!(!VerificationStatus::Failed("error".to_string()).is_verified());
        assert!(!VerificationStatus::Skipped.is_verified());

        assert!(VerificationStatus::Failed("error".to_string()).is_failed());
        assert!(!VerificationStatus::Verified.is_failed());
    }

    // ---- Cycle 69: Additional proof DB tests ----

    #[test]
    fn test_function_id_key_format() {
        let id = FunctionId::new("mymod", "myfn", 42);
        assert_eq!(id.key(), "mymod::myfn");
    }

    #[test]
    fn test_function_id_simple_defaults() {
        let id = FunctionId::simple("test_fn");
        assert_eq!(id.module, "main");
        assert_eq!(id.name, "test_fn");
        assert_eq!(id.signature_hash, 0);
    }

    #[test]
    fn test_proof_database_empty() {
        let db = ProofDatabase::new();
        assert!(db.is_empty());
        assert_eq!(db.len(), 0);
    }

    #[test]
    fn test_proof_database_clear() {
        let mut db = ProofDatabase::new();
        let id = FunctionId::simple("foo");
        db.store_function_proof(&id, FunctionProofResult::default());
        assert_eq!(db.len(), 1);

        db.clear();
        assert!(db.is_empty());
        assert_eq!(db.stats().functions_stored, 0);
    }

    #[test]
    fn test_proof_database_get_proven_facts_empty() {
        let db = ProofDatabase::new();
        let id = FunctionId::simple("missing");
        assert!(db.get_proven_facts(&id).is_empty());
    }

    #[test]
    fn test_proof_database_cache_miss() {
        let mut db = ProofDatabase::new();
        let id = FunctionId::simple("not_stored");
        assert!(db.get_function_proof(&id).is_none());
        assert_eq!(db.stats().cache_misses, 1);
    }

    #[test]
    fn test_proof_database_multiple_functions() {
        let mut db = ProofDatabase::new();

        for name in ["f1", "f2", "f3"] {
            let id = FunctionId::simple(name);
            db.store_function_proof(&id, FunctionProofResult {
                status: VerificationStatus::Verified,
                ..FunctionProofResult::default()
            });
        }

        assert_eq!(db.len(), 3);
        assert!(db.is_verified(&FunctionId::simple("f1")));
        assert!(db.is_verified(&FunctionId::simple("f2")));
        assert!(db.is_verified(&FunctionId::simple("f3")));
    }

    #[test]
    fn test_proof_database_overwrite() {
        let mut db = ProofDatabase::new();
        let id = FunctionId::simple("foo");

        db.store_function_proof(&id, FunctionProofResult {
            status: VerificationStatus::Failed("old".to_string()),
            ..FunctionProofResult::default()
        });
        assert!(!db.is_verified(&id));

        db.store_function_proof(&id, FunctionProofResult {
            status: VerificationStatus::Verified,
            ..FunctionProofResult::default()
        });
        assert!(db.is_verified(&id));
        assert_eq!(db.len(), 1);
    }

    #[test]
    fn test_cache_path_for() {
        let path = PathBuf::from("src/main.bmb");
        let cache = ProofDatabase::cache_path_for(&path);
        assert_eq!(cache, PathBuf::from("src/main.bmb.proofcache"));
    }

    #[test]
    fn test_verification_status_eq() {
        assert_eq!(VerificationStatus::Verified, VerificationStatus::Verified);
        assert_eq!(VerificationStatus::Skipped, VerificationStatus::Skipped);
        assert_eq!(VerificationStatus::Timeout, VerificationStatus::Timeout);
        assert_eq!(VerificationStatus::Unknown, VerificationStatus::Unknown);
        assert_ne!(VerificationStatus::Verified, VerificationStatus::Skipped);
    }

    #[test]
    fn test_function_proof_result_default() {
        let result = FunctionProofResult::default();
        assert_eq!(result.status, VerificationStatus::Unknown);
        assert!(result.proven_facts.is_empty());
        assert_eq!(result.smt_queries, 0);
        assert_eq!(result.verified_at, 0);
    }
}
