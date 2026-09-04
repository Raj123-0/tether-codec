use crate::Vec;
// Block framing reader for compressed and raw blocks.

use crate::entropy::{AdaptiveTable, RansDecoder};
use crate::predictor::delta_xor::{decode_delta_i64_block, decode_delta2_i64_block, decode_xor_u64_block};
use crate::predictor::adaptive_linear::decode_adaptive_fir_block;
use crate::predictor::PredictorMode;
use crate::stream::bitstream::BitReader;

pub struct BlockReader;

impl BlockReader {
    /// Read one block from `data[*offset..]` and append decoded `u64` words to `samples_out`.
    /// Returns Ok(true) if a block was decoded, Ok(false) if end of data reached,
    /// or Err(&str) if corrupted.
    pub fn read_block_u64(
        data: &[u8],
        offset: &mut usize,
        table: &mut AdaptiveTable,
        samples_out: &mut Vec<u64>,
    ) -> Result<bool, &'static str> {
        if *offset >= data.len() {
            return Ok(false);
        }

        let mode_byte = data[*offset];
        *offset += 1;

        let mode = PredictorMode::from_u8(mode_byte).ok_or("Unknown predictor mode")?;

        match mode {
            PredictorMode::Raw => {
                if *offset + 2 > data.len() {
                    return Err("Truncated raw block header");
                }
                let sample_count = u16::from_le_bytes(data[*offset..*offset + 2].try_into().unwrap()) as usize;
                *offset += 2;

                let payload_len = sample_count * 8;
                if *offset + payload_len > data.len() {
                    return Err("Truncated raw block payload");
                }

                for i in 0..sample_count {
                    let start = *offset + i * 8;
                    let val = u64::from_le_bytes(data[start..start + 8].try_into().unwrap());
                    samples_out.push(val);
                }
                *offset += payload_len;
                Ok(true)
            }
            PredictorMode::Constant => {
                if *offset + 2 + 8 > data.len() {
                    return Err("Truncated constant block header");
                }
                let sample_count = u16::from_le_bytes(data[*offset..*offset + 2].try_into().unwrap()) as usize;
                *offset += 2;
                let val = u64::from_le_bytes(data[*offset..*offset + 8].try_into().unwrap());
                *offset += 8;

                samples_out.extend(core::iter::repeat(val).take(sample_count));
                Ok(true)
            }
            PredictorMode::Delta => {
                if *offset + 2 + 2 + 2 + 2 + 8 > data.len() {
                    return Err("Truncated delta block header");
                }

                let sample_count = u16::from_le_bytes(data[*offset..*offset + 2].try_into().unwrap()) as usize;
                *offset += 2;
                let symbol_count = u16::from_le_bytes(data[*offset..*offset + 2].try_into().unwrap()) as usize;
                *offset += 2;
                let extra_len = u16::from_le_bytes(data[*offset..*offset + 2].try_into().unwrap()) as usize;
                *offset += 2;
                let rans_len = u16::from_le_bytes(data[*offset..*offset + 2].try_into().unwrap()) as usize;
                *offset += 2;
                let initial_sample = i64::from_le_bytes(data[*offset..*offset + 8].try_into().unwrap());
                *offset += 8;

                if *offset + extra_len + rans_len > data.len() {
                    return Err("Truncated delta block payloads");
                }

                let extra_bytes = &data[*offset..*offset + extra_len];
                *offset += extra_len;
                let rans_bytes = &data[*offset..*offset + rans_len];
                *offset += rans_len;

                let mut rans_dec = RansDecoder::new(rans_bytes)?;
                let symbols = rans_dec.decode_block(symbol_count, table);

                for &s in &symbols {
                    table.observe(s);
                }

                let mut bit_reader = BitReader::new(extra_bytes);
                let mut decoded_i64 = Vec::with_capacity(sample_count);
                decode_delta_i64_block(&symbols, initial_sample, &mut bit_reader, sample_count, &mut decoded_i64)?;

                for val in decoded_i64 {
                    samples_out.push(val as u64);
                }

                Ok(true)
            }
            PredictorMode::DeltaOfDelta => {
                if *offset + 2 + 2 + 2 + 2 + 16 > data.len() {
                    return Err("Truncated delta-of-delta block header");
                }

                let sample_count = u16::from_le_bytes(data[*offset..*offset + 2].try_into().unwrap()) as usize;
                *offset += 2;
                let symbol_count = u16::from_le_bytes(data[*offset..*offset + 2].try_into().unwrap()) as usize;
                *offset += 2;
                let extra_len = u16::from_le_bytes(data[*offset..*offset + 2].try_into().unwrap()) as usize;
                *offset += 2;
                let rans_len = u16::from_le_bytes(data[*offset..*offset + 2].try_into().unwrap()) as usize;
                *offset += 2;
                let prev1 = i64::from_le_bytes(data[*offset..*offset + 8].try_into().unwrap());
                *offset += 8;
                let prev2 = i64::from_le_bytes(data[*offset..*offset + 8].try_into().unwrap());
                *offset += 8;

                if *offset + extra_len + rans_len > data.len() {
                    return Err("Truncated delta-of-delta block payloads");
                }

                let extra_bytes = &data[*offset..*offset + extra_len];
                *offset += extra_len;
                let rans_bytes = &data[*offset..*offset + rans_len];
                *offset += rans_len;

                let mut rans_dec = RansDecoder::new(rans_bytes)?;
                let symbols = rans_dec.decode_block(symbol_count, table);

                for &s in &symbols {
                    table.observe(s);
                }

                let mut bit_reader = BitReader::new(extra_bytes);
                let mut decoded_i64 = Vec::with_capacity(sample_count);
                decode_delta2_i64_block(&symbols, prev1, prev2, &mut bit_reader, sample_count, &mut decoded_i64)?;

                for val in decoded_i64 {
                    samples_out.push(val as u64);
                }

                Ok(true)
            }
            PredictorMode::AdaptiveLinear => {
                if *offset + 2 + 2 + 2 + 2 + 24 > data.len() {
                    return Err("Truncated adaptive FIR block header");
                }

                let sample_count = u16::from_le_bytes(data[*offset..*offset + 2].try_into().unwrap()) as usize;
                *offset += 2;
                let symbol_count = u16::from_le_bytes(data[*offset..*offset + 2].try_into().unwrap()) as usize;
                *offset += 2;
                let extra_len = u16::from_le_bytes(data[*offset..*offset + 2].try_into().unwrap()) as usize;
                *offset += 2;
                let rans_len = u16::from_le_bytes(data[*offset..*offset + 2].try_into().unwrap()) as usize;
                *offset += 2;

                let h0 = i64::from_le_bytes(data[*offset..*offset + 8].try_into().unwrap());
                *offset += 8;
                let h1 = i64::from_le_bytes(data[*offset..*offset + 8].try_into().unwrap());
                *offset += 8;
                let h2 = i64::from_le_bytes(data[*offset..*offset + 8].try_into().unwrap());
                *offset += 8;

                if *offset + extra_len + rans_len > data.len() {
                    return Err("Truncated adaptive FIR block payloads");
                }

                let extra_bytes = &data[*offset..*offset + extra_len];
                *offset += extra_len;
                let rans_bytes = &data[*offset..*offset + rans_len];
                *offset += rans_len;

                let mut rans_dec = RansDecoder::new(rans_bytes)?;
                let symbols = rans_dec.decode_block(symbol_count, table);

                for &s in &symbols {
                    table.observe(s);
                }

                let mut bit_reader = BitReader::new(extra_bytes);
                let mut decoded_i64 = Vec::with_capacity(sample_count);
                decode_adaptive_fir_block(&symbols, [h0, h1, h2], &mut bit_reader, sample_count, &mut decoded_i64)?;

                for val in decoded_i64 {
                    samples_out.push(val as u64);
                }

                Ok(true)
            }
            PredictorMode::Xor => {
                if *offset + 2 + 2 + 2 + 2 + 8 > data.len() {
                    return Err("Truncated XOR block header");
                }

                let sample_count = u16::from_le_bytes(data[*offset..*offset + 2].try_into().unwrap()) as usize;
                *offset += 2;
                let symbol_count = u16::from_le_bytes(data[*offset..*offset + 2].try_into().unwrap()) as usize;
                *offset += 2;
                let extra_len = u16::from_le_bytes(data[*offset..*offset + 2].try_into().unwrap()) as usize;
                *offset += 2;
                let rans_len = u16::from_le_bytes(data[*offset..*offset + 2].try_into().unwrap()) as usize;
                *offset += 2;
                let initial_sample = u64::from_le_bytes(data[*offset..*offset + 8].try_into().unwrap());
                *offset += 8;

                if *offset + extra_len + rans_len > data.len() {
                    return Err("Truncated XOR block payloads");
                }

                let extra_bytes = &data[*offset..*offset + extra_len];
                *offset += extra_len;
                let rans_bytes = &data[*offset..*offset + rans_len];
                *offset += rans_len;

                let mut rans_dec = RansDecoder::new(rans_bytes)?;
                let symbols = rans_dec.decode_block(symbol_count, table);

                for &s in &symbols {
                    table.observe(s);
                }

                let mut bit_reader = BitReader::new(extra_bytes);
                decode_xor_u64_block(&symbols, initial_sample, &mut bit_reader, sample_count, samples_out)?;

                Ok(true)
            }
            PredictorMode::LinearRamp => {
                if *offset + 2 + 8 + 8 > data.len() {
                    return Err("Truncated linear ramp block header");
                }
                let sample_count = u16::from_le_bytes(data[*offset..*offset + 2].try_into().unwrap()) as usize;
                *offset += 2;
                let start_val = i64::from_le_bytes(data[*offset..*offset + 8].try_into().unwrap());
                *offset += 8;
                let delta = i64::from_le_bytes(data[*offset..*offset + 8].try_into().unwrap());
                *offset += 8;

                for i in 0..sample_count {
                    let val = start_val.wrapping_add((i as i64).wrapping_mul(delta));
                    samples_out.push(val as u64);
                }
                Ok(true)
            }
            PredictorMode::RepeatHistory => {
                if *offset + 2 + 2 > data.len() {
                    return Err("Truncated repeat history block header");
                }
                let sample_count = u16::from_le_bytes(data[*offset..*offset + 2].try_into().unwrap()) as usize;
                *offset += 2;
                let dist = u16::from_le_bytes(data[*offset..*offset + 2].try_into().unwrap()) as usize;
                *offset += 2;

                if dist == 0 || dist > samples_out.len() {
                    return Err("Invalid history repeat distance");
                }

                for _ in 0..sample_count {
                    let val = samples_out[samples_out.len() - dist];
                    samples_out.push(val);
                }
                Ok(true)
            }
            PredictorMode::DecimalFloat => {
                if *offset + 1 + 2 + 2 + 2 + 2 + 8 > data.len() {
                    return Err("Truncated decimal float block header");
                }
                let scale_code = data[*offset] as usize;
                *offset += 1;
                let sample_count = u16::from_le_bytes(data[*offset..*offset + 2].try_into().unwrap()) as usize;
                *offset += 2;
                let symbol_count = u16::from_le_bytes(data[*offset..*offset + 2].try_into().unwrap()) as usize;
                *offset += 2;
                let extra_len = u16::from_le_bytes(data[*offset..*offset + 2].try_into().unwrap()) as usize;
                *offset += 2;
                let rans_len = u16::from_le_bytes(data[*offset..*offset + 2].try_into().unwrap()) as usize;
                *offset += 2;
                let initial_sample = i64::from_le_bytes(data[*offset..*offset + 8].try_into().unwrap());
                *offset += 8;

                if *offset + extra_len + rans_len > data.len() {
                    return Err("Truncated decimal float block payloads");
                }

                const SCALES: [f64; 7] = [1.0, 10.0, 100.0, 1000.0, 10000.0, 100000.0, 1000000.0];
                if scale_code >= SCALES.len() {
                    return Err("Invalid decimal float scale code");
                }
                let scale = SCALES[scale_code];

                let extra_bytes = &data[*offset..*offset + extra_len];
                *offset += extra_len;
                let rans_bytes = &data[*offset..*offset + rans_len];
                *offset += rans_len;

                let mut rans_dec = RansDecoder::new(rans_bytes)?;
                let symbols = rans_dec.decode_block(symbol_count, table);

                for &s in &symbols {
                    table.observe(s);
                }

                let mut bit_reader = BitReader::new(extra_bytes);
                let mut decoded_i64 = Vec::with_capacity(sample_count);
                decode_delta_i64_block(&symbols, initial_sample, &mut bit_reader, sample_count, &mut decoded_i64)?;

                for val in decoded_i64 {
                    let f = (val as f64) / scale;
                    samples_out.push(f.to_bits());
                }

                Ok(true)
            }
        }
    }
}
