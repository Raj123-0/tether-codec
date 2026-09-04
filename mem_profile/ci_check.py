#!/usr/bin/env python3
"""
CI Memory Ceiling Enforcement Checker.
Verifies that Tether encoder and decoder working set RAM remains strictly
under the configured memory ceiling (<= 32 KB default).
"""

import os
import sys
import subprocess
import shutil
from pathlib import Path

CEILING_BYTES = 32 * 1024 # 32 KB

def run_cargo_memory_test():
    """Execute the Rust memory ceiling test suite directly."""
    print("[CI Check] Executing Rust memory ceiling unit tests...")
    env = os.environ.copy()
    cargo_bin = Path.home() / ".cargo" / "bin"
    if cargo_bin.exists():
        env["PATH"] = str(cargo_bin) + os.pathsep + env.get("PATH", "")
    
    cargo_path = shutil.which("cargo", path=env.get("PATH"))
    if not cargo_path:
        print("[FAIL] cargo executable not found in PATH", file=sys.stderr)
        return False

    cmd = [cargo_path, "test", "--test", "memory_ceiling", "--", "--test-threads=1", "--nocapture"]
    result = subprocess.run(cmd, capture_output=True, text=True, env=env)
    print(result.stdout)
    if result.returncode != 0:
        print(result.stderr, file=sys.stderr)
        print("[FAIL] Rust memory ceiling test failed!", file=sys.stderr)
        return False
    return True

def main():
    print("=== Tether CI Memory Budget Enforcement Check ===")
    print(f"Hard Ceiling: {CEILING_BYTES} bytes ({CEILING_BYTES / 1024:.1f} KB)\n")

    if not run_cargo_memory_test():
        sys.exit(1)

    print("[SUCCESS] Memory ceiling contract strictly enforced and verified.")
    sys.exit(0)

if __name__ == "__main__":
    main()
