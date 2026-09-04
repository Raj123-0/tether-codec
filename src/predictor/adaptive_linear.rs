use crate::Vec;
// Lightweight integer-only adaptive linear predictor (Order-3 FIR with sign-sign LMS).
// Zero floating-point operations, zero division, fixed small state (36 bytes).

use crate::predictor::delta_xor::{pack_residuals, unpack_residuals, zigzag_encode_i64, zigzag_decode_i64};
use crate::stream::bitstream::{BitReader, BitWriter};

pub const FIR_SCALE_SHIFT: u32 = 8;
pub const FIR_ROUND: i64 = 1 << (FIR_SCALE_SHIFT - 1);
pub const COEFF_MIN: i32 = -2048;
pub const COEFF_MAX: i32 = 2048;

#[inline]
fn sgn(val: i64) -> i32 {
    if val > 0 { 1 } else if val < 0 { -1 } else { 0 }
}

#[derive(Clone, Copy, Debug)]
pub struct AdaptiveLinearPredictor {
    pub coeffs: [i32; 3],
    pub history: [i64; 3],
}

impl AdaptiveLinearPredictor {
    /// Initialize with order-2 linear extrapolation coefficients (2 * x[n-1] - x[n-2]).
    pub fn new(initial_history: [i64; 3]) -> Self {
        Self {
            coeffs: [512, -256, 0], // scale = 256
            history: initial_history,
        }
    }

    #[inline]
    pub fn predict(&self) -> i64 {
        let p = (self.coeffs[0] as i64).wrapping_mul(self.history[0])
            .wrapping_add((self.coeffs[1] as i64).wrapping_mul(self.history[1]))
            .wrapping_add((self.coeffs[2] as i64).wrapping_mul(self.history[2]))
            .wrapping_add(FIR_ROUND);
        p >> FIR_SCALE_SHIFT
    }

    #[inline]
    pub fn update(&mut self, actual: i64, pred: i64) {
        let err = actual.wrapping_sub(pred);
        let sgn_e = sgn(err);

        self.coeffs[0] = (self.coeffs[0] + sgn_e * sgn(self.history[0])).clamp(COEFF_MIN, COEFF_MAX);
        self.coeffs[1] = (self.coeffs[1] + sgn_e * sgn(self.history[1])).clamp(COEFF_MIN, COEFF_MAX);
        self.coeffs[2] = (self.coeffs[2] + sgn_e * sgn(self.history[2])).clamp(COEFF_MIN, COEFF_MAX);

        self.history[2] = self.history[1];
        self.history[1] = self.history[0];
        self.history[0] = actual;
    }
}

/// Encode a block using the adaptive linear FIR predictor with Zero-Run packing.
pub fn encode_adaptive_fir_block(
    samples: &[i64],
    initial_history: [i64; 3],
    symbols_out: &mut Vec<u8>,
    writer: &mut BitWriter,
) -> [i64; 3] {
    let mut fir = AdaptiveLinearPredictor::new(initial_history);
    let mut residuals = Vec::with_capacity(samples.len());
    for &val in samples {
        let pred = fir.predict();
        let err = val.wrapping_sub(pred);
        residuals.push(zigzag_encode_i64(err));
        fir.update(val, pred);
    }
    pack_residuals(&residuals, symbols_out, writer);
    fir.history
}

/// Decode a block using the adaptive linear FIR predictor.
pub fn decode_adaptive_fir_block(
    symbols: &[u8],
    initial_history: [i64; 3],
    reader: &mut BitReader,
    target_count: usize,
    samples_out: &mut Vec<i64>,
) -> Result<[i64; 3], &'static str> {
    let mut residuals = Vec::with_capacity(target_count);
    unpack_residuals(symbols, reader, target_count, &mut residuals)?;
    let mut fir = AdaptiveLinearPredictor::new(initial_history);
    for z in residuals {
        let err = zigzag_decode_i64(z);
        let pred = fir.predict();
        let val = pred.wrapping_add(err);
        samples_out.push(val);
        fir.update(val, pred);
    }
    Ok(fir.history)
}
