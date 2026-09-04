"""
Python wrapper for Tether: Low-Memory, High-Speed Lossless Compressor
for Streaming Numeric Data.
"""

import os
import sys
import subprocess
import tempfile
import struct
import shutil
from pathlib import Path

def _find_binary():
    # 1. Environment override
    env_bin = os.environ.get("TETHER_BIN")
    if env_bin and os.path.isfile(env_bin):
        return env_bin

    # 2. Local build directory
    root = Path(__file__).parent
    release_exe = root / "target" / "release" / ("tether.exe" if sys.platform == "win32" else "tether")
    if release_exe.is_file():
        return str(release_exe)

    debug_exe = root / "target" / "debug" / ("tether.exe" if sys.platform == "win32" else "tether")
    if debug_exe.is_file():
        return str(debug_exe)

    # 3. System PATH
    which = shutil.which("tether")
    if which:
        return which

    raise FileNotFoundError("Tether CLI binary not found. Build with 'cargo build --release' first.")

def compress(data: bytes, profile: str = "embedded-32kb", dtype: str = "auto") -> bytes:
    """Compress a raw byte stream into Tether format."""
    binary = _find_binary()
    with tempfile.NamedTemporaryFile(delete=False, suffix=".bin") as in_f,          tempfile.NamedTemporaryFile(delete=False, suffix=".tth") as out_f:
        in_name = in_f.name
        out_name = out_f.name
        in_f.write(data)

    try:
        cmd = [binary, "compress", in_name, "-o", out_name, "-p", profile, "-t", dtype]
        res = subprocess.run(cmd, capture_output=True, text=True)
        if res.returncode != 0:
            raise RuntimeError(f"Tether compression failed: {res.stderr}")
        with open(out_name, "rb") as f:
            return f.read()
    finally:
        if os.path.exists(in_name): os.remove(in_name)
        if os.path.exists(out_name): os.remove(out_name)

def decompress(compressed_data: bytes) -> bytes:
    """Decompress a Tether-compressed byte stream."""
    binary = _find_binary()
    with tempfile.NamedTemporaryFile(delete=False, suffix=".tth") as in_f,          tempfile.NamedTemporaryFile(delete=False, suffix=".out") as out_f:
        in_name = in_f.name
        out_name = out_f.name
        in_f.write(compressed_data)

    try:
        cmd = [binary, "decompress", in_name, "-o", out_name]
        res = subprocess.run(cmd, capture_output=True, text=True)
        if res.returncode != 0:
            raise RuntimeError(f"Tether decompression failed: {res.stderr}")
        with open(out_name, "rb") as f:
            return f.read()
    finally:
        if os.path.exists(in_name): os.remove(in_name)
        if os.path.exists(out_name): os.remove(out_name)

def compress_i64(samples: list[int], profile: str = "embedded-32kb") -> bytes:
    """Compress a list of 64-bit signed integers."""
    raw = struct.pack(f"<{len(samples)}q", *samples)
    return compress(raw, profile=profile, dtype="i64")

def decompress_i64(compressed_data: bytes) -> list[int]:
    """Decompress into a list of 64-bit signed integers."""
    raw = decompress(compressed_data)
    num_words = len(raw) // 8
    return list(struct.unpack(f"<{num_words}q", raw[:num_words * 8]))

def compress_f64(samples: list[float], profile: str = "embedded-32kb") -> bytes:
    """Compress a list of 64-bit floating-point numbers."""
    raw = struct.pack(f"<{len(samples)}d", *samples)
    return compress(raw, profile=profile, dtype="f64")

def decompress_f64(compressed_data: bytes) -> list[float]:
    """Decompress into a list of 64-bit floating-point numbers."""
    raw = decompress(compressed_data)
    num_words = len(raw) // 8
    return list(struct.unpack(f"<{num_words}d", raw[:num_words * 8]))
