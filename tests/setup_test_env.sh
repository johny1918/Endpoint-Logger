#!/bin/bash
# Test Environment Setup Script
# Sets up all three services needed for integration testing
# Usage: ./tests/setup_test_env.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "╔════════════════════════════════════════════════════════════╗"
echo "║  Endpoint Logger - Test Environment Setup                  ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Cleanup on exit
cleanup() {
    echo ""
    echo -e "${YELLOW}Cleaning up...${NC}"
    jobs -p | xargs -r kill 2>/dev/null || true
    exit 0
}

trap cleanup EXIT INT TERM

# ============================================================================
# CHECK PREREQUISITES
# ============================================================================

echo -e "${BLUE}Checking prerequisites...${NC}"

# Check Node.js
if ! command -v node &> /dev/null; then
    echo "✗ Node.js not found. Install with: curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash - && sudo apt-get install -y nodejs"
    exit 1
fi
echo -e "${GREEN}✓${NC} Node.js $(node --version)"

# Check Rust/Cargo
if ! command -v cargo &> /dev/null; then
    echo "✗ Cargo not found. Install Rust from https://rustup.rs/"
    exit 1
fi
echo -e "${GREEN}✓${NC} Cargo $(cargo --version | cut -d' ' -f2)"

# Check Python
if ! command -v python3 &> /dev/null; then
    echo "✗ Python 3 not found. Install with: sudo apt-get install python3"
    exit 1
fi
echo -e "${GREEN}✓${NC} Python $(python3 --version | cut -d' ' -f2)"

# Check pip dependencies
echo -n "Checking Python dependencies... "
if ! python3 -c "import requests, websockets" 2>/dev/null; then
    echo -e "${YELLOW}missing${NC}"
    echo -e "${YELLOW}⚠ Python packages 'requests' and 'websockets' are required${NC}"
    echo ""
    echo "Install with one of these commands:"
    echo "  sudo apt-get install python3-requests python3-websockets"
    echo "  pip install requests websockets"
    echo "  pip3 install requests websockets"
    echo ""
    echo "Then run: bash tests/setup_test_env.sh"
    exit 1
else
    echo -e "${GREEN}✓${NC}"
fi

echo ""

# ============================================================================
# SETUP TEST TARGET APP
# ============================================================================

echo -e "${BLUE}[1/3] Setting up target application (port 8080)...${NC}"

TEST_TARGET_DIR="/tmp/endpoint-logger-test-target"
rm -rf "$TEST_TARGET_DIR"
mkdir -p "$TEST_TARGET_DIR"

cat > "$TEST_TARGET_DIR/server.js" << 'EOF'
const http = require('http');

const server = http.createServer((req, res) => {
  let body = '';
  req.on('data', chunk => body += chunk);
  req.on('end', () => {
    console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
    
    // Parse body if exists
    let parsedBody = {};
    if (body) {
      try {
        parsedBody = JSON.parse(body);
      } catch (e) {
        parsedBody = { raw: body };
      }
    }
    
    // Route handling
    if (req.url.startsWith('/api/users')) {
      if (req.method === 'DELETE') {
        res.writeHead(204);
        res.end();
      } else if (req.method === 'PUT') {
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ id: 1, ...parsedBody }));
      } else {
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify([
          { id: 1, name: 'John Doe', email: 'john@example.com' },
          { id: 2, name: 'Jane Smith', email: 'jane@example.com' }
        ]));
      }
    } else if (req.url === '/api/data') {
      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ received: parsedBody, timestamp: Date.now() }));
    } else if (req.url === '/api/error') {
      res.writeHead(500, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ error: 'Internal Server Error', code: 'INTERNAL_ERROR' }));
    } else if (req.url === '/api/notfound') {
      res.writeHead(404, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ error: 'Not Found', path: req.url }));
    } else if (req.url.startsWith('/test/')) {
      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ test: true, path: req.url }));
    } else {
      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ 
        status: 'ok',
        message: 'Test server running',
        method: req.method,
        path: req.url,
        timestamp: new Date().toISOString()
      }));
    }
  });
});

server.listen(8080, '127.0.0.1', () => {
  console.log('Target application running on http://localhost:8080');
});

server.on('error', (err) => {
  console.error('Server error:', err);
  process.exit(1);
});

process.on('SIGTERM', () => {
  console.log('Target server shutting down...');
  server.close(() => process.exit(0));
});
EOF

cd "$TEST_TARGET_DIR"
node server.js > /tmp/target-app.log 2>&1 &
TARGET_PID=$!
echo -e "${GREEN}✓${NC} Target app started (PID: $TARGET_PID)"
sleep 1

# Verify target app
if ! curl -s http://localhost:8080/api/users > /dev/null 2>&1; then
    echo "✗ Target app failed to start"
    cat /tmp/target-app.log
    kill $TARGET_PID 2>/dev/null || true
    exit 1
fi

echo ""

# ============================================================================
# BUILD AND RUN ENDPOINT LOGGER
# ============================================================================

echo -e "${BLUE}[2/3] Building and starting Endpoint Logger (port 3000)...${NC}"

cd "$PROJECT_ROOT"

# Check if already built
if [ ! -f "target/debug/endpoint_logger" ]; then
    echo "  Building Rust backend..."
    cargo build --quiet 2>&1 | grep -v "Compiling endpoint_logger" | head -5
fi

# Start the logger
cargo run --quiet -- --target http://localhost:8080 > /tmp/endpoint-logger.log 2>&1 &
LOGGER_PID=$!
echo -e "${GREEN}✓${NC} Endpoint Logger started (PID: $LOGGER_PID)"
sleep 2

# Verify logger
if ! curl -s http://localhost:3000/api/logs > /dev/null 2>&1; then
    echo "✗ Endpoint Logger failed to start"
    cat /tmp/endpoint-logger.log
    kill $LOGGER_PID 2>/dev/null || true
    kill $TARGET_PID 2>/dev/null || true
    exit 1
fi

echo ""

# ============================================================================
# VERIFY ALL SERVICES
# ============================================================================

echo -e "${BLUE}[3/3] Verifying services...${NC}"

sleep 1

services=(
    "Target App (8080)"
    "Endpoint Logger (3000)"
)

for service in "${services[@]}"; do
    name="${service%% *}"
    port="${service##*( )}"
    port="${port%\)}"
    
    if curl -s "http://localhost:${port%%)}/api" > /dev/null 2>&1 || curl -s "http://localhost:${port%%)}/api/logs" > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC} $service online"
    else
        echo -e "${YELLOW}⚠${NC} $service may not be responding"
    fi
done

echo ""

# ============================================================================
# TEST QUICK CONNECTION
# ============================================================================

echo -e "${BLUE}Testing quick connectivity...${NC}"

# Send a test request
curl -s -o /dev/null http://localhost:3000/api/users
sleep 0.5

# Check if it was logged
if curl -s http://localhost:3000/api/logs | grep -q "api/users"; then
    echo -e "${GREEN}✓${NC} End-to-end connectivity verified"
else
    echo -e "${YELLOW}⚠${NC} Could not verify end-to-end connectivity"
fi

echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo -e "${GREEN}✓ Test environment ready!${NC}"
echo ""
echo "In another terminal, you can now run:"
echo -e "  ${BLUE}cd $PROJECT_ROOT${NC}"
echo -e "  ${BLUE}python3 tests/integration_test.py${NC}"
echo "or:"
echo -e "  ${BLUE}bash tests/quick_test.sh${NC}"
echo ""
echo "Services running (press Ctrl+C to stop all):"
echo "  - Target App:       http://localhost:8080"
echo "  - Endpoint Logger:  http://localhost:3000"
echo "  - API:              http://localhost:3000/api/logs"
echo "  - WebSocket:        ws://localhost:3000/ws"
echo ""
echo "Logs:"
echo "  - Target App:       /tmp/target-app.log"
echo "  - Endpoint Logger:  /tmp/endpoint-logger.log"
echo ""

# Keep running
wait $TARGET_PID $LOGGER_PID 2>/dev/null
