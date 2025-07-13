#!/usr/bin/env python3

import asyncio
import aiohttp
import json
import time
import statistics
from typing import List, Dict, Any

class MCPStressTester:
    def __init__(self, base_url: str):
        self.base_url = base_url
        self.endpoint = f"{base_url}/mcp"
        
    async def make_request(self, session: aiohttp.ClientSession, request_id: int) -> Dict[str, Any]:
        """Make a single MCP request"""
        payload = {
            "jsonrpc": "2.0",
            "id": request_id,
            "method": "tools/call",
            "params": {
                "name": "bf2042_weapons_by_category",
                "arguments": {
                    "category": "Assault Rifles"
                }
            }
        }
        
        start_time = time.time()
        try:
            async with session.post(self.endpoint, json=payload) as response:
                result = await response.json()
                end_time = time.time()
                
                return {
                    "id": request_id,
                    "status": response.status,
                    "duration": end_time - start_time,
                    "success": response.status == 200 and "result" in result,
                    "error": result.get("error") if "error" in result else None
                }
        except Exception as e:
            end_time = time.time()
            return {
                "id": request_id,
                "status": 0,
                "duration": end_time - start_time,
                "success": False,
                "error": str(e)
            }
    
    async def stress_test(self, num_requests: int = 100, concurrency: int = 10) -> Dict[str, Any]:
        """Run stress test with specified parameters"""
        print(f"Starting stress test: {num_requests} requests with {concurrency} concurrent connections")
        print(f"Target: {self.endpoint}")
        
        connector = aiohttp.TCPConnector(limit=concurrency, limit_per_host=concurrency)
        timeout = aiohttp.ClientTimeout(total=30)
        
        async with aiohttp.ClientSession(connector=connector, timeout=timeout) as session:
            # Create semaphore to limit concurrency
            semaphore = asyncio.Semaphore(concurrency)
            
            async def limited_request(request_id: int):
                async with semaphore:
                    return await self.make_request(session, request_id)
            
            # Start timing
            test_start = time.time()
            
            # Execute all requests
            tasks = [limited_request(i) for i in range(num_requests)]
            results = await asyncio.gather(*tasks)
            
            test_end = time.time()
            
            return self.analyze_results(results, test_end - test_start)
    
    def analyze_results(self, results: List[Dict[str, Any]], total_time: float) -> Dict[str, Any]:
        """Analyze test results and return statistics"""
        successful_requests = [r for r in results if r["success"]]
        failed_requests = [r for r in results if not r["success"]]
        
        durations = [r["duration"] for r in results]
        successful_durations = [r["duration"] for r in successful_requests]
        
        stats = {
            "total_requests": len(results),
            "successful_requests": len(successful_requests),
            "failed_requests": len(failed_requests),
            "success_rate": len(successful_requests) / len(results) * 100,
            "total_test_time": total_time,
            "requests_per_second": len(results) / total_time,
            "successful_rps": len(successful_requests) / total_time,
        }
        
        if durations:
            stats.update({
                "avg_response_time": statistics.mean(durations),
                "min_response_time": min(durations),
                "max_response_time": max(durations),
                "median_response_time": statistics.median(durations),
            })
            
            if len(durations) > 1:
                stats["response_time_stddev"] = statistics.stdev(durations)
        
        if successful_durations:
            stats.update({
                "avg_successful_response_time": statistics.mean(successful_durations),
                "median_successful_response_time": statistics.median(successful_durations),
            })
        
        # Collect error summary
        error_summary = {}
        for result in failed_requests:
            error_key = str(result.get("error", "Unknown error"))
            error_summary[error_key] = error_summary.get(error_key, 0) + 1
        
        stats["errors"] = error_summary
        
        return stats
    
    def print_results(self, stats: Dict[str, Any]):
        """Print formatted test results"""
        print("\n" + "="*60)
        print("STRESS TEST RESULTS")
        print("="*60)
        
        print(f"Total Requests: {stats['total_requests']}")
        print(f"Successful: {stats['successful_requests']} ({stats['success_rate']:.1f}%)")
        print(f"Failed: {stats['failed_requests']}")
        print(f"Total Test Time: {stats['total_test_time']:.2f}s")
        print(f"Requests/Second: {stats['requests_per_second']:.2f}")
        print(f"Successful RPS: {stats['successful_rps']:.2f}")
        
        if "avg_response_time" in stats:
            print(f"\nResponse Times:")
            print(f"  Average: {stats['avg_response_time']*1000:.1f}ms")
            print(f"  Median: {stats['median_response_time']*1000:.1f}ms")
            print(f"  Min: {stats['min_response_time']*1000:.1f}ms")
            print(f"  Max: {stats['max_response_time']*1000:.1f}ms")
            
            if "response_time_stddev" in stats:
                print(f"  Std Dev: {stats['response_time_stddev']*1000:.1f}ms")
        
        if stats["errors"]:
            print(f"\nErrors:")
            for error, count in stats["errors"].items():
                print(f"  {error}: {count}")
        
        print("="*60)

async def main():
    import argparse
    
    parser = argparse.ArgumentParser(description="Stress test MCP server")
    parser.add_argument("--url", default="http://52.24.122.97:80", help="Base URL of MCP server")
    parser.add_argument("--requests", type=int, default=100, help="Number of requests to send")
    parser.add_argument("--concurrency", type=int, default=10, help="Number of concurrent connections")
    
    args = parser.parse_args()
    
    tester = MCPStressTester(args.url)
    
    try:
        stats = await tester.stress_test(args.requests, args.concurrency)
        tester.print_results(stats)
    except KeyboardInterrupt:
        print("\nTest interrupted by user")
    except Exception as e:
        print(f"Test failed: {e}")

if __name__ == "__main__":
    asyncio.run(main())

# python stress_test.py --requests 200 --concurrency 20