//! REPL (Read-Eval-Print Loop) for BMB

use crate::interp::Interpreter;
use crate::lexer::tokenize;
use crate::parser::parse;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result as RlResult};
use std::path::PathBuf;

const PROMPT: &str = "> ";
const HISTORY_FILE: &str = ".bmb_history";

/// REPL state
pub struct Repl {
    editor: DefaultEditor,
    interpreter: Interpreter,
    history_path: Option<PathBuf>,
}

impl Repl {
    /// Create a new REPL
    pub fn new() -> RlResult<Self> {
        let editor = DefaultEditor::new()?;
        let interpreter = Interpreter::new();

        // Try to find history file in home directory
        let history_path = dirs_home().map(|h| h.join(HISTORY_FILE));

        let mut repl = Repl {
            editor,
            interpreter,
            history_path,
        };

        // Load history if available
        if let Some(ref path) = repl.history_path {
            let _ = repl.editor.load_history(path);
        }

        Ok(repl)
    }

    /// Run the REPL
    pub fn run(&mut self) -> RlResult<()> {
        println!("BMB REPL v0.45");
        println!("Type :help for help, :quit to exit.\n");

        loop {
            match self.editor.readline(PROMPT) {
                Ok(line) => {
                    let line = line.trim();

                    if line.is_empty() {
                        continue;
                    }

                    // Add to history
                    let _ = self.editor.add_history_entry(line);

                    // Handle commands
                    if line.starts_with(':') {
                        if self.handle_command(line) {
                            break;
                        }
                        continue;
                    }

                    // Try to parse and evaluate
                    self.eval_input(line);
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    println!("Goodbye!");
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {err}");
                    break;
                }
            }
        }

        // Save history
        if let Some(ref path) = self.history_path {
            let _ = self.editor.save_history(path);
        }

        Ok(())
    }

    /// Handle REPL commands (starting with :)
    fn handle_command(&mut self, cmd: &str) -> bool {
        match cmd {
            ":quit" | ":q" | ":exit" => {
                println!("Goodbye!");
                true
            }
            ":help" | ":h" | ":?" => {
                self.print_help();
                false
            }
            ":clear" => {
                print!("\x1B[2J\x1B[1;1H");
                false
            }
            _ => {
                println!("Unknown command: {cmd}");
                println!("Type :help for help.");
                false
            }
        }
    }

    /// Print help message
    fn print_help(&self) {
        println!("BMB REPL Commands:");
        println!("  :help, :h, :?   Show this help");
        println!("  :quit, :q       Exit the REPL");
        println!("  :clear          Clear the screen");
        println!();
        println!("You can enter:");
        println!("  - Expressions: 1 + 2, if true then 1 else 2");
        println!("  - Function definitions: fn add(a: i32, b: i32) -> i32 = a + b;");
        println!("  - Function calls: add(1, 2)");
        println!();
        println!("Built-in functions:");
        println!("  println(x)      Print value with newline");
        println!("  print(x)        Print value without newline");
        println!("  assert(cond)    Assert condition is true");
        println!("  abs(n)          Absolute value");
        println!("  min(a, b)       Minimum of two values");
        println!("  max(a, b)       Maximum of two values");
    }

    /// Evaluate user input (v0.45: improved type inference)
    fn eval_input(&mut self, input: &str) {
        // If it's a function definition, use directly
        if input.starts_with("fn ") || input.starts_with("pub fn ") {
            self.eval_source(input);
            return;
        }

        // v0.45: Try multiple return types to support more expressions
        // Order: i64 (most common), bool, f64, string, () for side effects
        let return_types = ["i64", "bool", "f64", "string", "()"];
        let mut last_error: Option<String> = None;

        for ret_type in return_types {
            let source = format!("fn __repl__() -> {ret_type} = {input};");

            // Tokenize
            let tokens = match tokenize(&source) {
                Ok(t) => t,
                Err(e) => {
                    last_error = Some(format!("Lexer error: {}", e.message()));
                    continue;
                }
            };

            // Parse
            let program = match parse("<repl>", &source, tokens) {
                Ok(p) => p,
                Err(e) => {
                    last_error = Some(format!("Parse error: {}", e.message()));
                    continue;
                }
            };

            // Type check (without function registration for now)
            let mut checker = crate::types::TypeChecker::new();
            if checker.check_program(&program).is_err() {
                // Type check failed, try next type
                continue;
            }

            // Type check passed, now run it
            self.interpreter.load(&program);
            match self.interpreter.run(&program) {
                Ok(value) => {
                    // Don't print Unit values (like from println)
                    if !matches!(value, crate::interp::Value::Unit) {
                        println!("{value}");
                    }
                }
                Err(err) => {
                    eprintln!("Runtime error: {}", err.message);
                }
            }
            return;
        }

        // If no type worked, show the last error or a generic message
        if let Some(err) = last_error {
            eprintln!("{err}");
        } else {
            // Try to get a better error message with i64
            let source = format!("fn __repl__() -> i64 = {input};");
            if let Ok(tokens) = tokenize(&source)
                && let Ok(program) = parse("<repl>", &source, tokens)
            {
                let mut checker = crate::types::TypeChecker::new();
                if let Err(err) = checker.check_program(&program) {
                    eprintln!("Type error: {}", err.message());
                    return;
                }
            }
            eprintln!("Could not evaluate expression");
        }
    }

    /// Evaluate a complete source string (for function definitions)
    fn eval_source(&mut self, source: &str) {
        // Tokenize
        let tokens = match tokenize(source) {
            Ok(tokens) => tokens,
            Err(err) => {
                eprintln!("Lexer error: {}", err.message());
                return;
            }
        };

        // Parse
        match parse("<repl>", source, tokens) {
            Ok(program) => {
                // Load any function definitions
                self.interpreter.load(&program);

                // Run the program (which will call __repl__ or main)
                match self.interpreter.run(&program) {
                    Ok(value) => {
                        // Don't print Unit values (like from println)
                        if !matches!(value, crate::interp::Value::Unit) {
                            println!("{value}");
                        }
                    }
                    Err(err) => {
                        eprintln!("Runtime error: {}", err.message);
                    }
                }
            }
            Err(err) => {
                eprintln!("Parse error: {}", err.message());
            }
        }
    }
}

impl Default for Repl {
    fn default() -> Self {
        Self::new().expect("Failed to create REPL")
    }
}

/// Get home directory
fn dirs_home() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var("USERPROFILE").ok().map(PathBuf::from)
    }
    #[cfg(not(windows))]
    {
        std::env::var("HOME").ok().map(PathBuf::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_new() {
        let repl = Repl::new();
        assert!(repl.is_ok());
    }

    #[test]
    fn test_handle_command_quit() {
        let mut repl = Repl::new().unwrap();
        assert!(repl.handle_command(":quit"));
        assert!(repl.handle_command(":q"));
        assert!(repl.handle_command(":exit"));
    }

    #[test]
    fn test_handle_command_help() {
        let mut repl = Repl::new().unwrap();
        assert!(!repl.handle_command(":help"));
        assert!(!repl.handle_command(":h"));
        assert!(!repl.handle_command(":?"));
    }

    #[test]
    fn test_handle_command_clear() {
        let mut repl = Repl::new().unwrap();
        assert!(!repl.handle_command(":clear"));
    }

    #[test]
    fn test_handle_command_unknown() {
        let mut repl = Repl::new().unwrap();
        assert!(!repl.handle_command(":unknown"));
    }

    #[test]
    fn test_dirs_home_returns_some() {
        // On any real system, HOME or USERPROFILE should be set
        let home = dirs_home();
        assert!(home.is_some());
    }

    #[test]
    fn test_repl_default() {
        let repl = Repl::default();
        // Just verify it doesn't panic
        assert!(repl.history_path.is_some());
    }

    #[test]
    fn test_constants() {
        assert_eq!(PROMPT, "> ");
        assert_eq!(HISTORY_FILE, ".bmb_history");
    }

    // --- Cycle 1227: Additional REPL Tests ---

    #[test]
    fn test_eval_input_expression() {
        let mut repl = Repl::new().unwrap();
        // eval_input should not panic for a simple expression
        repl.eval_input("1 + 2");
    }

    #[test]
    fn test_eval_input_invalid_expression() {
        let mut repl = Repl::new().unwrap();
        // Should not panic, just print error
        repl.eval_input("@#$%");
    }

    #[test]
    fn test_eval_source_function_def() {
        let mut repl = Repl::new().unwrap();
        // Define a function via eval_source
        repl.eval_source("fn add(a: i64, b: i64) -> i64 = a + b;");
        // Should not panic
    }

    #[test]
    fn test_eval_source_invalid() {
        let mut repl = Repl::new().unwrap();
        repl.eval_source("this is not valid bmb code $$$");
        // Should not panic, just print error
    }

    #[test]
    fn test_handle_command_returns_correctly() {
        let mut repl = Repl::new().unwrap();
        // Quit commands return true
        assert!(repl.handle_command(":quit"));
        assert!(repl.handle_command(":q"));
        assert!(repl.handle_command(":exit"));
        // Non-quit commands return false
        assert!(!repl.handle_command(":help"));
        assert!(!repl.handle_command(":clear"));
        assert!(!repl.handle_command(":anything_else"));
    }

    #[test]
    fn test_eval_input_bool_expression() {
        let mut repl = Repl::new().unwrap();
        repl.eval_input("true");
    }

    #[test]
    fn test_eval_input_fn_definition_prefix() {
        let mut repl = Repl::new().unwrap();
        // Lines starting with "fn " are treated as function definitions
        repl.eval_input("fn foo() -> i64 = 42;");
    }

    #[test]
    fn test_eval_input_pub_fn_prefix() {
        let mut repl = Repl::new().unwrap();
        // Lines starting with "pub fn " are also function definitions
        repl.eval_input("pub fn bar() -> i64 = 99;");
    }

    // ================================================================
    // Additional REPL tests (Cycle 1234)
    // ================================================================

    #[test]
    fn test_eval_input_string_literal() {
        let mut repl = Repl::new().unwrap();
        repl.eval_input("\"hello\"");
    }

    #[test]
    fn test_eval_input_if_expression() {
        let mut repl = Repl::new().unwrap();
        repl.eval_input("if true then 1 else 2");
    }

    #[test]
    fn test_eval_source_multiple_functions() {
        let mut repl = Repl::new().unwrap();
        repl.eval_source("fn double(x: i64) -> i64 = x * 2;");
        repl.eval_source("fn triple(x: i64) -> i64 = x * 3;");
        // Both should be loaded without error
    }

    #[test]
    fn test_eval_input_float_literal() {
        let mut repl = Repl::new().unwrap();
        repl.eval_input("3.14");
    }

    #[test]
    fn test_eval_input_comparison() {
        let mut repl = Repl::new().unwrap();
        repl.eval_input("1 < 2");
    }

    #[test]
    fn test_eval_input_nested_arithmetic() {
        let mut repl = Repl::new().unwrap();
        repl.eval_input("(1 + 2) * (3 + 4)");
    }

    #[test]
    fn test_history_path_exists() {
        let repl = Repl::new().unwrap();
        assert!(repl.history_path.is_some());
        let path = repl.history_path.unwrap();
        assert!(path.to_string_lossy().contains(".bmb_history"));
    }

    #[test]
    fn test_eval_input_let_expression() {
        let mut repl = Repl::new().unwrap();
        // let expressions should work (evaluates to the body)
        repl.eval_input("let x = 42 in x + 1");
    }

    #[test]
    fn test_eval_source_lexer_error() {
        let mut repl = Repl::new().unwrap();
        // Completely invalid tokens
        repl.eval_source("###");
        // Should not panic
    }

    #[test]
    fn test_eval_input_boolean_literal() {
        let mut repl = Repl::new().unwrap();
        repl.eval_input("false");
    }

    // ================================================================
    // Additional REPL tests (Cycle 1237)
    // ================================================================

    #[test]
    fn test_eval_input_subtraction() {
        let mut repl = Repl::new().unwrap();
        repl.eval_input("10 - 3");
    }

    #[test]
    fn test_eval_input_multiplication() {
        let mut repl = Repl::new().unwrap();
        repl.eval_input("6 * 7");
    }

    #[test]
    fn test_eval_input_division() {
        let mut repl = Repl::new().unwrap();
        repl.eval_input("42 / 6");
    }

    #[test]
    fn test_eval_input_modulo() {
        let mut repl = Repl::new().unwrap();
        repl.eval_input("10 % 3");
    }

    #[test]
    fn test_eval_input_negative_literal() {
        let mut repl = Repl::new().unwrap();
        repl.eval_input("0 - 1");
    }

    #[test]
    fn test_eval_source_call_defined_function() {
        let mut repl = Repl::new().unwrap();
        repl.eval_source("fn square(n: i64) -> i64 = n * n;");
        // Function should be loaded, calling it should not panic
        repl.eval_input("square(5)");
    }

    #[test]
    fn test_handle_command_all_quit_variants() {
        let mut repl = Repl::new().unwrap();
        assert!(repl.handle_command(":quit"));
        assert!(repl.handle_command(":q"));
        assert!(repl.handle_command(":exit"));
    }

    #[test]
    fn test_eval_input_empty_like_expression() {
        let mut repl = Repl::new().unwrap();
        // Various tokens that should not crash
        repl.eval_input("0");
    }

    #[test]
    fn test_eval_input_large_number() {
        let mut repl = Repl::new().unwrap();
        repl.eval_input("1000000");
    }

    #[test]
    fn test_eval_source_parse_error_message() {
        let mut repl = Repl::new().unwrap();
        // Should not panic, just print error
        repl.eval_source("fn incomplete(");
    }
}
