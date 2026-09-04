# Contributing to Tether

Thank you for contributing to Tether!

## Core Principles
1. **Lossless Guarantee**: Every change must maintain bit-exact round-trip capability across all supported types.
2. **Strict Memory Contract**: Tether guarantees an explicit memory ceiling (default <= 32 KB, configurable to 4 KB). No change may introduce unbounded memory allocation or grow allocations with stream length.
3. **Honest Benchmarking**: Every benchmark must report all four metrics together: Compression Ratio, Encode Throughput, Decode Throughput, and Peak RAM (Encoder & Decoder). Never report ratio in isolation.
4. **Embedded First**: The core library must compile under `no_std` for ARM Cortex-M targets (`thumbv7em-none-eabihf`).

## Development Workflow
```bash
# Run all unit and property tests
cargo test --verbose

# Run isolated memory ceiling contract test
cargo test --test memory_ceiling -- --test-threads=1 --nocapture

# Run CI memory check
python mem_profile/ci_check.py

# Verify embedded target cross-compilation
cargo build --target thumbv7em-none-eabihf --no-default-features --features alloc --release

# Run benchmarks
python benchmarks/run_benchmarks.py
```
