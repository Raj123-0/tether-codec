use crate::Vec;
// 32-bit streaming range Asymmetric Numeral Systems (rANS) encoder.

use super::adaptive_table::{AdaptiveTable, TOTAL_BITS};

pub const RANS_L: u32 = 1 << 23; // Lower bound
const MAX_X_BASE: u32 = (1 << 31) >> TOTAL_BITS; // (1 << 21)

pub struct RansEncoder {
    pub state: u32,
    pub bytes: Vec<u8>,
}

impl RansEncoder {
    pub fn new() -> Self {
        Self {
            state: RANS_L,
            bytes: Vec::with_capacity(512),
        }
    }

    /// Reset encoder state for a new block.
    pub fn reset(&mut self) {
        self.state = RANS_L;
        self.bytes.clear();
    }

    /// Encode a single symbol using the given table.
    #[inline]
    pub fn encode_symbol(&mut self, symbol: u8, table: &AdaptiveTable) {
        let f = table.freq[symbol as usize] as u32;
        let c = table.cum[symbol as usize] as u32;

        let max_x = MAX_X_BASE * f;
        while self.state >= max_x {
            self.bytes.push((self.state & 0xFF) as u8);
            self.state >>= 8;
        }

        let q = self.state / f;
        let r = self.state % f;
        self.state = (q << TOTAL_BITS) + c + r;
    }

    /// Encode a slice of symbols (backwards, for forward-decoding compatibility).
    pub fn encode_block(&mut self, symbols: &[u8], table: &AdaptiveTable) {
        for &s in symbols.iter().rev() {
            self.encode_symbol(s, table);
        }
    }

    /// Finish encoding and produce payload: 4-byte state + forward-ordered byte stream.
    pub fn finish(self) -> Vec<u8> {
        let mut out = Vec::with_capacity(4 + self.bytes.len());
        out.extend_from_slice(&self.state.to_le_bytes());
        // Reverse emitted bytes so the forward decoder can consume them sequentially
        for &b in self.bytes.iter().rev() {
            out.push(b);
        }
        out
    }
}

impl Default for RansEncoder {
    fn default() -> Self {
        Self::new()
    }
}
