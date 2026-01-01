//! Build Pipeline
//!
//! This module orchestrates the full compilation pipeline:
//! BMB Source → AST → MIR → LLVM IR → Object File → Executable

use std::path::PathBuf;

#[cfg(feature = "llvm")]
use std::path::Path;
#[cfg(feature = "llvm")]
use std::process::Command;

use thiserror::Error;

use crate::codegen::{CodeGen, CodeGenError};
use crate::mir::lower_program;
use crate::parser::parse;
use crate::lexer::tokenize;
use crate::types::TypeChecker;

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
}

impl BuildConfig {
    /// Create a new build configuration with defaults
    pub fn new(input: PathBuf) -> Self {
        let output = input.with_extension(if cfg!(windows) { "exe" } else { "" });
        Self {
            input,
            output,
            opt_level: OptLevel::Debug,
            output_type: OutputType::Executable,
            emit_ir: false,
            verbose: false,
        }
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
        println!("  Parsed {} functions", program.items.len());
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
    let mir = lower_program(&program);

    if config.verbose {
        println!("  Generated MIR for {} functions", mir.functions.len());
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

        let codegen = CodeGen::with_opt_level(codegen_opt);

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
        let codegen = CodeGen::new();
        let obj_path = config.output.with_extension("o");
        codegen.compile(&mir, &obj_path)?;
        Ok(())
    }
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
        cmd.arg("-lc");
    }

    #[cfg(target_os = "macos")]
    {
        cmd.arg("-lSystem");
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
        let p = PathBuf::from(path);
        if p.exists() {
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
    let candidates = if cfg!(target_os = "windows") {
        vec!["gcc", "clang", "lld", "lld-link"]
    } else if cfg!(target_os = "macos") {
        vec!["ld", "clang", "gcc"]
    } else {
        vec!["ld", "lld", "clang", "gcc"]
    };

    for linker in candidates {
        if Command::new(linker).arg("--version").output().is_ok() {
            return Ok(linker.to_string());
        }
    }

    // Default to cc
    Ok("cc".to_string())
}
