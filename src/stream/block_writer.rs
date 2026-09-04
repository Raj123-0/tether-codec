// Block framing writer for compressed and raw blocks.

// Block framing writer for compressed and raw blocks.

use crate::Vec;
use crate::entropy::{AdaptiveTable, RansEncoder};
use crate::predictor::delta_xor::{encode_delta_i64_block, encode_delta2_i64_block, encode_xor_u64_block};
use crate::predictor::adaptive_linear::encode_adaptive_fir_block;
use crate::predictor::selector::PredictorSelector;
use crate::predictor::PredictorMode;
use crate::stream::bitstream::BitWriter;

pub struct BlockWriter;

impl BlockWriter {
    /// Write an integer block automatically selecting the best predictor
    /// (Constant vs DeltaOfDelta vs Adaptive Linear FIR vs Delta).
    pub fn write_auto_i64_block(
        samples: &[i64],
        initial_sample: i64,
        initial_history: [i64; 3],
        table: &mut AdaptiveTable,
        out: &mut Vec<u8>,
    ) -> (i64, [i64; 3]) {
        if samples.is_empty() {
            return (initial_sample, initial_history);
        }

        let mode = PredictorSelector::select_integer_mode(samples, initial_sample, initial_history);
        match mode {
            PredictorMode::Constant => {
                Self::write_constant_block(samples.len(), samples[0] as u64, out);
                let first = samples[0];
                (first, [first, first, first])
            }
            PredictorMode::LinearRamp => {
                let delta = samples[1].wrapping_sub(samples[0]);
                Self::write_linear_ramp_block(samples.len(), samples[0], delta, out);
                let last_val = *samples.last().unwrap();
                let len = samples.len();
                let h0 = last_val;
                let h1 = if len > 1 { samples[len - 2] } else { initial_history[0] };
                let h2 = if len > 2 { samples[len - 3] } else { initial_history[1] };
                (last_val, [h0, h1, h2])
            }
            PredictorMode::DeltaOfDelta => {
                Self::write_delta2_i64_block(samples, initial_history[0], initial_history[1], table, out);
                let last_val = *samples.last().unwrap();
                let len = samples.len();
                let h0 = last_val;
                let h1 = if len > 1 { samples[len - 2] } else { initial_history[0] };
                let h2 = if len > 2 { samples[len - 3] } else { initial_history[1] };
                (last_val, [h0, h1, h2])
            }
            PredictorMode::AdaptiveLinear => {
                let new_hist = Self::write_adaptive_fir_block(samples, initial_history, table, out);
                let last_val = *samples.last().unwrap();
                (last_val, new_hist)
            }
            _ => {
                Self::write_delta_i64_block(samples, initial_sample, table, out);
                let last_val = *samples.last().unwrap();
                let len = samples.len();
                let h0 = last_val;
                let h1 = if len > 1 { samples[len - 2] } else { initial_history[0] };
                let h2 = if len > 2 { samples[len - 3] } else { initial_history[1] };
                (last_val, [h0, h1, h2])
            }
        }
    }

    /// Write an 11-byte constant block (mode: 4, count: u16, val: u64).
    pub fn write_constant_block(count: usize, val: u64, out: &mut Vec<u8>) {
        out.push(PredictorMode::Constant as u8);
        out.extend_from_slice(&(count as u16).to_le_bytes());
        out.extend_from_slice(&val.to_le_bytes());
    }

    /// Write a 19-byte linear ramp block (mode: 6, count: u16, start_val: i64, delta: i64).
    pub fn write_linear_ramp_block(count: usize, start_val: i64, delta: i64, out: &mut Vec<u8>) {
        out.push(PredictorMode::LinearRamp as u8);
        out.extend_from_slice(&(count as u16).to_le_bytes());
        out.extend_from_slice(&start_val.to_le_bytes());
        out.extend_from_slice(&delta.to_le_bytes());
    }

    /// Write a 5-byte repeat history block (mode: 7, count: u16, dist: u16).
    pub fn write_repeat_history_block(count: usize, dist: usize, out: &mut Vec<u8>) {
        out.push(PredictorMode::RepeatHistory as u8);
        out.extend_from_slice(&(count as u16).to_le_bytes());
        out.extend_from_slice(&(dist as u16).to_le_bytes());
    }

    /// Find an exact periodic or repeated match in recent history.
    pub fn find_repeat_history_match(chunk: &[i64], history: &[i64], max_dist: usize) -> Option<usize> {
        if history.is_empty() || chunk.is_empty() {
            return None;
        }
        let max_d = history.len().min(max_dist);
        let h_len = history.len();
        for d in 1..=max_d {
            if chunk[0] != history[h_len - d] {
                continue;
            }
            let mut matched = true;
            for k in 1..chunk.len() {
                let expected = if k >= d {
                    chunk[k - d]
                } else {
                    history[h_len - d + k]
                };
                if chunk[k] != expected {
                    matched = false;
                    break;
                }
            }
            if matched {
                return Some(d);
            }
        }
        None
    }

    /// Write an integer block with history match fast path, falling back to auto predictor selection.
    pub fn write_auto_i64_block_with_history(
        samples: &[i64],
        initial_sample: i64,
        initial_history: [i64; 3],
        history_window: &[i64],
        max_history_dist: usize,
        table: &mut AdaptiveTable,
        out: &mut Vec<u8>,
    ) -> (i64, [i64; 3]) {
        if samples.is_empty() {
            return (initial_sample, initial_history);
        }

        if let Some(dist) = Self::find_repeat_history_match(samples, history_window, max_history_dist) {
            Self::write_repeat_history_block(samples.len(), dist, out);
            let last_val = *samples.last().unwrap();
            let len = samples.len();
            let h0 = last_val;
            let h1 = if len > 1 { samples[len - 2] } else { initial_history[0] };
            let h2 = if len > 2 { samples[len - 3] } else { initial_history[1] };
            return (last_val, [h0, h1, h2]);
        }

        Self::write_auto_i64_block(samples, initial_sample, initial_history, table, out)
    }

    /// Write a block using integer delta prediction + rANS.
    pub fn write_delta_i64_block(
        samples: &[i64],
        initial_sample: i64,
        table: &mut AdaptiveTable,
        out: &mut Vec<u8>,
    ) {
        if samples.is_empty() {
            return;
        }

        // Fast path for constant block
        let first = samples[0];
        if samples.iter().all(|&x| x == first) {
            Self::write_constant_block(samples.len(), first as u64, out);
            return;
        }

        let mut symbols = Vec::with_capacity(samples.len());
        let mut bit_writer = BitWriter::with_capacity(samples.len() * 4);

        encode_delta_i64_block(samples, initial_sample, &mut symbols, &mut bit_writer);

        let extra_bytes = bit_writer.finish();

        let mut rans = RansEncoder::new();
        rans.encode_block(&symbols, table);
        let rans_payload = rans.finish();

        let raw_size = samples.len() * 8;
        let compressed_size = 1 + 2 + 2 + 2 + 2 + 8 + extra_bytes.len() + rans_payload.len();

        if compressed_size >= raw_size {
            let u64_slice = unsafe { core::slice::from_raw_parts(samples.as_ptr() as *const u64, samples.len()) };
            Self::write_raw_u64_block(u64_slice, out);
            return;
        }

        // Only update table if block is actually written as compressed!
        for &s in &symbols {
            table.observe(s);
        }

        out.push(PredictorMode::Delta as u8);
        out.extend_from_slice(&(samples.len() as u16).to_le_bytes());
        out.extend_from_slice(&(symbols.len() as u16).to_le_bytes());
        out.extend_from_slice(&(extra_bytes.len() as u16).to_le_bytes());
        out.extend_from_slice(&(rans_payload.len() as u16).to_le_bytes());
        out.extend_from_slice(&initial_sample.to_le_bytes());

        out.extend_from_slice(&extra_bytes);
        out.extend_from_slice(&rans_payload);
    }

    /// Write a block using Delta-of-Delta (second difference) prediction + rANS.
    pub fn write_delta2_i64_block(
        samples: &[i64],
        prev1: i64,
        prev2: i64,
        table: &mut AdaptiveTable,
        out: &mut Vec<u8>,
    ) {
        if samples.is_empty() {
            return;
        }

        // Fast path for constant block
        let first = samples[0];
        if samples.iter().all(|&x| x == first) {
            Self::write_constant_block(samples.len(), first as u64, out);
            return;
        }

        let mut symbols = Vec::with_capacity(samples.len());
        let mut bit_writer = BitWriter::with_capacity(samples.len() * 4);

        encode_delta2_i64_block(samples, prev1, prev2, &mut symbols, &mut bit_writer);

        let extra_bytes = bit_writer.finish();

        let mut rans = RansEncoder::new();
        rans.encode_block(&symbols, table);
        let rans_payload = rans.finish();

        let raw_size = samples.len() * 8;
        let compressed_size = 1 + 2 + 2 + 2 + 2 + 16 + extra_bytes.len() + rans_payload.len();

        if compressed_size >= raw_size {
            let u64_slice = unsafe { core::slice::from_raw_parts(samples.as_ptr() as *const u64, samples.len()) };
            Self::write_raw_u64_block(u64_slice, out);
            return;
        }

        for &s in &symbols {
            table.observe(s);
        }

        out.push(PredictorMode::DeltaOfDelta as u8);
        out.extend_from_slice(&(samples.len() as u16).to_le_bytes());
        out.extend_from_slice(&(symbols.len() as u16).to_le_bytes());
        out.extend_from_slice(&(extra_bytes.len() as u16).to_le_bytes());
        out.extend_from_slice(&(rans_payload.len() as u16).to_le_bytes());
        out.extend_from_slice(&prev1.to_le_bytes());
        out.extend_from_slice(&prev2.to_le_bytes());

        out.extend_from_slice(&extra_bytes);
        out.extend_from_slice(&rans_payload);
    }

    /// Write a block using Adaptive Linear FIR prediction + rANS.
    pub fn write_adaptive_fir_block(
        samples: &[i64],
        initial_history: [i64; 3],
        table: &mut AdaptiveTable,
        out: &mut Vec<u8>,
    ) -> [i64; 3] {
        if samples.is_empty() {
            return initial_history;
        }

        let mut symbols = Vec::with_capacity(samples.len());
        let mut bit_writer = BitWriter::with_capacity(samples.len() * 4);

        let new_history = encode_adaptive_fir_block(samples, initial_history, &mut symbols, &mut bit_writer);
        let extra_bytes = bit_writer.finish();

        let mut rans = RansEncoder::new();
        rans.encode_block(&symbols, table);
        let rans_payload = rans.finish();

        let raw_size = samples.len() * 8;
        let compressed_size = 1 + 2 + 2 + 2 + 2 + 24 + extra_bytes.len() + rans_payload.len();

        if compressed_size >= raw_size {
            let u64_slice = unsafe { core::slice::from_raw_parts(samples.as_ptr() as *const u64, samples.len()) };
            Self::write_raw_u64_block(u64_slice, out);
            return new_history;
        }

        for &s in &symbols {
            table.observe(s);
        }

        out.push(PredictorMode::AdaptiveLinear as u8);
        out.extend_from_slice(&(samples.len() as u16).to_le_bytes());
        out.extend_from_slice(&(symbols.len() as u16).to_le_bytes());
        out.extend_from_slice(&(extra_bytes.len() as u16).to_le_bytes());
        out.extend_from_slice(&(rans_payload.len() as u16).to_le_bytes());
        out.extend_from_slice(&initial_history[0].to_le_bytes());
        out.extend_from_slice(&initial_history[1].to_le_bytes());
        out.extend_from_slice(&initial_history[2].to_le_bytes());

        out.extend_from_slice(&extra_bytes);
        out.extend_from_slice(&rans_payload);

        new_history
    }

    /// Write a block using Gorilla XOR prediction + rANS.
    pub fn write_xor_u64_block(
        samples: &[u64],
        initial_sample: u64,
        table: &mut AdaptiveTable,
        out: &mut Vec<u8>,
    ) {
        if samples.is_empty() {
            return;
        }

        // Fast path for constant float / word block
        let first = samples[0];
        if samples.iter().all(|&x| x == first) {
            Self::write_constant_block(samples.len(), first, out);
            return;
        }

        let mut symbols = Vec::with_capacity(samples.len());
        let mut bit_writer = BitWriter::with_capacity(samples.len() * 4);

        encode_xor_u64_block(samples, initial_sample, &mut symbols, &mut bit_writer);

        let extra_bytes = bit_writer.finish();

        let mut rans = RansEncoder::new();
        rans.encode_block(&symbols, table);
        let rans_payload = rans.finish();

        let raw_size = samples.len() * 8;
        let compressed_size = 1 + 2 + 2 + 2 + 2 + 8 + extra_bytes.len() + rans_payload.len();

        if compressed_size >= raw_size {
            Self::write_raw_u64_block(samples, out);
            return;
        }

        for &s in &symbols {
            table.observe(s);
        }

        out.push(PredictorMode::Xor as u8);
        out.extend_from_slice(&(samples.len() as u16).to_le_bytes());
        out.extend_from_slice(&(symbols.len() as u16).to_le_bytes());
        out.extend_from_slice(&(extra_bytes.len() as u16).to_le_bytes());
        out.extend_from_slice(&(rans_payload.len() as u16).to_le_bytes());
        out.extend_from_slice(&initial_sample.to_le_bytes());

        out.extend_from_slice(&extra_bytes);
        out.extend_from_slice(&rans_payload);
    }

    /// Decimal scale factors for lossless float quantization
    pub const DECIMAL_SCALES: [f64; 7] = [1.0, 10.0, 100.0, 1000.0, 10000.0, 100000.0, 1000000.0];

    /// Find an exact lossless decimal scale factor where recovering via `(scaled as f64) / scale`
    /// produces the exact bit-level IEEE 754 float for every sample in the block.
    pub fn find_lossless_decimal_scale(samples: &[u64]) -> Option<(u8, f64)> {
        'scale_loop: for (idx, &scale) in Self::DECIMAL_SCALES.iter().enumerate().skip(1) {
            for &w in samples {
                let f = f64::from_bits(w);
                if !f.is_finite() {
                    return None;
                }
                let scaled_val = f * scale;
                let scaled = (scaled_val + if scaled_val >= 0.0 { 0.5 } else { -0.5 }) as i64;
                let recovered = (scaled as f64) / scale;
                if recovered.to_bits() != w {
                    continue 'scale_loop;
                }
            }
            return Some((idx as u8, scale));
        }
        None
    }

    /// Try writing a block using Lossless Decimal Quantization + integer Delta + rANS.
    pub fn try_write_decimal_float_block(
        scaled_samples: &[i64],
        initial_sample: i64,
        scale_code: u8,
        table: &mut AdaptiveTable,
        out: &mut Vec<u8>,
    ) -> bool {
        let mut symbols = Vec::with_capacity(scaled_samples.len());
        let mut bit_writer = BitWriter::with_capacity(scaled_samples.len() * 4);
        encode_delta_i64_block(scaled_samples, initial_sample, &mut symbols, &mut bit_writer);
        let extra_bytes = bit_writer.finish();

        let mut rans = RansEncoder::new();
        rans.encode_block(&symbols, table);
        let rans_payload = rans.finish();

        let raw_size = scaled_samples.len() * 8;
        let compressed_size = 1 + 1 + 2 + 2 + 2 + 2 + 8 + extra_bytes.len() + rans_payload.len();
        if compressed_size >= raw_size {
            return false;
        }

        for &s in &symbols {
            table.observe(s);
        }

        out.push(PredictorMode::DecimalFloat as u8);
        out.push(scale_code);
        out.extend_from_slice(&(scaled_samples.len() as u16).to_le_bytes());
        out.extend_from_slice(&(symbols.len() as u16).to_le_bytes());
        out.extend_from_slice(&(extra_bytes.len() as u16).to_le_bytes());
        out.extend_from_slice(&(rans_payload.len() as u16).to_le_bytes());
        out.extend_from_slice(&initial_sample.to_le_bytes());

        out.extend_from_slice(&extra_bytes);
        out.extend_from_slice(&rans_payload);
        true
    }

    /// Automatically select between Constant, Lossless Decimal Float, and Gorilla XOR for u64/f64 blocks.
    pub fn write_auto_u64_block(
        samples: &[u64],
        initial_sample: u64,
        table: &mut AdaptiveTable,
        out: &mut Vec<u8>,
    ) {
        if samples.is_empty() {
            return;
        }

        // Fast path for constant float / word block
        let first = samples[0];
        if samples.iter().all(|&x| x == first) {
            Self::write_constant_block(samples.len(), first, out);
            return;
        }

        // Lossless Decimal Quantization check
        if let Some((scale_code, scale)) = Self::find_lossless_decimal_scale(samples) {
            let mut scaled = Vec::with_capacity(samples.len());
            for &w in samples {
                let f = f64::from_bits(w);
                let sv = f * scale;
                scaled.push((sv + if sv >= 0.0 { 0.5 } else { -0.5 }) as i64);
            }
            let init_f = f64::from_bits(initial_sample);
            let init_sv = init_f * scale;
            let initial_scaled = (init_sv + if init_sv >= 0.0 { 0.5 } else { -0.5 }) as i64;

            if Self::try_write_decimal_float_block(&scaled, initial_scaled, scale_code, table, out) {
                return;
            }
        }

        // Fall back to Gorilla XOR
        Self::write_xor_u64_block(samples, initial_sample, table, out);
    }

    /// Write an uncompressed raw block.
    pub fn write_raw_u64_block(samples: &[u64], out: &mut Vec<u8>) {
        out.push(PredictorMode::Raw as u8);
        out.extend_from_slice(&(samples.len() as u16).to_le_bytes());
        for &s in samples {
            out.extend_from_slice(&s.to_le_bytes());
        }
    }
}
