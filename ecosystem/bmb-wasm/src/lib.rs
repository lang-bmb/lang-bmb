// BMB WASM bindings for browser playground
// Exposes check() and run() APIs to JavaScript via wasm-bindgen

use std::cell::RefCell;
use wasm_bindgen::prelude::*;

// Thread-local output buffer: captures println output during WASM execution
thread_local! {
    static WASM_OUTPUT: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

fn wasm_println(args: &[bmb::interp::Value]) -> bmb::interp::InterpResult<bmb::interp::Value> {
    let line = args.iter()
        .map(|v| format!("{v}"))
        .collect::<Vec<_>>()
        .join(" ");
    WASM_OUTPUT.with(|buf| buf.borrow_mut().push(line));
    Ok(bmb::interp::Value::Unit)
}

fn wasm_print_no_newline(args: &[bmb::interp::Value]) -> bmb::interp::InterpResult<bmb::interp::Value> {
    let text = args.iter()
        .map(|v| format!("{v}"))
        .collect::<Vec<_>>()
        .join(" ");
    WASM_OUTPUT.with(|buf| {
        let mut b = buf.borrow_mut();
        if let Some(last) = b.last_mut() {
            last.push_str(&text);
        } else {
            b.push(text);
        }
    });
    Ok(bmb::interp::Value::Unit)
}

fn take_output() -> String {
    WASM_OUTPUT.with(|buf| {
        let lines = buf.borrow().join("\n");
        buf.borrow_mut().clear();
        lines
    })
}

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[derive(serde::Serialize)]
struct CheckResult {
    success: bool,
    errors: Vec<String>,
}

#[derive(serde::Serialize)]
struct RunResult {
    success: bool,
    stdout: String,
    error: Option<String>,
    execution_time_ms: f64,
}

/// Type-check BMB source code.
/// Returns JSON: { success: bool, errors: string[] }
#[wasm_bindgen]
pub fn check(source: &str) -> String {
    let result = bmb_check(source);
    serde_json::to_string(&result).unwrap_or_else(|e| format!(r#"{{"success":false,"errors":["{e}"]}}"#))
}

/// Run BMB source code via the interpreter.
/// Returns JSON: { success: bool, stdout: string, error?: string, execution_time_ms: number }
#[wasm_bindgen]
pub fn run(source: &str) -> String {
    let result = bmb_run(source);
    serde_json::to_string(&result).unwrap_or_else(|e| format!(r#"{{"success":false,"stdout":"","error":"{e}","execution_time_ms":0}}"#))
}

/// Return the BMB compiler version string.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn bmb_check(source: &str) -> CheckResult {
    use bmb::lexer::tokenize;
    use bmb::parser::parse;
    use bmb::types::TypeChecker;

    let tokens = match tokenize(source) {
        Ok(t) => t,
        Err(e) => {
            return CheckResult {
                success: false,
                errors: vec![format!("Lex error: {e:?}")],
            };
        }
    };

    let program = match parse("<wasm>", source, tokens) {
        Ok(p) => p,
        Err(e) => {
            return CheckResult {
                success: false,
                errors: vec![format!("Parse error: {e:?}")],
            };
        }
    };

    let mut checker = TypeChecker::new();
    match checker.check_program(&program) {
        Ok(()) => CheckResult { success: true, errors: vec![] },
        Err(e) => CheckResult {
            success: false,
            errors: vec![format!("{e:?}")],
        },
    }
}

fn bmb_run(source: &str) -> RunResult {
    use bmb::lexer::tokenize;
    use bmb::parser::parse;
    use bmb::types::TypeChecker;
    use bmb::interp::Interpreter;

    let start = web_time_ms();

    let tokens = match tokenize(source) {
        Ok(t) => t,
        Err(e) => {
            return RunResult {
                success: false,
                stdout: String::new(),
                error: Some(format!("Lex error: {e:?}")),
                execution_time_ms: web_time_ms() - start,
            };
        }
    };

    let program = match parse("<wasm>", source, tokens) {
        Ok(p) => p,
        Err(e) => {
            return RunResult {
                success: false,
                stdout: String::new(),
                error: Some(format!("Parse error: {e:?}")),
                execution_time_ms: web_time_ms() - start,
            };
        }
    };

    let mut checker = TypeChecker::new();
    if let Err(e) = checker.check_program(&program) {
        return RunResult {
            success: false,
            stdout: String::new(),
            error: Some(format!("{e:?}")),
            execution_time_ms: web_time_ms() - start,
        };
    }

    // Clear output buffer, register WASM println, run
    take_output();
    let mut interp = Interpreter::new();
    // Replace print builtins with WASM output-capturing versions
    interp.register_builtin("print", wasm_print_no_newline);
    interp.register_builtin("println", wasm_println);
    interp.register_builtin("println_str", wasm_println);
    interp.register_builtin("println_f64", wasm_println);
    interp.register_builtin("write_stdout", wasm_print_no_newline);

    match interp.run(&program) {
        Ok(_) => RunResult {
            success: true,
            stdout: take_output(),
            error: None,
            execution_time_ms: web_time_ms() - start,
        },
        Err(e) => RunResult {
            success: false,
            stdout: take_output(),
            error: Some(format!("{e:?}")),
            execution_time_ms: web_time_ms() - start,
        },
    }
}

fn web_time_ms() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as f64
    }
}
