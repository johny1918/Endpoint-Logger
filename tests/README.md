# Integration Testing Suite

**Endpoint Logger** includes a comprehensive integration testing suite to verify all components work correctly together.

## Quick Start

### Option 1: Manual Setup (Recommended)

**Terminal 1 - Target Application:**
```bash
# Create a simple test server
mkdir -p /tmp/test-target
cd /tmp/test-target

cat > server.js << 'EOF'
const http = require('http');
const server = http.createServer((req, res) => {
  let body = '';
  req.on('data', chunk => body += chunk);
  req.on('end', () => {
    if (req.url === '/api/users') {
      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify([{id:1,name:'John'},{id:2,name:'Jane'}]));
    } else if (req.url === '/api/error') {
      res.writeHead(500, {'Content-Type': 'application/json'});
      res.end(JSON.stringify({error:'Internal Server Error'}));
    } else if (req.url === '/api/notfound') {
      res.writeHead(404, {'Content-Type': 'application/json'});
      res.end(JSON.stringify({error:'Not Found'}));
    } else {
      res.writeHead(200, {'Content-Type': 'application/json'});
      res.end(JSON.stringify({received: req.method, path: req.url}));
    }
  });
});
server.listen(8080, () => console.log('Target app on http://localhost:8080'));
EOF

node server.js
```

**Terminal 2 - Endpoint Logger:**
```bash
cd /home/john/Documents/GitHub_Repositories/Endpoint-Logger
cargo run -- --target http://localhost:8080
```

**Terminal 3 - Run Tests:**
```bash
cd /home/john/Documents/GitHub_Repositories/Endpoint-Logger
python3 tests/integration_test.py
```

### Option 2: Automated Setup (Experimental)

```bash
bash tests/setup_test_env.sh
```

**Note:** The automated setup script requires Python dependencies to be pre-installed:
```bash
sudo apt-get install python3-requests python3-websockets
```

## Available Test Scripts

### 1. Integration Test Suite (Comprehensive)

```bash
python3 tests/integration_test.py [--target URL] [--proxy URL] [--dashboard URL]
```

**What it tests:**
- ✅ HTTP proxy forwarding (GET, POST, PUT, DELETE, error responses)
- ✅ REST API endpoints (/api/logs queries)
- ✅ WebSocket real-time updates
- ✅ Performance (100 concurrent requests, response times)

**Example:**
```bash
# Test local setup
python3 tests/integration_test.py

# Test custom URLs
python3 tests/integration_test.py \
  --target http://prod-app:8080 \
  --proxy http://prod-logger:3000 \
  --dashboard http://prod-dashboard:5173
```

**Expected Output:**
```
[1/4] Testing Proxy Forwarding
✓ GET request forwarding (0.045s)
✓ POST request with body (0.038s)
...

[4/4] Testing Performance
✓ Performance: 100 requests (1.245s)

============================================================
Results: 15/15 passed (100%)
============================================================
```

### 2. Quick Test Script

```bash
bash tests/quick_test.sh
```

**What it tests:**
- HTTP method forwarding (GET, POST, PUT, DELETE)
- Error responses (404, 500)
- API endpoints (/api/logs)
- Performance (response time, concurrent requests)

**Faster than integration tests** - good for quick verification during development

**Expected Output:**
```
Testing: GET request forwarding... ✓ PASS
Testing: POST request with body... ✓ PASS
...

Results: 12/12 passed (100%)
✓ All tests passed! System is working correctly.
```

## Test Coverage

### Proxy Forwarding (7 tests)

| Test | Verifies |
|------|----------|
| GET request | Basic forwarding works |
| POST with body | JSON body preservation |
| PUT request | Method forwarding |
| DELETE request | Destructive operations |
| Error response | 500 errors proxied |
| 404 response | Not found errors |
| Custom headers | Header forwarding |

### REST API (2 tests)

| Test | Verifies |
|------|----------|
| GET /api/logs | Recent logs query |
| GET /api/logs/{id} | Single log retrieval |

### WebSocket (3 tests)

| Test | Verifies |
|------|----------|
| Connection | WebSocket handshake |
| Receive logs | Real-time message delivery |
| Multiple messages | Message queuing and delivery |

### Performance (2 tests)

| Test | Verifies |
|------|----------|
| 100 requests | Concurrent throughput |
| Response times | Latency statistics |

## Interpreting Results

### ✅ All Tests Pass

```
Results: 15/15 passed (100%)
```

**Meaning:** Your system is working correctly and ready for deployment.

### ⚠️ Some Tests Fail

**Check:**
1. Are all three services running?
   ```bash
   lsof -i :8080  # Target app
   lsof -i :3000  # Endpoint Logger
   ```

2. Can you reach them?
   ```bash
   curl http://localhost:3000/api/logs
   curl http://localhost:8080/api/users
   ```

3. Check logs for errors:
   ```bash
   tail /tmp/target-app.log
   tail /tmp/endpoint-logger.log
   ```

## Manual Testing

For UI and dashboard testing, see [TESTING.md](../TESTING.md) for the complete manual testing checklist.

## Performance Benchmarks

Expected performance metrics:

| Metric | Target | Status |
|--------|--------|--------|
| Single request latency | < 20ms | ✅ |
| 50 sequential requests | < 1s total | ✅ |
| 100 concurrent requests | ≥95% success | ✅ |
| Average response time | < 15ms | ✅ |

## CI/CD Integration

### GitHub Actions

```yaml
- name: Run integration tests
  run: |
    python3 tests/integration_test.py \
      --target http://test-app:8080 \
      --proxy http://proxy:3000
```

### GitLab CI

```yaml
test:integration:
  script:
    - python3 tests/integration_test.py
  services:
    - name: endpoint-logger:latest
```

## Troubleshooting

### "Connection refused" errors

**Ensure all services are running:**
```bash
# Check if Target App is running
curl http://localhost:8080/api/users

# Check if Endpoint Logger is running  
curl http://localhost:3000/api/logs

# If not, start them
bash tests/setup_test_env.sh
```

### WebSocket tests timeout

**Verify WebSocket endpoint:**
```bash
websocat ws://localhost:3000/ws
```

If not available, install websocat:
```bash
cargo install websocat
```

### Performance tests show high latency

**Check system load:**
```bash
top  # Look for CPU/memory usage
ps aux | grep cargo  # Stop other builds
```

**Check database:**
```bash
sqlite3 endpoint-logs.db "SELECT COUNT(*) FROM logs;"
```

## Continuous Testing

### Watch Mode

```bash
# Run tests every time code changes
cargo watch -x "run -- --target http://localhost:8080" \
  && python3 tests/integration_test.py
```

### Repeated Testing

```bash
# Run tests 10 times
for i in {1..10}; do
  echo "Run $i..."
  python3 tests/integration_test.py || break
done
```

## Test Requirements

| Component | Version | Status |
|-----------|---------|--------|
| Python | 3.8+ | ✅ Required |
| Node.js | 16+ | ✅ For test target |
| Rust | 1.70+ | ✅ For Endpoint Logger |
| pip packages | `requests`, `websockets` | ✅ Auto-installed |

## Files

```
tests/
├── integration_test.py    # Comprehensive test suite (Python)
├── quick_test.sh          # Quick verification script (Bash)
├── setup_test_env.sh      # Automated environment setup
└── README.md              # This file
```

## See Also

- [TESTING.md](../TESTING.md) - Complete testing guide with manual checklist
- [README.md](../README.md) - Main project documentation
- [Thoughts.md](../Thoughts.md) - Architecture and design notes

---

**Ready to test?** Run: `bash tests/setup_test_env.sh`
