use crate::Vec;
// Delta, Delta-of-Delta, and Gorilla-style XOR predictors with Zero-Run residual packing.

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

/// Classify an unsigned 64-bit non-zero residual into (symbol, extra_bit_count, extra_value).
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

/// Pack residuals into symbols with Zero-Run encoding and extra bits.
/// Symbols:
///   0: single zero residual
///   1..127: small literal residuals (1..127)
///   128..192: prefix for larger residuals (128 + bit_width)
///   193..255: run of zeros of length 2..64 (193 -> 2 zeros, 255 -> 64 zeros)
pub fn pack_residuals(
    residuals: &[u64],
    symbols_out: &mut Vec<u8>,
    writer: &mut BitWriter,
) {
    let mut i = 0;
    let n = residuals.len();
    while i < n {
        let r = residuals[i];
        if r == 0 {
            let mut run = 0usize;
            while i < n && residuals[i] == 0 && run < 64 {
                run += 1;
                i += 1;
            }
            if run == 1 {
                symbols_out.push(0);
            } else {
                symbols_out.push(193 + (run - 2) as u8);
            }
        } else {
            let (sym, extra_count, extra_val) = classify_residual_u64(r);
            symbols_out.push(sym);
            if extra_count > 0 {
                writer.write_bits(extra_val, extra_count);
            }
            i += 1;
        }
    }
}

/// Unpack symbols and extra bits back into exact target_count residuals.
pub fn unpack_residuals(
    symbols: &[u8],
    reader: &mut BitReader,
    target_count: usize,
    residuals_out: &mut Vec<u64>,
) -> Result<(), &'static str> {
    for &sym in symbols {
        if sym == 0 {
            residuals_out.push(0);
        } else if sym >= 193 {
            let run = (sym - 193) as usize + 2;
            if residuals_out.len() + run > target_count {
                return Err("Zero-run length exceeds target sample count");
            }
            residuals_out.extend(core::iter::repeat(0).take(run));
        } else if sym < 128 {
            residuals_out.push(sym as u64);
        } else {
            let w = sym - 128;
            if w == 0 || w > 64 {
                return Err("Corrupted residual symbol bit-width");
            }
            let extra_bits = w - 1;
            let extra_val = reader.read_bits(extra_bits);
            let val = (1u64 << (w - 1)) | extra_val;
            residuals_out.push(val);
        }
    }
    if residuals_out.len() != target_count {
        return Err("Decoded residual count does not match expected sample count");
    }
    Ok(())
}

/// Delta encoding for integer slices.
pub fn encode_delta_i64_block(
    samples: &[i64],
    initial_sample: i64,
    symbols_out: &mut Vec<u8>,
    writer: &mut BitWriter,
) {
    let mut prev = initial_sample;
    let mut residuals = Vec::with_capacity(samples.len());
    for &val in samples {
        let diff = val.wrapping_sub(prev);
        residuals.push(zigzag_encode_i64(diff));
        prev = val;
    }
    pack_residuals(&residuals, symbols_out, writer);
}

/// Delta decoding for integer slices.
pub fn decode_delta_i64_block(
    symbols: &[u8],
    initial_sample: i64,
    reader: &mut BitReader,
    target_count: usize,
    samples_out: &mut Vec<i64>,
) -> Result<(), &'static str> {
    let mut residuals = Vec::with_capacity(target_count);
    unpack_residuals(symbols, reader, target_count, &mut residuals)?;
    let mut prev = initial_sample;
    for z in residuals {
        let diff = zigzag_decode_i64(z);
        let val = prev.wrapping_add(diff);
        samples_out.push(val);
        prev = val;
    }
    Ok(())
}

/// Delta-of-Delta (second-order difference) encoding for integer slices.
/// Predicts: p_i = 2 * x_{i-1} - x_{i-2}
pub fn encode_delta2_i64_block(
    samples: &[i64],
    prev1: i64,
    prev2: i64,
    symbols_out: &mut Vec<u8>,
    writer: &mut BitWriter,
) -> (i64, i64) {
    let mut p1 = prev1;
    let mut p2 = prev2;
    let mut residuals = Vec::with_capacity(samples.len());
    for &val in samples {
        let pred = p1.wrapping_add(p1.wrapping_sub(p2));
        let diff = val.wrapping_sub(pred);
        residuals.push(zigzag_encode_i64(diff));
        p2 = p1;
        p1 = val;
    }
    pack_residuals(&residuals, symbols_out, writer);
    (p1, p2)
}

/// Delta-of-Delta decoding for integer slices.
pub fn decode_delta2_i64_block(
    symbols: &[u8],
    prev1: i64,
    prev2: i64,
    reader: &mut BitReader,
    target_count: usize,
    samples_out: &mut Vec<i64>,
) -> Result<(i64, i64), &'static str> {
    let mut residuals = Vec::with_capacity(target_count);
    unpack_residuals(symbols, reader, target_count, &mut residuals)?;
    let mut p1 = prev1;
    let mut p2 = prev2;
    for z in residuals {
        let diff = zigzag_decode_i64(z);
        let pred = p1.wrapping_add(p1.wrapping_sub(p2));
        let val = pred.wrapping_add(diff);
        samples_out.push(val);
        p2 = p1;
        p1 = val;
    }
    Ok((p1, p2))
}

/// Gorilla XOR encoding for 64-bit float / word bitstreams with Zero-Run packing.
pub fn encode_xor_u64_block(
    samples: &[u64],
    initial_sample: u64,
    symbols_out: &mut Vec<u8>,
    writer: &mut BitWriter,
) {
    let mut prev = initial_sample;
    let mut residuals = Vec::with_capacity(samples.len());
    for &val in samples {
        let xor_diff = val ^ prev;
        residuals.push(xor_diff);
        prev = val;
    }
    pack_residuals(&residuals, symbols_out, writer);
}

/// Gorilla XOR decoding for 64-bit float / word bitstreams.
pub fn decode_xor_u64_block(
    symbols: &[u8],
    initial_sample: u64,
    reader: &mut BitReader,
    target_count: usize,
    samples_out: &mut Vec<u64>,
) -> Result<(), &'static str> {
    let mut residuals = Vec::with_capacity(target_count);
    unpack_residuals(symbols, reader, target_count, &mut residuals)?;
    let mut prev = initial_sample;
    for xor_diff in residuals {
        let val = prev ^ xor_diff;
        samples_out.push(val);
        prev = val;
    }
    Ok(())
}
