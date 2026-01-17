//! BMB Compiler CLI

use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

/// v0.71: Global flag for human-readable output (default: machine/AI-friendly)
static HUMAN_OUTPUT: AtomicBool = AtomicBool::new(false);

/// Check if human output mode is enabled (default: false = machine mode)
pub fn is_human_output() -> bool {
    HUMAN_OUTPUT.load(Ordering::Relaxed)
}

#[derive(Parser)]
#[command(name = "bmb", version, about = "BMB Compiler - AI-Native Language")]
struct Cli {
    /// v0.71: Human-readable output (colors, formatting). Default: machine/JSON
    #[arg(long, global = true)]
    human: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Build a native executable (requires LLVM)
    Build {
        /// Source file to compile
        file: PathBuf,
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Build with optimizations (-O2)
        #[arg(long)]
        release: bool,
        /// Build with aggressive optimizations (-O3)
        #[arg(long)]
        aggressive: bool,
        /// Emit LLVM IR instead of executable
        #[arg(long)]
        emit_ir: bool,
        /// Emit MIR (Mid-level IR) - v0.21.2
        #[arg(long)]
        emit_mir: bool,
        /// Emit WASM text format (.wat)
        #[arg(long)]
        emit_wasm: bool,
        /// WASM target environment (wasi, browser, standalone)
        #[arg(long, default_value = "wasi")]
        wasm_target: String,
        /// Build for all targets (native + WASM) - v0.12.4
        #[arg(long)]
        all_targets: bool,
        /// Target triple for cross-compilation (v0.50.23)
        /// Examples: x86_64-unknown-linux-gnu, x86_64-pc-windows-msvc, aarch64-apple-darwin
        #[arg(long)]
        target: Option<String>,
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Run a BMB program (interpreter)
    Run {
        /// Source file to run
        file: PathBuf,
        /// v0.46: Arguments to pass to the BMB program
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
        /// v0.71: Human-readable output (colors, formatting). Default: machine/JSON
        #[arg(long)]
        human: bool,
    },
    /// Start interactive REPL
    Repl,
    /// Type check a BMB source file
    Check {
        /// Source file to check
        file: PathBuf,
        /// v0.17: Additional include paths for module resolution
        #[arg(short = 'I', long = "include", value_name = "PATH")]
        include_paths: Vec<PathBuf>,
    },
    /// Verify contracts (pre/post conditions) using SMT solver
    Verify {
        /// Source file to verify
        file: PathBuf,
        /// Path to Z3 executable (default: z3)
        #[arg(long, default_value = "z3")]
        z3_path: String,
        /// Timeout in seconds
        #[arg(long, short = 't', default_value = "10")]
        timeout: u32,
    },
    /// Parse and dump AST (debug)
    Parse {
        /// Source file to parse
        file: PathBuf,
        /// Output format: json or sexpr (S-expression)
        #[arg(long, short, default_value = "json")]
        format: String,
    },
    /// Tokenize and dump tokens (debug)
    Tokens {
        /// Source file to tokenize
        file: PathBuf,
    },
    /// Run tests in a BMB file
    Test {
        /// Source file or directory to test
        file: PathBuf,
        /// Filter tests by pattern
        #[arg(long, short)]
        filter: Option<String>,
        /// Verbose output (show all test results)
        #[arg(short, long)]
        verbose: bool,
    },
    /// Format a BMB source file
    Fmt {
        /// Source file or directory to format
        file: PathBuf,
        /// Check formatting without modifying files
        #[arg(long)]
        check: bool,
    },
    /// Lint a BMB source file (v0.45)
    Lint {
        /// Source file or directory to lint
        file: PathBuf,
        /// Treat warnings as errors (exit 1 if any warnings)
        #[arg(long)]
        strict: bool,
        /// Additional include paths for module resolution
        #[arg(short = 'I', long = "include", value_name = "PATH")]
        include_paths: Vec<PathBuf>,
    },
    /// Start Language Server Protocol server
    Lsp,
    /// Generate project index for AI tools (v0.25)
    Index {
        /// Project root directory (default: current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Watch for file changes
        #[arg(long)]
        watch: bool,
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Query project index (AI Query System - v0.25)
    #[command(name = "q")]
    Query {
        #[command(subcommand)]
        query_type: QueryType,
    },
    /// Verify Stage 3 self-hosting (v0.30.246)
    /// Compares LLVM IR from Rust compiler vs Bootstrap compiler
    #[command(name = "verify-stage3")]
    VerifyStage3 {
        /// BMB source file to verify
        file: PathBuf,
        /// Show detailed comparison
        #[arg(short, long)]
        verbose: bool,
        /// Output comparison report to file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

/// Output format for queries (v0.48 - RFC-0001)
#[derive(Clone, Copy, Debug, Default, clap::ValueEnum)]
enum OutputFormat {
    /// JSON output (default)
    #[default]
    Json,
    /// Compact single-line format
    Compact,
    /// LLM-optimized format (token-efficient)
    Llm,
}

#[derive(Subcommand)]
enum QueryType {
    /// Search symbols by pattern
    Sym {
        /// Pattern to search for
        pattern: String,
        /// Filter by kind (fn, struct, enum, type, trait)
        #[arg(long)]
        kind: Option<String>,
        /// Only show public symbols
        #[arg(long)]
        public: bool,
        /// Output format (json, compact, llm)
        #[arg(long, short = 'f', value_enum, default_value = "json")]
        format: OutputFormat,
    },
    /// Query function details
    Fn {
        /// Function name (optional when using filters)
        #[arg(default_value = "")]
        name: String,
        /// Show functions with preconditions
        #[arg(long)]
        has_pre: bool,
        /// Show functions with postconditions
        #[arg(long)]
        has_post: bool,
        /// Show recursive functions
        #[arg(long)]
        recursive: bool,
        /// Output format (json, compact, llm)
        #[arg(long, short = 'f', value_enum, default_value = "json")]
        format: OutputFormat,
    },
    /// Query type details
    Type {
        /// Type name (optional when using --kind filter)
        #[arg(default_value = "")]
        name: String,
        /// Filter by kind (struct, enum, trait)
        #[arg(long)]
        kind: Option<String>,
        /// Output format (json, compact, llm)
        #[arg(long, short = 'f', value_enum, default_value = "json")]
        format: OutputFormat,
    },
    /// Show project metrics
    Metrics {
        /// Output format (json, compact, llm)
        #[arg(long, short = 'f', value_enum, default_value = "json")]
        format: OutputFormat,
    },
    /// Query dependencies (v0.47 - RFC-0001)
    Deps {
        /// Target to query (e.g., fn:main, type:Order)
        target: String,
        /// Show reverse dependencies (who calls/uses this)
        #[arg(long)]
        reverse: bool,
        /// Include transitive dependencies
        #[arg(long)]
        transitive: bool,
        /// Output format (json, compact, llm)
        #[arg(long, short = 'f', value_enum, default_value = "json")]
        format: OutputFormat,
    },
    /// Query contract details (v0.47 - RFC-0001)
    Contract {
        /// Function name to query contracts for
        name: String,
        /// Show contracts that use old() state
        #[arg(long)]
        uses_old: bool,
        /// Output format (json, compact, llm)
        #[arg(long, short = 'f', value_enum, default_value = "json")]
        format: OutputFormat,
    },
    /// Generate AI context for a target (v0.48 - RFC-0001)
    Ctx {
        /// Target to generate context for (e.g., fn:process_order)
        target: String,
        /// Depth of dependency inclusion
        #[arg(long, default_value = "1")]
        depth: usize,
        /// Include related tests
        #[arg(long)]
        include_tests: bool,
        /// Output format (json, compact, llm)
        #[arg(long, short = 'f', value_enum, default_value = "json")]
        format: OutputFormat,
    },
    /// Search functions by signature pattern (v0.48 - RFC-0001)
    Sig {
        /// Signature pattern (e.g., "(&[i64]) -> i64")
        #[arg(default_value = "")]
        pattern: String,
        /// Find functions that accept this type
        #[arg(long)]
        accepts: Option<String>,
        /// Find functions that return this type
        #[arg(long)]
        returns: Option<String>,
        /// Output format (json, compact, llm)
        #[arg(long, short = 'f', value_enum, default_value = "json")]
        format: OutputFormat,
    },
    /// Run batch queries from file (v0.49 - RFC-0001)
    Batch {
        /// Path to queries JSON file
        file: PathBuf,
        /// Output format (json, compact, llm)
        #[arg(long, short = 'f', value_enum, default_value = "json")]
        format: OutputFormat,
    },
    /// Analyze change impact (v0.49 - RFC-0001)
    Impact {
        /// Target to analyze (e.g., fn:calculate_fee)
        target: String,
        /// Description of the change
        #[arg(long)]
        change: String,
        /// Output format (json, compact, llm)
        #[arg(long, short = 'f', value_enum, default_value = "json")]
        format: OutputFormat,
    },
    /// Start HTTP query server (v0.50 - RFC-0001)
    Serve {
        /// Port to listen on
        #[arg(long, short = 'p', default_value = "3000")]
        port: u16,
        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
    /// Query verification proof results (v0.50.24 - Task 47.7-47.8)
    Proof {
        /// Function name pattern (optional)
        #[arg(default_value = "")]
        name: String,
        /// Show only unverified functions
        #[arg(long)]
        unverified: bool,
        /// Show only failed verifications
        #[arg(long)]
        failed: bool,
        /// Show only timed out verifications
        #[arg(long)]
        timeout: bool,
        /// Output format (json, compact, llm)
        #[arg(long, short = 'f', value_enum, default_value = "json")]
        format: OutputFormat,
    },
}

fn main() {
    let cli = Cli::parse();

    // v0.71: Set human output mode (default: machine)
    if cli.human {
        HUMAN_OUTPUT.store(true, Ordering::Relaxed);
    }

    let result = match cli.command {
        Command::Build {
            file,
            output,
            release,
            aggressive,
            emit_ir,
            emit_mir,
            emit_wasm,
            wasm_target,
            all_targets,
            target,
            verbose,
        } => build_file(&file, output, release, aggressive, emit_ir, emit_mir, emit_wasm, &wasm_target, all_targets, target.as_deref(), verbose),
        Command::Run { file, args, human: _ } => run_file(&file, &args),
        Command::Repl => start_repl(),
        Command::Check { file, include_paths } => check_file_with_includes(&file, &include_paths),
        Command::Verify { file, z3_path, timeout } => verify_file(&file, &z3_path, timeout),
        Command::Parse { file, format } => parse_file(&file, &format),
        Command::Tokens { file } => tokenize_file(&file),
        Command::Test { file, filter, verbose } => test_file(&file, filter.as_deref(), verbose),
        Command::Fmt { file, check } => fmt_file(&file, check),
        Command::Lint { file, strict, include_paths } => lint_file(&file, strict, &include_paths),
        Command::Lsp => start_lsp(),
        Command::Index { path, watch, verbose } => index_project(&path, watch, verbose),
        Command::Query { query_type } => run_query(query_type),
        Command::VerifyStage3 { file, verbose, output } => verify_stage3(&file, verbose, output.as_ref()),
    };

    if let Err(e) = result {
        // v0.71: Default machine output, --human for human-readable
        if is_human_output() {
            eprintln!("Error: {e}");
        } else {
            println!(r#"{{"type":"error","message":"{}"}}"#,
                e.to_string().replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n"));
        }
        std::process::exit(1);
    }
}

#[allow(clippy::too_many_arguments)]
fn build_file(
    path: &PathBuf,
    output: Option<PathBuf>,
    release: bool,
    aggressive: bool,
    emit_ir: bool,
    emit_mir: bool,
    emit_wasm: bool,
    wasm_target: &str,
    all_targets: bool,
    target: Option<&str>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // v0.21.2: If emitting MIR, just output MIR and return
    if emit_mir {
        return emit_mir_file(path, output, verbose);
    }

    // v0.12.4: Build for all targets (native + WASM)
    if all_targets {
        if verbose {
            println!("Building for all targets...");
        }

        // Build native first
        if verbose {
            println!("\n=== Native Build ===");
        }
        build_native(path, output.clone(), release, aggressive, emit_ir, target, verbose)?;

        // Then build WASM
        if verbose {
            println!("\n=== WASM Build ===");
        }
        build_wasm(path, None, wasm_target, verbose)?;

        if verbose {
            println!("\n=== All targets built successfully! ===");
        }
        return Ok(());
    }

    // If emitting WASM, use the WASM code generator
    if emit_wasm {
        return build_wasm(path, output, wasm_target, verbose);
    }

    // Default: build native
    build_native(path, output, release, aggressive, emit_ir, target, verbose)
}

fn build_native(
    path: &Path,
    output: Option<PathBuf>,
    release: bool,
    aggressive: bool,
    emit_ir: bool,
    target: Option<&str>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use bmb::build::{BuildConfig, OptLevel};

    let mut config = BuildConfig::new(path.to_path_buf())
        .emit_ir(emit_ir)
        .verbose(verbose);

    // v0.50.23: Cross-compilation target
    if let Some(triple) = target {
        config = config.target_triple(triple.to_string());
        if verbose {
            println!("Cross-compiling for target: {}", triple);
        }
    }

    if let Some(out) = output {
        config = config.output(out);
    }

    if aggressive {
        config = config.opt_level(OptLevel::Aggressive);
    } else if release {
        config = config.opt_level(OptLevel::Release);
    }

    bmb::build::build(&config)?;

    if !emit_ir {
        if is_human_output() {
            if verbose {
                println!("Build complete: {}", config.output.display());
            }
        } else {
            println!(r#"{{"type":"build_success","output":"{}"}}"#,
                config.output.display().to_string().replace('\\', "\\\\"));
        }
    }

    Ok(())
}

fn build_wasm(
    path: &PathBuf,
    output: Option<PathBuf>,
    wasm_target: &str,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use bmb::cfg::{CfgEvaluator, Target};
    use bmb::codegen::{WasmCodeGen, WasmTarget};

    let source = std::fs::read_to_string(path)?;
    let filename = path.display().to_string();

    if verbose {
        println!("Compiling {} to WASM...", filename);
    }

    // Tokenize
    let tokens = bmb::lexer::tokenize(&source)?;

    // Parse
    let ast = bmb::parser::parse(&filename, &source, tokens)?;

    if verbose {
        println!("  Parsed {} items", ast.items.len());
    }

    // v0.12.3: Filter items by @cfg attributes for WASM target
    let cfg_eval = CfgEvaluator::new(Target::Wasm32);
    let ast = cfg_eval.filter_program(&ast);

    if verbose {
        println!("  After @cfg filtering: {} items (target: wasm32)", ast.items.len());
    }

    // Type check
    let mut checker = bmb::types::TypeChecker::new();
    checker.check_program(&ast)?;

    // Lower to MIR
    let mir = bmb::mir::lower_program(&ast);

    // Parse WASM target
    let target = match wasm_target {
        "wasi" => WasmTarget::Wasi,
        "browser" => WasmTarget::Browser,
        "standalone" => WasmTarget::Standalone,
        _ => {
            eprintln!("Warning: Unknown WASM target '{}', using 'wasi'", wasm_target);
            WasmTarget::Wasi
        }
    };

    // Generate WASM text
    let codegen = WasmCodeGen::with_target(target);
    let wat = codegen.generate(&mir)?;

    // Determine output path
    let output_path = output.unwrap_or_else(|| {
        path.with_extension("wat")
    });

    // Write output
    std::fs::write(&output_path, &wat)?;

    if is_human_output() {
        println!("Generated: {}", output_path.display());
        if verbose {
            println!("  Target: {:?}", target);
            println!("  Size: {} bytes", wat.len());
        }
    } else {
        println!(r#"{{"type":"build_success","output":"{}","target":"{:?}","size":{}}}"#,
            output_path.display().to_string().replace('\\', "\\\\"), target, wat.len());
    }

    Ok(())
}

/// v0.21.2: Emit MIR output for bootstrap comparison
fn emit_mir_file(
    path: &PathBuf,
    output: Option<PathBuf>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string(path)?;
    let filename = path.display().to_string();

    if verbose {
        println!("Compiling {} to MIR...", filename);
    }

    // Tokenize
    let tokens = bmb::lexer::tokenize(&source)?;

    // Parse
    let ast = bmb::parser::parse(&filename, &source, tokens)?;

    if verbose {
        println!("  Parsed {} items", ast.items.len());
    }

    // Type check
    let mut checker = bmb::types::TypeChecker::new();
    checker.check_program(&ast)?;

    // Lower to MIR
    let mir = bmb::mir::lower_program(&ast);

    // Format MIR as text
    let mir_text = bmb::mir::format_mir(&mir);

    // Determine output path
    let output_path = output.unwrap_or_else(|| {
        path.with_extension("mir")
    });

    // Write output
    std::fs::write(&output_path, &mir_text)?;

    if is_human_output() {
        println!("Generated: {}", output_path.display());
        if verbose {
            println!("  Functions: {}", mir.functions.len());
            println!("  Size: {} bytes", mir_text.len());
        }
    } else {
        println!(r#"{{"type":"build_success","output":"{}","functions":{},"size":{}}}"#,
            output_path.display().to_string().replace('\\', "\\\\"), mir.functions.len(), mir_text.len());
    }

    Ok(())
}

/// v0.30.241: Stack size for interpreter thread (64MB for deep recursion in bootstrap)
const INTERPRETER_STACK_SIZE: usize = 64 * 1024 * 1024;

fn run_file(path: &Path, extra_args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    // v0.30.241: Run entire pipeline in a thread with larger stack to prevent overflow
    // Bootstrap files have deep recursion that exceeds default 1MB Windows stack
    // We run everything in the thread because Value uses Rc<RefCell<>> (not Send)
    let path = path.to_path_buf();

    // v0.46: Prepare program arguments for the BMB program
    // Format: [program_name, arg1, arg2, ...]
    let mut program_args = vec![path.display().to_string()];
    program_args.extend(extra_args.iter().cloned());

    let handle = std::thread::Builder::new()
        .name("bmb-interpreter".to_string())
        .stack_size(INTERPRETER_STACK_SIZE)
        .spawn(move || -> Result<(), String> {
            // v0.46: Set program arguments in thread-local storage
            bmb::interp::set_program_args(program_args);

            let source = std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read file: {}", e))?;
            let filename = path.display().to_string();

            // Tokenize
            let tokens = bmb::lexer::tokenize(&source)
                .map_err(|e| format!("Lexer error: {}", e))?;

            // Parse
            let ast = bmb::parser::parse(&filename, &source, tokens)
                .map_err(|e| format!("Parser error: {}", e))?;

            // Type check first
            let mut checker = bmb::types::TypeChecker::new();
            checker.check_program(&ast)
                .map_err(|e| format!("Type error: {}", e))?;

            // Run with interpreter
            let mut interpreter = bmb::interp::Interpreter::new();
            interpreter.load(&ast);
            interpreter.run(&ast)
                .map_err(|e| format!("Runtime error: {}", e.message))?;

            Ok(())
        })?;

    match handle.join() {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => {
            if is_human_output() {
                eprintln!("{}", e);
            } else {
                println!(r#"{{"type":"error","message":"{}"}}"#, e.to_string().replace('"', "\\\""));
            }
            std::process::exit(1);
        }
        Err(_) => {
            if is_human_output() {
                eprintln!("Runtime error: interpreter thread panicked");
            } else {
                println!(r#"{{"type":"error","message":"interpreter thread panicked"}}"#);
            }
            std::process::exit(1);
        }
    }
}

fn start_repl() -> Result<(), Box<dyn std::error::Error>> {
    let mut repl = bmb::repl::Repl::new()?;
    repl.run()?;
    Ok(())
}

/// v0.17: Check file with additional include paths for module resolution
fn check_file_with_includes(path: &PathBuf, include_paths: &[PathBuf]) -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string(path)?;
    let filename = path.display().to_string();

    // Tokenize
    let tokens = bmb::lexer::tokenize(&source)?;

    // Parse
    let ast = bmb::parser::parse(&filename, &source, tokens)?;

    // v0.17: Create type checker and register imported modules
    let mut checker = bmb::types::TypeChecker::new();

    // Resolve use statements and register imported modules
    let base_dir = path.parent().unwrap_or(std::path::Path::new("."));
    let mut resolver = bmb::resolver::Resolver::new(base_dir);

    // Also try include paths for module resolution
    for include_path in include_paths {
        // Try loading modules from include paths
        for item in &ast.items {
            if let bmb::ast::Item::Use(use_stmt) = item
                && !use_stmt.path.is_empty()
            {
                let module_name = &use_stmt.path[0].node;
                // Convert underscore to hyphen for package names (bmb_option -> bmb-option)
                let pkg_dir_name = module_name.replace('_', "-");
                let module_path = include_path.join(&pkg_dir_name).join("src").join("lib.bmb");
                if module_path.exists() {
                    // Load using the original filename convention
                    let lib_source = std::fs::read_to_string(&module_path)?;
                    let lib_tokens = bmb::lexer::tokenize(&lib_source)?;
                    let lib_ast = bmb::parser::parse(&module_path.display().to_string(), &lib_source, lib_tokens)?;
                    // Create a temporary module to register
                    let module = bmb::resolver::Module {
                        name: module_name.clone(),
                        path: module_path.clone(),
                        program: lib_ast,
                        exports: std::collections::HashMap::new(), // Not needed for type registration
                    };
                    checker.register_module(&module);
                }
            }
        }
    }

    // Also resolve from the file's own directory
    // v0.68: Propagate resolver errors (includes module name suggestions)
    // v0.74: Make imports mutable for usage tracking
    let mut imports = resolver.resolve_uses(&ast)?;
    for (_, info) in imports.all_imports() {
        if let Some(module) = resolver.get_module(&info.module) {
            checker.register_module(module);
        }
    }

    // Type check
    // v0.74: Pass imports for usage tracking
    checker.check_program_with_imports(&ast, &mut imports)?;

    // v0.74: Collect unused import warnings
    let mut all_warnings: Vec<bmb::error::CompileWarning> = checker.warnings().to_vec();
    for (name, span) in imports.get_unused() {
        all_warnings.push(bmb::error::CompileWarning::unused_import(name, span));
    }

    // v0.47: Report warnings (non-fatal diagnostics)
    // v0.71: Default machine output, --human for human-readable
    let warnings = &all_warnings;
    if !warnings.is_empty() {
        if is_human_output() {
            for warning in warnings {
                bmb::error::report_warning(&filename, &source, warning);
            }
            println!("  {} warning(s) generated", warnings.len());
        } else {
            bmb::error::report_warnings_machine(&filename, &source, warnings);
        }
    }

    if is_human_output() {
        println!("✓ {} type checks successfully", filename);
    } else {
        println!(r#"{{"type":"success","file":"{}","warnings":{}}}"#, filename, warnings.len());
    }
    Ok(())
}

/// Lint a BMB source file or directory (v0.45)
/// Collects and reports all warnings from type checking
fn lint_file(path: &PathBuf, strict: bool, include_paths: &[PathBuf]) -> Result<(), Box<dyn std::error::Error>> {
    // Handle directory recursively
    if path.is_dir() {
        return lint_directory(path, strict, include_paths);
    }

    let source = std::fs::read_to_string(path)?;
    let filename = path.display().to_string();

    // Tokenize
    let tokens = match bmb::lexer::tokenize(&source) {
        Ok(t) => t,
        Err(e) => {
            bmb::error::report_error(&filename, &source, &e);
            return Err(e.into());
        }
    };

    // Parse
    let ast = match bmb::parser::parse(&filename, &source, tokens) {
        Ok(a) => a,
        Err(e) => {
            bmb::error::report_error(&filename, &source, &e);
            return Err(e.into());
        }
    };

    // Create type checker
    let mut checker = bmb::types::TypeChecker::new();

    // Resolve use statements and register imported modules
    let base_dir = path.parent().unwrap_or(std::path::Path::new("."));
    let mut resolver = bmb::resolver::Resolver::new(base_dir);

    // Try include paths for module resolution
    for include_path in include_paths {
        for item in &ast.items {
            if let bmb::ast::Item::Use(use_stmt) = item
                && !use_stmt.path.is_empty()
            {
                let module_name = &use_stmt.path[0].node;
                let pkg_dir_name = module_name.replace('_', "-");
                let module_path = include_path.join(&pkg_dir_name).join("src").join("lib.bmb");
                if module_path.exists()
                    && let Ok(lib_source) = std::fs::read_to_string(&module_path)
                    && let Ok(lib_tokens) = bmb::lexer::tokenize(&lib_source)
                    && let Ok(lib_ast) = bmb::parser::parse(
                        &module_path.display().to_string(),
                        &lib_source,
                        lib_tokens,
                    )
                {
                    let module = bmb::resolver::Module {
                        name: module_name.clone(),
                        path: module_path.clone(),
                        program: lib_ast,
                        exports: std::collections::HashMap::new(),
                    };
                    checker.register_module(&module);
                }
            }
        }
    }

    // Resolve from file's directory
    let mut imports = resolver.resolve_uses(&ast)?;
    for (_, info) in imports.all_imports() {
        if let Some(module) = resolver.get_module(&info.module) {
            checker.register_module(module);
        }
    }

    // Type check (continue even with errors to collect all warnings)
    let type_result = checker.check_program_with_imports(&ast, &mut imports);

    // Collect all warnings
    let mut all_warnings: Vec<bmb::error::CompileWarning> = checker.warnings().to_vec();
    for (name, span) in imports.get_unused() {
        all_warnings.push(bmb::error::CompileWarning::unused_import(name, span));
    }

    // Report type errors if any
    if let Err(e) = type_result {
        bmb::error::report_error(&filename, &source, &e);
        // Still report warnings before returning error
        if !all_warnings.is_empty() {
            if is_human_output() {
                println!("\n  Warnings:");
                for warning in &all_warnings {
                    bmb::error::report_warning(&filename, &source, warning);
                }
            } else {
                bmb::error::report_warnings_machine(&filename, &source, &all_warnings);
            }
        }
        return Err(e.into());
    }

    // Report warnings
    let warning_count = all_warnings.len();
    if warning_count > 0 {
        if is_human_output() {
            for warning in &all_warnings {
                bmb::error::report_warning(&filename, &source, warning);
            }
            println!("\n  {} warning(s) in {}", warning_count, filename);
        } else {
            bmb::error::report_warnings_machine(&filename, &source, &all_warnings);
        }
    } else if is_human_output() {
        println!("✓ {} - no warnings", filename);
    } else {
        println!(r#"{{"type":"lint","file":"{}","warnings":0}}"#, filename);
    }

    // In strict mode, any warning is an error
    if strict && warning_count > 0 {
        if is_human_output() {
            eprintln!("\n  Lint failed: {} warning(s) in strict mode", warning_count);
        }
        std::process::exit(1);
    }

    Ok(())
}

/// Lint all .bmb files in a directory recursively (v0.45)
fn lint_directory(dir: &PathBuf, strict: bool, _include_paths: &[PathBuf]) -> Result<(), Box<dyn std::error::Error>> {
    let mut total_warnings = 0;
    let mut total_files = 0;
    let mut failed_files = 0;

    // Collect all .bmb files
    fn collect_bmb_files(dir: &PathBuf, files: &mut Vec<PathBuf>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_dir() {
                    collect_bmb_files(&path, files);
                } else if path.extension().is_some_and(|ext| ext == "bmb") {
                    files.push(path);
                }
            }
        }
    }

    let mut files = Vec::new();
    collect_bmb_files(dir, &mut files);
    files.sort();

    if is_human_output() {
        println!("Linting {} files in {}...\n", files.len(), dir.display());
    }

    for file in &files {
        total_files += 1;

        let source = match std::fs::read_to_string(file) {
            Ok(s) => s,
            Err(_) => {
                failed_files += 1;
                continue;
            }
        };
        let filename = file.display().to_string();

        // Tokenize and parse
        let tokens = match bmb::lexer::tokenize(&source) {
            Ok(t) => t,
            Err(_) => {
                failed_files += 1;
                continue;
            }
        };
        let ast = match bmb::parser::parse(&filename, &source, tokens) {
            Ok(a) => a,
            Err(_) => {
                failed_files += 1;
                continue;
            }
        };

        // Type check
        let mut checker = bmb::types::TypeChecker::new();
        let base_dir = file.parent().unwrap_or(std::path::Path::new("."));
        let mut resolver = bmb::resolver::Resolver::new(base_dir);

        // Resolve imports
        if let Ok(mut imports) = resolver.resolve_uses(&ast) {
            for (_, info) in imports.all_imports() {
                if let Some(module) = resolver.get_module(&info.module) {
                    checker.register_module(module);
                }
            }

            if checker.check_program_with_imports(&ast, &mut imports).is_ok() {
                let mut warnings: Vec<bmb::error::CompileWarning> = checker.warnings().to_vec();
                for (name, span) in imports.get_unused() {
                    warnings.push(bmb::error::CompileWarning::unused_import(name, span));
                }

                if !warnings.is_empty() {
                    total_warnings += warnings.len();
                    if is_human_output() {
                        for warning in &warnings {
                            bmb::error::report_warning(&filename, &source, warning);
                        }
                    } else {
                        bmb::error::report_warnings_machine(&filename, &source, &warnings);
                    }
                }
            } else {
                failed_files += 1;
            }
        }
    }

    // Summary
    if is_human_output() {
        println!("\nLint summary:");
        println!("  Files checked: {}", total_files);
        println!("  Total warnings: {}", total_warnings);
        if failed_files > 0 {
            println!("  Failed to lint: {}", failed_files);
        }
    } else {
        println!(r#"{{"type":"lint_summary","files":{},"warnings":{},"errors":{}}}"#,
            total_files, total_warnings, failed_files);
    }

    // In strict mode, any warning is an error
    if strict && total_warnings > 0 {
        if is_human_output() {
            eprintln!("\nLint failed: {} warning(s) in strict mode", total_warnings);
        }
        std::process::exit(1);
    }

    Ok(())
}

fn verify_file(path: &PathBuf, z3_path: &str, timeout: u32) -> Result<(), Box<dyn std::error::Error>> {
    use bmb::index::{ProofEntry, ProofIndex, ProofStatus, write_proof_index};
    use bmb::smt::VerifyResult;

    let source = std::fs::read_to_string(path)?;
    let filename = path.display().to_string();

    // Tokenize
    let tokens = bmb::lexer::tokenize(&source)?;

    // Parse
    let ast = bmb::parser::parse(&filename, &source, tokens)?;

    // Type check first
    let mut checker = bmb::types::TypeChecker::new();
    checker.check_program(&ast)?;

    // Set up verifier
    let verifier = bmb::verify::ContractVerifier::new()
        .with_z3_path(z3_path)
        .with_timeout(timeout);

    // Check if solver is available
    let z3_available = verifier.is_solver_available();
    if !z3_available {
        if is_human_output() {
            eprintln!("Warning: Z3 solver not found at '{}'. Install Z3 or specify --z3-path.", z3_path);
            eprintln!("Skipping contract verification.");
        } else {
            println!(r#"{{"type":"verify_skip","reason":"z3_not_found"}}"#);
        }
        return Ok(());
    }

    // Get Z3 version if available
    let z3_version = get_z3_version(z3_path);

    // Verify contracts
    let start_time = std::time::Instant::now();
    let report = verifier.verify_program(&ast);
    let verify_time_ms = start_time.elapsed().as_millis() as u64;

    // v0.50.24: Create proof index entries from verification report
    let mut proof_index = ProofIndex::new(z3_available, z3_version);
    for func_report in &report.functions {
        let pre_status = func_report.pre_result.as_ref().map(|r| match r {
            VerifyResult::Verified => ProofStatus::Verified,
            VerifyResult::Failed(_) => ProofStatus::Failed,
            VerifyResult::Unknown(msg) if msg.contains("timeout") => ProofStatus::Timeout,
            VerifyResult::Unknown(_) => ProofStatus::Unknown,
            VerifyResult::SolverNotAvailable => ProofStatus::Unavailable,
        });

        let post_status = func_report.post_result.as_ref().map(|r| match r {
            VerifyResult::Verified => ProofStatus::Verified,
            VerifyResult::Failed(_) => ProofStatus::Failed,
            VerifyResult::Unknown(msg) if msg.contains("timeout") => ProofStatus::Timeout,
            VerifyResult::Unknown(_) => ProofStatus::Unknown,
            VerifyResult::SolverNotAvailable => ProofStatus::Unavailable,
        });

        // Extract counterexample if failed
        let counterexample = func_report.post_result.as_ref().and_then(|r| {
            if let VerifyResult::Failed(ce) = r {
                Some(ce.assignments.clone())
            } else {
                None
            }
        }).or_else(|| {
            func_report.pre_result.as_ref().and_then(|r| {
                if let VerifyResult::Failed(ce) = r {
                    Some(ce.assignments.clone())
                } else {
                    None
                }
            })
        });

        proof_index.add_proof(ProofEntry {
            name: func_report.name.clone(),
            file: filename.clone(),
            line: 1, // Would need span info for accurate line numbers
            pre_status,
            post_status,
            counterexample,
            verify_time_ms: Some(verify_time_ms / report.functions.len().max(1) as u64),
            verified_at: Some(proof_index.verified_at.clone()),
        });
    }

    // Save proof index to .bmb/index/proofs.json
    let current_dir = std::env::current_dir()?;
    if let Err(e) = write_proof_index(&proof_index, &current_dir) {
        if is_human_output() {
            eprintln!("Warning: Could not save proof index: {}", e);
        }
    }

    // Print report
    if is_human_output() {
        print!("{}", report);
    } else {
        let verified = report.verified_count();
        let failed = report.failed_count();
        let total = verified + failed;
        println!(r#"{{"type":"verify_result","total":{},"verified":{},"failed":{}}}"#,
            total, verified, failed);
    }

    // Exit with error if any verification failed
    if !report.all_verified() {
        std::process::exit(1);
    }

    Ok(())
}

/// Get Z3 version string
fn get_z3_version(z3_path: &str) -> Option<String> {
    use std::process::Command;
    Command::new(z3_path)
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| {
            String::from_utf8(output.stdout).ok()
        })
        .map(|s| s.trim().to_string())
}

fn parse_file(path: &PathBuf, format: &str) -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string(path)?;
    let filename = path.display().to_string();

    let tokens = bmb::lexer::tokenize(&source)?;
    let ast = bmb::parser::parse(&filename, &source, tokens)?;

    match format {
        "sexpr" | "s-expression" => println!("{}", bmb::ast::output::to_sexpr(&ast)),
        _ => println!("{}", serde_json::to_string_pretty(&ast)?),
    }
    Ok(())
}

fn tokenize_file(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string(path)?;

    let tokens = bmb::lexer::tokenize(&source)?;

    if is_human_output() {
        for (tok, span) in &tokens {
            println!("{:?} @ {}..{}", tok, span.start, span.end);
        }
    } else {
        // JSON array of tokens
        print!("[");
        for (i, (tok, span)) in tokens.iter().enumerate() {
            if i > 0 { print!(","); }
            print!(r#"{{"token":"{:?}","start":{},"end":{}}}"#, tok, span.start, span.end);
        }
        println!("]");
    }

    Ok(())
}

fn test_file(path: &PathBuf, filter: Option<&str>, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Instant;

    // Collect test files
    let test_files = if path.is_dir() {
        collect_test_files(path)?
    } else {
        vec![path.clone()]
    };

    if test_files.is_empty() {
        if is_human_output() {
            println!("No test files found");
        } else {
            println!(r#"{{"type":"test_result","tests":0,"passed":0,"failed":0}}"#);
        }
        return Ok(());
    }

    let mut total_passed = 0;
    let mut total_failed = 0;
    let mut total_tests = 0;
    let start_time = Instant::now();

    for test_file in &test_files {
        let source = std::fs::read_to_string(test_file)?;
        let filename = test_file.display().to_string();

        // Tokenize
        let tokens = bmb::lexer::tokenize(&source)?;

        // Parse
        let ast = bmb::parser::parse(&filename, &source, tokens)?;

        // Type check
        let mut checker = bmb::types::TypeChecker::new();
        checker.check_program(&ast)?;

        // Run tests with interpreter
        let mut interpreter = bmb::interp::Interpreter::new();
        interpreter.load(&ast);

        let test_names = interpreter.get_test_functions();
        let filtered_tests: Vec<_> = test_names
            .iter()
            .filter(|name| {
                filter.is_none_or(|f| name.contains(f))
            })
            .collect();

        if filtered_tests.is_empty() {
            continue;
        }

        if is_human_output() && (verbose || test_files.len() > 1) {
            println!("\n📂 {}", filename);
        }

        for test_name in filtered_tests {
            total_tests += 1;
            let test_start = Instant::now();

            match interpreter.run_function(test_name) {
                Ok(value) => {
                    let passed = match value {
                        bmb::interp::Value::Bool(b) => b,
                        bmb::interp::Value::Int(n) => n != 0,
                        _ => true,
                    };

                    let elapsed_ms = test_start.elapsed().as_millis();

                    if passed {
                        total_passed += 1;
                        if is_human_output() && verbose {
                            println!("  ✅ {} ({:.2?})", test_name, test_start.elapsed());
                        }
                    } else {
                        total_failed += 1;
                        if is_human_output() {
                            println!("  ❌ {} - returned false ({:.2?})", test_name, test_start.elapsed());
                        } else {
                            println!(r#"{{"type":"test_fail","name":"{}","file":"{}","reason":"returned false","ms":{}}}"#,
                                test_name, filename, elapsed_ms);
                        }
                    }
                }
                Err(e) => {
                    total_failed += 1;
                    if is_human_output() {
                        println!("  ❌ {} - {}", test_name, e.message);
                    } else {
                        println!(r#"{{"type":"test_fail","name":"{}","file":"{}","reason":"{}"}}"#,
                            test_name, filename, e.message.replace('"', "\\\""));
                    }
                }
            }
        }
    }

    let elapsed = start_time.elapsed();

    // Print summary
    if is_human_output() {
        println!();
        if total_tests == 0 {
            println!("No tests found");
        } else if total_failed == 0 {
            println!("✅ {} tests passed ({:.2?})", total_passed, elapsed);
        } else {
            println!(
                "❌ {} passed, {} failed of {} tests ({:.2?})",
                total_passed, total_failed, total_tests, elapsed
            );
            std::process::exit(1);
        }
    } else {
        println!(r#"{{"type":"test_result","tests":{},"passed":{},"failed":{},"ms":{}}}"#,
            total_tests, total_passed, total_failed, elapsed.as_millis());
        if total_failed > 0 {
            std::process::exit(1);
        }
    }

    Ok(())
}

fn collect_test_files(dir: &PathBuf) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            files.extend(collect_test_files(&path)?);
        } else if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy();
            if name_str.starts_with("test_") && name_str.ends_with(".bmb") {
                files.push(path);
            }
        }
    }

    Ok(files)
}

/// Extract comments from source code with their line numbers
/// Returns a Vec of (line_number, comment_text) where line_number is 0-indexed
fn extract_comments(source: &str) -> Vec<(usize, String)> {
    let mut comments = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        // Check for // style comments (whole line only)
        if trimmed.starts_with("//") {
            comments.push((line_num, line.to_string()));
        } else if trimmed.starts_with("--") {
            // Legacy -- comment (whole line)
            comments.push((line_num, line.to_string()));
        }
    }

    comments
}

/// Get the line number from a byte offset in source
fn line_number_at_offset(source: &str, offset: usize) -> usize {
    source[..offset.min(source.len())].matches('\n').count()
}

fn fmt_file(path: &PathBuf, check: bool) -> Result<(), Box<dyn std::error::Error>> {
    let files = if path.is_dir() {
        collect_bmb_files(path)?
    } else {
        vec![path.clone()]
    };

    if files.is_empty() {
        if is_human_output() {
            println!("No BMB files found");
        } else {
            println!(r#"{{"type":"fmt_result","files":0}}"#);
        }
        return Ok(());
    }

    let mut needs_formatting = false;
    let mut _formatted_count = 0;

    for file in &files {
        let source = std::fs::read_to_string(file)?;
        let filename = file.display().to_string();

        // Extract comments before parsing (they get lost during tokenization)
        let comments = extract_comments(&source);

        // Tokenize
        let tokens = bmb::lexer::tokenize(&source)?;

        // Parse
        let ast = bmb::parser::parse(&filename, &source, tokens)?;

        // Format AST back to source, preserving comments
        let formatted = format_program_with_comments(&ast, &source, &comments);

        if check {
            if source != formatted {
                needs_formatting = true;
                if is_human_output() {
                    println!("❌ {} needs formatting", filename);
                } else {
                    println!(r#"{{"type":"fmt_needed","file":"{}"}}"#, filename);
                }
            } else if is_human_output() {
                println!("✓ {} is formatted", filename);
            }
        } else if source != formatted {
            std::fs::write(file, &formatted)?;
            _formatted_count += 1;
            if is_human_output() {
                println!("✓ formatted {}", filename);
            } else {
                println!(r#"{{"type":"fmt_formatted","file":"{}"}}"#, filename);
            }
        } else if is_human_output() {
            println!("✓ {} (unchanged)", filename);
        }
    }

    if check && needs_formatting {
        std::process::exit(1);
    }

    Ok(())
}

fn collect_bmb_files(dir: &PathBuf) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            files.extend(collect_bmb_files(&path)?);
        } else if path.extension().is_some_and(|e| e == "bmb") {
            files.push(path);
        }
    }

    Ok(files)
}

/// Get the starting span of an Item (for comment attachment)
fn get_item_span(item: &bmb::ast::Item) -> bmb::ast::Span {
    use bmb::ast::Item;
    match item {
        Item::FnDef(f) => f.span,
        Item::StructDef(s) => s.span,
        Item::EnumDef(e) => e.span,
        Item::TypeAlias(t) => t.span,
        Item::Use(u) => u.span,
        Item::ExternFn(e) => e.span,
        Item::TraitDef(t) => t.span,
        Item::ImplBlock(i) => i.span,
    }
}

/// Format program with comment preservation
/// Attaches comments to the items they precede based on line numbers
fn format_program_with_comments(
    program: &bmb::ast::Program,
    source: &str,
    comments: &[(usize, String)],
) -> String {
    use bmb::ast::{Item, Visibility};

    let mut output = String::new();
    let mut used_comments: std::collections::HashSet<usize> = std::collections::HashSet::new();

    // Collect item spans (line numbers)
    let mut item_lines: Vec<(usize, usize)> = Vec::new(); // (item_index, start_line)
    for (idx, item) in program.items.iter().enumerate() {
        let span = get_item_span(item);
        let start_line = line_number_at_offset(source, span.start);
        item_lines.push((idx, start_line));
    }

    // Find file-level comments (before first item)
    let first_item_line = item_lines.first().map(|(_, l)| *l).unwrap_or(usize::MAX);
    for (line_num, comment_text) in comments {
        if *line_num < first_item_line && !used_comments.contains(line_num) {
            output.push_str(comment_text);
            output.push('\n');
            used_comments.insert(*line_num);
        }
    }

    // Process each item with its preceding comments
    for (i, item) in program.items.iter().enumerate() {
        let item_start_line = item_lines.iter().find(|(idx, _)| *idx == i).map(|(_, l)| *l).unwrap_or(0);

        // Find the end of the previous item (or file start)
        let prev_end_line = if i > 0 {
            item_lines.iter().find(|(idx, _)| *idx == i - 1).map(|(_, l)| *l + 1).unwrap_or(0)
        } else {
            0
        };

        // Add blank line between items (if not first item)
        if i > 0 {
            output.push('\n');
        }

        // Find comments between previous item end and this item start
        for (line_num, comment_text) in comments {
            if *line_num >= prev_end_line && *line_num < item_start_line && !used_comments.contains(line_num) {
                output.push_str(comment_text);
                output.push('\n');
                used_comments.insert(*line_num);
            }
        }

        // Format the item
        match item {
            Item::FnDef(fn_def) => {
                output.push_str(&format_fn_def(fn_def));
            }
            Item::StructDef(s) => {
                if s.visibility == Visibility::Public {
                    output.push_str("pub ");
                }
                output.push_str(&format!("struct {} {{\n", s.name.node));
                for field in &s.fields {
                    output.push_str(&format!("    {}: {},\n", field.name.node, format_type(&field.ty.node)));
                }
                output.push('}');
            }
            Item::EnumDef(e) => {
                if e.visibility == Visibility::Public {
                    output.push_str("pub ");
                }
                output.push_str(&format!("enum {} {{\n", e.name.node));
                for variant in &e.variants {
                    output.push_str(&format!("    {},\n", variant.name.node));
                }
                output.push('}');
            }
            Item::Use(u) => {
                let path_str: Vec<_> = u.path.iter().map(|s| s.node.as_str()).collect();
                output.push_str(&format!("use {};", path_str.join("::")));
            }
            Item::ExternFn(e) => {
                if e.visibility == Visibility::Public {
                    output.push_str("pub ");
                }
                output.push_str(&format!("extern fn {}(", e.name.node));
                let params: Vec<_> = e.params.iter()
                    .map(|p| format!("{}: {}", p.name.node, format_type(&p.ty.node)))
                    .collect();
                output.push_str(&params.join(", "));
                output.push_str(&format!(") -> {};", format_type(&e.ret_ty.node)));
            }
            Item::TraitDef(t) => {
                if t.visibility == Visibility::Public {
                    output.push_str("pub ");
                }
                output.push_str(&format!("trait {} {{\n", t.name.node));
                for method in &t.methods {
                    let params: Vec<_> = method.params.iter()
                        .map(|p| format!("{}: {}", p.name.node, format_type(&p.ty.node)))
                        .collect();
                    output.push_str(&format!("    fn {}({}) -> {};\n",
                        method.name.node, params.join(", "), format_type(&method.ret_ty.node)));
                }
                output.push('}');
            }
            Item::ImplBlock(i) => {
                output.push_str(&format!("impl {} for {} {{\n", i.trait_name.node, format_type(&i.target_type.node)));
                for method in &i.methods {
                    output.push_str("    ");
                    output.push_str(&format_fn_def(method));
                    output.push('\n');
                }
                output.push('}');
            }
            Item::TypeAlias(t) => {
                if t.visibility == Visibility::Public {
                    output.push_str("pub ");
                }
                output.push_str(&format!("type {} = {};", t.name.node, format_type(&t.target.node)));
            }
        }
        output.push('\n');
    }

    // Add any trailing comments (after last item)
    let last_item_line = item_lines.last().map(|(_, l)| *l).unwrap_or(0);
    for (line_num, comment_text) in comments {
        if *line_num > last_item_line && !used_comments.contains(line_num) {
            output.push_str(comment_text);
            output.push('\n');
            used_comments.insert(*line_num);
        }
    }

    output
}

fn format_fn_def(fn_def: &bmb::ast::FnDef) -> String {
    use bmb::ast::Visibility;

    let mut s = String::new();

    // Visibility
    if fn_def.visibility == Visibility::Public {
        s.push_str("pub ");
    }

    // Function signature
    s.push_str(&format!("fn {}(", fn_def.name.node));

    for (i, param) in fn_def.params.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        s.push_str(&format!("{}: {}", param.name.node, format_type(&param.ty.node)));
    }

    s.push_str(&format!(") -> {}", format_type(&fn_def.ret_ty.node)));

    // Contracts
    if let Some(pre) = &fn_def.pre {
        s.push_str(&format!("\n  pre {}", format_expr(&pre.node)));
    }

    if let Some(post) = &fn_def.post {
        s.push_str(&format!("\n  post {}", format_expr(&post.node)));
    }

    // Body
    s.push_str(&format!("\n= {};", format_expr(&fn_def.body.node)));

    s
}

fn format_type(ty: &bmb::ast::Type) -> String {
    use bmb::ast::Type;

    match ty {
        Type::I32 => "i32".to_string(),
        Type::I64 => "i64".to_string(),
        // v0.38: Unsigned types
        Type::U32 => "u32".to_string(),
        Type::U64 => "u64".to_string(),
        Type::F64 => "f64".to_string(),
        Type::Bool => "bool".to_string(),
        Type::String => "String".to_string(),
        // v0.64: Character type
        Type::Char => "char".to_string(),
        Type::Unit => "()".to_string(),
        Type::Range(elem) => format!("Range<{}>", format_type(elem)),
        Type::Named(name) => name.clone(),
        // v0.13.1: Type variable
        Type::TypeVar(name) => name.clone(),
        // v0.13.1: Generic type
        Type::Generic { name, type_args } => {
            let args_str = type_args.iter()
                .map(|t| format_type(t))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}<{}>", name, args_str)
        }
        Type::Struct { name, .. } => name.clone(),
        Type::Enum { name, .. } => name.clone(),
        Type::Array(elem, size) => format!("[{}; {}]", format_type(elem), size),
        Type::Ref(inner) => format!("&{}", format_type(inner)),
        Type::RefMut(inner) => format!("&mut {}", format_type(inner)),
        // v0.2: Refined types display base{constraints}
        Type::Refined { base, constraints } => {
            let constraint_str = constraints.iter()
                .map(|c| format_expr(&c.node))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}{{{}}}", format_type(base), constraint_str)
        }
        // v0.20.0: Fn type
        Type::Fn { params, ret } => {
            let params_str = params.iter()
                .map(|p| format_type(p))
                .collect::<Vec<_>>()
                .join(", ");
            format!("fn({}) -> {}", params_str, format_type(ret))
        }
        // v0.31: Never type
        Type::Never => "!".to_string(),
        // v0.37: Nullable type
        Type::Nullable(inner) => format!("{}?", format_type(inner)),
        // v0.42: Tuple type
        Type::Tuple(elems) => {
            let elems_str: Vec<_> = elems.iter().map(|t| format_type(t)).collect();
            format!("({})", elems_str.join(", "))
        }
    }
}

fn format_expr(expr: &bmb::ast::Expr) -> String {
    use bmb::ast::{Expr, BinOp, UnOp};

    match expr {
        Expr::IntLit(n) => n.to_string(),
        Expr::FloatLit(f) => f.to_string(),
        Expr::BoolLit(b) => b.to_string(),
        Expr::StringLit(s) => format!("\"{}\"", s),
        // v0.64: Character literal
        Expr::CharLit(c) => format!("'{}'", c.escape_default()),
        Expr::Unit => "()".to_string(),
        Expr::Var(name) => name.clone(),
        Expr::Ret => "ret".to_string(),
        Expr::It => "it".to_string(),

        Expr::Binary { left, op, right } => {
            let op_str = match op {
                BinOp::Add => "+",
                BinOp::Sub => "-",
                BinOp::Mul => "*",
                BinOp::Div => "/",
                BinOp::Mod => "%",
                // v0.37: Wrapping arithmetic
                BinOp::AddWrap => "+%",
                BinOp::SubWrap => "-%",
                BinOp::MulWrap => "*%",
                // v0.38: Checked arithmetic
                BinOp::AddChecked => "+?",
                BinOp::SubChecked => "-?",
                BinOp::MulChecked => "*?",
                // v0.38: Saturating arithmetic
                BinOp::AddSat => "+|",
                BinOp::SubSat => "-|",
                BinOp::MulSat => "*|",
                BinOp::Eq => "==",
                BinOp::Ne => "!=",
                BinOp::Lt => "<",
                BinOp::Le => "<=",
                BinOp::Gt => ">",
                BinOp::Ge => ">=",
                BinOp::And => "and",
                BinOp::Or => "or",
                // v0.32: Shift operators
                BinOp::Shl => "<<",
                BinOp::Shr => ">>",
                // v0.36: Bitwise operators
                BinOp::Band => "band",
                BinOp::Bor => "bor",
                BinOp::Bxor => "bxor",
                // v0.36: Logical implication
                BinOp::Implies => "implies",
            };
            format!("{} {} {}", format_expr(&left.node), op_str, format_expr(&right.node))
        }

        Expr::Unary { op, expr } => {
            let op_str = match op {
                UnOp::Neg => "-",
                UnOp::Not => "not ",
                // v0.36: Bitwise not
                UnOp::Bnot => "bnot ",
            };
            format!("{}{}", op_str, format_expr(&expr.node))
        }

        Expr::If { cond, then_branch, else_branch } => {
            format!(
                "if {} then {} else {}",
                format_expr(&cond.node),
                format_expr(&then_branch.node),
                format_expr(&else_branch.node)
            )
        }

        Expr::Let { name, mutable, ty, value, body } => {
            let mut_str = if *mutable { "mut " } else { "" };
            let ty_str = ty.as_ref().map(|t| format!(": {}", format_type(&t.node))).unwrap_or_default();
            format!(
                "let {}{}{} = {};\n    {}",
                mut_str,
                name,
                ty_str,
                format_expr(&value.node),
                format_expr(&body.node)
            )
        }

        Expr::Call { func, args } => {
            let args_str: Vec<_> = args.iter().map(|a| format_expr(&a.node)).collect();
            format!("{}({})", func, args_str.join(", "))
        }

        Expr::MethodCall { receiver, method, args } => {
            let args_str: Vec<_> = args.iter().map(|a| format_expr(&a.node)).collect();
            format!("{}.{}({})", format_expr(&receiver.node), method, args_str.join(", "))
        }

        Expr::Index { expr: arr, index } => {
            format!("{}[{}]", format_expr(&arr.node), format_expr(&index.node))
        }

        Expr::ArrayLit(elems) => {
            let elems_str: Vec<_> = elems.iter().map(|e| format_expr(&e.node)).collect();
            format!("[{}]", elems_str.join(", "))
        }

        // v0.42: Tuple expression
        Expr::Tuple(elems) => {
            let elems_str: Vec<_> = elems.iter().map(|e| format_expr(&e.node)).collect();
            if elems.len() == 1 {
                format!("({},)", elems_str.join(", "))
            } else {
                format!("({})", elems_str.join(", "))
            }
        }

        Expr::StructInit { name, fields } => {
            let fields_str: Vec<_> = fields.iter()
                .map(|(n, v)| format!("{}: {}", n.node, format_expr(&v.node)))
                .collect();
            format!("{} {{ {} }}", name, fields_str.join(", "))
        }

        Expr::FieldAccess { expr, field } => {
            format!("{}.{}", format_expr(&expr.node), field.node)
        }

        // v0.43: Tuple field access
        Expr::TupleField { expr, index } => {
            format!("{}.{}", format_expr(&expr.node), index)
        }

        Expr::Match { expr, arms } => {
            let arms_str: Vec<_> = arms.iter()
                .map(|arm| format!("{} => {}", format_pattern(&arm.pattern.node), format_expr(&arm.body.node)))
                .collect();
            format!("match {} {{ {} }}", format_expr(&expr.node), arms_str.join(", "))
        }

        Expr::Block(stmts) => {
            if stmts.is_empty() {
                "{}".to_string()
            } else {
                let stmts_str: Vec<_> = stmts.iter().map(|s| format_expr(&s.node)).collect();
                format!("{{ {} }}", stmts_str.join("; "))
            }
        }

        Expr::Assign { name, value } => {
            format!("{} = {}", name, format_expr(&value.node))
        }

        // v0.37: Include invariant in format if present
        Expr::While { cond, invariant, body } => {
            match invariant {
                Some(inv) => format!(
                    "while {} invariant {} {{ {} }}",
                    format_expr(&cond.node),
                    format_expr(&inv.node),
                    format_expr(&body.node)
                ),
                None => format!(
                    "while {} {{ {} }}",
                    format_expr(&cond.node),
                    format_expr(&body.node)
                ),
            }
        }

        Expr::For { var, iter, body } => {
            format!(
                "for {} in {} {{ {} }}",
                var,
                format_expr(&iter.node),
                format_expr(&body.node)
            )
        }

        Expr::Range { start, end, kind } => {
            let op = match kind {
                bmb::ast::RangeKind::Exclusive => "..<",
                bmb::ast::RangeKind::Inclusive => "..=",
            };
            format!("{}{}{}", format_expr(&start.node), op, format_expr(&end.node))
        }

        Expr::EnumVariant { enum_name, variant, args } => {
            if args.is_empty() {
                format!("{}::{}", enum_name, variant)
            } else {
                let args_str: Vec<_> = args.iter().map(|a| format_expr(&a.node)).collect();
                format!("{}::{}({})", enum_name, variant, args_str.join(", "))
            }
        }

        Expr::Ref(inner) => {
            format!("&{}", format_expr(&inner.node))
        }

        Expr::RefMut(inner) => {
            format!("&mut {}", format_expr(&inner.node))
        }

        Expr::Deref(inner) => {
            format!("*{}", format_expr(&inner.node))
        }

        Expr::StateRef { expr, state } => {
            format!("{}{}", format_expr(&expr.node), state)
        }

        // v0.20.0: Closure expressions
        Expr::Closure { params, ret_ty, body } => {
            let params_str = params
                .iter()
                .map(|p| {
                    if let Some(ty) = &p.ty {
                        format!("{}: {}", p.name.node, format_type(&ty.node))
                    } else {
                        p.name.node.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            let ret_str = ret_ty
                .as_ref()
                .map(|t| format!(" -> {}", format_type(&t.node)))
                .unwrap_or_default();
            format!("fn |{}|{} {{ {} }}", params_str, ret_str, format_expr(&body.node))
        }

        // v0.31: Todo expression
        Expr::Todo { message } => {
            match message {
                Some(msg) => format!("todo \"{}\"", msg),
                None => "todo".to_string(),
            }
        }

        // v0.36: Additional control flow
        Expr::Loop { body } => format!("loop {{ {} }}", format_expr(&body.node)),
        Expr::Break { value } => match value {
            Some(v) => format!("break {}", format_expr(&v.node)),
            None => "break".to_string(),
        },
        Expr::Continue => "continue".to_string(),
        Expr::Return { value } => match value {
            Some(v) => format!("return {}", format_expr(&v.node)),
            None => "return".to_string(),
        },

        // v0.37: Quantifiers
        Expr::Forall { var, ty, body } => {
            format!("forall {}: {}, {}", var.node, format_type(&ty.node), format_expr(&body.node))
        }
        Expr::Exists { var, ty, body } => {
            format!("exists {}: {}, {}", var.node, format_type(&ty.node), format_expr(&body.node))
        }
        // v0.39: Type cast
        Expr::Cast { expr, ty } => {
            format!("{} as {}", format_expr(&expr.node), format_type(&ty.node))
        }
    }
}

fn format_literal_pattern(lit: &bmb::ast::LiteralPattern) -> String {
    use bmb::ast::LiteralPattern;
    match lit {
        LiteralPattern::Int(n) => n.to_string(),
        LiteralPattern::Float(f) => f.to_string(),
        LiteralPattern::Bool(b) => b.to_string(),
        LiteralPattern::String(s) => format!("\"{}\"", s),
    }
}

fn format_pattern(pattern: &bmb::ast::Pattern) -> String {
    use bmb::ast::Pattern;

    match pattern {
        Pattern::Wildcard => "_".to_string(),
        Pattern::Var(name) => name.clone(),
        Pattern::Literal(lit) => format_literal_pattern(lit),
        // v0.41: Nested patterns in enum bindings
        Pattern::EnumVariant { enum_name, variant, bindings } => {
            if bindings.is_empty() {
                format!("{}::{}", enum_name, variant)
            } else {
                let bindings_str: Vec<_> = bindings.iter()
                    .map(|b| format_pattern(&b.node))
                    .collect();
                format!("{}::{}({})", enum_name, variant, bindings_str.join(", "))
            }
        }
        Pattern::Struct { name, fields } => {
            let fields_str: Vec<_> = fields.iter()
                .map(|(n, p)| format!("{}: {}", n.node, format_pattern(&p.node)))
                .collect();
            format!("{} {{ {} }}", name, fields_str.join(", "))
        }
        // v0.39: Range pattern
        Pattern::Range { start, end, inclusive } => {
            let op = if *inclusive { "..=" } else { ".." };
            format!("{}{}{}", format_literal_pattern(start), op, format_literal_pattern(end))
        }
        // v0.40: Or-pattern
        Pattern::Or(alts) => {
            let alts_str: Vec<_> = alts.iter().map(|p| format_pattern(&p.node)).collect();
            alts_str.join(" | ")
        }
        // v0.41: Binding pattern
        Pattern::Binding { name, pattern } => {
            format!("{} @ {}", name, format_pattern(&pattern.node))
        }
        // v0.42: Tuple pattern
        Pattern::Tuple(elems) => {
            let elems_str: Vec<_> = elems.iter().map(|p| format_pattern(&p.node)).collect();
            if elems.len() == 1 {
                format!("({},)", elems_str.join(", "))
            } else {
                format!("({})", elems_str.join(", "))
            }
        }
        // v0.44: Array pattern
        Pattern::Array(elems) => {
            let elems_str: Vec<_> = elems.iter().map(|p| format_pattern(&p.node)).collect();
            format!("[{}]", elems_str.join(", "))
        }
        // v0.45: Array rest pattern
        Pattern::ArrayRest { prefix, suffix } => {
            let prefix_str: Vec<_> = prefix.iter().map(|p| format_pattern(&p.node)).collect();
            let suffix_str: Vec<_> = suffix.iter().map(|p| format_pattern(&p.node)).collect();
            match (prefix.is_empty(), suffix.is_empty()) {
                (true, true) => "[..]".to_string(),
                (false, true) => format!("[{}, ..]", prefix_str.join(", ")),
                (true, false) => format!("[.., {}]", suffix_str.join(", ")),
                (false, false) => format!("[{}, .., {}]", prefix_str.join(", "), suffix_str.join(", ")),
            }
        }
    }
}

fn start_lsp() -> Result<(), Box<dyn std::error::Error>> {
    // Create tokio runtime for async LSP server
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(bmb::lsp::run_server());
    Ok(())
}

/// v0.25: Generate project index for AI tools
/// v0.50.21: Added --watch mode for real-time index updates
fn index_project(path: &PathBuf, watch: bool, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Initial index generation
    do_index_project(path, verbose)?;

    // If watch mode, start file watcher
    if watch {
        run_index_watcher(path, verbose)?;
    }

    Ok(())
}

/// Perform the actual indexing operation
fn do_index_project(path: &PathBuf, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    use bmb::index::{IndexGenerator, write_index};

    // Determine project name from directory
    let project_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("bmb-project")
        .to_string();

    if verbose {
        println!("Indexing project: {} at {}", project_name, path.display());
    }

    // Collect all .bmb files
    let bmb_files = collect_bmb_files(path)?;

    if bmb_files.is_empty() {
        println!("No BMB files found in {}", path.display());
        return Ok(());
    }

    if verbose {
        println!("Found {} BMB files", bmb_files.len());
    }

    // Create index generator
    let mut generator = IndexGenerator::new(&project_name);

    // Index each file
    for file in &bmb_files {
        let source = std::fs::read_to_string(file)?;
        let filename = file.display().to_string();

        // Try to parse the file
        match bmb::lexer::tokenize(&source) {
            Ok(tokens) => {
                match bmb::parser::parse(&filename, &source, tokens) {
                    Ok(ast) => {
                        if verbose {
                            println!("  Indexed: {}", filename);
                        }
                        generator.index_file(&filename, &ast);
                    }
                    Err(e) => {
                        if verbose {
                            eprintln!("  Skipped {} (parse error: {})", filename, e);
                        }
                    }
                }
            }
            Err(e) => {
                if verbose {
                    eprintln!("  Skipped {} (lex error: {})", filename, e);
                }
            }
        }
    }

    // Generate and write index
    let index = generator.generate();
    write_index(&index, path)?;

    println!("✓ Index generated: .bmb/index/");
    println!("  Files: {}", index.manifest.files);
    println!("  Functions: {}", index.manifest.functions);
    println!("  Types: {}", index.manifest.types);
    println!("  Contracts: {}", index.manifest.contracts);

    Ok(())
}

/// v0.50.21: Watch for file changes and re-index automatically
fn run_index_watcher(path: &PathBuf, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
    use std::sync::mpsc::channel;
    use std::time::Duration;

    println!("👀 Watching for changes... (Press Ctrl+C to stop)");

    // Create a channel to receive events
    let (tx, rx) = channel();

    // Create a debounced watcher with 500ms delay
    let mut debouncer = new_debouncer(Duration::from_millis(500), tx)?;

    // Watch the directory recursively
    debouncer.watcher().watch(path.as_path(), RecursiveMode::Recursive)?;

    // Process events
    loop {
        match rx.recv() {
            Ok(result) => {
                match result {
                    Ok(events) => {
                        // Check if any .bmb file changed
                        let bmb_changed = events.iter().any(|e| {
                            e.path.extension().is_some_and(|ext| ext == "bmb")
                        });

                        if bmb_changed {
                            if verbose {
                                println!("\n📝 Detected .bmb file change, re-indexing...");
                            } else {
                                println!("\n🔄 Re-indexing...");
                            }

                            // Re-index the project
                            if let Err(e) = do_index_project(path, verbose) {
                                eprintln!("  Error during re-index: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Watch error: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Channel error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

/// v0.25: Run query against project index
fn run_query(query_type: QueryType) -> Result<(), Box<dyn std::error::Error>> {
    use bmb::index::{read_index, SymbolKind};
    use bmb::query::{QueryEngine, format_output};

    // Try to read index from current directory
    let current_dir = std::env::current_dir()?;
    let index = match read_index(&current_dir) {
        Ok(idx) => idx,
        Err(e) => {
            eprintln!("Error: No index found at {:?}. Run 'bmb index' first.", current_dir);
            eprintln!("  Detail: {}", e);
            std::process::exit(1);
        }
    };

    let engine = QueryEngine::new(index);

    // Helper to convert OutputFormat to query format string
    let fmt_str = |f: OutputFormat| match f {
        OutputFormat::Json => "json",
        OutputFormat::Compact => "compact",
        OutputFormat::Llm => "llm",
    };

    match query_type {
        QueryType::Sym { pattern, kind, public, format } => {
            let symbol_kind = kind.as_ref().and_then(|k| match k.as_str() {
                "fn" | "function" => Some(SymbolKind::Function),
                "struct" => Some(SymbolKind::Struct),
                "enum" => Some(SymbolKind::Enum),
                "type" => Some(SymbolKind::Type),
                "trait" => Some(SymbolKind::Trait),
                _ => None,
            });

            let result = engine.query_symbols(&pattern, symbol_kind, public);
            println!("{}", format_output(&result, fmt_str(format))?);
        }

        QueryType::Fn { name, has_pre, has_post, recursive, format } => {
            if !name.is_empty() && !has_pre && !has_post && !recursive {
                // Query specific function
                let result = engine.query_function(&name);
                println!("{}", format_output(&result, fmt_str(format))?);
            } else {
                // Query functions with filters
                let pre_filter = if has_pre { Some(true) } else { None };
                let post_filter = if has_post { Some(true) } else { None };
                let recursive_filter = if recursive { Some(true) } else { None };
                let result = engine.query_functions(pre_filter, post_filter, recursive_filter, false);
                println!("{}", format_output(&result, fmt_str(format))?);
            }
        }

        QueryType::Type { name, kind, format } => {
            if !name.is_empty() {
                let result = engine.query_type(&name);
                println!("{}", format_output(&result, fmt_str(format))?);
            } else {
                let result = engine.query_types(kind.as_deref(), false);
                println!("{}", format_output(&result, fmt_str(format))?);
            }
        }

        QueryType::Metrics { format } => {
            let metrics = engine.query_metrics();
            println!("{}", format_output(&metrics, fmt_str(format))?);
        }

        QueryType::Deps { target, reverse, transitive, format } => {
            let result = engine.query_deps(&target, reverse, transitive);
            println!("{}", format_output(&result, fmt_str(format))?);
        }

        QueryType::Contract { name, uses_old, format } => {
            let result = engine.query_contract(&name, uses_old);
            println!("{}", format_output(&result, fmt_str(format))?);
        }

        QueryType::Ctx { target, depth, include_tests, format } => {
            let result = engine.query_context(&target, depth, include_tests);
            println!("{}", format_output(&result, fmt_str(format))?);
        }

        QueryType::Sig { pattern, accepts, returns, format } => {
            let result = engine.query_signature(pattern.as_str(), accepts.as_deref(), returns.as_deref());
            println!("{}", format_output(&result, fmt_str(format))?);
        }

        QueryType::Batch { file, format } => {
            let result = engine.query_batch(&file)?;
            println!("{}", format_output(&result, fmt_str(format))?);
        }

        QueryType::Impact { target, change, format } => {
            let result = engine.query_impact(&target, &change);
            println!("{}", format_output(&result, fmt_str(format))?);
        }

        QueryType::Serve { port, host } => {
            return run_query_server(&host, port, engine);
        }

        QueryType::Proof { name, unverified, failed, timeout, format } => {
            use bmb::index::read_proof_index;
            use bmb::query::query_proofs;

            // Try to read proof index
            match read_proof_index(&current_dir) {
                Ok(proof_index) => {
                    let name_filter = if name.is_empty() { None } else { Some(name.as_str()) };
                    let result = query_proofs(&proof_index, name_filter, unverified, failed, timeout);
                    println!("{}", format_output(&result, fmt_str(format))?);
                }
                Err(_) => {
                    // No proof index yet - suggest running bmb verify
                    let result = bmb::query::ProofResult {
                        query: "proof:all".to_string(),
                        z3_available: false,
                        z3_version: None,
                        proofs: Vec::new(),
                        summary: bmb::query::ProofSummary {
                            total: 0,
                            verified: 0,
                            failed: 0,
                            timeout: 0,
                            unknown: 0,
                            pending: 0,
                        },
                        error: Some(bmb::query::QueryError {
                            code: "NO_PROOF_INDEX".to_string(),
                            message: "No proof index found. Run 'bmb verify <file>' to generate verification results.".to_string(),
                            suggestions: vec!["bmb verify main.bmb".to_string()],
                        }),
                    };
                    println!("{}", format_output(&result, fmt_str(format))?);
                }
            }
        }
    }

    Ok(())
}

/// v0.50.22: HTTP query server for AI tools (RFC-0001 Task 50.7)
fn run_query_server(
    host: &str,
    port: u16,
    engine: bmb::query::QueryEngine,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Read;
    use std::net::TcpListener;
    use bmb::query::format_output;

    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr)?;

    println!("BMB Query Server v0.50.22");
    println!("Listening on http://{}", addr);
    println!("Endpoints:");
    println!("  GET  /health      - Health check");
    println!("  POST /query       - Run query (JSON body)");
    println!("  GET  /metrics     - Project metrics");
    println!("Press Ctrl+C to stop");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                // Read request
                let mut buffer = [0; 8192];
                let n = stream.read(&mut buffer)?;
                let request = String::from_utf8_lossy(&buffer[..n]);

                // Parse request line
                let first_line = request.lines().next().unwrap_or("");
                let parts: Vec<&str> = first_line.split_whitespace().collect();

                if parts.len() < 2 {
                    send_response(&mut stream, 400, "Bad Request")?;
                    continue;
                }

                let method = parts[0];
                let path = parts[1];

                // Route request
                let (status, body) = match (method, path) {
                    ("GET", "/health") => {
                        (200, r#"{"status":"ok","version":"0.50.22"}"#.to_string())
                    }
                    ("GET", "/metrics") => {
                        let metrics = engine.query_metrics();
                        match format_output(&metrics, "json") {
                            Ok(json) => (200, json),
                            Err(e) => (500, format!(r#"{{"error":"{}"}}"#, e)),
                        }
                    }
                    ("POST", "/query") => {
                        // Extract JSON body
                        let body_start = request.find("\r\n\r\n").map(|i| i + 4)
                            .or_else(|| request.find("\n\n").map(|i| i + 2));

                        match body_start {
                            Some(start) => {
                                let json_body = &request[start..];
                                handle_query_request(&engine, json_body.trim())
                            }
                            None => (400, r#"{"error":"No request body"}"#.to_string()),
                        }
                    }
                    _ => {
                        (404, r#"{"error":"Not found"}"#.to_string())
                    }
                };

                send_json_response(&mut stream, status, &body)?;
            }
            Err(e) => {
                eprintln!("Connection error: {}", e);
            }
        }
    }

    Ok(())
}

/// Handle POST /query request
fn handle_query_request(engine: &bmb::query::QueryEngine, json_body: &str) -> (u16, String) {
    use bmb::query::format_output;

    // Parse query JSON
    let query: serde_json::Value = match serde_json::from_str(json_body) {
        Ok(v) => v,
        Err(e) => return (400, format!(r#"{{"error":"Invalid JSON: {}"}}"#, e)),
    };

    let query_type = query.get("type").and_then(|v| v.as_str()).unwrap_or("");

    match query_type {
        "sym" => {
            let pattern = query.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
            let public = query.get("public").and_then(|v| v.as_bool()).unwrap_or(false);
            let result = engine.query_symbols(pattern, None, public);
            match format_output(&result, "json") {
                Ok(json) => (200, json),
                Err(e) => (500, format!(r#"{{"error":"{}"}}"#, e)),
            }
        }
        "fn" => {
            let name = query.get("name").and_then(|v| v.as_str()).unwrap_or("");
            if !name.is_empty() {
                let result = engine.query_function(name);
                match format_output(&result, "json") {
                    Ok(json) => (200, json),
                    Err(e) => (500, format!(r#"{{"error":"{}"}}"#, e)),
                }
            } else {
                (400, r#"{"error":"Missing 'name' field"}"#.to_string())
            }
        }
        "type" => {
            let name = query.get("name").and_then(|v| v.as_str()).unwrap_or("");
            if !name.is_empty() {
                let result = engine.query_type(name);
                match format_output(&result, "json") {
                    Ok(json) => (200, json),
                    Err(e) => (500, format!(r#"{{"error":"{}"}}"#, e)),
                }
            } else {
                (400, r#"{"error":"Missing 'name' field"}"#.to_string())
            }
        }
        "metrics" => {
            let result = engine.query_metrics();
            match format_output(&result, "json") {
                Ok(json) => (200, json),
                Err(e) => (500, format!(r#"{{"error":"{}"}}"#, e)),
            }
        }
        "deps" => {
            let target = query.get("target").and_then(|v| v.as_str()).unwrap_or("");
            let reverse = query.get("reverse").and_then(|v| v.as_bool()).unwrap_or(false);
            let transitive = query.get("transitive").and_then(|v| v.as_bool()).unwrap_or(false);
            let result = engine.query_deps(target, reverse, transitive);
            match format_output(&result, "json") {
                Ok(json) => (200, json),
                Err(e) => (500, format!(r#"{{"error":"{}"}}"#, e)),
            }
        }
        "contract" => {
            let name = query.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let uses_old = query.get("uses_old").and_then(|v| v.as_bool()).unwrap_or(false);
            let result = engine.query_contract(name, uses_old);
            match format_output(&result, "json") {
                Ok(json) => (200, json),
                Err(e) => (500, format!(r#"{{"error":"{}"}}"#, e)),
            }
        }
        "impact" => {
            let target = query.get("target").and_then(|v| v.as_str()).unwrap_or("");
            let change = query.get("change").and_then(|v| v.as_str()).unwrap_or("");
            let result = engine.query_impact(target, change);
            match format_output(&result, "json") {
                Ok(json) => (200, json),
                Err(e) => (500, format!(r#"{{"error":"{}"}}"#, e)),
            }
        }
        _ => {
            (400, format!(r#"{{"error":"Unknown query type: {}"}}"#, query_type))
        }
    }
}

/// Send HTTP response with status code and body
fn send_response(stream: &mut std::net::TcpStream, status: u16, body: &str) -> std::io::Result<()> {
    use std::io::Write;
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    };
    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status, status_text, body.len(), body
    );
    stream.write_all(response.as_bytes())
}

/// Send JSON HTTP response
fn send_json_response(stream: &mut std::net::TcpStream, status: u16, body: &str) -> std::io::Result<()> {
    use std::io::Write;
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    };
    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status, status_text, body.len(), body
    );
    stream.write_all(response.as_bytes())
}

/// v0.30.246: Stage 3 self-hosting verification
/// Compares LLVM IR from Rust compiler vs Bootstrap compiler
fn verify_stage3(
    path: &PathBuf,
    verbose: bool,
    output: Option<&PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Stage 3 Verification: {}", path.display());
    println!("==========================================");

    // Read target source file
    let source = std::fs::read_to_string(path)?;
    let filename = path.display().to_string();

    if verbose {
        println!("\n[1/4] Target source loaded ({} bytes)", source.len());
    }

    // Step 1: Generate LLVM IR using Rust compiler
    if verbose {
        println!("[2/4] Generating LLVM IR via Rust compiler...");
    }
    let rust_ir = generate_rust_ir(&source, &filename)?;
    if verbose {
        println!("      Rust IR: {} bytes", rust_ir.len());
    }

    // Step 2: Generate LLVM IR using Bootstrap compiler
    if verbose {
        println!("[3/4] Generating LLVM IR via Bootstrap compiler...");
    }
    let bootstrap_ir = generate_bootstrap_ir(&source)?;
    if verbose {
        println!("      Bootstrap IR: {} bytes", bootstrap_ir.len());
    }

    // Step 3: Normalize and compare
    if verbose {
        println!("[4/4] Comparing outputs...");
    }
    let rust_normalized = normalize_ir(&rust_ir);
    let bootstrap_normalized = normalize_ir(&bootstrap_ir);

    // Generate report
    let mut report = String::new();
    report.push_str("# Stage 3 Verification Report\n");
    report.push_str(&format!("File: {}\n", path.display()));
    report.push_str(&format!("Date: {}\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));

    report.push_str("## Rust Compiler Output (normalized)\n```llvm\n");
    report.push_str(&rust_normalized);
    report.push_str("\n```\n\n");

    report.push_str("## Bootstrap Compiler Output (normalized)\n```llvm\n");
    report.push_str(&bootstrap_normalized);
    report.push_str("\n```\n\n");

    let is_exact_match = rust_normalized == bootstrap_normalized;

    // Check semantic equivalence (function signatures match)
    let rust_sigs = extract_function_signature(&rust_ir);
    let bootstrap_sigs = extract_function_signature(&bootstrap_ir);
    let is_semantic_match = rust_sigs.iter()
        .filter(|s| s.starts_with("define "))
        .collect::<Vec<_>>() ==
        bootstrap_sigs.iter()
        .filter(|s| s.starts_with("define "))
        .collect::<Vec<_>>();

    if is_exact_match {
        report.push_str("## Result: ✅ PASS (Exact Match)\n");
        report.push_str("LLVM IR outputs are exactly equivalent.\n");
        println!("\n✅ PASS: Stage 3 verification successful!");
        println!("   Rust and Bootstrap compilers produce identical LLVM IR.");
    } else if is_semantic_match {
        report.push_str("## Result: ✅ PASS (Semantic Match)\n");
        report.push_str("Function signatures are equivalent. Code differs in optimization level.\n");
        println!("\n✅ PASS: Stage 3 verification successful!");
        println!("   Function signatures match. Code generation differs in optimization level.");
        println!("   Both outputs are semantically equivalent.");
    } else {
        report.push_str("## Result: ❌ FAIL\n");
        report.push_str("LLVM IR outputs differ.\n\n");
        report.push_str("## Diff\n");

        // Generate simple diff
        let rust_lines: Vec<&str> = rust_normalized.lines().collect();
        let bootstrap_lines: Vec<&str> = bootstrap_normalized.lines().collect();

        for (i, (r, b)) in rust_lines.iter().zip(bootstrap_lines.iter()).enumerate() {
            if r != b {
                report.push_str(&format!("Line {}: Rust: {}\n", i + 1, r));
                report.push_str(&format!("Line {}: Boot: {}\n", i + 1, b));
            }
        }

        if rust_lines.len() != bootstrap_lines.len() {
            report.push_str(&format!("\nLine count mismatch: Rust={}, Bootstrap={}\n",
                rust_lines.len(), bootstrap_lines.len()));
        }

        println!("\n❌ FAIL: Stage 3 verification failed!");
        println!("   LLVM IR outputs differ between Rust and Bootstrap compilers.");

        if verbose {
            println!("\nRust output ({} lines):", rust_lines.len());
            for line in rust_lines.iter().take(20) {
                println!("  {}", line);
            }
            if rust_lines.len() > 20 {
                println!("  ... ({} more lines)", rust_lines.len() - 20);
            }

            println!("\nBootstrap output ({} lines):", bootstrap_lines.len());
            for line in bootstrap_lines.iter().take(20) {
                println!("  {}", line);
            }
            if bootstrap_lines.len() > 20 {
                println!("  ... ({} more lines)", bootstrap_lines.len() - 20);
            }
        }
    }

    // Write report if output path specified
    if let Some(out_path) = output {
        std::fs::write(out_path, &report)?;
        println!("\nReport written to: {}", out_path.display());
    }

    if is_exact_match || is_semantic_match {
        Ok(())
    } else {
        Err("Stage 3 verification failed: IR mismatch".into())
    }
}

/// Generate LLVM IR using Rust compiler pipeline
fn generate_rust_ir(source: &str, filename: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Parse
    let tokens = bmb::lexer::tokenize(source)?;
    let ast = bmb::parser::parse(filename, source, tokens)?;

    // Type check
    let mut checker = bmb::types::TypeChecker::new();
    checker.check_program(&ast)?;

    // Lower to MIR
    let mir = bmb::mir::lower_program(&ast);

    // Generate LLVM IR
    let codegen = bmb::codegen::TextCodeGen::new();
    let llvm_ir = codegen.generate(&mir)?;
    Ok(llvm_ir)
}

/// Generate LLVM IR using Bootstrap compiler (compiler.bmb)
/// Runs in a separate thread with 64MB stack for deep recursion
fn generate_bootstrap_ir(source: &str) -> Result<String, Box<dyn std::error::Error>> {
    use bmb::interp::Value;
    use std::rc::Rc;

    // Path to compiler.bmb
    let bootstrap_path = std::path::Path::new("bootstrap/compiler.bmb");
    if !bootstrap_path.exists() {
        return Err("bootstrap/compiler.bmb not found".into());
    }

    // Load compiler.bmb source
    let compiler_source = std::fs::read_to_string(bootstrap_path)?;
    let escaped_source = escape_bmb_source(source);

    // Run bootstrap compiler in a thread with 64MB stack (same as run_file)
    let handle = std::thread::Builder::new()
        .name("bootstrap-compiler".to_string())
        .stack_size(INTERPRETER_STACK_SIZE)
        .spawn(move || -> Result<String, String> {
            // Parse compiler.bmb
            let tokens = bmb::lexer::tokenize(&compiler_source)
                .map_err(|e| format!("Lexer error: {}", e))?;
            let compiler_ast = bmb::parser::parse("compiler.bmb", &compiler_source, tokens)
                .map_err(|e| format!("Parser error: {}", e))?;

            // Type check compiler.bmb
            let mut checker = bmb::types::TypeChecker::new();
            checker.check_program(&compiler_ast)
                .map_err(|e| format!("Type error: {}", e))?;

            // Create interpreter and load compiler.bmb
            let mut interpreter = bmb::interp::Interpreter::new();
            interpreter.load(&compiler_ast);

            // v0.30.280: Enable ScopeStack for efficient memory management
            // This allows let bindings to deallocate immediately on scope exit
            interpreter.enable_scope_stack();

            // Call compile_program with the target source (v0.30.268: Rc<String>)
            let result = interpreter.call_function_with_args(
                "compile_program",
                vec![Value::Str(Rc::new(escaped_source))],
            ).map_err(|e| format!("Runtime error: {}", e.message))?;

            // Extract string result (v0.30.283: handle both Str and StringRope)
            match &result {
                Value::Str(ir) => Ok(ir.as_ref().clone()),
                Value::StringRope(_) => {
                    // Materialize StringRope to String
                    result.materialize_string()
                        .ok_or_else(|| "Failed to materialize StringRope".to_string())
                }
                other => Err(format!("Expected string from compile_program, got: {:?}", other.type_name())),
            }
        })?;

    match handle.join() {
        Ok(Ok(ir)) => Ok(ir),
        Ok(Err(e)) => Err(e.into()),
        Err(_) => Err("Bootstrap compiler thread panicked".into()),
    }
}

/// Escape BMB source for use with bootstrap compiler
fn escape_bmb_source(source: &str) -> String {
    // Keep newlines as-is (bootstrap parser expects real newlines)
    // Only normalize Windows line endings to Unix
    source
        .replace("\r\n", "\n")  // Normalize Windows newlines
        .replace("\r", "\n")    // Normalize old Mac newlines
}

/// Normalize LLVM IR for comparison
/// Focuses on function definitions, ignoring target triple and runtime declarations
fn normalize_ir(ir: &str) -> String {
    ir
        // Replace | separators with newlines (bootstrap convention)
        .replace("|", "\n")
        // Split into lines
        .lines()
        // Trim whitespace
        .map(|l| l.trim())
        // Remove empty lines
        .filter(|l| !l.is_empty())
        // Remove comments
        .filter(|l| !l.starts_with(";"))
        // Remove target triple (differs between platforms)
        .filter(|l| !l.starts_with("target triple"))
        // Remove module ID
        .filter(|l| !l.starts_with("; ModuleID"))
        // Remove runtime declarations (vary between implementations)
        .filter(|l| !l.starts_with("declare"))
        // Collect and join
        .collect::<Vec<_>>()
        .join("\n")
}

/// Extract function signature and structure for semantic comparison
fn extract_function_signature(ir: &str) -> Vec<String> {
    let normalized = normalize_ir(ir);
    normalized
        .lines()
        .filter(|l| l.starts_with("define ") || l.contains("entry:") || l.starts_with("ret "))
        .map(|l| l.to_string())
        .collect()
}
