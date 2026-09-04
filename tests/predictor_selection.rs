use tether::entropy::AdaptiveTable;
use tether::stream::block_writer::BlockWriter;
use tether::stream::block_reader::BlockReader;
use tether::predictor::selector::PredictorSelector;
use tether::predictor::PredictorMode;

#[test]
fn test_adaptive_fir_bit_exact_roundtrip() {
    let mut table_enc = AdaptiveTable::new_skewed();
    let mut table_dec = AdaptiveTable::new_skewed();

    let n = 1000;
    let original: Vec<i64> = (0..n).map(|i| (i * 12 + 50) as i64).collect();

    let mut compressed = Vec::new();
    let mut history = [original[0], original[0], original[0]];

    for chunk in original.chunks(256) {
        history = BlockWriter::write_adaptive_fir_block(chunk, history, &mut table_enc, &mut compressed);
    }

    println!("Adaptive FIR Linear ramp 1000 i64: raw {} B -> compressed {} B ({:.2}x)",
             n * 8, compressed.len(), (n * 8) as f64 / compressed.len() as f64);

    let mut decoded = Vec::new();
    let mut offset = 0;
    while BlockReader::read_block_u64(&compressed, &mut offset, &mut table_dec, &mut decoded).unwrap() {}

    let decoded_i64: Vec<i64> = decoded.into_iter().map(|x| x as i64).collect();
    assert_eq!(decoded_i64, original);
}

#[test]
fn test_selector_ablation_comparison() {
    // 1. Linear trend dataset: Adaptive FIR should dominate
    let linear: Vec<i64> = (0..500).map(|i| (i * 15 + 100) as i64).collect();
    let mode_linear = PredictorSelector::select_integer_mode(&linear, linear[0], [linear[0]; 3]);
    assert_eq!(mode_linear, PredictorMode::AdaptiveLinear, "Linear trend should select AdaptiveLinear");

    // 2. Smooth Sine Wave: Adaptive FIR should dominate
    let sine: Vec<i64> = (0..500).map(|i| ((i as f64 * 0.05).sin() * 5000.0) as i64).collect();
    let mode_sine = PredictorSelector::select_integer_mode(&sine, sine[0], [sine[0]; 3]);
    assert_eq!(mode_sine, PredictorMode::AdaptiveLinear, "Smooth sine wave should select AdaptiveLinear");

    // 3. Random Walk: Delta should be selected or preferred
    let mut rw = vec![0i64];
    let mut seed = 54321u64;
    for _ in 0..500 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let step = ((seed >> 32) % 3) as i64 - 1; // -1, 0, +1
        rw.push(rw.last().unwrap() + step);
    }
    let mode_rw = PredictorSelector::select_integer_mode(&rw, rw[0], [rw[0]; 3]);
    assert_eq!(mode_rw, PredictorMode::Delta, "Random walk should select simpler Delta predictor");

    println!("Selector ablation results verified: Linear -> FIR, Sine -> FIR, RandomWalk -> Delta");
}

#[test]
fn test_auto_selector_end_to_end() {
    let mut table_enc = AdaptiveTable::new_skewed();
    let mut table_dec = AdaptiveTable::new_skewed();

    // Stream with changing dynamics: first 500 samples are smooth sine, next 500 samples are random steps
    let mut stream = Vec::with_capacity(1000);
    for i in 0..500 {
        stream.push(((i as f64 * 0.05).sin() * 8000.0) as i64);
    }
    let mut last = *stream.last().unwrap();
    let mut seed = 112233u64;
    for _ in 0..500 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let delta = ((seed >> 32) % 5) as i64 - 2;
        last += delta;
        stream.push(last);
    }

    let mut compressed = Vec::new();
    let mut initial_sample = stream[0];
    let mut initial_history = [stream[0]; 3];

    for chunk in stream.chunks(256) {
        let (s, h) = BlockWriter::write_auto_i64_block(chunk, initial_sample, initial_history, &mut table_enc, &mut compressed);
        initial_sample = s;
        initial_history = h;
    }

    println!("Mixed dynamics 1000 i64 samples: raw {} B -> compressed {} B ({:.2}x)",
             stream.len() * 8, compressed.len(), (stream.len() * 8) as f64 / compressed.len() as f64);

    let mut decoded = Vec::new();
    let mut offset = 0;
    while BlockReader::read_block_u64(&compressed, &mut offset, &mut table_dec, &mut decoded).unwrap() {}

    let decoded_i64: Vec<i64> = decoded.into_iter().map(|x| x as i64).collect();
    assert_eq!(decoded_i64, stream);
}
