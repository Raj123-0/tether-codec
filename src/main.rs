// Tether command-line interface.

use std::env;
use std::fs;
use std::process;
use std::time::Instant;
use tether::memory_budget::MemoryBudget;
use tether::{compress, decompress, compress_f64, compress_i64};

fn print_usage() {
    println!(
        r#"Tether: Low-memory, high-speed lossless compression for streaming numeric data

USAGE:
    tether compress INPUT -o OUTPUT [OPTIONS]
    tether decompress INPUT -o OUTPUT
    tether info FILE

OPTIONS:
    -p, --profile PROFILE   Memory budget profile: micro-4kb, embedded-16kb,
                            embedded-32kb (default), desktop-64kb
    -t, --type TYPE         Data interpretation: auto (default), f64, i64
    -v, --verbose           Print compression and performance metrics
    -h, --help              Print this help information
"#
    );
}

fn parse_profile(s: &str) -> Option<MemoryBudget> {
    match s.to_lowercase().as_str() {
        "micro-4kb" | "4k" | "micro" => Some(MemoryBudget::micro_4kb()),
        "embedded-16kb" | "16k" => Some(MemoryBudget::embedded_16kb()),
        "embedded-32kb" | "32k" | "default" => Some(MemoryBudget::embedded_32kb()),
        "desktop-64kb" | "64k" | "desktop" => Some(MemoryBudget::desktop_64kb()),
        _ => None,
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "-h" | "--help" | "help" => {
            print_usage();
        }
        "-V" | "--version" => {
            println!("tether-codec v0.1.0");
        }
        "compress" => {
            if args.len() < 3 {
                eprintln!("Error: Missing input path for compression");
                process::exit(1);
            }
            let input_path = &args[2];
            let mut output_path = String::new();
            let mut budget = MemoryBudget::embedded_32kb();
            let mut data_type = "auto";
            let mut verbose = false;

            let mut i = 3;
            while i < args.len() {
                match args[i].as_str() {
                    "-o" | "--output" => {
                        if i + 1 < args.len() {
                            output_path = args[i + 1].clone();
                            i += 1;
                        }
                    }
                    "-p" | "--profile" => {
                        if i + 1 < args.len() {
                            if let Some(b) = parse_profile(&args[i + 1]) {
                                budget = b;
                            } else {
                                eprintln!("Unknown profile: {}", args[i + 1]);
                                process::exit(1);
                            }
                            i += 1;
                        }
                    }
                    "-t" | "--type" => {
                        if i + 1 < args.len() {
                            data_type = match args[i + 1].as_str() {
                                "f64" => "f64",
                                "i64" => "i64",
                                _ => "auto",
                            };
                            i += 1;
                        }
                    }
                    "-v" | "--verbose" => {
                        verbose = true;
                    }
                    _ => {}
                }
                i += 1;
            }

            if output_path.is_empty() {
                output_path = format!("{}.tth", input_path);
            }

            let input_bytes = match fs::read(input_path) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("Failed to read {}: {}", input_path, e);
                    process::exit(1);
                }
            };

            let start = Instant::now();
            let compressed = match data_type {
                "f64" if input_bytes.len() % 8 == 0 => {
                    let num_f = input_bytes.len() / 8;
                    let mut f_slice = Vec::with_capacity(num_f);
                    for chunk in input_bytes.chunks_exact(8) {
                        f_slice.push(f64::from_le_bytes(chunk.try_into().unwrap()));
                    }
                    compress_f64(&f_slice, budget)
                }
                "i64" if input_bytes.len() % 8 == 0 => {
                    let num_i = input_bytes.len() / 8;
                    let mut i_slice = Vec::with_capacity(num_i);
                    for chunk in input_bytes.chunks_exact(8) {
                        i_slice.push(i64::from_le_bytes(chunk.try_into().unwrap()));
                    }
                    compress_i64(&i_slice, budget)
                }
                _ => compress(&input_bytes, budget),
            };
            let elapsed = start.elapsed();

            if let Err(e) = fs::write(&output_path, &compressed) {
                eprintln!("Failed to write {}: {}", output_path, e);
                process::exit(1);
            }

            let raw_len = input_bytes.len();
            let comp_len = compressed.len();
            let ratio = if comp_len > 0 { raw_len as f64 / comp_len as f64 } else { 0.0 };
            let mb_s = (raw_len as f64 / (1024.0 * 1024.0)) / elapsed.as_secs_f64().max(1e-9);

            println!("Compressed {} -> {}", input_path, output_path);
            println!("  Raw size:        {} bytes", raw_len);
            println!("  Compressed size: {} bytes ({:.2}x ratio)", comp_len, ratio);
            println!("  RAM ceiling:     {} bytes (Profile: {:?})", budget.max_encoder_bytes, budget.profile);
            if verbose {
                println!("  Throughput:      {:.2} MB/s (took {:?})", mb_s, elapsed);
            }
        }
        "decompress" => {
            if args.len() < 3 {
                eprintln!("Error: Missing input path for decompression");
                process::exit(1);
            }
            let input_path = &args[2];
            let mut output_path = String::new();
            let mut verbose = false;

            let mut i = 3;
            while i < args.len() {
                match args[i].as_str() {
                    "-o" | "--output" => {
                        if i + 1 < args.len() {
                            output_path = args[i + 1].clone();
                            i += 1;
                        }
                    }
                    "-v" | "--verbose" => {
                        verbose = true;
                    }
                    _ => {}
                }
                i += 1;
            }

            if output_path.is_empty() {
                if input_path.ends_with(".tth") {
                    output_path = input_path[..input_path.len() - 4].to_string();
                } else {
                    output_path = format!("{}.out", input_path);
                }
            }

            let compressed = match fs::read(input_path) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("Failed to read {}: {}", input_path, e);
                    process::exit(1);
                }
            };

            let start = Instant::now();
            let decompressed = match decompress(&compressed) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("Decompression failed: {}", e);
                    process::exit(1);
                }
            };
            let elapsed = start.elapsed();

            if let Err(e) = fs::write(&output_path, &decompressed) {
                eprintln!("Failed to write {}: {}", output_path, e);
                process::exit(1);
            }

            let raw_len = decompressed.len();
            let mb_s = (raw_len as f64 / (1024.0 * 1024.0)) / elapsed.as_secs_f64().max(1e-9);

            println!("Decompressed {} -> {}", input_path, output_path);
            println!("  Decompressed:    {} bytes", raw_len);
            if verbose {
                println!("  Throughput:      {:.2} MB/s (took {:?})", mb_s, elapsed);
            }
        }
        "info" => {
            if args.len() < 3 {
                eprintln!("Error: Missing file path");
                process::exit(1);
            }
            let path = &args[2];
            let bytes = match fs::read(path) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("Failed to read {}: {}", path, e);
                    process::exit(1);
                }
            };

            if bytes.len() < 7 || &bytes[0..4] != tether::MAGIC {
                eprintln!("Error: Not a valid Tether compressed file");
                process::exit(1);
            }

            let prof = match bytes[4] {
                0 => "Micro4KB (<= 4 KB RAM)",
                1 => "Embedded16KB (<= 16 KB RAM)",
                2 => "Embedded32KB (<= 32 KB RAM)",
                3 => "Desktop64KB (<= 64 KB RAM)",
                _ => "Custom / Unknown",
            };

            let dtype = match bytes[5] {
                1 => "i64 (64-bit signed integer)",
                2 => "f64 (64-bit floating point)",
                _ => "Raw bytes / Generic",
            };

            println!("Tether File: {}", path);
            println!("  Total Size:      {} bytes", bytes.len());
            println!("  Memory Profile:  {}", prof);
            println!("  Data Type:       {}", dtype);
        }
        cmd => {
            eprintln!("Unknown command: {}", cmd);
            print_usage();
            process::exit(1);
        }
    }
}
