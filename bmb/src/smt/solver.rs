//! Z3 SMT solver interface via external process
//!
//! Invokes Z3 on generated SMT-LIB2 files and parses results.

use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};

/// SMT solver interface
pub struct SmtSolver {
    /// Path to Z3 executable
    z3_path: String,
    /// Timeout in seconds
    timeout: u32,
}

impl SmtSolver {
    /// Create a new solver with default Z3 path
    pub fn new() -> Self {
        Self {
            z3_path: "z3".to_string(),
            timeout: 10,
        }
    }

    /// Set Z3 executable path
    pub fn with_path(mut self, path: &str) -> Self {
        self.z3_path = path.to_string();
        self
    }

    /// Set timeout in seconds
    pub fn with_timeout(mut self, seconds: u32) -> Self {
        self.timeout = seconds;
        self
    }

    /// Check if Z3 is available
    pub fn is_available(&self) -> bool {
        Command::new(&self.z3_path)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Run Z3 on the given SMT-LIB2 script
    pub fn solve(&self, smt_script: &str) -> Result<SolverResult, SolverError> {
        let mut child = Command::new(&self.z3_path)
            .arg("-in")
            .arg(format!("-T:{}", self.timeout))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| SolverError::ProcessError(format!("failed to start z3: {}", e)))?;

        // Write SMT script to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(smt_script.as_bytes())
                .map_err(|e| SolverError::ProcessError(format!("failed to write to z3: {}", e)))?;
        }

        // Get output
        let output = child.wait_with_output()
            .map_err(|e| SolverError::ProcessError(format!("failed to wait for z3: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !stderr.is_empty() && !stderr.contains("warning") {
            return Err(SolverError::Z3Error(stderr.to_string()));
        }

        self.parse_result(&stdout)
    }

    fn parse_result(&self, output: &str) -> Result<SolverResult, SolverError> {
        let lines: Vec<&str> = output.lines().collect();

        if lines.is_empty() {
            return Err(SolverError::ParseError("empty output".into()));
        }

        match lines[0].trim() {
            "sat" => {
                // Parse model
                let model = self.parse_model(&lines[1..]);
                Ok(SolverResult::Sat(model))
            }
            "unsat" => Ok(SolverResult::Unsat),
            "unknown" => Ok(SolverResult::Unknown),
            "timeout" => Ok(SolverResult::Timeout),
            other => Err(SolverError::ParseError(format!("unexpected result: {}", other))),
        }
    }

    fn parse_model(&self, lines: &[&str]) -> HashMap<String, String> {
        let mut model = HashMap::new();
        let full_output = lines.join("\n");

        // Simple parser for Z3 model output format:
        // (model
        //   (define-fun x () Int 5)
        //   (define-fun y () Bool true)
        // )

        // Find define-fun declarations
        for line in lines {
            let line = line.trim();
            if line.starts_with("(define-fun ")
                && let Some(parsed) = self.parse_define_fun(line)
            {
                model.insert(parsed.0, parsed.1);
            }
        }

        // If simple parsing failed, try to extract from full output
        if model.is_empty() && full_output.contains("define-fun") {
            // More robust parsing using simple string parsing
            self.parse_model_fallback(&full_output, &mut model);
        }

        model
    }

    fn parse_define_fun(&self, line: &str) -> Option<(String, String)> {
        // Parse: (define-fun name () Type value)
        let line = line.trim_start_matches("(define-fun ").trim_end_matches(')');
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() >= 4 {
            let name = parts[0].to_string();
            // Skip "()" and type, get value
            let value = parts[3..].join(" ").trim_end_matches(')').to_string();
            Some((name, value))
        } else {
            None
        }
    }

    fn parse_model_fallback(&self, output: &str, model: &mut HashMap<String, String>) {
        // Parse define-fun expressions more carefully
        // Format: (define-fun name () Type value)
        let mut chars = output.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '(' {
                // Check if this is define-fun
                let mut token = String::new();
                while let Some(&next) = chars.peek() {
                    if next.is_whitespace() || next == '(' || next == ')' {
                        break;
                    }
                    token.push(chars.next().unwrap());
                }

                if token == "define-fun" {
                    // Skip whitespace
                    while chars.peek().is_some_and(|c| c.is_whitespace()) {
                        chars.next();
                    }

                    // Get variable name
                    let mut name = String::new();
                    while let Some(&next) = chars.peek() {
                        if next.is_whitespace() || next == '(' || next == ')' {
                            break;
                        }
                        name.push(chars.next().unwrap());
                    }

                    // Skip to value (after "()" and type)
                    let mut depth = 0;
                    let mut found_type = false;
                    while let Some(c) = chars.next() {
                        if c == '(' {
                            depth += 1;
                        } else if c == ')' {
                            if depth > 0 {
                                depth -= 1;
                            } else {
                                // End of define-fun without value
                                break;
                            }
                        } else if depth == 0 && !c.is_whitespace() && found_type {
                            // This is the start of the value
                            let mut value = String::from(c);
                            let mut val_depth = if c == '(' { 1 } else { 0 };

                            while let Some(&next) = chars.peek() {
                                if val_depth == 0 && (next == ')' || next.is_whitespace()) {
                                    break;
                                }
                                let next = chars.next().unwrap();
                                if next == '(' {
                                    val_depth += 1;
                                } else if next == ')' {
                                    val_depth -= 1;
                                }
                                value.push(next);
                            }

                            model.insert(name, value);
                            break;
                        } else if depth == 0 && !c.is_whitespace() {
                            // This is the type
                            found_type = true;
                            // Skip rest of type
                            while chars.peek().is_some_and(|c| !c.is_whitespace()) {
                                chars.next();
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Default for SmtSolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Result from SMT solver
#[derive(Debug, Clone)]
pub enum SolverResult {
    /// Satisfiable with model
    Sat(HashMap<String, String>),
    /// Unsatisfiable
    Unsat,
    /// Unknown (solver gave up)
    Unknown,
    /// Timeout
    Timeout,
}

/// Errors from solver
#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum SolverError {
    ProcessError(String),
    Z3Error(String),
    ParseError(String),
}

impl std::fmt::Display for SolverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SolverError::ProcessError(msg) => write!(f, "process error: {}", msg),
            SolverError::Z3Error(msg) => write!(f, "Z3 error: {}", msg),
            SolverError::ParseError(msg) => write!(f, "parse error: {}", msg),
        }
    }
}

impl std::error::Error for SolverError {}

/// Result of contract verification
#[derive(Debug, Clone)]
pub enum VerifyResult {
    /// Contract is verified (negation is unsatisfiable)
    Verified,
    /// Contract failed with counterexample
    Failed(Counterexample),
    /// Solver could not determine (timeout/unknown)
    Unknown(String),
    /// Z3 not available
    SolverNotAvailable,
}

/// Counterexample showing why verification failed
#[derive(Debug, Clone)]
pub struct Counterexample {
    /// Variable assignments that violate the contract
    pub assignments: Vec<(String, String)>,
}

impl Counterexample {
    pub fn from_model(model: HashMap<String, String>) -> Self {
        let mut assignments: Vec<_> = model.into_iter().collect();
        assignments.sort_by(|a, b| a.0.cmp(&b.0));
        Self { assignments }
    }
}

impl std::fmt::Display for Counterexample {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Counterexample:")?;
        for (name, value) in &self.assignments {
            if name == "__ret__" {
                writeln!(f, "  ret = {}", value)?;
            } else {
                writeln!(f, "  {} = {}", name, value)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solver_creation() {
        let solver = SmtSolver::new();
        assert_eq!(solver.timeout, 10);
    }

    #[test]
    fn test_solver_with_options() {
        let solver = SmtSolver::new()
            .with_path("/usr/bin/z3")
            .with_timeout(30);
        assert_eq!(solver.z3_path, "/usr/bin/z3");
        assert_eq!(solver.timeout, 30);
    }
}
