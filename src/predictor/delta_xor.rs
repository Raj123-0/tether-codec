use crate::Vec;
// Delta and Gorilla-style XOR baseline predictors for integer and floating-point streams.

use crate::stream::bitstream::{BitReader, BitWriter};

#[inline]
pub fn zigzag_encode_i64(n: i64) -> u64 {
    ((n << 1) ^ (n >> 63)) as u64
}

#[inline]
pub fn zigzag_decode_i64(z: u64) -> i64 {
    ((z >> 1) as i64) ^ (-((z & 1) as i64))
}

#[inline]
pub fn zigzag_encode_i32(n: i32) -> u32 {
    ((n << 1) ^ (n >> 31)) as u32
}

#[inline]
pub fn zigzag_decode_i32(z: u32) -> i32 {
    ((z >> 1) as i32) ^ (-((z & 1) as i32))
}

/// Classify an unsigned 64-bit residual into (symbol, extra_bit_count, extra_value).
#[inline]
pub fn classify_residual_u64(val: u64) -> (u8, u8, u64) {
    if val < 128 {
        (val as u8, 0, 0)
    } else {
        let w = (64 - val.leading_zeros()) as u8; // 8 to 64
        let sym = 128 + w;
        let extra_bits = w - 1;
        let mask = if extra_bits == 64 { u64::MAX } else { (1u64 << extra_bits) - 1 };
        let extra_val = val & mask;
        (sym, extra_bits, extra_val)
    }
}

/// Reconstruct an unsigned 64-bit residual from symbol and extra bits safely.
#[inline]
pub fn reconstruct_residual_u64(symbol: u8, reader: &mut BitReader) -> u64 {
    if symbol < 128 {
        symbol as u64
    } else {
        let w = symbol.saturating_sub(128);
        if w == 0 {
            0
        } else if w <= 64 {
            let extra_bits = w - 1;
            let extra_val = reader.read_bits(extra_bits);
            (1u64 << (w - 1)) | extra_val
        } else {
            // Out-of-range symbol fallback
            reader.read_bits(64)
        }
    }
}

/// Delta encoding for integer slices (computes wrapping differences and zigzags).
pub fn encode_delta_i64_block(
    samples: &[i64],
    initial_sample: i64,
    symbols_out: &mut Vec<u8>,
    writer: &mut BitWriter,
) {
    let mut prev = initial_sample;
    for &val in samples {
        let diff = val.wrapping_sub(prev);
        let z = zigzag_encode_i64(diff);
        let (sym, extra_count, extra_val) = classify_residual_u64(z);
        symbols_out.push(sym);
        writer.write_bits(extra_val, extra_count);
        prev = val;
    }
}

/// Delta decoding for integer slices.
pub fn decode_delta_i64_block(
    symbols: &[u8],
    initial_sample: i64,
    reader: &mut BitReader,
    samples_out: &mut Vec<i64>,
) {
    let mut prev = initial_sample;
    for &sym in symbols {
        let z = reconstruct_residual_u64(sym, reader);
        let diff = zigzag_decode_i64(z);
        let val = prev.wrapping_add(diff);
        samples_out.push(val);
        prev = val;
    }
}

/// Gorilla XOR encoding for 64-bit float / word bitstreams.
pub fn encode_xor_u64_block(
    samples: &[u64],
    initial_sample: u64,
    symbols_out: &mut Vec<u8>,
    writer: &mut BitWriter,
) {
    let mut prev = initial_sample;
    for &val in samples {
        let xor_diff = val ^ prev;
        let (sym, extra_count, extra_val) = classify_residual_u64(xor_diff);
        symbols_out.push(sym);
        writer.write_bits(extra_val, extra_count);
        prev = val;
    }
}

/// Gorilla XOR decoding for 64-bit float / word bitstreams.
pub fn decode_xor_u64_block(
    symbols: &[u8],
    initial_sample: u64,
    reader: &mut BitReader,
    samples_out: &mut Vec<u64>,
) {
    let mut prev = initial_sample;
    for &sym in symbols {
        let xor_diff = reconstruct_residual_u64(sym, reader);
        let val = prev ^ xor_diff;
        samples_out.push(val);
        prev = val;
    }
}
