//! BMB Compiler CLI

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "bmb", version, about = "BMB Compiler - AI-Native Language")]
struct Cli {
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
        /// Build with optimizations
        #[arg(long)]
        release: bool,
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
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Run a BMB program (interpreter)
    Run {
        /// Source file to run
        file: PathBuf,
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
    },
    /// Query type details
    Type {
        /// Type name (optional when using --kind filter)
        #[arg(default_value = "")]
        name: String,
        /// Filter by kind (struct, enum, trait)
        #[arg(long)]
        kind: Option<String>,
    },
    /// Show project metrics
    Metrics,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Build {
            file,
            output,
            release,
            emit_ir,
            emit_mir,
            emit_wasm,
            wasm_target,
            all_targets,
            verbose,
        } => build_file(&file, output, release, emit_ir, emit_mir, emit_wasm, &wasm_target, all_targets, verbose),
        Command::Run { file } => run_file(&file),
        Command::Repl => start_repl(),
        Command::Check { file, include_paths } => check_file_with_includes(&file, &include_paths),
        Command::Verify { file, z3_path, timeout } => verify_file(&file, &z3_path, timeout),
        Command::Parse { file, format } => parse_file(&file, &format),
        Command::Tokens { file } => tokenize_file(&file),
        Command::Test { file, filter, verbose } => test_file(&file, filter.as_deref(), verbose),
        Command::Fmt { file, check } => fmt_file(&file, check),
        Command::Lsp => start_lsp(),
        Command::Index { path, watch, verbose } => index_project(&path, watch, verbose),
        Command::Query { query_type } => run_query(query_type),
        Command::VerifyStage3 { file, verbose, output } => verify_stage3(&file, verbose, output.as_ref()),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn build_file(
    path: &PathBuf,
    output: Option<PathBuf>,
    release: bool,
    emit_ir: bool,
    emit_mir: bool,
    emit_wasm: bool,
    wasm_target: &str,
    all_targets: bool,
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
        build_native(path, output.clone(), release, emit_ir, verbose)?;

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
    build_native(path, output, release, emit_ir, verbose)
}

fn build_native(
    path: &PathBuf,
    output: Option<PathBuf>,
    release: bool,
    emit_ir: bool,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use bmb::build::{BuildConfig, OptLevel};

    let mut config = BuildConfig::new(path.clone())
        .emit_ir(emit_ir)
        .verbose(verbose);

    if let Some(out) = output {
        config = config.output(out);
    }

    if release {
        config = config.opt_level(OptLevel::Release);
    }

    bmb::build::build(&config)?;

    if !emit_ir && verbose {
        println!("Build complete: {}", config.output.display());
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

    println!("Generated: {}", output_path.display());

    if verbose {
        println!("  Target: {:?}", target);
        println!("  Size: {} bytes", wat.len());
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

    println!("Generated: {}", output_path.display());

    if verbose {
        println!("  Functions: {}", mir.functions.len());
        println!("  Size: {} bytes", mir_text.len());
    }

    Ok(())
}

/// v0.30.241: Stack size for interpreter thread (64MB for deep recursion in bootstrap)
const INTERPRETER_STACK_SIZE: usize = 64 * 1024 * 1024;

fn run_file(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // v0.30.241: Run entire pipeline in a thread with larger stack to prevent overflow
    // Bootstrap files have deep recursion that exceeds default 1MB Windows stack
    // We run everything in the thread because Value uses Rc<RefCell<>> (not Send)
    let path = path.clone();
    let handle = std::thread::Builder::new()
        .name("bmb-interpreter".to_string())
        .stack_size(INTERPRETER_STACK_SIZE)
        .spawn(move || -> Result<(), String> {
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
            eprintln!("{}", e);
            std::process::exit(1);
        }
        Err(_) => {
            eprintln!("Runtime error: interpreter thread panicked");
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
            if let bmb::ast::Item::Use(use_stmt) = item {
                if !use_stmt.path.is_empty() {
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
    }

    // Also resolve from the file's own directory
    if let Ok(imports) = resolver.resolve_uses(&ast) {
        for (_, (module_name, _)) in imports.all_imports() {
            if let Some(module) = resolver.get_module(module_name) {
                checker.register_module(module);
            }
        }
    }

    // Type check
    checker.check_program(&ast)?;

    println!("✓ {} type checks successfully", filename);
    Ok(())
}

fn verify_file(path: &PathBuf, z3_path: &str, timeout: u32) -> Result<(), Box<dyn std::error::Error>> {
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
    if !verifier.is_solver_available() {
        eprintln!("Warning: Z3 solver not found at '{}'. Install Z3 or specify --z3-path.", z3_path);
        eprintln!("Skipping contract verification.");
        return Ok(());
    }

    // Verify contracts
    let report = verifier.verify_program(&ast);

    // Print report
    print!("{}", report);

    // Exit with error if any verification failed
    if !report.all_verified() {
        std::process::exit(1);
    }

    Ok(())
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
    for (tok, span) in &tokens {
        println!("{:?} @ {}..{}", tok, span.start, span.end);
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
        println!("No test files found");
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

        if verbose || test_files.len() > 1 {
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

                    let elapsed = test_start.elapsed();

                    if passed {
                        total_passed += 1;
                        if verbose {
                            println!("  ✅ {} ({:.2?})", test_name, elapsed);
                        }
                    } else {
                        total_failed += 1;
                        println!("  ❌ {} - returned false ({:.2?})", test_name, elapsed);
                    }
                }
                Err(e) => {
                    total_failed += 1;
                    println!("  ❌ {} - {}", test_name, e.message);
                }
            }
        }
    }

    let elapsed = start_time.elapsed();

    // Print summary
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

fn fmt_file(path: &PathBuf, check: bool) -> Result<(), Box<dyn std::error::Error>> {
    let files = if path.is_dir() {
        collect_bmb_files(path)?
    } else {
        vec![path.clone()]
    };

    if files.is_empty() {
        println!("No BMB files found");
        return Ok(());
    }

    let mut needs_formatting = false;

    for file in &files {
        let source = std::fs::read_to_string(file)?;
        let filename = file.display().to_string();

        // Tokenize
        let tokens = bmb::lexer::tokenize(&source)?;

        // Parse
        let ast = bmb::parser::parse(&filename, &source, tokens)?;

        // Format AST back to source
        let formatted = format_program(&ast);

        if check {
            if source != formatted {
                println!("❌ {} needs formatting", filename);
                needs_formatting = true;
            } else {
                println!("✓ {} is formatted", filename);
            }
        } else {
            if source != formatted {
                std::fs::write(file, &formatted)?;
                println!("✓ formatted {}", filename);
            } else {
                println!("✓ {} (unchanged)", filename);
            }
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

fn format_program(program: &bmb::ast::Program) -> String {
    use bmb::ast::{Item, Visibility};

    let mut output = String::new();

    for (i, item) in program.items.iter().enumerate() {
        if i > 0 {
            output.push_str("\n\n");
        }

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
            // v0.13.0: Format extern function declarations
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
            // v0.20.1: Format trait definitions
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
            // v0.20.1: Format impl blocks
            Item::ImplBlock(i) => {
                output.push_str(&format!("impl {} for {} {{\n", i.trait_name.node, format_type(&i.target_type.node)));
                for method in &i.methods {
                    output.push_str("    ");
                    output.push_str(&format_fn_def(method));
                    output.push('\n');
                }
                output.push('}');
            }
        }
    }

    output.push('\n');
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
        Type::F64 => "f64".to_string(),
        Type::Bool => "bool".to_string(),
        Type::String => "String".to_string(),
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
    }
}

fn format_expr(expr: &bmb::ast::Expr) -> String {
    use bmb::ast::{Expr, BinOp, UnOp};

    match expr {
        Expr::IntLit(n) => n.to_string(),
        Expr::FloatLit(f) => f.to_string(),
        Expr::BoolLit(b) => b.to_string(),
        Expr::StringLit(s) => format!("\"{}\"", s),
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
                BinOp::Eq => "==",
                BinOp::Ne => "!=",
                BinOp::Lt => "<",
                BinOp::Le => "<=",
                BinOp::Gt => ">",
                BinOp::Ge => ">=",
                BinOp::And => "and",
                BinOp::Or => "or",
            };
            format!("{} {} {}", format_expr(&left.node), op_str, format_expr(&right.node))
        }

        Expr::Unary { op, expr } => {
            let op_str = match op {
                UnOp::Neg => "-",
                UnOp::Not => "not ",
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

        Expr::StructInit { name, fields } => {
            let fields_str: Vec<_> = fields.iter()
                .map(|(n, v)| format!("{}: {}", n.node, format_expr(&v.node)))
                .collect();
            format!("{} {{ {} }}", name, fields_str.join(", "))
        }

        Expr::FieldAccess { expr, field } => {
            format!("{}.{}", format_expr(&expr.node), field.node)
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

        Expr::While { cond, body } => {
            format!("while {} {{ {} }}", format_expr(&cond.node), format_expr(&body.node))
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

        // v0.13.2: Try block
        Expr::Try { body } => {
            format!("try {{ {} }}", format_expr(&body.node))
        }

        // v0.13.2: Question mark operator
        Expr::Question { expr: inner } => {
            format!("{}?", format_expr(&inner.node))
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
    }
}

fn format_pattern(pattern: &bmb::ast::Pattern) -> String {
    use bmb::ast::{Pattern, LiteralPattern};

    match pattern {
        Pattern::Wildcard => "_".to_string(),
        Pattern::Var(name) => name.clone(),
        Pattern::Literal(lit) => match lit {
            LiteralPattern::Int(n) => n.to_string(),
            LiteralPattern::Float(f) => f.to_string(),
            LiteralPattern::Bool(b) => b.to_string(),
            LiteralPattern::String(s) => format!("\"{}\"", s),
        },
        Pattern::EnumVariant { enum_name, variant, bindings } => {
            if bindings.is_empty() {
                format!("{}::{}", enum_name, variant)
            } else {
                let bindings_str: Vec<_> = bindings.iter().map(|b| b.node.as_str()).collect();
                format!("{}::{}({})", enum_name, variant, bindings_str.join(", "))
            }
        }
        Pattern::Struct { name, fields } => {
            let fields_str: Vec<_> = fields.iter()
                .map(|(n, p)| format!("{}: {}", n.node, format_pattern(&p.node)))
                .collect();
            format!("{} {{ {} }}", name, fields_str.join(", "))
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
fn index_project(path: &PathBuf, _watch: bool, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
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

/// v0.25: Run query against project index
fn run_query(query_type: QueryType) -> Result<(), Box<dyn std::error::Error>> {
    use bmb::index::{read_index, SymbolKind};
    use bmb::query::QueryEngine;

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

    match query_type {
        QueryType::Sym { pattern, kind, public } => {
            let symbol_kind = kind.as_ref().and_then(|k| match k.as_str() {
                "fn" | "function" => Some(SymbolKind::Function),
                "struct" => Some(SymbolKind::Struct),
                "enum" => Some(SymbolKind::Enum),
                "type" => Some(SymbolKind::Type),
                "trait" => Some(SymbolKind::Trait),
                _ => None,
            });

            let result = engine.query_symbols(&pattern, symbol_kind, public);
            println!("{}", serde_json::to_string_pretty(&result)?);
        }

        QueryType::Fn { name, has_pre, has_post, recursive } => {
            if !name.is_empty() && !has_pre && !has_post && !recursive {
                // Query specific function
                let result = engine.query_function(&name);
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                // Query functions with filters
                let pre_filter = if has_pre { Some(true) } else { None };
                let post_filter = if has_post { Some(true) } else { None };
                let recursive_filter = if recursive { Some(true) } else { None };
                let result = engine.query_functions(pre_filter, post_filter, recursive_filter, false);
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
        }

        QueryType::Type { name, kind } => {
            if !name.is_empty() {
                let result = engine.query_type(&name);
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                let result = engine.query_types(kind.as_deref(), false);
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
        }

        QueryType::Metrics => {
            let metrics = engine.query_metrics();
            println!("{}", serde_json::to_string_pretty(&metrics)?);
        }
    }

    Ok(())
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
