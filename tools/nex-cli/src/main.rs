use clap::{Parser, Subcommand};
use std::fs;

#[derive(Parser)]
#[command(name = "nex")]
#[command(about = "NEX language toolkit")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check syntax of NEX files
    Check {
        /// Files to check
        files: Vec<String>,
    },
    /// Format NEX files
    Format {
        /// Files to format
        files: Vec<String>,
        /// Write changes to files
        #[arg(short, long)]
        write: bool,
    },
    /// Convert NEX to JSON
    ToJson {
        /// Input NEX file
        file: String,
    },
    /// Convert JSON to NEX
    FromJson {
        /// Input JSON file
        file: String,
    },
    /// Start LSP server
    Lsp,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { files } => {
            for file in files {
                let content = fs::read_to_string(&file)?;
                match nex_parser::parse(&content) {
                    Ok(_) => println!("✓ {}: OK", file),
                    Err(err) => {
                        eprintln!("✗ {}: {}", file, err);
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::Format { files, write } => {
            for file in files {
                let content = fs::read_to_string(&file)?;
                match nex_parser::parse(&content) {
                    Ok(parsed) => {
                        let formatted = nex_formatter::format(&parsed);
                        if write {
                            fs::write(&file, formatted)?;
                            println!("Formatted {}", file);
                        } else {
                            println!("{}", formatted);
                        }
                    }
                    Err(err) => {
                        eprintln!("Error parsing {}: {}", file, err);
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::ToJson { file } => {
            let content = fs::read_to_string(&file)?;
            match nex_parser::parse(&content) {
                Ok(parsed) => {
                    let json = nex_json::nex_to_json(&parsed);
                    println!("{}", serde_json::to_string_pretty(&json)?);
                }
                Err(err) => {
                    eprintln!("Error parsing {}: {}", file, err);
                    std::process::exit(1);
                }
            }
        }
        Commands::FromJson { file } => {
            let content = fs::read_to_string(&file)?;
            let json: serde_json::Value = serde_json::from_str(&content)?;
            let nex = nex_json::json_to_nex(&json);
            println!("{}", nex_formatter::format(&nex));
        }
        Commands::Lsp => {
            nex_lsp::run_lsp().await?;
        }
    }

    Ok(())
}
