# Aegis: Safe Agent Execution Runtime

A secure sandbox for executing AI agent code using Rhai — a Rust-embedded scripting language.

## Overview

Aegis provides a constrained execution environment for AI agents. It leverages [Rhai](https://rhai.rs) to run untrusted code with fine-grained capability control, resource limits, and policy-based access control.

## Features

- **Capability-based Access Control** — Grant specific capabilities (HTTP, file I/O, KV store) per tool
- **Resource Limits** — Memory, operations, call depth, and timeout limits per execution
- **Policy Validation** — Domain allowlists, path allowlists, and key prefix restrictions
- **JSON-RPC Server** — Built-in MCP-style server for remote execution
- **Multiple Tool Policies** — Define different policies for different agent tools

## Quick Start

```bash
# Build
cargo build --release

# Run a script
cargo run -- run --execute 'log("Hello from Aegis!"); yield(42)'

# Use a specific policy
cargo run -- run --policy policy.yaml --tool weather_fetch --execute 'http_get("https://api.weather.com/forecast")'
```

## Capabilities

| Capability | Description |
|-----------|------------|
| `log` | Print to stdout |
| `yield` | Return a result |
| `sleep` | Delay execution |
| `http_get` | HTTP GET requests |
| `kv_set` | Set key-value pairs |
| `kv_get` | Get key-value pairs |
| `file_read` | Read files |
| `file_write` | Write files |
| `file_list` | List directory contents |

## Policy Configuration

```yaml
default_capabilities:
  - log
  - yield

tools:
  default:
    capabilities:
      - http_get
      - kv_set
      - kv_get
    resource_limits:
      memory_mb: 64
      max_operations: 1000000
      max_call_depth: 64
      timeout_seconds: 30

  weather_fetch:
    capabilities:
      - http_get
      - kv_set
    allowed_domains:
      - api.weather.com
      - api.openweathermap.org
    allowed_key_prefixes:
      - "weather:"
```

## JSON-RPC Server

Start the server:

```bash
cargo run -- serve --port 4788
```

Call via HTTP:

```bash
curl -X POST http://localhost:4788 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "execute",
    "params": {
      "code": "log(\"Hello\"); yield(42)"
    },
    "id": 1
  }'
```

## Use as a Library

```rust
use aegis_ai_runtime::{Aegis, Policy};

let policy = Policy::from_yaml(policy_yaml)?;
let aegis = Aegis::new().with_policy(&policy, "default");

match aegis.execute("log('hello'); yield(42)") {
    Ok(result) => println!("Result: {:?}", result),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Installation

```bash
# From crates.io
cargo install aegis-ai-runtime

# Or build from source
git clone https://github.com/meganerd/aegis-ai-runtime.git
cd aegis-ai-runtime
cargo install --path .
```

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                      Aegis                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │   Grants   │  │   Limits   │  │    Policy   │       │
│  │  (Caps)   │  │ (Memory,   │  │ (Domains,   │       │
│  │           │  │  Ops,      │  │  Paths,    │       │
│  │           │  │  Depth)   │  │  Keys)     │       │
│  └─────────────┘  └─────────────┘  └─────────────┘       │
│         │               │               │                   │
│         └─────────────┴───────────────┘                   │
│                       │                                │
│                       ▼                                │
│              ┌─────────────────┐                     │
│              │  Rhai Engine     │                     │
│              │  (Sandboxed)     │                     │
│              └─────────────────┘                     │
└─────────────────────────────────────────────────────────┘
```

## License

MIT OR Apache-2.0

## Links

- [GitHub](https://github.com/meganerd/aegis-ai-runtime)
- [GitLab](https://gitlab.zarquon.space/meganerd/aegis)
- [crates.io](https://crates.io/crates/aegis-ai-runtime)