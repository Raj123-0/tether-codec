use tether::entropy::{AdaptiveTable, RansEncoder, RansDecoder};

#[test]
fn test_rans_single_symbol() {
    let table = AdaptiveTable::new_uniform();
    let symbols = vec![42u8];

    let mut encoder = RansEncoder::new();
    encoder.encode_block(&symbols, &table);
    let payload = encoder.finish();

    let mut decoder = RansDecoder::new(&payload).unwrap();
    let decoded = decoder.decode_block(symbols.len(), &table);

    assert_eq!(decoded, symbols);
}

#[test]
fn test_rans_all_zeros_static() {
    let table = AdaptiveTable::new_skewed();
    let symbols = vec![0u8; 1024];

    let mut encoder = RansEncoder::new();
    encoder.encode_block(&symbols, &table);
    let payload = encoder.finish();

    let mut decoder = RansDecoder::new(&payload).unwrap();
    let decoded = decoder.decode_block(symbols.len(), &table);

    assert_eq!(decoded, symbols);
}

#[test]
fn test_rans_all_zeros_adapted() {
    let mut counts = [1u32; 256];
    counts[0] = 5000;
    let table = AdaptiveTable::from_counts(&counts);
    let symbols = vec![0u8; 1024];

    let mut encoder = RansEncoder::new();
    encoder.encode_block(&symbols, &table);
    let payload = encoder.finish();

    println!("1024 zeros adapted payload length: {} bytes ({:.2}x)", 
             payload.len(), 1024.0 / payload.len() as f64);
    assert!(payload.len() < 100);

    let mut decoder = RansDecoder::new(&payload).unwrap();
    let decoded = decoder.decode_block(symbols.len(), &table);

    assert_eq!(decoded, symbols);
}

#[test]
fn test_rans_all_max() {
    let table = AdaptiveTable::new_uniform();
    let symbols = vec![255u8; 512];

    let mut encoder = RansEncoder::new();
    encoder.encode_block(&symbols, &table);
    let payload = encoder.finish();

    let mut decoder = RansDecoder::new(&payload).unwrap();
    let decoded = decoder.decode_block(symbols.len(), &table);

    assert_eq!(decoded, symbols);
}

#[test]
fn test_rans_skewed_distribution() {
    let mut table = AdaptiveTable::new_skewed();
    
    let mut symbols = Vec::new();
    let mut seed = 123456789u64;
    let mut prng = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (seed >> 32) as u32
    };

    for _ in 0..1000 {
        let r = prng() % 100;
        let sym = if r < 60 {
            0
        } else if r < 85 {
            (prng() % 4) as u8
        } else if r < 98 {
            (prng() % 32) as u8
        } else {
            (prng() % 256) as u8
        };
        symbols.push(sym);
        table.observe(sym);
    }

    let mut encoder = RansEncoder::new();
    encoder.encode_block(&symbols, &table);
    let payload = encoder.finish();

    let mut decoder = RansDecoder::new(&payload).unwrap();
    let decoded = decoder.decode_block(symbols.len(), &table);

    assert_eq!(decoded, symbols);
    println!("1000 skewed symbols compressed to {} bytes ({:.2}x)", 
             payload.len(), 1000.0 / payload.len() as f64);
}

#[test]
fn test_rans_rescaling_determinism() {
    let mut table1 = AdaptiveTable::new_skewed();
    let mut table2 = AdaptiveTable::new_skewed();

    let stream = [0, 1, 0, 0, 2, 5, 0, 12, 0, 0, 1, 0];
    for &s in stream.iter().cycle().take(5000) {
        table1.observe(s);
        table2.observe(s);
    }

    assert_eq!(table1.freq, table2.freq);
    assert_eq!(table1.cum, table2.cum);
    assert_eq!(table1.lut, table2.lut);
}
