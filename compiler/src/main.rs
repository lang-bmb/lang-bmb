use std::process::ExitCode;

const USAGE: &str = "usage:
  bmb check <file.bmb>...   check sources; diagnostics as JSONL on stdout
  bmb verify [--emit-smt DIR] <file.bmb>...
                            check and prove; diagnostics and per-definition status as JSONL
  bmb emit-llvm <file.bmb>... -o OUT.ll
                            LLVM IR; unproved obligations keep runtime checks
  bmb build <file.bmb>... -o OUT
                            check, prove (all obligations) and build an executable
  bmb fmt [--check] <file.bmb>...
                            print the normal form (ADR-0019); --check fails if a file differs
  bmb hash <file.bmb>...    definition hashes (body, iface) as JSONL
  bmb map [L0|L1|L2|all] <file.bmb>...
                            standard maps as JSONL (ADR-0020)
  bmb query <units|surface|edges|slice|find|contracts|status> [args] -- <file.bmb>...
                            read queries as JSONL (ADR-0020)
  bmb import DIR <file.bmb>...
                            create or replace workspace DIR from files, in normal form
  bmb apply DIR OPS.json    commit a transaction (JSON array of unit/put/remove/remove_unit ops)
                            to workspace DIR; rejected as a whole on any structural or type error
  bmb mcp DIR               MCP server (stdio) for workspace DIR: query, map, verify, build, tx_*
  bmb codes [--md]          list diagnostic codes as JSONL (or the Markdown reference page)

Any <file.bmb> argument may also be a workspace directory (its src/*.bmb).
  bmb doctor                check the pinned toolchain (z3, LLVM) as JSONL";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("check") if args.len() > 1 => {
            let files = match read_files(&args[1..]) {
                Ok(f) => f,
                Err(code) => return code,
            };
            let out = bmb::check_sources(&files);
            print!("{}", out.diagnostics);
            if out.ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Some("doctor") => {
            let mut ok = true;
            for (tool, want, res) in [
                (
                    "z3",
                    bmb::toolchain::z3_version(),
                    bmb::toolchain::check_z3(),
                ),
                (
                    "llvm",
                    bmb::toolchain::llvm_major(),
                    bmb::toolchain::check_llvm(),
                ),
            ] {
                let (status, detail) = match res {
                    Ok(v) => ("ok", v),
                    Err(e) => {
                        ok = false;
                        ("error", e)
                    }
                };
                println!(
                    "{{\"tool\":{},\"required\":{},\"status\":{},\"detail\":{}}}",
                    bmb::diag::json_str(tool),
                    bmb::diag::json_str(want),
                    bmb::diag::json_str(status),
                    bmb::diag::json_str(&detail)
                );
            }
            if ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Some("verify") if args.len() > 1 => {
            let mut rest = &args[1..];
            let mut emit = None;
            if rest.first().map(String::as_str) == Some("--emit-smt") && rest.len() > 2 {
                emit = Some(rest[1].clone());
                rest = &rest[2..];
            }
            let files = match read_files(rest) {
                Ok(f) => f,
                Err(code) => return code,
            };
            let out = bmb::verify_sources(&files);
            if let Some(dir) = emit {
                if let Err(e) = std::fs::create_dir_all(&dir) {
                    eprintln!("bmb: cannot create {dir}: {e}");
                    return ExitCode::from(2);
                }
                for (def, script) in &out.scripts {
                    let file = format!("{dir}/{}.smt2", def.replace("::", "."));
                    if let Err(e) = std::fs::write(&file, script) {
                        eprintln!("bmb: cannot write {file}: {e}");
                        return ExitCode::from(2);
                    }
                }
            }
            print!("{}", out.output);
            if out.ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Some(cmd @ ("emit-llvm" | "build")) => {
            let Some(pos) = args.iter().position(|a| a == "-o") else {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            };
            let Some(out_path) = args.get(pos + 1).cloned() else {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            };
            let inputs: Vec<String> = args[1..pos]
                .iter()
                .chain(args[pos + 2..].iter())
                .cloned()
                .collect();
            let files = match read_files(&inputs) {
                Ok(f) => f,
                Err(code) => return code,
            };
            let out = bmb::emit_llvm_sources(&files);
            print!("{}", out.verify.output);
            let Some(ir) = out.ir else {
                return ExitCode::from(1);
            };
            if cmd == "emit-llvm" {
                if let Err(e) = std::fs::write(&out_path, ir) {
                    eprintln!("bmb: cannot write {out_path}: {e}");
                    return ExitCode::from(2);
                }
                return if out.verify.ok {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::from(1)
                };
            }
            // build: no binary without a complete proof.
            if !out.verify.ok || out.checks > 0 {
                eprintln!("bmb: build refused: the program is not fully proved");
                return ExitCode::from(1);
            }
            if !out.has_main {
                eprintln!("bmb: build needs a `main`");
                return ExitCode::from(1);
            }
            match bmb::toolchain::build_executable(&ir, &out_path) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("bmb: {e}");
                    ExitCode::from(1)
                }
            }
        }
        Some("fmt") if args.len() > 1 => {
            let check = args[1] == "--check";
            let paths = if check { &args[2..] } else { &args[1..] };
            let files = match read_files(paths) {
                Ok(f) => f,
                Err(code) => return code,
            };
            match bmb::format_sources(&files) {
                Err(diags) => {
                    print!("{diags}");
                    ExitCode::from(1)
                }
                Ok(units) => {
                    let text = units.values().cloned().collect::<Vec<_>>().join("\n");
                    if !check {
                        print!("{text}");
                        return ExitCode::SUCCESS;
                    }
                    let given = files
                        .iter()
                        .map(|(_, t)| t.as_str())
                        .collect::<Vec<_>>()
                        .join("\n");
                    if given == text {
                        ExitCode::SUCCESS
                    } else {
                        eprintln!("bmb: not in normal form");
                        ExitCode::from(1)
                    }
                }
            }
        }
        Some("hash") if args.len() > 1 => {
            let files = match read_files(&args[1..]) {
                Ok(f) => f,
                Err(code) => return code,
            };
            match bmb::hash_sources(&files) {
                Ok(out) => {
                    print!("{out}");
                    ExitCode::SUCCESS
                }
                Err(diags) => {
                    print!("{diags}");
                    ExitCode::from(1)
                }
            }
        }
        Some("map") if args.len() > 1 => {
            let (level, paths) = if matches!(args[1].as_str(), "L0" | "L1" | "L2" | "all") {
                (args[1].clone(), &args[2..])
            } else {
                ("all".to_string(), &args[1..])
            };
            let files = match read_files(paths) {
                Ok(f) => f,
                Err(code) => return code,
            };
            respond(bmb::map_request(&files, &["map".into(), level]))
        }
        Some("query") if args.len() > 2 => {
            let Some(sep) = args.iter().position(|a| a == "--") else {
                eprintln!("bmb: put `--` between the query and the files");
                return ExitCode::from(2);
            };
            let files = match read_files(&args[sep + 1..]) {
                Ok(f) => f,
                Err(code) => return code,
            };
            let mut req = vec!["query".to_string()];
            req.extend(args[1..sep].iter().cloned());
            respond(bmb::map_request(&files, &req))
        }
        Some("import") if args.len() > 2 => {
            let files = match read_files(&args[2..]) {
                Ok(f) => f,
                Err(code) => return code,
            };
            match bmb::format_sources(&files) {
                Err(diags) => {
                    print!("{diags}");
                    ExitCode::from(1)
                }
                Ok(units) => {
                    match bmb::workspace::write_units(std::path::Path::new(&args[1]), &units) {
                        Ok(()) => ExitCode::SUCCESS,
                        Err(e) => {
                            eprintln!("bmb: {e}");
                            ExitCode::from(2)
                        }
                    }
                }
            }
        }
        Some("apply") if args.len() == 3 => {
            let text = if args[2] == "-" {
                let mut s = String::new();
                if std::io::Read::read_to_string(&mut std::io::stdin(), &mut s).is_err() {
                    eprintln!("bmb: cannot read stdin");
                    return ExitCode::from(2);
                }
                s
            } else {
                match std::fs::read_to_string(&args[2]) {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("bmb: cannot read {}: {e}", args[2]);
                        return ExitCode::from(2);
                    }
                }
            };
            let ops = match bmb::workspace::parse_ops(&text) {
                Ok(o) => o,
                Err(e) => {
                    eprintln!("bmb: {e}");
                    return ExitCode::from(2);
                }
            };
            match bmb::workspace::commit(std::path::Path::new(&args[1]), &ops) {
                Ok(c) => {
                    print!("{}", c.diagnostics);
                    println!("{}", commit_line(&c));
                    if c.ok {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::from(1)
                    }
                }
                Err(e) => {
                    eprintln!("bmb: {e}");
                    ExitCode::from(2)
                }
            }
        }
        Some("mcp") if args.len() == 2 => match bmb::mcp::serve(std::path::Path::new(&args[1])) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("bmb: {e}");
                ExitCode::from(2)
            }
        },
        Some("codes") if args.get(1).map(String::as_str) == Some("--md") => {
            print!("{}", bmb::diag::codes_markdown());
            ExitCode::SUCCESS
        }
        Some("codes") => {
            print!("{}", bmb::diag::codes_jsonl());
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn commit_line(c: &bmb::workspace::Commit) -> String {
    let list = |xs: &[String]| {
        xs.iter()
            .map(|x| bmb::diag::json_str(x))
            .collect::<Vec<_>>()
            .join(",")
    };
    format!(
        "{{\"committed\":{},\"changed\":[{}],\"removed\":[{}]}}",
        c.ok,
        list(&c.changed),
        list(&c.removed)
    )
}

fn read_files(paths: &[String]) -> Result<Vec<(String, String)>, ExitCode> {
    let mut files = Vec::new();
    for path in paths {
        if std::path::Path::new(path).is_dir() {
            match bmb::workspace::read_dir_sources(std::path::Path::new(path)) {
                Ok(fs) => files.extend(fs),
                Err(e) => {
                    eprintln!("bmb: {e}");
                    return Err(ExitCode::from(2));
                }
            }
            continue;
        }
        match std::fs::read_to_string(path) {
            Ok(text) => files.push((path.clone(), text)),
            Err(e) => {
                eprintln!("bmb: cannot read {path}: {e}");
                return Err(ExitCode::from(2));
            }
        }
    }
    Ok(files)
}

fn respond(r: Result<String, String>) -> ExitCode {
    match r {
        Ok(out) => {
            print!("{out}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            if e.starts_with('{') {
                print!("{e}");
            } else {
                eprintln!("bmb: {e}");
            }
            ExitCode::from(1)
        }
    }
}
