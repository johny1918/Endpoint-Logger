#!/usr/bin/env python3
"""
Integration Test Suite for Endpoint Logger

Tests all major user flows:
1. Proxy forwarding and logging
2. WebSocket real-time updates
3. REST API queries
4. Dashboard display and functionality

Usage:
    python3 integration_test.py [--target http://localhost:8080] [--proxy http://localhost:3000]
"""

import sys
import time
import json
import asyncio
import argparse
import subprocess
from datetime import datetime
from typing import Dict, List, Any, Optional
import requests
import websockets
from statistics import mean, stdev

# Color codes for terminal output
class Colors:
    RESET = '\033[0m'
    GREEN = '\033[92m'
    RED = '\033[91m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    CYAN = '\033[96m'

class TestResult:
    """Tracks test results"""
    def __init__(self):
        self.passed = 0
        self.failed = 0
        self.tests: List[Dict[str, Any]] = []
    
    def add_pass(self, name: str, duration: float):
        self.passed += 1
        self.tests.append({
            'name': name,
            'status': 'PASS',
            'duration': duration
        })
    
    def add_fail(self, name: str, error: str, duration: float):
        self.failed += 1
        self.tests.append({
            'name': name,
            'status': 'FAIL',
            'error': error,
            'duration': duration
        })
    
    def summary(self) -> str:
        total = self.passed + self.failed
        pct = (self.passed / total * 100) if total > 0 else 0
        return f"\n{'='*60}\nResults: {self.passed}/{total} passed ({pct:.1f}%)\n{'='*60}"

class ProxyTester:
    """Test proxy forwarding and logging"""
    
    def __init__(self, proxy_url: str, target_url: str):
        self.proxy_url = proxy_url.rstrip('/')
        self.target_url = target_url.rstrip('/')
        self.results = TestResult()
        self.logged_requests: List[Dict] = []
    
    def test_get_request(self):
        """Test GET request forwarding"""
        print(f"{Colors.BLUE}Testing GET request forwarding...{Colors.RESET}")
        start = time.time()
        try:
            response = requests.get(f"{self.proxy_url}/api/users")
            duration = time.time() - start
            
            if response.status_code == 200:
                self.results.add_pass("GET request forwarding", duration)
                print(f"  {Colors.GREEN}✓{Colors.RESET} GET request succeeded (status: {response.status_code})")
            else:
                self.results.add_fail("GET request forwarding", 
                                     f"Expected 200, got {response.status_code}", duration)
        except Exception as e:
            duration = time.time() - start
            self.results.add_fail("GET request forwarding", str(e), duration)
            print(f"  {Colors.RED}✗{Colors.RESET} {str(e)}")
    
    def test_post_request_with_body(self):
        """Test POST request with JSON body"""
        print(f"{Colors.BLUE}Testing POST request with body...{Colors.RESET}")
        start = time.time()
        try:
            data = {"name": "Test User", "email": "test@example.com"}
            response = requests.post(
                f"{self.proxy_url}/api/data",
                json=data,
                headers={"Content-Type": "application/json"}
            )
            duration = time.time() - start
            
            if response.status_code == 200:
                self.results.add_pass("POST request with body", duration)
                print(f"  {Colors.GREEN}✓{Colors.RESET} POST request succeeded")
            else:
                self.results.add_fail("POST request with body",
                                     f"Unexpected status: {response.status_code}", duration)
        except Exception as e:
            duration = time.time() - start
            self.results.add_fail("POST request with body", str(e), duration)
    
    def test_put_request(self):
        """Test PUT request"""
        print(f"{Colors.BLUE}Testing PUT request...{Colors.RESET}")
        start = time.time()
        try:
            response = requests.put(
                f"{self.proxy_url}/api/users/1",
                json={"name": "Updated"}
            )
            duration = time.time() - start
            
            if response.status_code in [200, 404]:
                self.results.add_pass("PUT request forwarding", duration)
                print(f"  {Colors.GREEN}✓{Colors.RESET} PUT request forwarded correctly")
            else:
                self.results.add_fail("PUT request forwarding",
                                     f"Unexpected status: {response.status_code}", duration)
        except Exception as e:
            duration = time.time() - start
            self.results.add_fail("PUT request forwarding", str(e), duration)
    
    def test_delete_request(self):
        """Test DELETE request"""
        print(f"{Colors.BLUE}Testing DELETE request...{Colors.RESET}")
        start = time.time()
        try:
            response = requests.delete(f"{self.proxy_url}/api/users/1")
            duration = time.time() - start
            
            if response.status_code in [200, 204, 404]:
                self.results.add_pass("DELETE request forwarding", duration)
                print(f"  {Colors.GREEN}✓{Colors.RESET} DELETE request forwarded")
            else:
                self.results.add_fail("DELETE request forwarding",
                                     f"Unexpected status: {response.status_code}", duration)
        except Exception as e:
            duration = time.time() - start
            self.results.add_fail("DELETE request forwarding", str(e), duration)
    
    def test_error_responses(self):
        """Test that error responses are proxied correctly"""
        print(f"{Colors.BLUE}Testing error response forwarding...{Colors.RESET}")
        start = time.time()
        try:
            response = requests.get(f"{self.proxy_url}/api/error")
            duration = time.time() - start
            
            if response.status_code == 500:
                self.results.add_pass("Error response forwarding", duration)
                print(f"  {Colors.GREEN}✓{Colors.RESET} 500 error forwarded correctly")
            else:
                self.results.add_fail("Error response forwarding",
                                     f"Expected 500, got {response.status_code}", duration)
        except Exception as e:
            duration = time.time() - start
            self.results.add_fail("Error response forwarding", str(e), duration)
    
    def test_404_response(self):
        """Test 404 response handling"""
        print(f"{Colors.BLUE}Testing 404 response...{Colors.RESET}")
        start = time.time()
        try:
            response = requests.get(f"{self.proxy_url}/api/notfound")
            duration = time.time() - start
            
            if response.status_code == 404:
                self.results.add_pass("404 response forwarding", duration)
                print(f"  {Colors.GREEN}✓{Colors.RESET} 404 response handled correctly")
            else:
                self.results.add_fail("404 response forwarding",
                                     f"Expected 404, got {response.status_code}", duration)
        except Exception as e:
            duration = time.time() - start
            self.results.add_fail("404 response forwarding", str(e), duration)
    
    def test_custom_headers_forwarded(self):
        """Test that custom headers are forwarded"""
        print(f"{Colors.BLUE}Testing custom headers forwarding...{Colors.RESET}")
        start = time.time()
        try:
            headers = {
                "X-Custom-Header": "test-value",
                "Authorization": "Bearer test-token"
            }
            response = requests.get(
                f"{self.proxy_url}/api/users",
                headers=headers
            )
            duration = time.time() - start
            
            if response.status_code == 200:
                self.results.add_pass("Custom headers forwarding", duration)
                print(f"  {Colors.GREEN}✓{Colors.RESET} Headers forwarded successfully")
            else:
                self.results.add_fail("Custom headers forwarding",
                                     f"Failed with status {response.status_code}", duration)
        except Exception as e:
            duration = time.time() - start
            self.results.add_fail("Custom headers forwarding", str(e), duration)

class APITester:
    """Test REST API endpoints"""
    
    def __init__(self, proxy_url: str):
        self.proxy_url = proxy_url.rstrip('/')
        self.results = TestResult()
    
    def test_get_recent_logs(self):
        """Test GET /api/logs endpoint"""
        print(f"{Colors.BLUE}Testing GET /api/logs...{Colors.RESET}")
        start = time.time()
        try:
            # First make a request to generate a log
            requests.get(f"{self.proxy_url}/api/users")
            time.sleep(0.5)  # Give time for logging
            
            # Query recent logs
            response = requests.get(f"{self.proxy_url}/api/logs?limit=10")
            duration = time.time() - start
            
            if response.status_code == 200:
                logs = response.json()
                if isinstance(logs, list) and len(logs) > 0:
                    self.results.add_pass("GET /api/logs", duration)
                    print(f"  {Colors.GREEN}✓{Colors.RESET} Retrieved {len(logs)} logs")
                else:
                    self.results.add_fail("GET /api/logs", "No logs returned", duration)
            else:
                self.results.add_fail("GET /api/logs",
                                     f"Status {response.status_code}", duration)
        except Exception as e:
            duration = time.time() - start
            self.results.add_fail("GET /api/logs", str(e), duration)
    
    def test_get_single_log(self):
        """Test GET /api/logs/{request_id}"""
        print(f"{Colors.BLUE}Testing GET /api/logs/{{id}}...{Colors.RESET}")
        start = time.time()
        try:
            # Get recent logs
            requests.get(f"{self.proxy_url}/api/users")
            time.sleep(0.5)
            
            response = requests.get(f"{self.proxy_url}/api/logs?limit=1")
            logs = response.json()
            
            if logs and 'request_id' in logs[0]:
                request_id = logs[0]['request_id']
                detail_response = requests.get(f"{self.proxy_url}/api/logs/{request_id}")
                duration = time.time() - start
                
                if detail_response.status_code == 200:
                    self.results.add_pass("GET /api/logs/{id}", duration)
                    print(f"  {Colors.GREEN}✓{Colors.RESET} Retrieved single log details")
                else:
                    self.results.add_fail("GET /api/logs/{id}",
                                         f"Status {detail_response.status_code}", duration)
            else:
                duration = time.time() - start
                self.results.add_fail("GET /api/logs/{id}", "No logs to query", duration)
        except Exception as e:
            duration = time.time() - start
            self.results.add_fail("GET /api/logs/{id}", str(e), duration)

class WebSocketTester:
    """Test WebSocket real-time updates"""
    
    def __init__(self, proxy_url: str):
        self.ws_url = proxy_url.rstrip('/').replace('http', 'ws') + '/ws'
        self.results = TestResult()
        self.received_messages: List[Dict] = []
    
    async def test_websocket_connection(self):
        """Test WebSocket connection"""
        print(f"{Colors.BLUE}Testing WebSocket connection...{Colors.RESET}")
        start = time.time()
        try:
            async with websockets.connect(self.ws_url) as websocket:
                # Connection successful
                self.results.add_pass("WebSocket connection", time.time() - start)
                print(f"  {Colors.GREEN}✓{Colors.RESET} WebSocket connected successfully")
                return websocket
        except Exception as e:
            duration = time.time() - start
            self.results.add_fail("WebSocket connection", str(e), duration)
            print(f"  {Colors.RED}✗{Colors.RESET} {str(e)}")
            return None
    
    async def test_websocket_receive_logs(self, timeout: int = 5):
        """Test receiving log updates via WebSocket"""
        print(f"{Colors.BLUE}Testing WebSocket receive logs...{Colors.RESET}")
        start = time.time()
        try:
            async with websockets.connect(self.ws_url) as websocket:
                # Make a request to trigger logging
                requests.get(f"http://localhost:3000/api/users")
                
                # Wait for message
                try:
                    message = await asyncio.wait_for(websocket.recv(), timeout=timeout)
                    duration = time.time() - start
                    
                    log = json.loads(message)
                    if 'request_id' in log:
                        self.results.add_pass("WebSocket receive logs", duration)
                        print(f"  {Colors.GREEN}✓{Colors.RESET} Received log via WebSocket")
                        self.received_messages.append(log)
                    else:
                        self.results.add_fail("WebSocket receive logs",
                                             "Invalid message format", duration)
                except asyncio.TimeoutError:
                    duration = time.time() - start
                    self.results.add_fail("WebSocket receive logs",
                                         "No message received (timeout)", duration)
        except Exception as e:
            duration = time.time() - start
            self.results.add_fail("WebSocket receive logs", str(e), duration)
    
    async def test_websocket_multiple_messages(self, count: int = 3):
        """Test receiving multiple messages via WebSocket"""
        print(f"{Colors.BLUE}Testing WebSocket multiple messages ({count})...{Colors.RESET}")
        start = time.time()
        try:
            async with websockets.connect(self.ws_url) as websocket:
                # Make multiple requests
                for i in range(count):
                    requests.get(f"http://localhost:3000/test/{i}")
                    await asyncio.sleep(0.1)
                
                # Collect messages
                messages = []
                try:
                    for _ in range(count):
                        message = await asyncio.wait_for(websocket.recv(), timeout=2)
                        messages.append(json.loads(message))
                except asyncio.TimeoutError:
                    pass
                
                duration = time.time() - start
                if len(messages) >= count - 1:
                    self.results.add_pass("WebSocket multiple messages", duration)
                    print(f"  {Colors.GREEN}✓{Colors.RESET} Received {len(messages)} messages")
                else:
                    self.results.add_fail("WebSocket multiple messages",
                                         f"Expected {count}, got {len(messages)}", duration)
        except Exception as e:
            duration = time.time() - start
            self.results.add_fail("WebSocket multiple messages", str(e), duration)

class PerformanceTester:
    """Test system performance"""
    
    def __init__(self, proxy_url: str):
        self.proxy_url = proxy_url.rstrip('/')
        self.results = TestResult()
    
    def test_100_requests(self):
        """Test proxy with 100 concurrent requests"""
        print(f"{Colors.BLUE}Testing performance (100 requests)...{Colors.RESET}")
        start = time.time()
        
        try:
            import concurrent.futures
            
            def make_request(i):
                try:
                    r = requests.get(f"{self.proxy_url}/api/users")
                    return r.status_code == 200
                except:
                    return False
            
            with concurrent.futures.ThreadPoolExecutor(max_workers=10) as executor:
                futures = [executor.submit(make_request, i) for i in range(100)]
                successes = sum(1 for f in concurrent.futures.as_completed(futures) if f.result())
            
            duration = time.time() - start
            rate = 100 / duration
            
            if successes >= 95:  # 95% success rate
                self.results.add_pass("Performance: 100 requests", duration)
                print(f"  {Colors.GREEN}✓{Colors.RESET} 100 requests in {duration:.2f}s ({rate:.1f} req/s)")
            else:
                self.results.add_fail("Performance: 100 requests",
                                     f"Only {successes}/100 succeeded", duration)
        except Exception as e:
            duration = time.time() - start
            self.results.add_fail("Performance: 100 requests", str(e), duration)
    
    def test_response_times(self, count: int = 50):
        """Measure response time statistics"""
        print(f"{Colors.BLUE}Testing response times ({count} requests)...{Colors.RESET}")
        start = time.time()
        
        try:
            times = []
            for i in range(count):
                req_start = time.time()
                requests.get(f"{self.proxy_url}/api/users")
                times.append((time.time() - req_start) * 1000)  # Convert to ms
            
            duration = time.time() - start
            avg = mean(times)
            
            if len(times) > 1:
                std_dev = stdev(times)
                print(f"    Avg: {avg:.2f}ms | Min: {min(times):.2f}ms | Max: {max(times):.2f}ms | StdDev: {std_dev:.2f}ms")
            else:
                print(f"    Response time: {avg:.2f}ms")
            
            self.results.add_pass("Response time statistics", duration)
        except Exception as e:
            duration = time.time() - start
            self.results.add_fail("Response time statistics", str(e), duration)

class DashboardTester:
    """Test dashboard functionality"""
    
    def __init__(self, proxy_url: str, dashboard_url: str):
        self.proxy_url = proxy_url.rstrip('/')
        self.dashboard_url = dashboard_url.rstrip('/')
        self.results = TestResult()
    
    def test_dashboard_loads(self):
        """Test dashboard page loads"""
        print(f"{Colors.BLUE}Testing dashboard loads...{Colors.RESET}")
        start = time.time()
        try:
            response = requests.get(f"{self.dashboard_url}/")
            duration = time.time() - start
            
            if response.status_code == 200 and 'html' in response.text.lower():
                self.results.add_pass("Dashboard page loads", duration)
                print(f"  {Colors.GREEN}✓{Colors.RESET} Dashboard loads successfully")
            else:
                self.results.add_fail("Dashboard page loads",
                                     f"Status {response.status_code}", duration)
        except Exception as e:
            duration = time.time() - start
            self.results.add_fail("Dashboard page loads", str(e), duration)
    
    def test_api_routes_respond(self):
        """Test that API routes respond correctly"""
        print(f"{Colors.BLUE}Testing API routes respond...{Colors.RESET}")
        
        routes = ['/api/logs', '/api/logs?limit=10']
        all_pass = True
        
        for route in routes:
            start = time.time()
            try:
                response = requests.get(f"{self.proxy_url}{route}")
                duration = time.time() - start
                
                if response.status_code == 200:
                    print(f"  {Colors.GREEN}✓{Colors.RESET} {route}")
                else:
                    print(f"  {Colors.RED}✗{Colors.RESET} {route} (status {response.status_code})")
                    all_pass = False
            except Exception as e:
                print(f"  {Colors.RED}✗{Colors.RESET} {route} ({str(e)})")
                all_pass = False
        
        if all_pass:
            self.results.add_pass("API routes respond", 0.1)
        else:
            self.results.add_fail("API routes respond", "Some routes failed", 0.1)

async def main():
    """Main test runner"""
    parser = argparse.ArgumentParser(description='Integration tests for Endpoint Logger')
    parser.add_argument('--target', default='http://localhost:8080',
                       help='Target application URL')
    parser.add_argument('--proxy', default='http://localhost:3000',
                       help='Endpoint Logger proxy URL')
    parser.add_argument('--dashboard', default='http://localhost:5173',
                       help='Dashboard URL')
    args = parser.parse_args()
    
    print(f"{Colors.CYAN}╔════════════════════════════════════════════════════════════╗{Colors.RESET}")
    print(f"{Colors.CYAN}║  Endpoint Logger - Integration Test Suite                  ║{Colors.RESET}")
    print(f"{Colors.CYAN}╚════════════════════════════════════════════════════════════╝{Colors.RESET}")
    print(f"\nConfiguration:")
    print(f"  Target:    {args.target}")
    print(f"  Proxy:     {args.proxy}")
    print(f"  Dashboard: {args.dashboard}\n")
    
    all_results = TestResult()
    
    # Test proxy forwarding
    print(f"{Colors.CYAN}[1/4] Testing Proxy Forwarding{Colors.RESET}")
    proxy_tester = ProxyTester(args.proxy, args.target)
    proxy_tester.test_get_request()
    proxy_tester.test_post_request_with_body()
    proxy_tester.test_put_request()
    proxy_tester.test_delete_request()
    proxy_tester.test_error_responses()
    proxy_tester.test_404_response()
    proxy_tester.test_custom_headers_forwarded()
    
    for test in proxy_tester.results.tests:
        if test['status'] == 'PASS':
            all_results.add_pass(test['name'], test['duration'])
        else:
            all_results.add_fail(test['name'], test.get('error', ''), test['duration'])
    
    # Test API
    print(f"\n{Colors.CYAN}[2/4] Testing REST API{Colors.RESET}")
    api_tester = APITester(args.proxy)
    api_tester.test_get_recent_logs()
    api_tester.test_get_single_log()
    
    for test in api_tester.results.tests:
        if test['status'] == 'PASS':
            all_results.add_pass(test['name'], test['duration'])
        else:
            all_results.add_fail(test['name'], test.get('error', ''), test['duration'])
    
    # Test WebSocket
    print(f"\n{Colors.CYAN}[3/4] Testing WebSocket{Colors.RESET}")
    ws_tester = WebSocketTester(args.proxy)
    await ws_tester.test_websocket_connection()
    await ws_tester.test_websocket_receive_logs()
    await ws_tester.test_websocket_multiple_messages(3)
    
    for test in ws_tester.results.tests:
        if test['status'] == 'PASS':
            all_results.add_pass(test['name'], test['duration'])
        else:
            all_results.add_fail(test['name'], test.get('error', ''), test['duration'])
    
    # Test performance
    print(f"\n{Colors.CYAN}[4/4] Testing Performance{Colors.RESET}")
    perf_tester = PerformanceTester(args.proxy)
    perf_tester.test_response_times(50)
    perf_tester.test_100_requests()
    
    for test in perf_tester.results.tests:
        if test['status'] == 'PASS':
            all_results.add_pass(test['name'], test['duration'])
        else:
            all_results.add_fail(test['name'], test.get('error', ''), test['duration'])
    
    # Print summary
    print(all_results.summary())
    print(f"\nTest Report:")
    for test in all_results.tests:
        status_color = Colors.GREEN if test['status'] == 'PASS' else Colors.RED
        status_symbol = '✓' if test['status'] == 'PASS' else '✗'
        print(f"  {status_color}{status_symbol}{Colors.RESET} {test['name']:45} ({test['duration']:.3f}s)")
        if test['status'] == 'FAIL':
            print(f"      → {test.get('error', 'Unknown error')}")
    
    # Exit with proper code
    sys.exit(0 if all_results.failed == 0 else 1)

if __name__ == '__main__':
    asyncio.run(main())
