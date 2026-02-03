# Contributing to Endpoint Logger

Thank you for your interest in contributing! We welcome contributions of all kinds: bug fixes, feature implementations, documentation improvements, and more.

## Code of Conduct

This project adheres to the [Contributor Covenant](https://www.contributor-covenant.org/). By participating, you are expected to uphold this code. Please report unacceptable behavior to the project maintainers.

## Getting Started

### Prerequisites

- **Rust 1.70+** - Install from [rustup.rs](https://rustup.rs/)
- **Node.js 18+** - Install from [nodejs.org](https://nodejs.org/)
- **Git** - For cloning and version control
- **SQLite3** - Usually pre-installed; check with `sqlite3 --version`

### Development Setup

1. **Fork & Clone**
```bash
git clone https://github.com/yourusername/endpoint-logger.git
cd endpoint-logger
```

2. **Install Dependencies**

**Rust Backend:**
```bash
cargo build
```

**Vue.js Frontend:**
```bash
cd dashboard
npm install
cd ..
```

3. **Run Development Servers**

**Terminal 1: Backend (with hot reload)**
```bash
cargo watch -x run
```

**Terminal 2: Frontend (Vite dev server)**
```bash
cd dashboard
npm run dev
# Runs on http://localhost:5173
```

**Terminal 3: Test Server (target app)**
```bash
# Create a simple test server
mkdir -p /tmp/test-target
cd /tmp/test-target

cat > server.js << 'EOF'
const http = require('http');
const server = http.createServer((req, res) => {
  res.writeHead(200, { 'Content-Type': 'application/json' });
  res.end(JSON.stringify({ message: 'Hello from test server' }));
});
server.listen(8080, () => console.log('Test server on 8080'));
EOF

node server.js
```

4. **Verify Setup**
```bash
# Test backend
curl http://localhost:3000/

# Test dashboard
# Open http://localhost:5173 in browser

# Test proxy
curl http://localhost:3000/api/logs
```

---

## Project Structure

```
endpoint-logger/
├── src/                          # Rust backend
│   ├── main.rs                   # Entry point, CLI setup
│   ├── config.rs                 # Configuration loading
│   ├── proxy/                    # HTTP proxy
│   │   ├── mod.rs                # Proxy setup
│   │   ├── interceptor.rs        # Request/response capture
│   │   └── forwarder.rs          # Target forwarding
│   ├── storage/                  # Data persistence
│   │   ├── mod.rs                # Storage trait
│   │   └── sqlite.rs             # SQLite implementation
│   ├── api/                      # REST & WebSocket
│   │   ├── mod.rs                # Route setup
│   │   ├── logs.rs               # Log endpoints
│   │   ├── websocket.rs          # WebSocket handler
│   │   └── broadcaster.rs        # Broadcast channel
│   ├── logging/                  # Async logging
│   │   ├── mod.rs                # Logging setup
│   │   └── worker.rs             # Background worker
│   └── lib.rs                    # Library exports
│
├── dashboard/                    # Vue.js frontend
│   ├── src/
│   │   ├── App.vue               # Root component
│   │   ├── main.js               # Entry point
│   │   ├── style.css             # Global styles
│   │   ├── components/           # Reusable components
│   │   │   ├── LogStream.vue     # Log display
│   │   │   ├── LogDetail.vue     # Detail modal
│   │   │   ├── Filter.vue        # Filter UI
│   │   │   └── ...
│   │   ├── views/                # Route pages
│   │   │   ├── LiveView.vue      # Real-time view
│   │   │   └── HistoryView.vue   # Historical view
│   │   ├── composables/          # Reusable logic
│   │   │   ├── useWebSocket.js   # WS connection
│   │   │   └── useApi.js         # REST client
│   │   ├── router/               # Vue Router
│   │   │   └── index.js
│   │   └── utils/                # Utilities
│   │       ├── api.js            # API functions
│   │       └── formatters.js     # Formatting
│   ├── package.json              # Dependencies
│   ├── vite.config.js            # Vite config
│   ├── index.html                # Entry HTML
│   └── dist/                     # Built assets (embedded)
│
├── tests/                        # Integration tests
│   ├── integration_test.py       # Python test suite
│   ├── quick_test.sh             # Quick bash tests
│   ├── setup_test_env.sh         # Setup script
│   └── README.md                 # Test documentation
│
├── Cargo.toml                    # Rust manifest
├── Cargo.lock                    # Dependency lock
├── package.json                  # Root npm (optional)
├── README.md                     # User documentation
├── CONFIGURATION.md              # Config reference
├── ARCHITECTURE.md               # Architecture guide
├── CONTRIBUTING.md               # This file
├── LICENSE                       # MIT License
└── Thoughts.md                   # Design notes
```

---

## Making Changes

### 1. Create a Branch

```bash
# Create feature branch
git checkout -b feature/my-feature

# Or bugfix branch
git checkout -b fix/my-bug
```

**Branch Naming:**
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation
- `refactor/` - Code improvements
- `test/` - Testing improvements

### 2. Make Your Changes

**Rust Code:**
- Follow `rustfmt` style (automatic with `cargo fmt`)
- Follow `clippy` recommendations (check with `cargo clippy`)
- Write unit tests for new functions
- Add documentation comments (///)

**Vue.js Code:**
- Follow ESLint rules (automatic with `npm run lint`)
- Use Prettier for formatting
- Use Vue 3 Composition API (script setup)
- Follow component naming conventions

**Documentation:**
- Use Markdown with proper formatting
- Link to related files
- Include code examples
- Update table of contents if needed

### 3. Test Your Changes

**Rust Backend:**
```bash
# Run unit tests
cargo test

# Run with logging
RUST_LOG=debug cargo run

# Format code
cargo fmt

# Check for issues
cargo clippy
```

**Vue.js Frontend:**
```bash
cd dashboard

# Lint
npm run lint

# Format
npm run format

# Unit tests (if available)
npm test

# Build
npm run build
```

**Integration Testing:**
```bash
# See TESTING.md for full procedures
bash tests/quick_test.sh

# Or full test suite
bash tests/setup_test_env.sh
```

### 4. Commit Your Changes

**Good Commit Messages:**
```bash
# Follow conventional commits format
git commit -m "feat: add log filtering by status code"
git commit -m "fix: resolve WebSocket reconnection issue"
git commit -m "docs: update configuration guide"
git commit -m "refactor: simplify proxy handler"
git commit -m "test: add unit tests for storage layer"
```

**Format:**
```
<type>(<scope>): <subject>

<body>

<footer>
```

**Example:**
```
feat(dashboard): add export to CSV functionality

Implement CSV export feature allowing users to download logs.
Includes options for date range filtering.

Fixes #123
```

### 5. Push & Create PR

```bash
git push origin feature/my-feature
```

Then create a Pull Request on GitHub with:
- Clear title
- Description of changes
- Link to related issue (if any)
- Screenshots/GIFs (if UI changes)
- Testing notes

---

## Code Style Guidelines

### Rust

**General Principles:**
- Use `cargo fmt` (mandatory)
- Fix `cargo clippy` warnings
- Use meaningful variable names
- Add doc comments for public functions
- Write tests for new functionality

**Example:**
```rust
/// Extracts the request body and limits size.
///
/// # Arguments
/// * `req` - The HTTP request
/// * `max_size` - Maximum bytes to read
///
/// # Returns
/// The body bytes, truncated if necessary
pub async fn extract_body(
    req: HttpRequest,
    max_size: usize,
) -> Result<Vec<u8>, Error> {
    // implementation
}

#[test]
fn test_extract_body_truncates() {
    // test code
}
```

### Vue.js

**General Principles:**
- Use `<script setup>` (Vue 3 Composition API)
- Use `ref()` and `computed()` for state
- Add prop types with JSDoc
- Meaningful component names
- CSS using scoped styles

**Example:**
```vue
<template>
  <div class="log-item" :class="statusClass">
    <span>{{ method }}</span>
    <span>{{ path }}</span>
    <span>{{ statusCode }}</span>
  </div>
</template>

<script setup>
import { computed } from 'vue';

const props = defineProps({
  method: String,
  path: String,
  statusCode: Number
});

const statusClass = computed(() => {
  if (props.statusCode >= 500) return 'error';
  if (props.statusCode >= 400) return 'warning';
  return 'success';
});
</script>

<style scoped>
.log-item {
  display: flex;
  gap: 1rem;
  padding: 0.5rem;
}
.error { color: red; }
.warning { color: orange; }
.success { color: green; }
</style>
```

---

## Pull Request Process

### Before Submitting

1. **Update from main**
```bash
git fetch origin
git rebase origin/main
```

2. **Run all checks**
```bash
cargo fmt
cargo clippy
cargo test

cd dashboard
npm run lint
npm run build
cd ..

bash tests/quick_test.sh
```

3. **Update documentation**
- If adding feature, update README.md
- If changing config, update CONFIGURATION.md
- If changing architecture, update ARCHITECTURE.md

### PR Checklist

- [ ] Commits follow conventional commits format
- [ ] Code follows style guidelines
- [ ] Tests pass locally
- [ ] New tests added for new functionality
- [ ] Documentation updated
- [ ] No breaking changes (or clearly documented)
- [ ] Issue is linked (if applicable)

### Review Process

- At least one maintainer review required
- CI checks must pass (GitHub Actions)
- All suggestions should be addressed
- Squash commits before merge (optional)

---

## Testing Guidelines

### Unit Tests (Rust)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry {
            request_id: "test-id".to_string(),
            method: "GET".to_string(),
            // ...
        };
        assert_eq!(entry.method, "GET");
    }

    #[tokio::test]
    async fn test_async_function() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

### Integration Tests

```bash
# Add to tests/ directory
# Follow patterns in tests/integration_test.py

# Run with:
bash tests/setup_test_env.sh
# or
python3 tests/integration_test.py
```

### Manual Testing

See [TESTING.md](TESTING.md) for comprehensive testing procedures.

---

## Performance Considerations

### Benchmarking

```bash
# Measure proxy latency
time curl -X POST http://localhost:3000/api/test \
  -H "Content-Type: application/json" \
  -d '{"key":"value"}'

# Measure database performance
sqlite3 endpoint-logs.db "SELECT COUNT(*) FROM logs;"

# Monitor memory
watch -n 1 'ps aux | grep endpoint-logger'
```

### Performance Goals

- Proxy overhead: < 10ms
- Database writes: 1000+ per second
- Dashboard update latency: < 100ms
- Memory per 1000 logs: 1MB
- Startup time: < 1 second

### Optimization Tips

1. Use async/await, not blocking operations
2. Batch database operations when possible
3. Limit in-memory log retention
4. Use indexes for frequently queried fields
5. Profile with `cargo flamegraph` if needed

---

## Documentation

### Writing Docs

1. **README.md** - User-facing overview
2. **CONFIGURATION.md** - All config options
3. **ARCHITECTURE.md** - System design details
4. **CONTRIBUTING.md** - Development guide (this file)
5. **TESTING.md** - Test procedures

### Code Comments

```rust
// Use /// for public APIs
/// Logs an HTTP request/response pair.
///
/// This is called internally by the proxy handler.
pub fn log_request(...) { }

// Use // for implementation details
// This is a workaround for...
let result = ...;
```

### Examples in Docs

```markdown
# Example Title

Use code fences with language:

\`\`\`bash
endpoint-logger --target http://localhost:8080
\`\`\`

\`\`\`rust
let logs = storage.get_recent(10).await?;
\`\`\`

\`\`\`javascript
const ws = new WebSocket('ws://localhost:3000/ws');
\`\`\`
```

---

## Release Process

### Version Numbering

We follow [Semantic Versioning](https://semver.org/):
- `MAJOR.MINOR.PATCH` (e.g., `1.2.3`)
- MAJOR: Breaking changes
- MINOR: New features (backwards compatible)
- PATCH: Bug fixes

### Release Checklist

1. Update version in `Cargo.toml`
2. Update version in `dashboard/package.json`
3. Create `CHANGELOG.md` entry
4. Build release binary: `cargo build --release`
5. Create git tag: `git tag v1.2.3`
6. Push tag: `git push origin v1.2.3`
7. GitHub Actions creates release automatically

---

## Getting Help

### Questions?

- **GitHub Issues** - Bug reports and feature requests
- **GitHub Discussions** - General questions
- **Code Comments** - Ask in PRs if confused

### Common Issues

**Cargo build fails:**
```bash
# Clean and rebuild
cargo clean
cargo build
```

**Node dependencies issue:**
```bash
cd dashboard
rm -rf node_modules package-lock.json
npm install
```

**Port already in use:**
```bash
# Find process using port
lsof -i :3000

# Kill it
kill -9 <PID>
```

**WebSocket connection fails:**
```bash
# Check backend is running
curl http://localhost:3000/

# Check browser console for errors
# Enable verbose mode in backend
endpoint-logger --verbose
```

---

## Recognition

Contributors will be recognized in:
- `CONTRIBUTORS.md` file
- Release notes
- GitHub contributors page

### Types of Contributions We Appreciate

- 🐛 Bug fixes
- ✨ New features
- 📚 Documentation improvements
- 🧪 Test additions
- 🎨 UI/UX improvements
- ⚡ Performance optimizations
- 🔒 Security improvements
- 🌐 Internationalization
- 📦 Dependency updates
- ♿ Accessibility improvements

---

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inspiring community for all. Please read our [Code of Conduct](CODE_OF_CONDUCT.md).

### Reporting Issues

If you encounter unacceptable behavior, please report it to the maintainers at [contact information].

---

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

## Questions?

Feel free to reach out:

- **GitHub Issues**: For bugs and features
- **GitHub Discussions**: For questions
- **Email**: [maintainer email]

Thank you for contributing! 🎉

---

## Additional Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Axum Documentation](https://docs.rs/axum/latest/axum/)
- [Vue.js Guide](https://vuejs.org/guide/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [SQLx Documentation](https://docs.rs/sqlx/latest/sqlx/)

**Happy coding! 🚀**
