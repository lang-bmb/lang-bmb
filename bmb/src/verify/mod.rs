//! Contract verification module
//!
//! Verifies function contracts (pre/post conditions) using SMT solving.
//!
//! # Phase 1 Infrastructure
//!
//! - `proof_db`: Proof database for caching verification results
//! - `summary`: Function summary extraction for inter-module verification
//! - `incremental`: Incremental verification to re-verify only changed functions

mod contract;
pub mod proof_db;
pub mod summary;
pub mod incremental;

pub use contract::{ContractVerifier, VerificationReport, FunctionReport};
pub use proof_db::{
    ProofDatabase, FunctionId, FunctionProofResult, VerificationStatus,
    ProofFact, ProofScope, ProofEvidence, ProofDbStats,
};
pub use summary::{
    FunctionSummary, TerminationStatus, SummaryChange,
    extract_summaries, extract_function_summary, compare_summaries,
};
pub use incremental::{IncrementalVerifier, IncrementalVerificationResult};
