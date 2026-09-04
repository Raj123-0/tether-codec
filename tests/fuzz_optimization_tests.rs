use tether::stream::block_writer::BlockWriter;
use tether::stream::block_reader::BlockReader;
use tether::entropy::AdaptiveTable;
use tether::memory_budget::MemoryBudget;
use tether::{compress_f64, decompress_f64, compress_i64, decompress_i64};
use tether::image::{compress_image, decompress_image};

#[test]
fn test_fuzz_all_constant_streams() {
    let mut table_enc = AdaptiveTable::new_skewed();
    let mut table_dec = AdaptiveTable::new_skewed();

    let values = [0i64, -1, 1, 42, -999999999, i64::MAX, i64::MIN, 1234567890123456789];
    for &val in &values {
        for len in [1, 2, 7, 64, 128, 255, 256, 512, 1024, 2048] {
            let data = vec![val; len];
            let mut comp = Vec::new();
            let mut hist = [val; 3];
            for chunk in data.chunks(256) {
                let (s, h) = BlockWriter::write_auto_i64_block(chunk, hist[0], hist, &mut table_enc, &mut comp);
                hist = [s, h[0], h[1]];
            }

            let mut dec = Vec::new();
            let mut offset = 0;
            while BlockReader::read_block_u64(&comp, &mut offset, &mut table_dec, &mut dec).unwrap() {}
            let dec_i64: Vec<i64> = dec.into_iter().map(|x| x as i64).collect();
            assert_eq!(dec_i64, data);
        }
    }
}

#[test]
fn test_fuzz_linear_trends_delta2() {
    let mut seed = 987654321u64;
    let mut lcg = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        seed
    };

    for _ in 0..200 {
        let slope = (lcg() % 2001) as i64 - 1000;
        let intercept = (lcg() % 2000001) as i64 - 1000000;
        let len = ((lcg() % 1000) + 10) as usize;

        let data: Vec<i64> = (0..len).map(|i| intercept.wrapping_add((i as i64).wrapping_mul(slope))).collect();

        let budget = MemoryBudget::embedded_32kb();
        let compressed = compress_i64(&data, budget);
        let decompressed = decompress_i64(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }
}

#[test]
fn test_fuzz_zero_runs_and_spikes() {
    let mut seed = 123456789u64;
    let mut lcg = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        seed
    };

    for _ in 0..100 {
        let mut data = Vec::new();
        let num_segments = (lcg() % 20) + 5;
        let mut curr = 0i64;
        for _ in 0..num_segments {
            let run_len = ((lcg() % 70) + 1) as usize;
            for _ in 0..run_len {
                data.push(curr);
            }
            curr = curr.wrapping_add((lcg() % 100000) as i64 - 50000);
        }

        let budget = MemoryBudget::embedded_32kb();
        let compressed = compress_i64(&data, budget);
        let decompressed = decompress_i64(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }
}

#[test]
fn test_fuzz_extreme_integers() {
    let extremes = vec![
        i64::MIN, i64::MAX, -1, 0, 1,
        i64::MIN + 1, i64::MAX - 1,
        -128, 127, -32768, 32767,
        i32::MIN as i64, i32::MAX as i64,
        0, 0, 0, 0, 0,
        i64::MIN, i64::MAX,
    ];

    let budget = MemoryBudget::micro_4kb();
    let compressed = compress_i64(&extremes, budget);
    let decompressed = decompress_i64(&compressed).unwrap();
    assert_eq!(decompressed, extremes);
}

#[test]
fn test_fuzz_floats_with_special_values() {
    let floats = vec![
        0.0f64, -0.0f64, 1.0, -1.0, 21.5, 21.5, 21.5, 21.50001,
        f64::MIN, f64::MAX, f64::MIN_POSITIVE, f64::EPSILON,
        f64::INFINITY, f64::NEG_INFINITY,
        1.23456789e-300, 9.87654321e300,
    ];

    let budget = MemoryBudget::embedded_32kb();
    let compressed = compress_f64(&floats, budget);
    let decompressed = decompress_f64(&compressed).unwrap();

    for (a, b) in floats.iter().zip(decompressed.iter()) {
        assert_eq!(a.to_bits(), b.to_bits());
    }
}

#[test]
fn test_fuzz_images_rgb_and_gray() {
    let mut seed = 55667788u64;
    let mut lcg = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        seed
    };

    for channels in [1u8, 3u8] {
        for &(w, h) in &[(16, 16), (32, 20), (50, 1), (1, 50), (64, 64)] {
            let n = (w * h * channels as u32) as usize;
            let mut pixels = Vec::with_capacity(n);
            for _ in 0..n {
                pixels.push((lcg() % 256) as u8);
            }

            let comp = compress_image(w, h, channels, &pixels).unwrap();
            let (dec_w, dec_h, dec_c, dec_pix) = decompress_image(&comp).unwrap();
            assert_eq!(dec_w, w);
            assert_eq!(dec_h, h);
            assert_eq!(dec_c, channels);
            assert_eq!(dec_pix, pixels);
        }
    }
}

#[test]
fn test_fuzz_truncated_blocks_fail_gracefully() {
    let data = vec![42i64; 500];
    let comp = compress_i64(&data, MemoryBudget::embedded_32kb());

    for trunc_len in 0..comp.len() {
        let truncated = &comp[..trunc_len];
        let res = decompress_i64(truncated);
        let _ = res;
    }
}
