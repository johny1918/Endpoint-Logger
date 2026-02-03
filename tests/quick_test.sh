#!/bin/bash
# Quick Manual Test Script for Endpoint Logger
# Usage: ./tests/quick_test.sh

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

PROXY_URL="${PROXY_URL:-http://localhost:3000}"
TARGET_URL="${TARGET_URL:-http://localhost:8080}"

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Endpoint Logger - Quick Test Script                       ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Configuration:"
echo "  Proxy Target: $PROXY_URL"
echo "  Target App:  $TARGET_URL"
echo ""

# Track test results
PASSED=0
FAILED=0

test_case() {
    local name=$1
    local command=$2
    local expected=$3
    
    echo -ne "Testing: $name... "
    
    result=$(eval "$command" 2>&1)
    
    if echo "$result" | grep -q "$expected"; then
        echo -e "${GREEN}✓ PASS${NC}"
        ((PASSED++))
    else
        echo -e "${RED}✗ FAIL${NC}"
        echo "  Expected: $expected"
        echo "  Got: $result"
        ((FAILED++))
    fi
}

# ============================================================================
# PROXY TESTS
# ============================================================================

echo -e "${BLUE}[1] PROXY FORWARDING TESTS${NC}"

test_case \
    "GET request forwarding" \
    "curl -s $PROXY_URL/api/users | head -c 50" \
    "^\\[.*\\]"

test_case \
    "POST request with body" \
    "curl -s -X POST $PROXY_URL/api/data -H 'Content-Type: application/json' -d '{\"test\":true}' | head -c 50" \
    "received|test|true"

test_case \
    "PUT request forwarding" \
    "curl -s -w '%{http_code}' -o /dev/null -X PUT $PROXY_URL/api/users/1 -d 'test'" \
    "200|404"

test_case \
    "DELETE request forwarding" \
    "curl -s -w '%{http_code}' -o /dev/null -X DELETE $PROXY_URL/api/users/1" \
    "200|204|404"

test_case \
    "Error response forwarding" \
    "curl -s -w '%{http_code}' -o /dev/null $PROXY_URL/api/error" \
    "500"

test_case \
    "404 response forwarding" \
    "curl -s -w '%{http_code}' -o /dev/null $PROXY_URL/api/notfound" \
    "404"

# ============================================================================
# API TESTS
# ============================================================================

echo ""
echo -e "${BLUE}[2] REST API TESTS${NC}"

# Wait for logs to be saved
sleep 1

test_case \
    "GET /api/logs returns array" \
    "curl -s '$PROXY_URL/api/logs?limit=5' | head -c 50" \
    "^\\[.*\\]"

test_case \
    "GET /api/logs has request_id" \
    "curl -s '$PROXY_URL/api/logs?limit=1'" \
    "request_id"

test_case \
    "GET /api/logs has method" \
    "curl -s '$PROXY_URL/api/logs?limit=1'" \
    "\"method\""

test_case \
    "GET /api/logs has status" \
    "curl -s '$PROXY_URL/api/logs?limit=1'" \
    "\"status\""

# ============================================================================
# RESPONSE TIME TESTS
# ============================================================================

echo ""
echo -e "${BLUE}[3] PERFORMANCE TESTS${NC}"

echo -n "Testing: Response time (should be < 50ms)... "
total_time=0
count=10
for i in $(seq 1 $count); do
    start_time=$(date +%s%N | cut -b1-13)
    curl -s "$PROXY_URL/api/users" > /dev/null
    end_time=$(date +%s%N | cut -b1-13)
    elapsed=$((end_time - start_time))
    total_time=$((total_time + elapsed))
done
avg_time=$((total_time / count))

if [ $avg_time -lt 50 ]; then
    echo -e "${GREEN}✓ PASS${NC} (avg: ${avg_time}ms)"
    ((PASSED++))
else
    echo -e "${YELLOW}⚠ WARN${NC} (avg: ${avg_time}ms - higher than expected)"
fi

# ============================================================================
# CONCURRENT REQUEST TEST
# ============================================================================

echo -n "Testing: 20 concurrent requests... "
failed=0
for i in $(seq 1 20); do
    (curl -s "$PROXY_URL/api/users" > /dev/null) || ((failed++)) &
done
wait

if [ $failed -eq 0 ]; then
    echo -e "${GREEN}✓ PASS${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ FAIL${NC} ($failed failures)"
    ((FAILED++))
fi

# ============================================================================
# SUMMARY
# ============================================================================

echo ""
total=$((PASSED + FAILED))
percentage=$((PASSED * 100 / total))

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Test Summary                                              ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Results: $PASSED/$total passed (${percentage}%)"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed! System is working correctly.${NC}"
    exit 0
else
    echo -e "${RED}✗ $FAILED test(s) failed. Please review errors above.${NC}"
    exit 1
fi
