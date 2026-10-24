use std::process::ExitCode;

const USAGE: &str = "usage:
  bmb check <file.bmb>...   check sources; diagnostics as JSONL on stdout
  bmb codes                 list diagnostic codes as JSONL";

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
