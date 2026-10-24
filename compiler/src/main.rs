use std::process::ExitCode;

const USAGE: &str = "usage:
  bmb check <file.bmb>...   check sources; diagnostics as JSONL on stdout
  bmb verify [--emit-smt DIR] <file.bmb>...
                            check and prove; diagnostics and per-definition status as JSONL
  bmb codes                 list diagnostic codes as JSONL
  bmb doctor                check the pinned toolchain (z3, LLVM) as JSONL";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("check") if args.len() > 1 => {
            let mut files = Vec::new();
            for path in &args[1..] {
                match std::fs::read_to_string(path) {
                    Ok(text) => files.push((path.clone(), text)),
                    Err(e) => {
                        eprintln!("bmb: cannot read {path}: {e}");
                        return ExitCode::from(2);
                    }
                }
            }
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
            let mut files = Vec::new();
            for path in rest {
                match std::fs::read_to_string(path) {
                    Ok(text) => files.push((path.clone(), text)),
                    Err(e) => {
                        eprintln!("bmb: cannot read {path}: {e}");
                        return ExitCode::from(2);
                    }
                }
            }
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
