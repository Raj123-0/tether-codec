// Adaptive frequency table for rANS entropy coding.

pub const ALPHABET_SIZE: usize = 256;
pub const TOTAL_BITS: u32 = 10;
pub const TOTAL_FREQ: u32 = 1 << TOTAL_BITS; // 1024
pub const RESCALE_LIMIT: u32 = 2048;

#[derive(Clone, Debug)]
pub struct AdaptiveTable {
    /// Normalized frequencies summing exactly to TOTAL_FREQ (all >= 1)
    pub freq: [u16; ALPHABET_SIZE],
    /// Cumulative frequencies: cum[0] = 0, cum[256] = TOTAL_FREQ
    pub cum: [u16; ALPHABET_SIZE + 1],
    /// Fast reverse lookup table: slot (0..1023) -> symbol (0..255)
    pub lut: [u8; TOTAL_FREQ as usize],
    /// Running raw observation counts before rescaling
    pub raw_counts: [u16; ALPHABET_SIZE],
    /// Total raw observations in current window
    pub total_raw: u32,
}

impl AdaptiveTable {
    /// Create a new uniform frequency table (every symbol has equal initial weight).
    pub fn new_uniform() -> Self {
        let raw = [4u16; ALPHABET_SIZE]; // 256 * 4 = 1024
        let mut table = Self {
            freq: [0; ALPHABET_SIZE],
            cum: [0; ALPHABET_SIZE + 1],
            lut: [0; TOTAL_FREQ as usize],
            raw_counts: raw,
            total_raw: 1024,
        };
        table.normalize();
        table
    }

    /// Create an adaptive table with initial bias toward small symbols (standard for residuals).
    pub fn new_skewed() -> Self {
        let mut raw = [1u16; ALPHABET_SIZE];
        raw[0] = 128;
        raw[1] = 64;
        raw[2] = 32;
        raw[3] = 16;
        for item in raw[4..16].iter_mut() {
            *item = 8;
        }
        for item in raw[16..64].iter_mut() {
            *item = 4;
        }
        for item in raw[64..128].iter_mut() {
            *item = 2;
        }
        let total: u32 = raw.iter().map(|&x| x as u32).sum();
        let mut table = Self {
            freq: [0; ALPHABET_SIZE],
            cum: [0; ALPHABET_SIZE + 1],
            lut: [0; TOTAL_FREQ as usize],
            raw_counts: raw,
            total_raw: total,
        };
        table.normalize();
        table
    }

    /// Build a table customized from an exact frequency count histogram.
    pub fn from_counts(counts: &[u32; ALPHABET_SIZE]) -> Self {
        let mut raw = [1u16; ALPHABET_SIZE];
        for i in 0..ALPHABET_SIZE {
            raw[i] = (counts[i] as u16).max(1);
        }
        let total: u32 = raw.iter().map(|&x| x as u32).sum();
        let mut table = Self {
            freq: [0; ALPHABET_SIZE],
            cum: [0; ALPHABET_SIZE + 1],
            lut: [0; TOTAL_FREQ as usize],
            raw_counts: raw,
            total_raw: total,
        };
        table.normalize();
        table
    }

    /// Observe a symbol occurrence and trigger rescaling if limit reached.
    #[inline]
    pub fn observe(&mut self, symbol: u8) {
        let idx = symbol as usize;
        self.raw_counts[idx] = self.raw_counts[idx].saturating_add(1);
        self.total_raw = self.total_raw.saturating_add(1);

        if self.total_raw >= RESCALE_LIMIT {
            self.rescale_and_normalize();
        }
    }

    /// Halve all raw counts (keeping >= 1) and renormalize.
    pub fn rescale_and_normalize(&mut self) {
        let mut new_total = 0u32;
        for c in self.raw_counts.iter_mut() {
            *c = (*c >> 1).max(1);
            new_total += *c as u32;
        }
        self.total_raw = new_total;
        self.normalize();
    }

    /// Deterministic pure-integer normalization of raw counts to sum to TOTAL_FREQ.
    pub fn normalize(&mut self) {
        let target_sum = TOTAL_FREQ;
        let num_symbols = ALPHABET_SIZE as u32;
        let total_raw = self.total_raw.max(1);

        let rem_target = target_sum - num_symbols;
        let mut allocated = 0u32;

        let mut remainders: [(u32, u8); ALPHABET_SIZE] = [(0, 0); ALPHABET_SIZE];

        for (i, rem) in remainders.iter_mut().enumerate().take(ALPHABET_SIZE) {
            let c = self.raw_counts[i] as u32;
            let prod = c * rem_target;
            let add = prod / total_raw;
            let r = prod % total_raw;

            self.freq[i] = (1 + add) as u16;
            allocated += add;
            *rem = (r, i as u8);
        }

        let leftover = (rem_target - allocated) as usize;
        if leftover > 0 {
            remainders.sort_unstable_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
            for item in remainders.iter().take(leftover.min(ALPHABET_SIZE)) {
                let idx = item.1 as usize;
                self.freq[idx] += 1;
            }
        }

        let mut current_cum = 0u16;
        for i in 0..ALPHABET_SIZE {
            self.cum[i] = current_cum;
            let f = self.freq[i];
            let end = current_cum + f;
            for slot in current_cum..end {
                self.lut[slot as usize] = i as u8;
            }
            current_cum = end;
        }
        self.cum[ALPHABET_SIZE] = current_cum;
        debug_assert_eq!(current_cum as u32, TOTAL_FREQ);
    }
}

impl Default for AdaptiveTable {
    fn default() -> Self {
        Self::new_skewed()
    }
}
