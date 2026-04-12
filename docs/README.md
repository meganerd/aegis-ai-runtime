# Aegis Documentation

## Overview

Aegis is a secure agent execution runtime built on [Rhai](https://rhai.rs), a fast and secure scripting language for Rust. It provides a sandboxed environment for executing untrusted AI agent code with fine-grained capability control.

## Key Concepts

### Sandboxed Execution

Aegis leverages Rhai's sandboxed execution model to run untrusted code. Unlike native Rust, Rhai:
- Does not have direct access to system APIs
- Has controlled function definitions
- Enforces resource limits (operations, memory, call depth)

### Capability System

Capabilities are explicit permissions granted to execute specific operations:

| Capability | Function | Description |
|-----------|----------|------------|
| `log` | `log(msg)` | Print to stdout |
| `yield` | `yield(value)` | Return a result |
| `sleep` | `sleep(millis)` | Delay execution |
| `http_get` | `http_get(url)` | HTTP GET requests |
| `kv_set` | `kv_set(key, value)` | Set key-value pairs |
| `kv_get` | `kv_get(key)` | Get key-value pairs |
| `file_read` | `file_read(path)` | Read files |
| `file_write` | `file_write(path, content)` | Write files |
| `file_list` | `file_list(path)` | List directory contents |

### Policy Configuration

Policies define what a tool can do:

```yaml
tools:
  tool_name:
    capabilities:
      - http_get
      - kv_set
    resource_limits:
      memory_mb: 64
      max_operations: 1000000
      max_call_depth: 64
      timeout_seconds: 30
    allowed_domains:
      - api.example.com
    allowed_paths:
      - /data/input/
      - /data/output/
    allowed_key_prefixes:
      - "app:"
```

### Validation Rules

- **Domain validation**: Prevents HTTP requests to non-allowed domains
- **Path validation**: Prevents file access outside allowed paths
- **Key validation**: Prevents setting/getting keys outside allowed prefixes

## Architecture

```
                    ┌─────────────────────────────────────┐
                    │           Aegis Runtime              │
                    │                                     │
┌──────────────┐   │   ┌─────────────────────────────┐    │
│   Policy     │───▶│   │    Rhai Engine            │    │
│  (YAML)      │   │   │    (Sandboxed)             │    │
└──────────────┘   │   └─────────────────────────────┘    │
                   │           │                           │
                   │     ┌─────┴─────┐                 │
                   │     │         │                   │
                   │  Grants   Limits             │
                   │ (Caps)  (Memory,           │
                   │          Ops, Depth)        │
                   └─────────────────────────────────────┘
```

## Use Cases

### 1. Script Execution

```rust
use aegis_ai_runtime::{Aegis, Policy};

let policy = Policy::from_yaml(policy_yaml)?;
let aegis = Aegis::new().with_policy(&policy, "default");

let result = aegis.execute("log('Hello'); yield(42)")?;
```

### 2. HTTP-Fetching Agent

```yaml
tools:
  web_fetch:
    capabilities:
      - http_get
      - kv_set
    allowed_domains:
      - api.weather.com
      - api.openweathermap.org
    allowed_key_prefixes:
      - "weather:"
```

### 3. File Processor

```yaml
tools:
  file_processor:
    capabilities:
      - file_read
      - file_write
    allowed_paths:
      - /data/input/
      - /data/output/
```

### 4. JSON-RPC Server

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
      "code": "log('test'); yield(42)"
    },
    "id": 1
  }'
```

## Resource Limits

| Limit | Default | Description |
|-------|---------|------------|
| `memory_mb` | 64 | Maximum memory in MB |
| `max_operations` | 1,000,000 | Operations per script |
| `max_call_depth` | 64 | Maximum call stack depth |
| `timeout_seconds` | 30 | Execution timeout |

## Configuration Files

### policy.yaml

The policy file defines tools and their capabilities:

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
```

## CLI Commands

```bash
# Run a script
cargo run -- run --execute 'log("hello")'

# Run a script file
cargo run -- run --file script.rhai

# Use a specific policy
cargo run -- run --policy policy.yaml --tool default

# List available policies
cargo run -- policies

# Validate a policy file
cargo run -- validate policy.yaml

# Start JSON-RPC server
cargo run -- serve --port 4788
```

## Security Considerations

1. **No native code execution** — Rhai cannot call arbitrary Rust functions
2. **Explicit capability grants** — Nothing is available by default
3. **Policy validation** — Domains, paths, and keys are validated
4. **Resource limits** — Prevents runaway execution
5. **No state leakage** — Each execution is isolated

## Installation

```bash
# From crates.io
cargo install aegis-ai-runtime

# Or build from source
git clone https://github.com/meganerd/aegis-ai-runtime.git
cd aegis-ai-runtime
cargo install --path .
```

## Links

- [crates.io](https://crates.io/crates/aegis-ai-runtime)
- [GitHub](https://github.com/meganerd/aegis-ai-runtime)
- [GitLab](https://gitlab.zarquon.space/meganerd/aegis)