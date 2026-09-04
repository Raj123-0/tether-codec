// Lightweight, zero-allocation-ready bit-level reader and writer using u128 accumulator.

use crate::Vec;

pub struct BitWriter {
    pub bytes: Vec<u8>,
    acc: u128,
    bit_count: u8,
}

impl BitWriter {
    pub fn new() -> Self {
        Self {
            bytes: Vec::with_capacity(256),
            acc: 0,
            bit_count: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            bytes: Vec::with_capacity(capacity),
            acc: 0,
            bit_count: 0,
        }
    }

    #[inline]
    pub fn write_bits(&mut self, val: u64, count: u8) {
        if count == 0 {
            return;
        }
        let mask = if count == 64 { u64::MAX } else { (1u64 << count) - 1 };
        let clean_val = (val & mask) as u128;

        self.acc |= clean_val << self.bit_count;
        self.bit_count += count;

        while self.bit_count >= 8 {
            self.bytes.push((self.acc & 0xFF) as u8);
            self.acc >>= 8;
            self.bit_count -= 8;
        }
    }

    pub fn finish(mut self) -> Vec<u8> {
        if self.bit_count > 0 {
            self.bytes.push((self.acc & 0xFF) as u8);
            self.acc = 0;
            self.bit_count = 0;
        }
        self.bytes
    }
}

pub struct BitReader<'a> {
    pub bytes: &'a [u8],
    pub pos: usize,
    acc: u128,
    bit_count: u8,
}

impl<'a> BitReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            pos: 0,
            acc: 0,
            bit_count: 0,
        }
    }

    #[inline]
    pub fn read_bits(&mut self, count: u8) -> u64 {
        if count == 0 {
            return 0;
        }

        while self.bit_count < count {
            if self.pos < self.bytes.len() {
                self.acc |= (self.bytes[self.pos] as u128) << self.bit_count;
                self.pos += 1;
                self.bit_count += 8;
            } else {
                self.bit_count = count;
                break;
            }
        }

        let mask = if count == 64 { u64::MAX } else { (1u64 << count) - 1 };
        let val = (self.acc as u64) & mask;
        self.acc >>= count;
        self.bit_count = self.bit_count.saturating_sub(count);
        val
    }
}

impl Default for BitWriter {
    fn default() -> Self {
        Self::new()
    }
}
