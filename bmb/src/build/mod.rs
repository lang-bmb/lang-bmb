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

use crate::cfg::{CfgEvaluator, Target};
use crate::codegen::CodeGenError;
#[cfg(feature = "llvm")]
use crate::codegen::CodeGen;
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
    /// Compilation target (v0.12.3)
    pub target: Target,
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
            target: Target::Native,
        }
    }

    /// Set compilation target (v0.12.3)
    pub fn target(mut self, target: Target) -> Self {
        self.target = target;
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
        use crate::codegen::TextCodeGen;
        use std::process::Command;

        // Use text-based LLVM IR generation + clang
        let codegen = TextCodeGen::new();
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
        let clang = find_clang().map_err(|e| BuildError::Linker(e))?;

        // Find runtime
        let runtime_path = find_runtime_c().map_err(|e| BuildError::Linker(e))?;

        if config.verbose {
            println!("  Using clang: {}", clang);
            println!("  Using runtime: {}", runtime_path.display());
        }

        // Compile IR to object file
        let obj_path = config.output.with_extension(if cfg!(windows) { "obj" } else { "o" });
        let mut cmd = Command::new(&clang);
        cmd.args(["-c", ir_path.to_str().unwrap(), "-o", obj_path.to_str().unwrap()]);

        let output_result = cmd.output()?;
        if !output_result.status.success() {
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            return Err(BuildError::Linker(format!("clang compile failed: {}", stderr)));
        }

        if config.verbose {
            println!("  Compiled to object file: {}", obj_path.display());
        }

        // Compile runtime
        let runtime_obj = config.output.with_file_name("runtime").with_extension(if cfg!(windows) { "obj" } else { "o" });
        let mut cmd = Command::new(&clang);
        cmd.args(["-c", runtime_path.to_str().unwrap(), "-o", runtime_obj.to_str().unwrap()]);

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
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            // target/release/ -> runtime/
            if let Some(grandparent) = parent.parent() {
                if let Some(project_root) = grandparent.parent() {
                    let runtime = project_root.join("runtime").join("runtime.c");
                    if runtime.exists() {
                        return Ok(runtime);
                    }
                }
            }
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
    if msvc_base.exists() {
        if let Ok(entries) = std::fs::read_dir(msvc_base) {
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
    }

    // Also try VS 2022 BuildTools location
    let msvc_bt_base = Path::new(r"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC");
    if msvc_bt_base.exists() {
        if let Ok(entries) = std::fs::read_dir(msvc_bt_base) {
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
    if sdk_base.exists() {
        if let Ok(entries) = std::fs::read_dir(sdk_base) {
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
    }

    // Find MSVC lib path
    let msvc_base = Path::new(r"C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC");
    if msvc_base.exists() {
        if let Ok(entries) = std::fs::read_dir(msvc_base) {
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
    }

    // Also try VS 2022 BuildTools location
    let msvc_bt_base = Path::new(r"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC");
    if msvc_bt_base.exists() {
        if let Ok(entries) = std::fs::read_dir(msvc_bt_base) {
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
    }

    if paths.is_empty() {
        None
    } else {
        Some(paths)
    }
}
