// Throughput micro-benchmark harness for Tether.

use std::time::Instant;
use tether::memory_budget::MemoryBudget;
use tether::{compress_i64, decompress_i64, compress_f64, decompress_f64};

fn bench_i64_stream(name: &str, samples: &[i64], budget: MemoryBudget, iterations: usize) {
    let raw_bytes = samples.len() * 8;
    
    // Warm up
    let compressed = compress_i64(samples, budget);
    let _ = decompress_i64(&compressed).unwrap();

    // Measure Encode
    let start_enc = Instant::now();
    for _ in 0..iterations {
        let _ = compress_i64(samples, budget);
    }
    let dur_enc = start_enc.elapsed();
    let enc_mb_s = (raw_bytes as f64 * iterations as f64 / (1024.0 * 1024.0)) / dur_enc.as_secs_f64();

    // Measure Decode
    let start_dec = Instant::now();
    for _ in 0..iterations {
        let _ = decompress_i64(&compressed).unwrap();
    }
    let dur_dec = start_dec.elapsed();
    let dec_mb_s = (raw_bytes as f64 * iterations as f64 / (1024.0 * 1024.0)) / dur_dec.as_secs_f64();

    let ratio = raw_bytes as f64 / compressed.len() as f64;
    println!("{:25} | Ratio: {:5.2}x | Enc: {:7.2} MB/s | Dec: {:7.2} MB/s | Size: {} -> {} B",
             name, ratio, enc_mb_s, dec_mb_s, raw_bytes, compressed.len());
}

fn bench_f64_stream(name: &str, samples: &[f64], budget: MemoryBudget, iterations: usize) {
    let raw_bytes = samples.len() * 8;
    let compressed = compress_f64(samples, budget);
    let _ = decompress_f64(&compressed).unwrap();

    let start_enc = Instant::now();
    for _ in 0..iterations {
        let _ = compress_f64(samples, budget);
    }
    let dur_enc = start_enc.elapsed();
    let enc_mb_s = (raw_bytes as f64 * iterations as f64 / (1024.0 * 1024.0)) / dur_enc.as_secs_f64();

    let start_dec = Instant::now();
    for _ in 0..iterations {
        let _ = decompress_f64(&compressed).unwrap();
    }
    let dur_dec = start_dec.elapsed();
    let dec_mb_s = (raw_bytes as f64 * iterations as f64 / (1024.0 * 1024.0)) / dur_dec.as_secs_f64();

    let ratio = raw_bytes as f64 / compressed.len() as f64;
    println!("{:25} | Ratio: {:5.2}x | Enc: {:7.2} MB/s | Dec: {:7.2} MB/s | Size: {} -> {} B",
             name, ratio, enc_mb_s, dec_mb_s, raw_bytes, compressed.len());
}

fn main() {
    println!("=== Tether Performance & Throughput Benchmark ===");
    println!("Profile: Embedded32KB (Hard RAM ceiling: 32 KB)
");

    let budget = MemoryBudget::embedded_32kb();
    let n = 50_000;
    let iters = 20;

    // 1. Sine wave
    let sine: Vec<i64> = (0..n).map(|i| ((i as f64 * 0.05).sin() * 10000.0) as i64).collect();
    bench_i64_stream("Sine Wave (i64)", &sine, budget, iters);

    // 2. Linear Trend
    let linear: Vec<i64> = (0..n).map(|i| (i * 5 + 100) as i64).collect();
    bench_i64_stream("Linear Trend (i64)", &linear, budget, iters);

    // 3. Constant Stream
    let constant: Vec<i64> = vec![424242i64; n];
    bench_i64_stream("Constant Stream (i64)", &constant, budget, iters);

    // 4. Float Temperature Telemetry
    let mut temp = 21.5f64;
    let mut float_temps = Vec::with_capacity(n);
    for i in 0..n {
        if i % 4 == 0 {
            temp += 0.05 * if (i / 100) % 2 == 0 { 1.0 } else { -1.0 };
        }
        float_temps.push(temp);
    }
    bench_f64_stream("Temperature (f64)", &float_temps, budget, iters);

    println!("
Benchmark completed successfully.");
}
