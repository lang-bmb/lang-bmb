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
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Build {
            file,
            output,
            release,
            emit_ir,
            emit_wasm,
            wasm_target,
            all_targets,
            verbose,
        } => build_file(&file, output, release, emit_ir, emit_wasm, &wasm_target, all_targets, verbose),
        Command::Run { file } => run_file(&file),
        Command::Repl => start_repl(),
        Command::Check { file } => check_file(&file),
        Command::Verify { file, z3_path, timeout } => verify_file(&file, &z3_path, timeout),
        Command::Parse { file } => parse_file(&file),
        Command::Tokens { file } => tokenize_file(&file),
        Command::Test { file, filter, verbose } => test_file(&file, filter.as_deref(), verbose),
        Command::Fmt { file, check } => fmt_file(&file, check),
        Command::Lsp => start_lsp(),
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
    emit_wasm: bool,
    wasm_target: &str,
    all_targets: bool,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
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

fn run_file(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string(path)?;
    let filename = path.display().to_string();

    // Tokenize
    let tokens = bmb::lexer::tokenize(&source)?;

    // Parse
    let ast = bmb::parser::parse(&filename, &source, tokens)?;

    // Type check first
    let mut checker = bmb::types::TypeChecker::new();
    checker.check_program(&ast)?;

    // Run with interpreter
    let mut interpreter = bmb::interp::Interpreter::new();
    interpreter.load(&ast);

    match interpreter.run(&ast) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Runtime error: {}", e.message);
            std::process::exit(1);
        }
    }
}

fn start_repl() -> Result<(), Box<dyn std::error::Error>> {
    let mut repl = bmb::repl::Repl::new()?;
    repl.run()?;
    Ok(())
}

fn check_file(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string(path)?;
    let filename = path.display().to_string();

    // Tokenize
    let tokens = bmb::lexer::tokenize(&source)?;

    // Parse
    let ast = bmb::parser::parse(&filename, &source, tokens)?;

    // Type check
    let mut checker = bmb::types::TypeChecker::new();
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

fn parse_file(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string(path)?;
    let filename = path.display().to_string();

    let tokens = bmb::lexer::tokenize(&source)?;
    let ast = bmb::parser::parse(&filename, &source, tokens)?;

    println!("{}", serde_json::to_string_pretty(&ast)?);
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
                filter.map_or(true, |f| name.contains(f))
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
        } else if path.extension().map_or(false, |e| e == "bmb") {
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
                output.push_str("}");
            }
            Item::EnumDef(e) => {
                if e.visibility == Visibility::Public {
                    output.push_str("pub ");
                }
                output.push_str(&format!("enum {} {{\n", e.name.node));
                for variant in &e.variants {
                    output.push_str(&format!("    {},\n", variant.name.node));
                }
                output.push_str("}");
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
