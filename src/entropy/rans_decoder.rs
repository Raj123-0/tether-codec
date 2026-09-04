use crate::Vec;
// 32-bit streaming range Asymmetric Numeral Systems (rANS) decoder.

use super::adaptive_table::{AdaptiveTable, TOTAL_BITS, TOTAL_FREQ};
use super::rans_encoder::RANS_L;

pub struct RansDecoder<'a> {
    pub state: u32,
    pub bytes: &'a [u8],
    pub cursor: usize,
}

impl<'a> RansDecoder<'a> {
    /// Initialize decoder from a payload produced by RansEncoder::finish().
    pub fn new(payload: &'a [u8]) -> Result<Self, &'static str> {
        if payload.len() < 4 {
            return Err("Payload too short for rANS state");
        }
        let state = u32::from_le_bytes(payload[0..4].try_into().unwrap());
        Ok(Self {
            state,
            bytes: &payload[4..],
            cursor: 0,
        })
    }

    /// Decode a single symbol using the given table.
    #[inline]
    pub fn decode_symbol(&mut self, table: &AdaptiveTable) -> u8 {
        let slot = (self.state & (TOTAL_FREQ - 1)) as u16;
        let symbol = table.lut[slot as usize];
        let c = table.cum[symbol as usize] as u32;
        let f = table.freq[symbol as usize] as u32;

        self.state = f * (self.state >> TOTAL_BITS) + (slot as u32 - c);

        while self.state < RANS_L && self.cursor < self.bytes.len() {
            self.state = (self.state << 8) | (self.bytes[self.cursor] as u32);
            self.cursor += 1;
        }

        symbol
    }

    /// Decode count symbols in forward order.
    pub fn decode_block(&mut self, count: usize, table: &AdaptiveTable) -> Vec<u8> {
        let mut symbols = Vec::with_capacity(count);
        for _ in 0..count {
            symbols.push(self.decode_symbol(table));
        }
        symbols
    }
}
