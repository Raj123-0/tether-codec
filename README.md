# Tether (`tether-codec`)

**Low-memory, high-speed lossless compression for streaming numeric and sensor time-series data.**

[![CI](https://github.com/Raj123-0/tether-codec/actions/workflows/ci.yml/badge.svg)](https://github.com/Raj123-0/tether-codec/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Embedded](https://img.shields.io/badge/embedded-thumbv7em%20(no__std)-success.svg)](ARCHITECTURE.md)

---

## 1. Why "Better Than Everything" Is the Wrong Frame

Compression has an inescapable three-way tradeoff: **ratio, speed, and memory footprint**.
- **zstd / Brotli / LZMA** achieve high ratios by maintaining large match-finding sliding windows and multi-megabyte hash tables. LZMA can require 100+ MB; even zstd's modest levels allocate hundreds of kilobytes to megabytes.
- **LZ4 / Snappy** achieve raw byte throughput with tiny state, but sacrifice ratio on numeric telemetry because byte-level dictionary matching fails on drifting floating-point mantissas or small integer deltas.
- **Context-mixing compressors (PAQ/CMIX)** achieve unmatched theoretical ratios, but run orders of magnitude too slow and memory-hungry for embedded microcontrollers.

No algorithm dominates all three axes simultaneously on arbitrary data. This is not an engineering oversight; it is a structural property of information theory.

### What Tether Actually Achieves
For a **specific, well-characterized data distribution**—numeric sensor telemetry, wearables, industrial time-series, and edge loggers—data exhibits strong local statistical structure: smooth trends, inertia, periodic oscillations, and low intrinsic entropy per sample. 

Tether pairs an **online predictive coding stage** (Gorilla XOR + Order-3 FIR adaptive linear predictor) with a **bounded-memory range Asymmetric Numeral Systems (rANS)** entropy coder operating under an **explicit, strictly enforced working RAM ceiling ($\le 32$ KB default, tunable down to $\le 4$ KB)**. It beats general-purpose compressors on the Pareto frontier that matters: **delivering superior compression ratio while consuming 100x less RAM than zstd, and 3x higher ratio than LZ4**.

---

## 2. Benchmark Results (Honest 4-Metric Evaluation)

All benchmarks are evaluated under stated memory budgets on real and synthetic sensor data. See [benchmarks/results.md](benchmarks/results.md) for the full leaderboard.

| Dataset | Codec | Ratio | Enc Speed | Dec Speed | Peak RAM (Enc / Dec) | Notes |
|:---|:---|:---:|:---:|:---:|:---:|:---|
| **Periodic Square Wave** (100k samples) | **Tether (32KB Profile)** | **402.21x** | **460.7 MB/s** | **941.3 MB/s** | **5.7 KB / 2.4 KB** | **Beats zstd & LZ4 under 5.7 KB RAM** |
| | Tether (4KB Micro Profile) | 201.97x | 309.7 MB/s | 973.0 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU |
| | zstd (32KB Window, L1) | 391.77x | 2735.5 MB/s | 3029.9 MB/s | 784.3 KB / 781.3 KB | Uses >137x more RAM |
| | LZ4 (Standard) | 79.87x | 6663.2 MB/s | 1431.4 MB/s | 794.1 KB / 1562.5 KB | 5x lower compression ratio |
| **Environmental IoT Telemetry** (100k samples) | **Tether (32KB Profile)** | **35.57x** | **436.2 MB/s** | **466.6 MB/s** | **5.7 KB / 2.4 KB** | **Lossless Decimal Quantization winner** |
| | Tether (4KB Micro Profile) | 25.73x | 384.3 MB/s | 429.3 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU |
| | zstd (32KB Window, L1) | 15.10x | 602.5 MB/s | 937.3 MB/s | 784.3 KB / 781.3 KB | 2.35x lower ratio, 137x more RAM |
| | Gorilla XOR | 7.87x | 7.6 MB/s | 9.1 MB/s | 2.0 KB / 2.0 KB | 4.5x lower ratio |
| **Wearable IMU Telemetry** (100k samples) | **Tether (32KB Profile)** | **8.76x** | **261.8 MB/s** | **298.2 MB/s** | **5.7 KB / 2.4 KB** | **Pareto frontier winner** |
| | Tether (4KB Micro Profile) | 7.81x | 241.5 MB/s | 264.9 MB/s | 3.2 KB / 2.2 KB | Ultra-constrained MCU |
| | zstd (32KB Window, L1) | 6.58x | 279.7 MB/s | 594.1 MB/s | 784.3 KB / 781.3 KB | Uses >130x more RAM |
| | LZ4 (Standard) | 2.78x | 714.0 MB/s | 1311.3 MB/s | 1065.7 KB / 1562.5 KB | Poor ratio on sensor data |
| **Industrial Power Telemetry** (100k samples) | **Tether (32KB Profile)** | **11.75x** | **234.3 MB/s** | **304.2 MB/s** | **5.7 KB / 2.4 KB** | **Pareto frontier winner (2.1x > zstd)** |
| | zstd (32KB Window, L1) | 5.60x | 262.5 MB/s | 588.2 MB/s | 784.3 KB / 781.3 KB | Uses >130x more RAM |
| | LZ4 (Standard) | 2.80x | 670.8 MB/s | 1278.2 MB/s | 1063.0 KB / 1562.5 KB | 4.2x lower ratio |
| **Synthetic Linear Ramp** (100k samples) | **Tether (32KB Profile)** | **31.83x** | **327.7 MB/s** | **479.5 MB/s** | **5.7 KB / 2.4 KB** | LinearRamp & Delta-of-Delta |
| | zstd (32KB Window, L1) | 7.53x | 266.6 MB/s | 283.8 MB/s | 784.3 KB / 781.3 KB | 4.2x lower ratio than Tether |
| **Synthetic Sine Wave** (100k samples) | **Tether (32KB Profile)** | **14.96x** | **249.6 MB/s** | **309.8 MB/s** | **5.7 KB / 2.4 KB** | 2.5x higher ratio than zstd |
| | zstd (32KB Window, L1) | 5.88x | 267.0 MB/s | 621.7 MB/s | 784.3 KB / 781.3 KB | Higher RAM, 2.5x worse ratio |
| **Adversarial Random** (100k samples) | **Tether (32KB Profile)** | **1.00x** | **183.1 MB/s** | **1072.8 MB/s** | **5.7 KB / 2.4 KB** | Raw fallback prevents expansion |
| | zstd (32KB Window, L1) | 1.00x | 818.0 MB/s | 2084.5 MB/s | 784.3 KB / 781.3 KB | Raw fallback |

### Where Tether Loses (Reported Honestly)
- **High-Entropy Random Floats with Irregular Mantissas**: When floats cannot be losslessly scaled to integers and have chaotic bit patterns, general-purpose compressors with large windows (64KB+) may find incidental byte matches that Tether's 32KB streaming predictor cannot.
- **Unstructured / Text / Blob data**: Tether is explicitly not designed for natural language, HTML, or pre-compressed payloads. Use zstd or LZ4 there.

---

## 3. Strict Memory Contract

Tether enforces compile-time and CI-verified runtime bounds on working memory:

| Memory Profile | Target Hardware | Max Allowed RAM | Peak Measured RAM (Enc / Dec) |
|:---|:---|:---:|:---:|
| `Micro4KB` | Cortex-M0/M3 (<= 8KB SRAM) | 4,096 B | **3.2 KB / 2.2 KB** |
| `Embedded16KB` | Cortex-M4/Wearables (<= 32KB SRAM) | 16,384 B | **5.7 KB / 2.4 KB** |
| `Embedded32KB` (Default) | Cortex-M7/IoT Telemetry | 32,768 B | **5.7 KB / 2.4 KB** |
| `Desktop64KB` | Linux SBC / Edge Gateways | 65,536 B | **10.5 KB / 3.6 KB** |

Automated verification is enforced on every push in CI via `tests/memory_ceiling.rs` (using a real tracking allocator) and `mem_profile/ci_check.py`.

---

## 4. Getting Started

### CLI Installation & Usage
```bash
cargo install tether-codec

# Compress a telemetry stream using default 32KB profile
tether compress sensor.bin -o sensor.tth --profile embedded-32kb

# Compress an f64 stream with the 4KB micro profile
tether compress temp_telemetry.bin -o temp.tth --profile micro-4kb --type f64

# Compress a 2D image row-by-row in O(width) RAM
tether compress-image screenshot.raw -o screenshot.tthi -w 256 -h 256

# Decompress
tether decompress sensor.tth -o sensor.bin
tether decompress-image screenshot.tthi -o screenshot.raw

# Inspect file metadata and memory profile
tether info sensor.tth
```

### Rust Public API
Add Tether to your `Cargo.toml`:
```toml
[dependencies]
tether-codec = "0.2.2"
```

Compress and decompress slices:
```rust
use tether::{compress_i64, decompress_i64, MemoryBudget};

let telemetry: Vec<i64> = vec![1000, 1004, 1007, 1012, 1015];
let budget = MemoryBudget::embedded_32kb();

// Compress
let compressed = compress_i64(&telemetry, budget);

// Decompress (guaranteed bit-exact)
let recovered = decompress_i64(&compressed).unwrap();
assert_eq!(recovered, telemetry);
```

For incremental streaming without buffering entire streams:
```rust
use tether::streaming::TetherWriter;
use tether::MemoryBudget;
use std::fs::File;

let file = File::create("stream.tth")?;
let mut writer = TetherWriter::new(file, MemoryBudget::embedded_32kb());

for sample in sensor_readings() {
    writer.write_sample(sample)?; // Flushes blocks automatically at 256 samples
}
writer.finish()?;
```

---

## 5. Embedded & `no_std` Support

The core codec compiles under `no_std` for bare-metal targets:
```bash
cargo build --target thumbv7em-none-eabihf --no-default-features --features alloc --release
```

---

## 6. Lossless Image Extension (§9)

Tether adapts its predictive-entropy engine to 2D spatial image compression using a **row-by-row streaming architecture**:
- Causal 2D spatial predictors: Plane ($A + B - C$), Paeth, MED (JPEG-LS), Sub, Up, Average.
- **Bounded $O(\text{row\_width})$ memory**: Keeping only 2 rows in RAM ($\approx 3.8$ KB for 1080p, $\approx 7.6$ KB for 4K).
- Residuals feed directly into the existing adaptive rANS entropy coder.

Benchmarked against PNG and Lossless WebP:
- **Flat UI Screenshot**: Tether achieves **8.11x** compression using **5.7 KB RAM**, whereas PNG requires 1.3 MB RAM and WebP requires 2.0 MB RAM.

---

## 7. Prior Art & Citations

Tether builds upon and synthesizes foundational prior art in predictive time-series coding and asymmetric numeral systems:
- **Facebook Gorilla** (VLDB 2015): Pelkonen et al., *Gorilla: A Fast, Scalable, In-Memory Time Series Database*.
- **FPC** (IEEE Trans. Computers 2006): Burtscher & Ratanaworabhan, *FPC: A High-Throughput Compression Algorithm for Double-Precision Floating-Point Data*.
- **rANS**: Jarek Duda, *Asymmetric numeral systems: entropy coding combining the speed of Huffman coding with the compression rate of arithmetic coding* (arXiv:1311.2540).
- **FLAC**: Josh Coalson, *FLAC - Free Lossless Audio Codec* (Linear Prediction Modeling).

---

## 8. License

Dual-licensed under either of:
- MIT License ([LICENSE](LICENSE))
- Apache License, Version 2.0
