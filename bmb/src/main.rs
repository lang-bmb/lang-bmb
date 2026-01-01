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
    /// Type check a BMB source file
    Check {
        /// Source file to check
        file: PathBuf,
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
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Check { file } => check_file(&file),
        Command::Parse { file } => parse_file(&file),
        Command::Tokens { file } => tokenize_file(&file),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
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
