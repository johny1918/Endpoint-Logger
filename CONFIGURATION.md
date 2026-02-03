# Configuration Guide

Endpoint Logger supports flexible configuration through multiple sources with a clear priority order.

## Configuration Priority

The logger applies configuration in this order (highest to lowest priority):

```
1. CLI Arguments        (--target, --port, --database, --config)
2. Environment Variables (TARGET_URL, PORT, DATABASE_PATH)
3. TOML Config File     (endpoint-logger.toml)
4. Built-in Defaults
```

The first source that provides a value wins. For example, a CLI argument will override an environment variable.

---

## Quick Start Examples

### Minimal (Just CLI)
```bash
endpoint-logger --target http://localhost:8080
```
- Proxy listens on: 3000 (default)
- Database: ./endpoint-logs.db (default)

### With Custom Port
```bash
endpoint-logger --target http://localhost:8080 --port 5000
```

### Using Environment Variables
```bash
export TARGET_URL=http://localhost:8080
export PORT=3000
export DATABASE_PATH=./my-logs.db

endpoint-logger
```

### Using TOML Configuration File
```bash
# Create endpoint-logger.toml
endpoint-logger
```

### Custom TOML File Path
```bash
endpoint-logger --config ./production.toml
```

### Override TOML with CLI
```bash
# Uses production.toml but overrides the target
endpoint-logger --config production.toml --target http://localhost:9000
```

---

## CLI Arguments

### Required
None! But you must provide `target_url` via CLI, ENV, or TOML file.

### Optional Flags

| Flag | Short | Environment | Default | Description |
|------|-------|-------------|---------|-------------|
| `--target` | `-t` | `TARGET_URL` | *(required)* | Target application URL |
| `--port` | `-p` | `PORT` | `3000` | Proxy server listening port |
| `--database` | `-d` | `DATABASE_PATH` | `./endpoint-logs.db` | SQLite database file path |
| `--config` | `-c` | - | `./endpoint-logger.toml` | TOML config file path |
| `--verbose` | `-v` | `VERBOSE` | `false` | Enable verbose logging |
| `--help` | `-h` | - | - | Show help message |
| `--version` | - | - | - | Show version |

### Examples

```bash
# Basic with custom port
endpoint-logger --target http://localhost:8080 --port 5000

# Short flags
endpoint-logger -t http://localhost:8080 -p 5000 -d ./logs.db

# Verbose mode for debugging
endpoint-logger --target http://localhost:8080 --verbose

# Show help
endpoint-logger --help

# Show version
endpoint-logger --version
```

---

## TOML Configuration File

### File Location

The logger looks for `endpoint-logger.toml` in the current working directory by default.

Specify a custom path with:
```bash
endpoint-logger --config /path/to/config.toml
```

### Basic Format

```toml
# endpoint-logger.toml

# Core Settings (required)
target_url = "http://localhost:8080"

# Optional Settings (with defaults shown)
proxy_port = 3000
database_path = "./endpoint-logs.db"
verbose = false
```

### Complete TOML Example

```toml
# Endpoint Logger Configuration

[proxy]
# Target application URL (required)
target_url = "http://localhost:8080"

# Port to listen on for proxy requests
proxy_port = 3000

# Request timeout in seconds
timeout_seconds = 30

[storage]
# SQLite database file path
database_path = "./endpoint-logs.db"

# Maximum log entries to keep (0 = unlimited)
max_entries = 50000

# Auto-cleanup retention days (0 = no auto-cleanup)
retention_days = 7

[logging]
# Log level: standard, verbose, debug
level = "standard"

# Log request bodies
log_request_body = true

# Log response bodies
log_response_body = true

# Maximum body size to log (in KB)
max_body_size_kb = 100

[dashboard]
# Dashboard port (usually same as proxy_port)
port = 3000

# Auto-open browser on startup
auto_open_browser = true

[privacy]
# Redact sensitive patterns (regex)
redact_patterns = [
    "password",
    "token",
    "secret",
    "api_key",
    "authorization",
    "credit_card"
]

# Headers to redact completely
redact_headers = [
    "Authorization",
    "Cookie",
    "X-API-Key",
    "X-Auth-Token"
]

# Paths to ignore completely
ignore_paths = [
    "/health",
    "/metrics",
    "/favicon.ico",
    "/_next/*"
]

# Never log bodies for these paths
never_log_bodies_for = [
    "/api/login",
    "/api/register",
    "/api/payment",
    "/api/auth/*"
]

# Anonymize IP addresses
anonymize_ip = false
```

### Minimal TOML File

The absolute minimum is:

```toml
target_url = "http://localhost:8080"
```

All other settings use defaults.

---

## Environment Variables

Endpoint Logger respects these environment variables:

| Variable | Description | Example |
|----------|-------------|---------|
| `TARGET_URL` | Target application URL (required) | `http://localhost:8080` |
| `PORT` | Proxy server listening port | `3000` |
| `DATABASE_PATH` | SQLite database file path | `./endpoint-logs.db` |
| `VERBOSE` | Enable verbose logging | `true` or `1` |

### Docker Example

```bash
docker run \
  -e TARGET_URL=http://app:8080 \
  -e PORT=3000 \
  -e DATABASE_PATH=/data/logs.db \
  -e VERBOSE=false \
  endpoint-logger
```

### Kubernetes Example

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: endpoint-logger
spec:
  containers:
  - name: logger
    image: endpoint-logger:latest
    env:
    - name: TARGET_URL
      value: "http://backend:8080"
    - name: PORT
      value: "3000"
    - name: DATABASE_PATH
      value: "/var/lib/endpoint-logger/logs.db"
    ports:
    - containerPort: 3000
    volumeMounts:
    - name: data
      mountPath: /var/lib/endpoint-logger
  volumes:
  - name: data
    emptyDir: {}
```

---

## Real-World Scenarios

### Scenario 1: Quick Local Development

```bash
# Just override the target, use defaults for everything else
endpoint-logger --target http://localhost:8080

# This gives you:
# - Proxy on localhost:3000
# - Database: ./endpoint-logs.db
# - Dashboard: http://localhost:3000
```

### Scenario 2: Per-Project Configuration

Create a project-specific config:

```bash
# my-project.toml
cat > my-project.toml <<EOF
[proxy]
target_url = "http://localhost:8080"
proxy_port = 5000

[storage]
database_path = "./project-logs.db"

[logging]
level = "verbose"
EOF

# Use it
endpoint-logger --config my-project.toml
```

### Scenario 3: Multiple Environments

```bash
# development.toml
[proxy]
target_url = "http://localhost:8080"
proxy_port = 3000
timeout_seconds = 30

[logging]
level = "verbose"

# production.toml
[proxy]
target_url = "http://production-api.example.com"
proxy_port = 3000
timeout_seconds = 60

[logging]
level = "standard"

[privacy]
redact_headers = [
    "Authorization",
    "Cookie",
    "X-API-Key"
]

# Use based on environment
endpoint-logger --config ${ENV}.toml
```

### Scenario 4: CI/CD Pipeline

```bash
#!/bin/bash
# .github/workflows/test.yml or similar

export TARGET_URL=http://test-app:8080
export DATABASE_PATH=/tmp/test-logs.db
export PORT=3000

endpoint-logger
```

### Scenario 5: Docker Compose

```yaml
version: '3.8'

services:
  app:
    build: .
    ports:
      - "8080:8080"

  endpoint-logger:
    image: endpoint-logger:latest
    environment:
      TARGET_URL: http://app:8080
      PORT: 3000
      DATABASE_PATH: /data/logs.db
    ports:
      - "3000:3000"
    volumes:
      - logger-data:/data
    depends_on:
      - app

volumes:
  logger-data:
```

---

## Validation Rules

### Target URL

**Required:** Yes  
**Format:** Must be valid HTTP(S) URL

Valid:
- ✅ `http://localhost:8080`
- ✅ `https://api.example.com`
- ✅ `http://192.168.1.100:3000`

Invalid:
- ❌ `localhost:8080` (missing scheme)
- ❌ `ftp://example.com` (wrong scheme)
- ❌ `:8080` (no host)

### Port

**Required:** No (defaults to 3000)  
**Range:** 1-65535

Valid:
- ✅ `3000`
- ✅ `8080`
- ✅ `80`
- ✅ `65535`

Invalid:
- ❌ `0` (too low)
- ❌ `70000` (too high)
- ❌ `abc` (not a number)

### Database Path

**Required:** No (defaults to `./endpoint-logs.db`)  
**Format:** Valid file path

Valid:
- ✅ `./endpoint-logs.db`
- ✅ `/var/log/endpoint-logger/logs.db`
- ✅ `../shared/logs.db`
- ✅ `C:\Users\John\logs.db` (Windows)

Invalid:
- ❌ Directory path without filename
- ❌ Non-writable directory

### Timeout

**Required:** No (defaults to 30)  
**Range:** 1-300 seconds

---

## Error Messages

### Missing Target URL

```
Error: Target URL is required.

Provide it via one of these methods:
  1. CLI: endpoint-logger --target http://localhost:8080
  2. ENV: export TARGET_URL=http://localhost:8080
  3. TOML: target_url = "http://localhost:8080" in endpoint-logger.toml

Usage: endpoint-logger --help
```

### Invalid URL Format

```
Error: Invalid target URL format: 'localhost:8080'

URL must be valid and start with http:// or https://

Example: endpoint-logger --target http://localhost:8080
```

### Invalid Port Number

```
Error: Invalid port: 70000

Port must be between 1 and 65535

Example: endpoint-logger --port 3000
```

### Config File Not Found

```
Error: Configuration file not found: custom.toml

Make sure the file exists or remove the --config flag to use defaults.

Current directory: /home/user/projects/myapp
```

### Invalid TOML Syntax

```
Error: Failed to parse configuration file: custom.toml

Invalid TOML syntax at line 5:
  target_url = http://localhost:8080  ← Missing quotes!

Should be: target_url = "http://localhost:8080"
```

---

## Troubleshooting

### Port Already in Use

```bash
# Error: Address already in use

# Solution 1: Use a different port
endpoint-logger --target http://localhost:8080 --port 5000

# Solution 2: Find what's using port 3000
lsof -i :3000

# Solution 3: Kill the process (if safe)
kill <PID>
```

### Database Locked

```bash
# Error: database is locked

# Cause: Another instance is running

# Solution 1: Kill other instances
pkill endpoint-logger

# Solution 2: Use different database
endpoint-logger --target http://localhost:8080 \
                --database /tmp/alternative-logs.db

# Solution 3: Wait for the other instance to close
```

### Target Unreachable

```bash
# Error: Failed to connect to target: Connection refused

# Check target is running
curl http://localhost:8080/

# Check target URL is correct
endpoint-logger --target http://localhost:8080  # Make sure port matches

# Check network (if remote)
ping example.com
```

### Permission Denied

```bash
# Error: Permission denied (os error 13)

# Cause: Can't write to database location

# Solution 1: Use writable directory
endpoint-logger --database ~/endpoint-logs.db

# Solution 2: Use /tmp (temporary)
endpoint-logger --database /tmp/logs.db

# Solution 3: Create directory with permissions
mkdir -p ./logs
chmod 755 ./logs
endpoint-logger --database ./logs/endpoint-logs.db
```

---

## Advanced Configuration

### Sensitive Data Protection

Redact sensitive patterns from logs:

```toml
[privacy]
# Redact specific patterns (case-insensitive regex)
redact_patterns = [
    "password",
    "token",
    "secret",
    "api_key",
    "authorization",
    "credit_card",
    "ssn",
    "social_security"
]

# Headers to redact completely
redact_headers = [
    "Authorization",
    "Cookie",
    "X-API-Key",
    "X-Auth-Token",
    "Stripe-Signature"
]

# Never log request/response bodies for these paths
never_log_bodies_for = [
    "/api/login",
    "/api/register",
    "/api/password-reset",
    "/api/auth/*",
    "/api/payment",
    "/api/checkout"
]

# Anonymize IP addresses (127.0.0.1 → 127.0.0.xxx)
anonymize_ip = true
```

### Ignore Paths

Don't log these paths at all:

```toml
[privacy]
ignore_paths = [
    "/health",
    "/metrics",
    "/favicon.ico",
    "/.well-known/*",
    "/_next/*",
    "/static/*"
]
```

### Retention Policy

Automatically clean up old logs:

```toml
[storage]
# Keep maximum 50,000 entries
max_entries = 50000

# Keep logs for 7 days
retention_days = 7

# Auto-cleanup runs daily
```

---

## Performance Tuning

### For High Traffic

```toml
[proxy]
# Longer timeout for slow endpoints
timeout_seconds = 60

[storage]
# Reduce max entries to prevent disk bloat
max_entries = 10000

# Aggressive cleanup
retention_days = 1

[logging]
# Don't log response bodies to save I/O
log_response_body = false

# Reduce max body size
max_body_size_kb = 50
```

### For Development

```toml
[logging]
level = "verbose"
log_request_body = true
log_response_body = true
max_body_size_kb = 500

[storage]
# Keep more entries for debugging
max_entries = 100000

# Keep longer for investigation
retention_days = 30
```

---

## Environment-Specific Examples

### Local Development

```bash
endpoint-logger \
  --target http://localhost:8080 \
  --port 3000 \
  --database ./dev-logs.db
```

### Testing

```bash
export TARGET_URL=http://test-api:8080
export PORT=3000
export DATABASE_PATH=/tmp/test-logs.db
export VERBOSE=true

endpoint-logger
```

### Staging

```toml
# staging.toml
[proxy]
target_url = "http://staging-api.internal:8080"
proxy_port = 3000
timeout_seconds = 45

[storage]
database_path = "/var/log/endpoint-logger/staging.db"
max_entries = 100000

[logging]
level = "standard"
```

### Production (Not Recommended)

```toml
# This is a DEVELOPMENT tool, not for production
# But if you must use it:

[proxy]
target_url = "https://production-api.example.com"
timeout_seconds = 60

[privacy]
# Redact everything sensitive
redact_patterns = [".*"]
redact_headers = [
    "Authorization",
    "Cookie",
    "X-*"
]

[storage]
# Minimal storage
max_entries = 1000
retention_days = 1
```

---

## Migration from Other Tools

### From Postman/Insomnia

```bash
# Run through proxy for automatic logging
endpoint-logger --target http://localhost:8080

# All manual requests now captured automatically
# Better than maintaining request collections
```

### From Application Logging

```bash
# No code changes needed
# Just point endpoint-logger at your app
endpoint-logger --target http://localhost:8080

# Captures all traffic without modifying application
```

### From Cloud Logging (Datadog, etc.)

```bash
# Use for local development/debugging
# Cloud logging for production monitoring
endpoint-logger --target http://localhost:8080 --port 3000

# Local development: http://localhost:3000
# Production: Your cloud service
```

---

## Support

For issues with configuration:

1. Check error message carefully
2. Verify CLI arguments with `--help`
3. Validate TOML syntax (JSON-compatible structure)
4. Test with minimal config: `endpoint-logger --target http://localhost:8080`
5. Enable verbose mode: `--verbose`
6. Check documentation: [README.md](README.md)

---

**Need help?** See [README.md](README.md) for more information.
