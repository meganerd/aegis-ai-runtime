use aegis::{Aegis, Policy};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "aegis")]
#[command(version = "0.1.0")]
#[command(about = "Aegis: A Safe Agent Execution Runtime")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Policy file (YAML)
    #[arg(short, long, default_value = "policy.yaml")]
    policy: PathBuf,

    /// Tool/policy name to use
    #[arg(short, long, default_value = "default")]
    tool: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute a Rhai script
    Run {
        /// Script file to execute
        file: Option<PathBuf>,

        /// Inline script
        #[arg(short, long)]
        execute: Option<String>,
    },
    /// List available policies
    Policies,
    /// Validate a policy file
    Validate {
        /// Policy file to validate
        file: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let cli = Cli::parse();

    // Load policy
    let policy_content = fs::read_to_string(&cli.policy)?;
    let policy = Policy::from_yaml(&policy_content)?;

    // Create Aegis instance with policy
    let aegis = Aegis::new().with_policy(&policy, &cli.tool);

    match cli.command {
        Commands::Run { file, execute } => {
            let code = if let Some(f) = file {
                fs::read_to_string(f)?
            } else if let Some(e) = execute {
                e
            } else {
                return Err("No script provided".into());
            };

            match aegis.execute(&code) {
                Ok(result) => {
                    println!("Result: {:?}", result);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Policies => {
            println!("Available policies:");
            for (name, _) in &policy.tools {
                println!("  - {}", name);
            }
        }
        Commands::Validate { file } => {
            let content = fs::read_to_string(file)?;
            match Policy::from_yaml(&content) {
                Ok(_) => println!("Policy is valid"),
                Err(e) => {
                    eprintln!("Policy error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}
