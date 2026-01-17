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
