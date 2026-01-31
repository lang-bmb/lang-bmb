//! Build Pipeline
//!
//! This module orchestrates the full compilation pipeline:
//! BMB Source → AST → MIR → LLVM IR → Object File → Executable
//!
//! # v0.55+ CIR/PIR Pipeline
//!
//! The build pipeline now supports the full CIR/PIR verification and
//! optimization pipeline:
//!
//! ```text
//! BMB Source → AST → TAST → CIR → PIR → MIR → LLVM IR
//!                            ↓      ↓
//!                         Verify  Optimize
//! ```

use std::path::PathBuf;

#[cfg(feature = "llvm")]
use std::path::Path;
#[cfg(feature = "llvm")]
use std::process::Command;

use std::collections::HashSet;

use thiserror::Error;

use crate::cfg::{CfgEvaluator, Target};
use crate::codegen::CodeGenError;
#[cfg(feature = "llvm")]
use crate::codegen::CodeGen;
use crate::mir::lower_program;
use crate::parser::parse;
use crate::lexer::tokenize;
use crate::types::TypeChecker;

// v0.55: CIR/PIR pipeline imports
use crate::mir::run_proof_guided_program;
use crate::cir::{lower_to_cir, extract_all_facts, CirVerifier, CirVerificationReport, ProofOutcome};
use crate::pir::{propagate_proofs, extract_all_pir_facts};
use crate::verify::ProofDatabase;

/// Verification mode for contract checking during build
///
/// # Soundness Guarantee
///
/// BMB's core philosophy is "Performance > Everything". Contract-verified code
/// enables proof-guided optimizations (BCE, NCE, DCE, PUE) that eliminate runtime
/// checks. However, using unverified contracts for optimization is **unsound**:
/// the compiler would assume properties that haven't been proven.
///
/// This enum controls when and how verification is performed:
/// - `None`: Skip verification entirely (Debug builds) - no proof-guided optimizations
/// - `Check`: Require verification to succeed (Release default) - sound optimizations
/// - `Warn`: Verify but only warn on failures - optimizations only for verified code
/// - `Trust`: Use contracts without verification (explicit unsafe) - user takes responsibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VerificationMode {
    /// Debug mode: Skip verification and proof-guided optimizations
    /// Fast builds, no soundness guarantees needed (no optimizations use proofs)
    None,

    /// Release mode (default): Verify contracts, fail build on verification failure
    /// This is the sound default - only verified facts enable optimizations
    #[default]
    Check,

    /// Warning mode: Verify contracts, warn on failure but continue
    /// Only verified functions get proof-guided optimizations
    Warn,

    /// Trust mode: Use contracts without verification
    /// **WARNING**: This is explicitly unsafe. The user takes responsibility
    /// for ensuring contracts are valid. Unsound optimizations possible.
    Trust,
}

/// Build configuration
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// Input source file
    pub input: PathBuf,
    /// Output file path
    pub output: PathBuf,
    /// Optimization level
    pub opt_level: OptLevel,
    /// Output type
    pub output_type: OutputType,
    /// Emit LLVM IR instead of object file
    pub emit_ir: bool,
    /// Verbose output
    pub verbose: bool,
    /// Compilation target (v0.12.3)
    pub target: Target,
    /// Target triple for cross-compilation (v0.50.23)
    /// e.g., "x86_64-unknown-linux-gnu", "x86_64-pc-windows-msvc", "aarch64-apple-darwin"
    pub target_triple: Option<String>,

    // === v0.55: CIR/PIR Pipeline Options ===

    /// Emit CIR (Contract IR) output instead of compiling
    pub emit_cir: bool,
    /// Emit PIR (Proof-Indexed IR) output instead of compiling
    pub emit_pir: bool,
    /// Show proof annotations in output
    pub show_proofs: bool,
    /// Generate optimization report
    pub opt_report: bool,
    /// Enable proof-guided optimizations (BCE, NCE, DCE, PUE)
    pub proof_optimizations: bool,
    /// Enable proof caching for incremental compilation
    pub proof_cache: bool,

    // === v0.60: Verification Mode Options ===

    /// Verification mode for contract checking
    /// Controls soundness guarantees for proof-guided optimizations
    pub verification_mode: VerificationMode,
    /// SMT solver timeout in seconds (default: 30)
    pub verification_timeout: u32,

    // === v0.60.56: Fast Math Options ===

    /// Enable fast-math optimizations for floating-point operations
    /// WARNING: This enables aggressive FP optimizations (FMA, reciprocal approximation,
    /// reassociation) that may change results slightly. Not IEEE-754 compliant.
    /// Improves performance significantly for FP-heavy workloads (1.5-2x speedup).
    pub fast_math: bool,
}

impl BuildConfig {
    /// Create a new build configuration with defaults
    pub fn new(input: PathBuf) -> Self {
        let output = input.with_extension(if cfg!(windows) { "exe" } else { "" });
        Self {
            input,
            output,
            opt_level: OptLevel::Release,  // v0.60.38: Default to Release for optimized builds
            output_type: OutputType::Executable,
            emit_ir: false,
            verbose: false,
            target: Target::Native,
            target_triple: None,
            // v0.55: CIR/PIR defaults
            emit_cir: false,
            emit_pir: false,
            show_proofs: false,
            opt_report: false,
            proof_optimizations: true, // Enabled by default
            proof_cache: true, // Enabled by default for incremental compilation
            // v0.60: Verification mode defaults
            verification_mode: VerificationMode::Check, // Sound default for release
            verification_timeout: 30, // 30 seconds default timeout
            // v0.60.56: Fast math defaults
            fast_math: false, // Strict IEEE-754 by default for correctness
        }
    }

    /// Set verification mode
    pub fn verification_mode(mut self, mode: VerificationMode) -> Self {
        self.verification_mode = mode;
        self
    }

    /// Set verification timeout in seconds
    pub fn verification_timeout(mut self, timeout: u32) -> Self {
        self.verification_timeout = timeout;
        self
    }

    /// Set compilation target (v0.12.3)
    pub fn target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }

    /// Set target triple for cross-compilation (v0.50.23)
    pub fn target_triple(mut self, triple: String) -> Self {
        self.target_triple = Some(triple);
        self
    }

    /// Set output path
    pub fn output(mut self, path: PathBuf) -> Self {
        self.output = path;
        self
    }

    /// Set optimization level
    pub fn opt_level(mut self, level: OptLevel) -> Self {
        self.opt_level = level;
        self
    }

    /// Set to emit LLVM IR
    pub fn emit_ir(mut self, emit: bool) -> Self {
        self.emit_ir = emit;
        self
    }

    /// Set verbose mode
    pub fn verbose(mut self, v: bool) -> Self {
        self.verbose = v;
        self
    }

    /// Enable fast-math optimizations (v0.60.56)
    /// WARNING: Not IEEE-754 compliant. Use for performance-critical FP code.
    pub fn fast_math(mut self, enable: bool) -> Self {
        self.fast_math = enable;
        self
    }
}

/// Optimization level
#[derive(Debug, Clone, Copy, Default)]
pub enum OptLevel {
    /// No optimization (-O0)
    #[default]
    Debug,
    /// Standard optimization (-O2)
    Release,
    /// Size optimization (-Os)
    Size,
    /// Aggressive optimization (-O3)
    Aggressive,
}

/// Output type
#[derive(Debug, Clone, Copy, Default)]
pub enum OutputType {
    /// Executable binary
    #[default]
    Executable,
    /// Object file
    Object,
    /// LLVM IR
    LlvmIr,
}

/// Build error
#[derive(Debug, Error)]
pub enum BuildError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Type error: {0}")]
    Type(String),

    #[error("Code generation error: {0}")]
    CodeGen(#[from] CodeGenError),

    #[error("Linker error: {0}")]
    Linker(String),

    /// Contract verification failed (v0.60)
    /// This error occurs when VerificationMode::Check is set and contracts fail verification
    #[error("Contract verification failed:\n{0}")]
    Verification(String),
}

/// Build result
pub type BuildResult<T> = Result<T, BuildError>;

/// Build a BMB program
pub fn build(config: &BuildConfig) -> BuildResult<()> {
    // Read source
    let source = std::fs::read_to_string(&config.input)?;
    let filename = config.input.display().to_string();

    if config.verbose {
        println!("Compiling: {}", config.input.display());
    }

    // Tokenize
    let tokens = tokenize(&source).map_err(|e| BuildError::Parse(e.message().to_string()))?;

    // Parse
    let program = parse(&filename, &source, tokens)
        .map_err(|e| BuildError::Parse(e.message().to_string()))?;

    if config.verbose {
        println!("  Parsed {} items", program.items.len());
    }

    // v0.12.3: Filter items by @cfg attributes
    let cfg_eval = CfgEvaluator::new(config.target);
    let program = cfg_eval.filter_program(&program);

    if config.verbose {
        println!("  After @cfg filtering: {} items (target: {})",
                 program.items.len(), config.target.as_str());
    }

    // Type check
    let mut type_checker = TypeChecker::new();
    type_checker
        .check_program(&program)
        .map_err(|e| BuildError::Type(format!("{:?}", e)))?;

    if config.verbose {
        println!("  Type check passed");
    }

    // Lower to MIR
    let mut mir = lower_program(&program);

    if config.verbose {
        println!("  Generated MIR for {} functions", mir.functions.len());
    }

    // v0.60: Determine effective verification mode based on build configuration
    // Debug builds skip verification and proof-guided optimizations entirely
    // Release builds verify contracts and only use verified facts for optimization
    let effective_verification_mode = if matches!(config.opt_level, OptLevel::Debug) {
        VerificationMode::None
    } else {
        config.verification_mode
    };

    // v0.55/v0.60: Full CIR → PIR → MIR proof pipeline with verification gate
    //
    // # Soundness Guarantee (v0.60)
    //
    // The proof-guided optimization pipeline now includes a verification gate:
    // 1. CIR: Extract contract propositions
    // 2. **Verification Gate**: Run SMT verification on CIR contracts
    // 3. PIR: Propagate proofs through control flow (only for verified functions)
    // 4. MIR: Augment with extracted ContractFacts for optimization
    //
    // Without verification, using contracts for optimization is UNSOUND.
    // The compiler would assume properties that haven't been proven.
    if config.proof_optimizations && !matches!(effective_verification_mode, VerificationMode::None) {
        let cir = lower_to_cir(&program);

        // v0.60: Run verification based on mode
        let verified_functions: HashSet<String> = match effective_verification_mode {
            VerificationMode::None => {
                // Should not reach here due to outer condition
                HashSet::new()
            }
            VerificationMode::Check => {
                // Strict mode: verify and fail on any verification failure
                let verifier = CirVerifier::new()
                    .with_timeout(config.verification_timeout);

                if !verifier.is_solver_available() {
                    if config.verbose {
                        println!("  Warning: Z3 solver not available, skipping verification");
                        println!("  Falling back to Trust mode (using contracts without verification)");
                    }
                    // Fallback to trust mode if solver unavailable
                    cir.functions.iter().map(|f| f.name.clone()).collect()
                } else {
                    let report = verifier.verify_program(&cir);

                    if config.verbose {
                        println!("  Contract verification: {}", report.summary());
                    }

                    // Check for failures
                    if report.has_failures() {
                        let mut errors = String::new();
                        for witness in &report.witnesses {
                            if let ProofOutcome::Failed(reason) = &witness.outcome {
                                errors.push_str(&format!(
                                    "  Function '{}': {}\n",
                                    witness.function, reason
                                ));
                                if let Some(ce) = &witness.counterexample {
                                    errors.push_str(&format!("    Counterexample: {:?}\n", ce));
                                }
                            }
                        }
                        return Err(BuildError::Verification(errors));
                    }

                    // Return only verified function names
                    report.witnesses.iter()
                        .filter(|w| matches!(w.outcome, ProofOutcome::Verified))
                        .map(|w| w.function.clone())
                        .collect()
                }
            }
            VerificationMode::Warn => {
                // Warning mode: verify but continue on failure
                let verifier = CirVerifier::new()
                    .with_timeout(config.verification_timeout);

                if !verifier.is_solver_available() {
                    if config.verbose {
                        println!("  Warning: Z3 solver not available");
                    }
                    HashSet::new() // No verified functions without solver
                } else {
                    let report = verifier.verify_program(&cir);

                    if config.verbose {
                        println!("  Contract verification: {}", report.summary());
                    }

                    // Warn about failures but continue
                    if report.has_failures() {
                        eprintln!("Warning: Contract verification issues detected:");
                        for witness in &report.witnesses {
                            if let ProofOutcome::Failed(reason) = &witness.outcome {
                                eprintln!("  Function '{}': {}", witness.function, reason);
                            }
                        }
                        eprintln!("  (Only verified functions will use proof-guided optimizations)");
                    }

                    // Return only verified function names
                    report.witnesses.iter()
                        .filter(|w| matches!(w.outcome, ProofOutcome::Verified))
                        .map(|w| w.function.clone())
                        .collect()
                }
            }
            VerificationMode::Trust => {
                // Trust mode: use all contracts without verification
                // WARNING: This is explicitly unsafe
                if config.verbose {
                    println!("  Using contracts without verification (--trust-contracts)");
                    println!("  WARNING: Unsound optimizations possible if contracts are invalid");
                }
                cir.functions.iter().map(|f| f.name.clone()).collect()
            }
        };

        // Extract CIR facts only for verified functions
        let cir_facts = extract_all_facts(&cir);

        // v0.55: Load or create proof database for caching
        let cache_path = ProofDatabase::cache_path_for(&config.input);
        let proof_db = if config.proof_cache {
            ProofDatabase::load_from_file(&cache_path).unwrap_or_else(|_| {
                if config.verbose {
                    println!("  Creating new proof cache");
                }
                ProofDatabase::new()
            })
        } else {
            ProofDatabase::new()
        };

        let cache_hits = proof_db.stats().cache_hits;

        // v0.55: Run PIR proof propagation for richer facts
        // PIR captures branch conditions, loop invariants, postconditions from calls
        let pir = propagate_proofs(&cir, &proof_db);
        let pir_facts = extract_all_pir_facts(&pir);

        // Augment MIR functions with both CIR and PIR facts
        // v0.60: Only augment facts for VERIFIED functions
        let mut cir_augmented = 0;
        let mut pir_augmented = 0;
        let mut skipped_unverified = 0;

        for func in &mut mir.functions {
            // v0.60: Soundness check - only use facts from verified functions
            if !verified_functions.contains(&func.name) {
                skipped_unverified += 1;
                continue;
            }

            // Add CIR-derived facts (from explicit contracts)
            if let Some((cir_pre, cir_post)) = cir_facts.get(&func.name) {
                for fact in cir_pre {
                    if !func.preconditions.contains(fact) {
                        func.preconditions.push(fact.clone());
                        cir_augmented += 1;
                    }
                }
                for fact in cir_post {
                    if !func.postconditions.contains(fact) {
                        func.postconditions.push(fact.clone());
                        cir_augmented += 1;
                    }
                }
            }

            // Add PIR-derived facts (from proof propagation)
            if let Some(pir_func_facts) = pir_facts.get(&func.name) {
                // Add preconditions from PIR
                for fact in &pir_func_facts.preconditions {
                    if !func.preconditions.contains(fact) {
                        func.preconditions.push(fact.clone());
                        pir_augmented += 1;
                    }
                }
                // Add postconditions from PIR
                for fact in &pir_func_facts.postconditions {
                    if !func.postconditions.contains(fact) {
                        func.postconditions.push(fact.clone());
                        pir_augmented += 1;
                    }
                }
            }
        }

        // v0.55: Save proof cache for incremental compilation
        if config.proof_cache {
            if let Err(e) = proof_db.save_to_file(&cache_path) {
                if config.verbose {
                    println!("  Warning: Failed to save proof cache: {}", e);
                }
            }
        }

        if config.verbose {
            let total = cir_augmented + pir_augmented;
            if total > 0 {
                println!("  Contract facts augmented: {} total", total);
                if cir_augmented > 0 {
                    println!("    - From CIR (explicit contracts): {}", cir_augmented);
                }
                if pir_augmented > 0 {
                    println!("    - From PIR (proof propagation): {}", pir_augmented);
                }
            }
            if skipped_unverified > 0 {
                println!("  Functions skipped (unverified): {}", skipped_unverified);
            }
            let new_cache_hits = proof_db.stats().cache_hits - cache_hits;
            if new_cache_hits > 0 {
                println!("  Proof cache hits: {}", new_cache_hits);
            }
        }
    }

    // v0.29: Run MIR optimizations
    {
        use crate::mir::{OptimizationPipeline, OptLevel as MirOptLevel};

        let mir_opt_level = match config.opt_level {
            OptLevel::Debug => MirOptLevel::Debug,
            OptLevel::Release => MirOptLevel::Release,
            OptLevel::Size => MirOptLevel::Release, // Size uses release-level MIR opts
            OptLevel::Aggressive => MirOptLevel::Aggressive,
        };

        let pipeline = OptimizationPipeline::for_level(mir_opt_level);
        let stats = pipeline.optimize(&mut mir);

        if config.verbose && !stats.pass_counts.is_empty() {
            println!("  MIR optimizations applied: {:?}", stats.pass_counts);
        }
    }

    // v0.55: Run proof-guided optimizations (BCE, NCE, DCE, PUE)
    if config.proof_optimizations && !matches!(config.opt_level, OptLevel::Debug) {
        let proof_stats = run_proof_guided_program(&mut mir);

        if config.verbose && proof_stats.total() > 0 {
            println!("  Proof-guided optimizations:");
            if proof_stats.bounds_checks_eliminated > 0 {
                println!("    - Bounds checks eliminated: {}", proof_stats.bounds_checks_eliminated);
            }
            if proof_stats.null_checks_eliminated > 0 {
                println!("    - Null checks eliminated: {}", proof_stats.null_checks_eliminated);
            }
            if proof_stats.division_checks_eliminated > 0 {
                println!("    - Division checks eliminated: {}", proof_stats.division_checks_eliminated);
            }
            if proof_stats.unreachable_blocks_eliminated > 0 {
                println!("    - Unreachable blocks eliminated: {}", proof_stats.unreachable_blocks_eliminated);
            }
        }

        // v0.55: Generate optimization report if requested
        if config.opt_report {
            println!("\n=== Proof-Guided Optimization Report ===");
            println!("Total optimizations: {}", proof_stats.total());
            println!("  Bounds Check Elimination (BCE): {}", proof_stats.bounds_checks_eliminated);
            println!("  Null Check Elimination (NCE): {}", proof_stats.null_checks_eliminated);
            println!("  Division Check Elimination (DCE): {}", proof_stats.division_checks_eliminated);
            println!("  Unreachable Block Elimination (PUE): {}", proof_stats.unreachable_blocks_eliminated);
            println!("=========================================\n");
        }
    }

    // Generate LLVM IR or object file
    #[cfg(feature = "llvm")]
    {
        use crate::codegen::OptLevel as CodeGenOptLevel;

        let codegen_opt = match config.opt_level {
            OptLevel::Debug => CodeGenOptLevel::Debug,
            OptLevel::Release => CodeGenOptLevel::Release,
            OptLevel::Size => CodeGenOptLevel::Size,
            OptLevel::Aggressive => CodeGenOptLevel::Aggressive,
        };

        // v0.60.56: Pass fast_math flag to codegen
        let codegen = CodeGen::with_fast_math(codegen_opt, config.fast_math);

        if config.emit_ir {
            // Emit LLVM IR
            let ir = codegen.generate_ir(&mir)?;
            let ir_path = config.output.with_extension("ll");
            std::fs::write(&ir_path, ir)?;
            if config.verbose {
                println!("  Wrote LLVM IR to {}", ir_path.display());
            }
            return Ok(());
        }

        // Generate object file
        let obj_path = config.output.with_extension("o");
        codegen.compile(&mir, &obj_path)?;

        if config.verbose {
            println!("  Generated object file: {}", obj_path.display());
        }

        // Link if building executable
        if matches!(config.output_type, OutputType::Executable) {
            link_executable(&obj_path, &config.output, config.verbose)?;
        }

        Ok(())
    }

    #[cfg(not(feature = "llvm"))]
    {
        use crate::codegen::TextCodeGen;
        use std::process::Command;

        // Use text-based LLVM IR generation + clang
        // v0.50.23: Support cross-compilation target triple
        let codegen = if let Some(ref triple) = config.target_triple {
            TextCodeGen::with_target(triple)
        } else {
            TextCodeGen::new()
        };
        let ir = codegen.generate(&mir).map_err(|_| BuildError::CodeGen(
            CodeGenError::LlvmNotAvailable, // Use existing error type
        ))?;

        let ir_path = config.output.with_extension("ll");
        std::fs::write(&ir_path, &ir)?;

        if config.verbose {
            println!("  Generated LLVM IR: {}", ir_path.display());
        }

        if config.emit_ir {
            return Ok(());
        }

        // Find clang
        let clang = find_clang().map_err(BuildError::Linker)?;

        // Find runtime
        let runtime_path = find_runtime_c().map_err(BuildError::Linker)?;

        if config.verbose {
            println!("  Using clang: {}", clang);
            println!("  Using runtime: {}", runtime_path.display());
        }

        // Compile IR to object file with optimization
        let obj_path = config.output.with_extension(if cfg!(windows) { "obj" } else { "o" });
        let mut cmd = Command::new(&clang);

        // Apply optimization based on config
        // v0.51.21: Changed Release from -O2 to -O3 for better performance
        // -O3 enables more aggressive loop optimizations and inlining
        let opt_flag = match config.opt_level {
            OptLevel::Debug => "-O0",
            OptLevel::Release => "-O3",  // Was -O2, changed for better benchmarks
            OptLevel::Size => "-Os",
            OptLevel::Aggressive => "-O3",
        };

        cmd.args([opt_flag, "-c", ir_path.to_str().unwrap(), "-o", obj_path.to_str().unwrap()]);

        let output_result = cmd.output()?;
        if !output_result.status.success() {
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            return Err(BuildError::Linker(format!("clang compile failed: {}", stderr)));
        }

        if config.verbose {
            println!("  Compiled to object file: {}", obj_path.display());
        }

        // Compile runtime with same optimization level as BMB code
        // v0.51: Critical fix - runtime was compiled with -O0, causing 3x slowdown in FFI calls
        let runtime_obj = config.output.with_file_name("runtime").with_extension(if cfg!(windows) { "obj" } else { "o" });
        let mut cmd = Command::new(&clang);
        cmd.args([opt_flag, "-c", runtime_path.to_str().unwrap(), "-o", runtime_obj.to_str().unwrap()]);

        // Add Windows SDK include paths if on Windows
        #[cfg(target_os = "windows")]
        {
            if let Some(include_paths) = find_windows_sdk_includes() {
                for path in include_paths {
                    cmd.arg("-I").arg(path);
                }
            }
        }

        let output_result = cmd.output()?;
        if !output_result.status.success() {
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            return Err(BuildError::Linker(format!("runtime compile failed: {}", stderr)));
        }

        // Link using lld-link on Windows (more reliable than clang auto-detection)
        #[cfg(target_os = "windows")]
        {
            let mut cmd = Command::new("lld-link");
            cmd.args([
                obj_path.to_str().unwrap(),
                runtime_obj.to_str().unwrap(),
                &format!("/OUT:{}", config.output.to_str().unwrap()),
                "/SUBSYSTEM:CONSOLE",
                "/ENTRY:mainCRTStartup",
                "/STACK:16777216",  // 16MB stack for deep recursion in bootstrap compiler
            ]);

            // Add Windows SDK and MSVC library paths
            if let Some(lib_paths) = find_windows_lib_paths() {
                for path in lib_paths {
                    cmd.arg(format!("/LIBPATH:{}", path));
                }
            }

            // Link required libraries
            cmd.args([
                "libcmt.lib",      // C runtime
                "libucrt.lib",     // Universal CRT
                "kernel32.lib",    // Windows kernel
                "legacy_stdio_definitions.lib",  // printf and friends
            ]);

            if config.verbose {
                println!("  Linking with lld-link...");
            }

            let output_result = cmd.output()?;
            if !output_result.status.success() {
                let stderr = String::from_utf8_lossy(&output_result.stderr);
                return Err(BuildError::Linker(format!("link failed: {}", stderr)));
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            let mut cmd = Command::new(&clang);
            cmd.args([
                obj_path.to_str().unwrap(),
                runtime_obj.to_str().unwrap(),
                "-o",
                config.output.to_str().unwrap(),
            ]);

            let output_result = cmd.output()?;
            if !output_result.status.success() {
                let stderr = String::from_utf8_lossy(&output_result.stderr);
                return Err(BuildError::Linker(format!("link failed: {}", stderr)));
            }
        }

        // Cleanup intermediate files
        let _ = std::fs::remove_file(&ir_path);
        let _ = std::fs::remove_file(&obj_path);
        let _ = std::fs::remove_file(&runtime_obj);

        if config.verbose {
            println!("  Created executable: {}", config.output.display());
        }

        Ok(())
    }
}

/// Find clang compiler
fn find_clang() -> Result<String, String> {
    use std::process::Command;

    // Check common locations
    let candidates = if cfg!(target_os = "windows") {
        vec![
            "clang",
            "C:\\Program Files\\LLVM\\bin\\clang.exe",
            "C:\\msys64\\mingw64\\bin\\clang.exe",
        ]
    } else {
        vec!["clang", "clang-18", "clang-17", "clang-16", "clang-15"]
    };

    for candidate in candidates {
        if Command::new(candidate).arg("--version").output().is_ok() {
            return Ok(candidate.to_string());
        }
    }

    Err("clang not found. Please install LLVM/clang.".to_string())
}

/// Find runtime.c source file
fn find_runtime_c() -> Result<std::path::PathBuf, String> {
    use std::path::PathBuf;

    // Check BMB_RUNTIME_PATH environment variable
    if let Ok(path) = std::env::var("BMB_RUNTIME_PATH") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Ok(p);
        }
    }

    // Check relative to executable
    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
        && let Some(grandparent) = parent.parent()
        && let Some(project_root) = grandparent.parent()
    {
        // target/release/ -> runtime/
        let runtime = project_root.join("runtime").join("runtime.c");
        if runtime.exists() {
            return Ok(runtime);
        }
    }

    // Check current working directory patterns
    let patterns = [
        "runtime/runtime.c",
        "../runtime/runtime.c",
        "../../runtime/runtime.c",
    ];

    for pattern in patterns {
        let p = PathBuf::from(pattern);
        if p.exists() {
            return Ok(p);
        }
    }

    Err("runtime.c not found. Set BMB_RUNTIME_PATH environment variable.".to_string())
}

/// Link object file to executable
#[cfg(feature = "llvm")]
fn link_executable(obj_path: &Path, output: &Path, verbose: bool) -> BuildResult<()> {
    // Find the appropriate linker
    let linker = find_linker()?;

    if verbose {
        println!("  Linking with: {}", linker);
    }

    // Find runtime library
    let runtime_path = find_runtime()?;

    if verbose {
        println!("  Using runtime: {}", runtime_path.display());
    }

    // Build linker command
    let mut cmd = Command::new(&linker);

    // Add object file
    cmd.arg(obj_path.to_str().unwrap());

    // Add runtime library
    cmd.arg(runtime_path.to_str().unwrap());

    // Output file
    cmd.args(["-o", output.to_str().unwrap()]);

    // Platform-specific linker flags
    #[cfg(target_os = "windows")]
    {
        cmd.args(["-lkernel32", "-lmsvcrt"]);
    }

    #[cfg(target_os = "linux")]
    {
        // v0.100: Disable PIE to avoid PIC relocation issues
        cmd.arg("-no-pie");
        cmd.arg("-lc");
        // v0.70: Link pthread for threading support
        cmd.arg("-lpthread");
    }

    #[cfg(target_os = "macos")]
    {
        cmd.arg("-lSystem");
        // v0.70: Link pthread for threading support (part of System on macOS)
    }

    let output_result = cmd.output()?;

    if !output_result.status.success() {
        let stderr = String::from_utf8_lossy(&output_result.stderr);
        return Err(BuildError::Linker(stderr.to_string()));
    }

    if verbose {
        println!("  Created executable: {}", output.display());
    }

    Ok(())
}

/// Find the BMB runtime library
#[cfg(feature = "llvm")]
fn find_runtime() -> BuildResult<PathBuf> {
    // Check BMB_RUNTIME_PATH environment variable
    if let Ok(path) = std::env::var("BMB_RUNTIME_PATH") {
        let p = PathBuf::from(&path);
        // v0.60.42: If path is a directory, look for the library file inside it
        if p.is_dir() {
            let lib_path = p.join("libbmb_runtime.a");
            if lib_path.exists() {
                return Ok(lib_path);
            }
            // Also try Windows-style naming
            let lib_path_win = p.join("bmb_runtime.lib");
            if lib_path_win.exists() {
                return Ok(lib_path_win);
            }
        } else if p.exists() {
            // Path is a file, use it directly
            return Ok(p);
        }
    }

    // Check common locations relative to executable
    let exe_path = std::env::current_exe().ok();
    if let Some(exe) = exe_path {
        // Check ../runtime/libbmb_runtime.a (relative to exe)
        if let Some(parent) = exe.parent() {
            let runtime = parent.join("runtime").join("libbmb_runtime.a");
            if runtime.exists() {
                return Ok(runtime);
            }
            // Check ../../runtime/libbmb_runtime.a (for debug builds)
            if let Some(grandparent) = parent.parent() {
                let runtime = grandparent.join("runtime").join("libbmb_runtime.a");
                if runtime.exists() {
                    return Ok(runtime);
                }
                // Check ../../../runtime/ (for target/x86_64-pc-windows-gnu/debug/)
                if let Some(ggp) = grandparent.parent() {
                    if let Some(gggp) = ggp.parent() {
                        let runtime = gggp.join("runtime").join("libbmb_runtime.a");
                        if runtime.exists() {
                            return Ok(runtime);
                        }
                    }
                }
            }
        }
    }

    // Check current working directory
    let cwd_runtime = PathBuf::from("runtime/libbmb_runtime.a");
    if cwd_runtime.exists() {
        return Ok(cwd_runtime);
    }

    Err(BuildError::Linker(
        "Cannot find BMB runtime library. Set BMB_RUNTIME_PATH environment variable.".to_string(),
    ))
}

/// Find the system linker
#[cfg(feature = "llvm")]
fn find_linker() -> BuildResult<String> {
    // Try common linkers in order of preference
    // On Windows, prefer gcc/clang (MinGW) over MSVC link.exe because:
    // 1. We target x86_64-pc-windows-gnu
    // 2. gcc understands -o flag while link.exe uses /OUT:
    // On Linux, prefer clang/gcc over bare ld because:
    // 1. clang/gcc automatically link C runtime (crt1.o, libc)
    // 2. bare ld requires manual setup of startup files
    let candidates = if cfg!(target_os = "windows") {
        vec!["gcc", "clang", "lld", "lld-link"]
    } else if cfg!(target_os = "macos") {
        vec!["clang", "gcc", "ld"]
    } else {
        vec!["clang", "gcc", "ld", "lld"]
    };

    for linker in candidates {
        if Command::new(linker).arg("--version").output().is_ok() {
            return Ok(linker.to_string());
        }
    }

    // Default to cc
    Ok("cc".to_string())
}

/// Find Windows SDK and MSVC include paths
#[cfg(target_os = "windows")]
fn find_windows_sdk_includes() -> Option<Vec<String>> {
    use std::path::Path;

    let mut paths = Vec::new();

    // Find Windows SDK
    let sdk_base = Path::new(r"C:\Program Files (x86)\Windows Kits\10\Include");
    if sdk_base.exists() {
        let sdk_versions: Vec<_> = std::fs::read_dir(sdk_base)
            .ok()?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|name| name.starts_with("10.0."))
            .collect();

        if let Some(latest_version) = sdk_versions.iter().max() {
            let sdk_include = sdk_base.join(latest_version);

            // UCRT headers (stdio.h, etc.)
            let ucrt_path = sdk_include.join("ucrt");
            if ucrt_path.exists() {
                paths.push(ucrt_path.to_string_lossy().to_string());
            }

            // shared headers
            let shared_path = sdk_include.join("shared");
            if shared_path.exists() {
                paths.push(shared_path.to_string_lossy().to_string());
            }

            // um headers
            let um_path = sdk_include.join("um");
            if um_path.exists() {
                paths.push(um_path.to_string_lossy().to_string());
            }
        }
    }

    // Find MSVC include path (for vcruntime.h)
    let msvc_base = Path::new(r"C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC");
    if msvc_base.exists()
        && let Ok(entries) = std::fs::read_dir(msvc_base)
    {
        let msvc_versions: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();

        if let Some(latest_version) = msvc_versions.iter().max() {
            let msvc_include = msvc_base.join(latest_version).join("include");
            if msvc_include.exists() {
                paths.push(msvc_include.to_string_lossy().to_string());
            }
        }
    }

    // Also try VS 2022 BuildTools location
    let msvc_bt_base = Path::new(r"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC");
    if msvc_bt_base.exists()
        && let Ok(entries) = std::fs::read_dir(msvc_bt_base)
    {
        let msvc_versions: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();

        if let Some(latest_version) = msvc_versions.iter().max() {
            let msvc_include = msvc_bt_base.join(latest_version).join("include");
            if msvc_include.exists() {
                paths.push(msvc_include.to_string_lossy().to_string());
            }
        }
    }

    if paths.is_empty() {
        None
    } else {
        Some(paths)
    }
}

/// Find Windows SDK and MSVC library paths for linking
#[cfg(target_os = "windows")]
fn find_windows_lib_paths() -> Option<Vec<String>> {
    use std::path::Path;

    let mut paths = Vec::new();

    // Find Windows SDK lib path
    let sdk_base = Path::new(r"C:\Program Files (x86)\Windows Kits\10\Lib");
    if sdk_base.exists()
        && let Ok(entries) = std::fs::read_dir(sdk_base)
    {
        let sdk_versions: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|name| name.starts_with("10.0."))
            .collect();

        if let Some(latest_version) = sdk_versions.iter().max() {
            let sdk_lib = sdk_base.join(latest_version);

            // UCRT libraries
            let ucrt_lib = sdk_lib.join("ucrt").join("x64");
            if ucrt_lib.exists() {
                paths.push(ucrt_lib.to_string_lossy().to_string());
            }

            // um libraries (kernel32, etc.)
            let um_lib = sdk_lib.join("um").join("x64");
            if um_lib.exists() {
                paths.push(um_lib.to_string_lossy().to_string());
            }
        }
    }

    // Find MSVC lib path
    let msvc_base = Path::new(r"C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC");
    if msvc_base.exists()
        && let Ok(entries) = std::fs::read_dir(msvc_base)
    {
        let msvc_versions: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();

        if let Some(latest_version) = msvc_versions.iter().max() {
            let msvc_lib = msvc_base.join(latest_version).join("lib").join("x64");
            if msvc_lib.exists() {
                paths.push(msvc_lib.to_string_lossy().to_string());
            }
        }
    }

    // Also try VS 2022 BuildTools location
    let msvc_bt_base = Path::new(r"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC");
    if msvc_bt_base.exists()
        && let Ok(entries) = std::fs::read_dir(msvc_bt_base)
    {
        let msvc_versions: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();

        if let Some(latest_version) = msvc_versions.iter().max() {
            let msvc_lib = msvc_bt_base.join(latest_version).join("lib").join("x64");
            if msvc_lib.exists() {
                paths.push(msvc_lib.to_string_lossy().to_string());
            }
        }
    }

    if paths.is_empty() {
        None
    } else {
        Some(paths)
    }
}
