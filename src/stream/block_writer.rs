// Block framing writer for compressed and raw blocks.

use crate::Vec;
use crate::entropy::{AdaptiveTable, RansEncoder};
use crate::predictor::delta_xor::{encode_delta_i64_block, encode_xor_u64_block};
use crate::predictor::adaptive_linear::encode_adaptive_fir_block;
use crate::predictor::selector::PredictorSelector;
use crate::predictor::PredictorMode;
use crate::stream::bitstream::BitWriter;

pub struct BlockWriter;

impl BlockWriter {
    /// Write an integer block automatically selecting the best predictor (Delta vs Adaptive Linear FIR).
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

        let mut symbols = Vec::with_capacity(samples.len());
        let mut bit_writer = BitWriter::with_capacity(samples.len() * 4);

        encode_delta_i64_block(samples, initial_sample, &mut symbols, &mut bit_writer);

        let extra_bytes = bit_writer.finish();

        let mut rans = RansEncoder::new();
        rans.encode_block(&symbols, table);
        let rans_payload = rans.finish();

        let raw_size = samples.len() * 8;
        let compressed_size = 1 + 2 + 2 + 2 + 8 + extra_bytes.len() + rans_payload.len();

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
        out.extend_from_slice(&(extra_bytes.len() as u16).to_le_bytes());
        out.extend_from_slice(&(rans_payload.len() as u16).to_le_bytes());
        out.extend_from_slice(&initial_sample.to_le_bytes());

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
        let compressed_size = 1 + 2 + 2 + 2 + 24 + extra_bytes.len() + rans_payload.len();

        if compressed_size >= raw_size {
            let u64_slice = unsafe { core::slice::from_raw_parts(samples.as_ptr() as *const u64, samples.len()) };
            Self::write_raw_u64_block(u64_slice, out);
            return new_history;
        }

        // Only update table if block is actually written as compressed!
        for &s in &symbols {
            table.observe(s);
        }

        out.push(PredictorMode::AdaptiveLinear as u8);
        out.extend_from_slice(&(samples.len() as u16).to_le_bytes());
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

        let mut symbols = Vec::with_capacity(samples.len());
        let mut bit_writer = BitWriter::with_capacity(samples.len() * 4);

        encode_xor_u64_block(samples, initial_sample, &mut symbols, &mut bit_writer);

        let extra_bytes = bit_writer.finish();

        let mut rans = RansEncoder::new();
        rans.encode_block(&symbols, table);
        let rans_payload = rans.finish();

        let raw_size = samples.len() * 8;
        let compressed_size = 1 + 2 + 2 + 2 + 8 + extra_bytes.len() + rans_payload.len();

        if compressed_size >= raw_size {
            Self::write_raw_u64_block(samples, out);
            return;
        }

        // Only update table if block is actually written as compressed!
        for &s in &symbols {
            table.observe(s);
        }

        out.push(PredictorMode::Xor as u8);
        out.extend_from_slice(&(samples.len() as u16).to_le_bytes());
        out.extend_from_slice(&(extra_bytes.len() as u16).to_le_bytes());
        out.extend_from_slice(&(rans_payload.len() as u16).to_le_bytes());
        out.extend_from_slice(&initial_sample.to_le_bytes());

        out.extend_from_slice(&extra_bytes);
        out.extend_from_slice(&rans_payload);
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
