use tether::entropy::AdaptiveTable;
use tether::stream::block_writer::BlockWriter;
use tether::stream::block_reader::BlockReader;

#[test]
fn test_roundtrip_delta_i64_sine() {
    let mut table_enc = AdaptiveTable::new_skewed();
    let mut table_dec = AdaptiveTable::new_skewed();

    let n = 1000;
    let mut original = Vec::with_capacity(n);
    for i in 0..n {
        let val = ((i as f64 * 0.05).sin() * 10000.0) as i64;
        original.push(val);
    }

    let block_size = 256;
    let mut compressed = Vec::new();
    let mut initial = 0i64;

    for chunk in original.chunks(block_size) {
        BlockWriter::write_delta_i64_block(chunk, initial, &mut table_enc, &mut compressed);
        initial = *chunk.last().unwrap();
    }

    println!("Sine wave 1000 i64 samples: raw {} B -> compressed {} B ({:.2}x)",
             n * 8, compressed.len(), (n * 8) as f64 / compressed.len() as f64);
    assert!(compressed.len() < (n * 8) / 3);

    let mut decoded = Vec::with_capacity(n);
    let mut offset = 0;
    while BlockReader::read_block_u64(&compressed, &mut offset, &mut table_dec, &mut decoded).unwrap() {}

    let decoded_i64: Vec<i64> = decoded.into_iter().map(|x| x as i64).collect();
    assert_eq!(decoded_i64, original);
}

#[test]
fn test_roundtrip_xor_f64_temperatures() {
    let mut table_enc = AdaptiveTable::new_skewed();
    let mut table_dec = AdaptiveTable::new_skewed();

    // Realistic IoT sensor telemetry: slow drift with repeated sensor readings at 10Hz
    let n = 1000;
    let mut original_f64 = Vec::with_capacity(n);
    let mut temp = 21.5f64;
    for i in 0..n {
        if i % 4 == 0 {
            temp += 0.05 * if (i / 100) % 2 == 0 { 1.0 } else { -1.0 };
        }
        original_f64.push(temp);
    }

    let original_u64: Vec<u64> = original_f64.iter().map(|&f| f.to_bits()).collect();

    let block_size = 256;
    let mut compressed = Vec::new();
    let mut initial = original_u64[0];

    for chunk in original_u64.chunks(block_size) {
        BlockWriter::write_xor_u64_block(chunk, initial, &mut table_enc, &mut compressed);
        initial = *chunk.last().unwrap();
    }

    println!("Temperature telemetry 1000 f64 samples: raw {} B -> compressed {} B ({:.2}x)",
             n * 8, compressed.len(), (n * 8) as f64 / compressed.len() as f64);
    assert!(compressed.len() < (n * 8) / 2);

    let mut decoded_u64 = Vec::with_capacity(n);
    let mut offset = 0;
    while BlockReader::read_block_u64(&compressed, &mut offset, &mut table_dec, &mut decoded_u64).unwrap() {}

    assert_eq!(decoded_u64, original_u64);

    let decoded_f64: Vec<f64> = decoded_u64.iter().map(|&b| f64::from_bits(b)).collect();
    assert_eq!(decoded_f64, original_f64);
}

#[test]
fn test_roundtrip_constant_stream() {
    let mut table_enc = AdaptiveTable::new_skewed();
    let mut table_dec = AdaptiveTable::new_skewed();

    let n = 1024;
    let original = vec![123456789i64; n];

    let mut compressed = Vec::new();
    let mut initial = original[0];
    for chunk in original.chunks(256) {
        BlockWriter::write_delta_i64_block(chunk, initial, &mut table_enc, &mut compressed);
        initial = *chunk.last().unwrap();
    }

    println!("Constant stream 1024 i64 samples: raw {} B -> compressed {} B ({:.2}x)",
             n * 8, compressed.len(), (n * 8) as f64 / compressed.len() as f64);
    assert!(compressed.len() < 500);

    let mut decoded = Vec::with_capacity(n);
    let mut offset = 0;
    while BlockReader::read_block_u64(&compressed, &mut offset, &mut table_dec, &mut decoded).unwrap() {}

    let decoded_i64: Vec<i64> = decoded.into_iter().map(|x| x as i64).collect();
    assert_eq!(decoded_i64, original);
}

#[test]
fn test_roundtrip_edge_cases_single_and_small() {
    let lengths = [1, 2, 3, 5, 17, 128, 255, 256, 257];
    for &len in &lengths {
        let mut table_enc = AdaptiveTable::new_skewed();
        let mut table_dec = AdaptiveTable::new_skewed();

        let original: Vec<i64> = (0..len).map(|i| (i * 37 - 100) as i64).collect();

        let mut compressed = Vec::new();
        let mut initial = 0i64;
        for chunk in original.chunks(256) {
            BlockWriter::write_delta_i64_block(chunk, initial, &mut table_enc, &mut compressed);
            initial = *chunk.last().unwrap();
        }

        let mut decoded = Vec::new();
        let mut offset = 0;
        while BlockReader::read_block_u64(&compressed, &mut offset, &mut table_dec, &mut decoded).unwrap() {}

        let decoded_i64: Vec<i64> = decoded.into_iter().map(|x| x as i64).collect();
        assert_eq!(decoded_i64, original, "Failed for length {}", len);
    }
}

#[test]
fn test_roundtrip_random_adversarial() {
    let mut table_enc = AdaptiveTable::new_skewed();
    let mut table_dec = AdaptiveTable::new_skewed();

    let mut seed = 987654321u64;
    let mut prng = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        seed
    };

    let n = 1000;
    let mut original = Vec::with_capacity(n);
    for _ in 0..n {
        original.push(prng());
    }

    let mut compressed = Vec::new();
    let mut initial = original[0];
    for chunk in original.chunks(256) {
        BlockWriter::write_xor_u64_block(chunk, initial, &mut table_enc, &mut compressed);
        initial = *chunk.last().unwrap();
    }

    println!("Adversarial random stream: raw {} B -> compressed {} B", n * 8, compressed.len());
    assert!(compressed.len() <= (n * 8) + 100);

    let mut decoded = Vec::new();
    let mut offset = 0;
    while BlockReader::read_block_u64(&compressed, &mut offset, &mut table_dec, &mut decoded).unwrap() {}

    assert_eq!(decoded, original);
}

#[test]
fn test_roundtrip_flat_ui_raw() {
    let raw = std::fs::read("data/images/flat_ui.raw").unwrap();
    let num_words = raw.len() / 8;
    let mut words = Vec::new();
    for i in 0..num_words {
        words.push(i64::from_le_bytes(raw[i*8..(i+1)*8].try_into().unwrap()));
    }

    let mut table_enc = tether::entropy::AdaptiveTable::new_skewed();
    let mut table_dec = tether::entropy::AdaptiveTable::new_skewed();

    let mut initial_sample = words[0];
    let mut initial_history = [words[0]; 3];

    for (b_idx, chunk) in words.chunks(256).enumerate() {
        let mut comp = Vec::new();
        let (s, h) = tether::stream::block_writer::BlockWriter::write_auto_i64_block(
            chunk, initial_sample, initial_history, &mut table_enc, &mut comp
        );
        initial_sample = s;
        initial_history = h;

        let mut dec = Vec::new();
        let mut offset = 0;
        tether::stream::block_reader::BlockReader::read_block_u64(&comp, &mut offset, &mut table_dec, &mut dec).unwrap();
        let dec_i64: Vec<i64> = dec.into_iter().map(|x| x as i64).collect();

        if dec_i64 != chunk {
            println!("Block {} mismatched!", b_idx);
            for i in 0..chunk.len() {
                if chunk[i] != dec_i64[i] {
                    println!("  Index {}: orig = {}, dec = {}", i, chunk[i], dec_i64[i]);
                    panic!("Mismatch in block {}", b_idx);
                }
            }
        }
    }
}

#[test]
fn test_roundtrip_linear_ramp_mode() {
    let n = 2560;
    let original: Vec<i64> = (0..n).map(|i| 1000 + (i as i64) * 7).collect();
    let budget = tether::MemoryBudget::embedded_32kb();
    let compressed = tether::compress_i64(&original, budget);

    println!("Linear ramp 2560 samples: raw {} B -> compressed {} B ({:.2}x)",
             n * 8, compressed.len(), (n * 8) as f64 / compressed.len() as f64);

    assert!(compressed.len() < 300, "Linear ramp should compress to < 300 bytes (> 68x ratio)");

    let decompressed = tether::decompress_i64(&compressed).unwrap();
    assert_eq!(decompressed, original);
}

#[test]
fn test_roundtrip_periodic_square_repeat_history() {
    let n = 10000;
    let period = 200;
    let original: Vec<i64> = (0..n).map(|i| if (i % period) < 100 { 5000 } else { -5000 }).collect();
    let budget = tether::MemoryBudget::embedded_32kb();
    let compressed = tether::compress_i64(&original, budget);

    println!("Periodic square 10000 samples: raw {} B -> compressed {} B ({:.2}x)",
             n * 8, compressed.len(), (n * 8) as f64 / compressed.len() as f64);

    assert!(compressed.len() < 1000, "Periodic square should compress to < 1000 bytes (> 80x ratio)");

    let decompressed = tether::decompress_i64(&compressed).unwrap();
    assert_eq!(decompressed, original);
}

#[test]
fn test_roundtrip_iot_decimal_floats() {
    let raw = std::fs::read("data/real_world/iot_environmental.bin").unwrap();
    let num_floats = raw.len() / 8;
    let mut floats = Vec::with_capacity(num_floats);
    for i in 0..num_floats {
        floats.push(f64::from_bits(u64::from_le_bytes(raw[i*8..(i+1)*8].try_into().unwrap())));
    }

    let budget = tether::MemoryBudget::embedded_32kb();
    let compressed = tether::compress_f64(&floats, budget);

    println!("IoT environmental 100000 f64: raw {} B -> compressed {} B ({:.2}x)",
             raw.len(), compressed.len(), raw.len() as f64 / compressed.len() as f64);

    assert!(compressed.len() < 25000, "IoT decimal telemetry should compress to < 25 KB (> 32x ratio)");

    let decompressed = tether::decompress_f64(&compressed).unwrap();
    assert_eq!(decompressed.len(), floats.len());
    for i in 0..floats.len() {
        assert_eq!(decompressed[i].to_bits(), floats[i].to_bits(), "Float mismatch at sample {}", i);
    }
}
