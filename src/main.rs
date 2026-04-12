use aegis_ai_runtime::{mcp::McpServer, Aegis, Policy};
use clap::{Parser, Subcommand};
use std::fs;
use std::io::{Read, Write};
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
    /// Start HTTP server (JSON-RPC)
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "4788")]
        port: u16,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Load policy
    let policy_content = fs::read_to_string(&cli.policy)?;
    let policy = Policy::from_yaml(&policy_content)?;

    match cli.command {
        Commands::Run { file, execute } => {
            // Create Aegis instance with policy
            let aegis = Aegis::new().with_policy(&policy, &cli.tool);

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
        Commands::Serve { port } => {
            // Create Aegis instance
            let aegis = Aegis::new();

            // Create MCP server
            let server = McpServer::new(aegis, policy);

            // Simple HTTP server
            let addr = format!("127.0.0.1:{}", port);
            let listener = std::net::TcpListener::bind(&addr)?;

            println!("Aegis JSON-RPC server running on http://{}", addr);

            for stream in listener.incoming() {
                let mut stream = stream?;
                let mut buffer = [0u8; 4096];
                let n = stream.read(&mut buffer)?;

                let body = String::from_utf8_lossy(&buffer[..n]);
                let response = server.handle_request(&body);

                let response_json = serde_json::to_string(&response).unwrap_or_default();
                let response_str = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                    response_json.len(),
                    response_json
                );
                stream.write_all(response_str.as_bytes())?;
            }
        }
    }

    Ok(())
}
